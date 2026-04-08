use minijinja::{context, Environment};
use serde::Serialize;

use crate::checker::Diagnostic;
pub fn format_text(diagnostics: &[Diagnostic]) -> String {
    let mut out = String::new();
    for d in diagnostics {
        let first_line = d
            .comment
            .content
            .lines()
            .next()
            .unwrap_or("")
            .trim_end();
        let raw_first = format!(
            "{}{}{}",
            d.comment.prefix,
            if first_line.is_empty() { "" } else { " " },
            first_line
        );
        out.push_str(&format!(
            "{}:{}:{}: warning[comment]: {}\n",
            d.file,
            d.comment.span.start_line,
            d.comment.span.start_col,
            raw_first,
        ));
    }
    out
}

#[derive(Serialize)]
struct JsonlRecord<'a> {
    file: &'a str,
    line: usize,
    column: usize,
    end_line: usize,
    end_column: usize,
    kind: &'a str,
    text: String,
    severity: &'static str,
}

/// JSONL format: one JSON object per diagnostic, newline-delimited.
pub fn format_jsonl(diagnostics: &[Diagnostic]) -> String {
    let mut out = String::new();
    for d in diagnostics {
        let kind_str = d.comment.kind.to_string();
        let record = JsonlRecord {
            file: &d.file,
            line: d.comment.span.start_line,
            column: d.comment.span.start_col,
            end_line: d.comment.span.end_line,
            end_column: d.comment.span.end_col,
            kind: &kind_str,
            text: d.comment.raw_text(),
            severity: "warning",
        };
        if let Ok(json) = serde_json::to_string(&record) {
            out.push_str(&json);
            out.push('\n');
        }
    }
    out
}

const DEFAULT_INSTRUCTION: &str = "\
Remove comments that merely restate the code, describe obvious behavior, or were left by a previous edit. \
Keep comments that explain WHY (non-obvious intent, business rules, trade-offs, workarounds), \
mark incomplete work (TODO/FIXME with context), \
or are required by tooling (linter directives, type annotations, license headers). \
When in doubt, remove the comment -- the code should speak for itself.";

const DEFAULT_TEMPLATE: &str = r#"<comment-checker>
<summary>Found {{ count }} flagged comment(s) in {{ groups }} group(s).</summary>
<flagged-comments>
{% for g in comments %}<comment file="{{ g.file }}" line="{{ g.line }}" type="{{ g.kind }}">
{{ g.text }}
</comment>
{% endfor %}</flagged-comments>
<instruction>{{ instruction }}</instruction>
</comment-checker>"#;

#[derive(Serialize)]
struct PromptGroup {
    file: String,
    line: String,
    kind: String,
    text: String,
}

fn group_diagnostics(diagnostics: &[Diagnostic]) -> Vec<PromptGroup> {
    let mut groups: Vec<PromptGroup> = Vec::new();

    for d in diagnostics {
        let text = d.comment.raw_text().lines().next().unwrap_or("").trim_end().to_string();
        let kind = d.comment.kind.to_string();

        if let Some(last) = groups.last_mut() {
            if last.file == d.file && last.kind == kind {
                let prev_end: usize = last.line.split('-').last()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                if d.comment.span.start_line <= prev_end + 2 {
                    last.line = format!(
                        "{}-{}",
                        last.line.split('-').next().unwrap(),
                        d.comment.span.start_line
                    );
                    last.text.push('\n');
                    last.text.push_str(&text);
                    continue;
                }
            }
        }

        groups.push(PromptGroup {
            file: d.file.clone(),
            line: d.comment.span.start_line.to_string(),
            kind,
            text,
        });
    }

    groups
}

pub fn format_prompt(
    diagnostics: &[Diagnostic],
    template: Option<&str>,
    instruction: Option<&str>,
) -> String {
    if diagnostics.is_empty() {
        return String::new();
    }

    let groups = group_diagnostics(diagnostics);
    let instruction = instruction.unwrap_or(DEFAULT_INSTRUCTION);
    let tmpl = template.unwrap_or(DEFAULT_TEMPLATE);
    let mut env = Environment::new();
    env.set_auto_escape_callback(|_| minijinja::AutoEscape::None);
    env.add_template("prompt", tmpl).unwrap_or_else(|_| {
        env.add_template("prompt", DEFAULT_TEMPLATE).unwrap();
    });

    env.get_template("prompt")
        .and_then(|t| {
            t.render(context! {
                count => diagnostics.len(),
                groups => groups.len(),
                comments => groups,
                instruction => instruction,
            })
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::Diagnostic;
    use crate::parser::comment::{Comment, CommentKind, Span};

    fn make_diag(file: &str, content: &str, line: usize) -> Diagnostic {
        Diagnostic {
            file: file.to_string(),
            comment: Comment {
                kind: CommentKind::Line,
                prefix: "//".to_string(),
                content: content.to_string(),
                span: Span {
                    start_line: line,
                    start_col: 0,
                    end_line: line,
                    end_col: content.len(),
                },
            },
        }
    }

    #[test]
    fn test_format_text() {
        let diags = vec![make_diag("foo.rs", "TODO: fix", 10)];
        let out = format_text(&diags);
        assert!(out.contains("foo.rs:10:0: warning[comment]: // TODO: fix"));
    }

    #[test]
    fn test_format_jsonl() {
        let diags = vec![make_diag("bar.rs", "FIXME", 5)];
        let out = format_jsonl(&diags);
        assert!(out.contains("\"file\":\"bar.rs\""));
        assert!(out.contains("\"line\":5"));
        assert!(out.contains("\"severity\":\"warning\""));
    }

    #[test]
    fn test_format_prompt_default_template() {
        let diags = vec![make_diag("a.rs", "some note", 1)];
        let out = format_prompt(&diags, None, None);
        assert!(out.contains("<comment-checker>"));
        assert!(out.contains("Found 1 flagged comment"));
        assert!(out.contains("file=\"a.rs\""));
    }

    #[test]
    fn test_format_prompt_merges_consecutive_lines() {
        let diags = vec![
            make_diag("a.rs", "first line", 1),
            make_diag("a.rs", "second line", 2),
            make_diag("a.rs", "third line", 3),
        ];
        let out = format_prompt(&diags, None, None);
        assert!(out.contains("3 flagged comment(s) in 1 group(s)"));
        assert!(out.contains("line=\"1-3\""));
        assert!(out.contains("// first line\n// second line\n// third line"));
    }

    #[test]
    fn test_format_prompt_splits_non_consecutive() {
        let diags = vec![
            make_diag("a.rs", "top", 1),
            make_diag("a.rs", "bottom", 10),
        ];
        let out = format_prompt(&diags, None, None);
        assert!(out.contains("in 2 group(s)"));
        assert!(out.contains("line=\"1\""));
        assert!(out.contains("line=\"10\""));
    }

    #[test]
    fn test_format_prompt_custom_template() {
        let diags = vec![make_diag("a.rs", "x", 1)];
        let out = format_prompt(
            &diags,
            Some("COUNT={{ count }} COMMENTS={% for c in comments %}{{ c.text }}{% endfor %}"),
            None,
        );
        assert!(out.starts_with("COUNT=1"));
        assert!(out.contains("COMMENTS=// x"));
    }
}

use serde::Serialize;

use crate::checker::Diagnostic;

/// Plain-text format: one line per diagnostic.
/// Format: `file:line:col: warning[comment]: first line of raw text`
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

const DEFAULT_TEMPLATE: &str = r#"<comment-checker>
<summary>Found {{count}} flagged comment(s) that may need attention.</summary>
<flagged-comments>{{comments}}</flagged-comments>
<instruction>Review each flagged comment. If the comment is outdated, inaccurate, or unnecessary, remove or update it. If the comment is valid and intentional, add a pattern to the allowlist in .comment-checker.toml to suppress this warning.</instruction>
</comment-checker>"#;

/// Prompt format: XML block suitable for injecting into an LLM prompt.
/// `template` may contain `{{comments}}` and `{{count}}` placeholders.
pub fn format_prompt(diagnostics: &[Diagnostic], template: Option<&str>) -> String {
    if diagnostics.is_empty() {
        return String::new();
    }
    let tmpl = template.unwrap_or(DEFAULT_TEMPLATE);

    let mut comments_block = String::new();
    for d in diagnostics {
        let raw = d
            .comment
            .raw_text()
            .lines()
            .next()
            .unwrap_or("")
            .trim_end()
            .to_string();
        comments_block.push_str(&format!(
            "<comment file=\"{}\" line=\"{}\" type=\"{}\">{}</comment>\n",
            d.file,
            d.comment.span.start_line,
            d.comment.kind,
            raw,
        ));
    }
    let comments_block = comments_block.trim_end_matches('\n');

    tmpl.replace("{{count}}", &diagnostics.len().to_string())
        .replace("{{comments}}", comments_block)
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
        let out = format_prompt(&diags, None);
        assert!(out.contains("<comment-checker>"));
        assert!(out.contains("Found 1 flagged comment"));
        assert!(out.contains("file=\"a.rs\""));
    }

    #[test]
    fn test_format_prompt_custom_template() {
        let diags = vec![make_diag("a.rs", "x", 1)];
        let out = format_prompt(&diags, Some("COUNT={{count}} COMMENTS={{comments}}"));
        assert!(out.starts_with("COUNT=1"));
        assert!(out.contains("COMMENTS=<comment"));
    }
}

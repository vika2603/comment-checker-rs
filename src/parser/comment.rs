use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    pub kind: CommentKind,
    pub prefix: String,
    pub content: String,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CommentKind {
    Line,
    Block,
    Doc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

impl Comment {
    pub fn from_raw(raw: &str, kind: CommentKind, span: Span) -> Self {
        let (prefix, content) = strip_comment(raw, kind);
        Self {
            kind,
            prefix: prefix.to_string(),
            content,
            span,
        }
    }

    pub fn raw_text(&self) -> String {
        if self.prefix.is_empty() {
            self.content.clone()
        } else {
            format!(
                "{}{}{}",
                self.prefix,
                if self.content.is_empty() { "" } else { " " },
                self.content
            )
        }
    }
}

fn strip_comment(raw: &str, kind: CommentKind) -> (&str, String) {
    let trimmed = raw.trim();
    let prefixes: &[&str] = match kind {
        CommentKind::Doc => &["///", "//!", "/**", "/*!", "\"\"\"", "'''"],
        CommentKind::Line => &["//", "#", "--"],
        CommentKind::Block => &["/*"],
    };
    for prefix in prefixes {
        if let Some(after) = trimmed.strip_prefix(prefix) {
            let is_block_style = kind == CommentKind::Block
                || (kind == CommentKind::Doc && (*prefix == "/**" || *prefix == "/*!"));
            let after = if is_block_style {
                after
                    .trim_end()
                    .strip_suffix("*/")
                    .unwrap_or(after)
            } else {
                after
            };
            // Strip a single leading space from line comments to preserve intentional indentation.
            let content = if is_block_style {
                after.trim().to_string()
            } else {
                after.strip_prefix(' ').unwrap_or(after).to_string()
            };
            return (prefix, content);
        }
    }
    ("", trimmed.to_string())
}

impl fmt::Display for CommentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommentKind::Line => write!(f, "line"),
            CommentKind::Block => write!(f, "block"),
            CommentKind::Doc => write!(f, "doc"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span() -> Span {
        Span {
            start_line: 1,
            start_col: 0,
            end_line: 1,
            end_col: 10,
        }
    }

    #[test]
    fn test_strip_line_comment() {
        let c = Comment::from_raw("// hello world", CommentKind::Line, span());
        assert_eq!(c.prefix, "//");
        assert_eq!(c.content, "hello world");
    }

    #[test]
    fn test_strip_doc_comment() {
        let c = Comment::from_raw("/// doc text", CommentKind::Doc, span());
        assert_eq!(c.prefix, "///");
        assert_eq!(c.content, "doc text");
    }

    #[test]
    fn test_strip_block_comment() {
        let c = Comment::from_raw("/* block */", CommentKind::Block, span());
        assert_eq!(c.prefix, "/*");
        assert_eq!(c.content, "block");
    }

    #[test]
    fn test_strip_doc_block_comment() {
        let c = Comment::from_raw("/** doc block */", CommentKind::Doc, span());
        assert_eq!(c.prefix, "/**");
        assert_eq!(c.content, "doc block");
    }

    #[test]
    fn test_raw_text_roundtrip() {
        let c = Comment::from_raw("// hello", CommentKind::Line, span());
        assert_eq!(c.raw_text(), "// hello");
    }

    #[test]
    fn test_empty_comment() {
        let c = Comment::from_raw("//", CommentKind::Line, span());
        assert_eq!(c.prefix, "//");
        assert_eq!(c.content, "");
        assert_eq!(c.raw_text(), "//");
    }
}

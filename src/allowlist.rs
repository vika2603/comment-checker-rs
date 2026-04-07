use regex::Regex;

use crate::parser::comment::Comment;

pub struct Allowlist {
    builtin: Vec<Regex>,
    user: Vec<Regex>,
}

impl Allowlist {
    pub fn new(user_patterns: &[String]) -> Result<Self, regex::Error> {
        let builtin = builtin_patterns()
            .into_iter()
            .map(|p| Regex::new(p))
            .collect::<Result<Vec<_>, _>>()?;

        let user = user_patterns
            .iter()
            .map(|p| Regex::new(p))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { builtin, user })
    }

    /// Returns true if the comment matches any built-in or user-defined pattern.
    /// Matching is performed against the stripped comment content (prefix removed).
    pub fn is_allowed(&self, comment: &Comment) -> bool {
        let content = comment.content.trim();
        self.builtin
            .iter()
            .chain(self.user.iter())
            .any(|re| re.is_match(content))
    }
}

pub fn builtin_patterns() -> Vec<&'static str> {
    vec![
        // Linter directives
        r"(?i)^eslint-disable",
        r"(?i)^eslint-enable",
        r"(?i)noqa",
        r"(?i)^@ts-ignore",
        r"(?i)^@ts-expect-error",
        r"(?i)^@ts-nocheck",
        r"(?i)^noinspection",
        r"(?i)rubocop:\s*(enable|disable)",
        r"(?i)^pylint:\s*(enable|disable)",
        r"(?i)^flake8:",
        // Rust attributes written in comments (rare but valid)
        r"(?i)^allow\(",
        r"(?i)^deny\(",
        r"(?i)^expect\(",
        r"(?i)^warn\(",
        r"(?i)^forbid\(",
        // Compiler pragmas
        r"(?i)^pragma\b",
        r"^go:(generate|build|embed|linkname|nosplit|noinline|noescape)",
        // Type annotations
        r"(?i)^type:\s*ignore",
        r"(?i)^pyright:",
        r"(?i)^mypy:",
        // Shebangs
        r"^!/usr/bin",
        r"^!/bin",
        // Encoding declarations
        r"-\*-\s*coding:",
        // License / copyright
        r"(?i)^Copyright\b",
        r"(?i)^License\b",
        r"(?i)^SPDX-License-Identifier:",
        // BDD step keywords
        r"(?i)^given\b",
        r"(?i)^when\b",
        r"(?i)^then\b",
        // Region markers and IDE annotations
        r"(?i)^#region\b",
        r"(?i)^#endregion\b",
        r"(?i)^MARK:",
        r"(?i)^pragma\s+mark\b",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::comment::{CommentKind, Span};

    fn make_comment(content: &str) -> Comment {
        Comment {
            kind: CommentKind::Line,
            prefix: "//".to_string(),
            content: content.to_string(),
            span: Span {
                start_line: 1,
                start_col: 0,
                end_line: 1,
                end_col: 10,
            },
        }
    }

    fn allowlist() -> Allowlist {
        Allowlist::new(&[]).expect("builtin patterns must be valid")
    }

    #[test]
    fn test_eslint_disable_allowed() {
        assert!(allowlist().is_allowed(&make_comment("eslint-disable-next-line")));
    }

    #[test]
    fn test_noqa_allowed() {
        assert!(allowlist().is_allowed(&make_comment("noqa")));
    }

    #[test]
    fn test_spdx_allowed() {
        assert!(allowlist().is_allowed(&make_comment("SPDX-License-Identifier: MIT")));
    }

    #[test]
    fn test_copyright_allowed() {
        assert!(allowlist().is_allowed(&make_comment("Copyright 2024 Example Corp")));
    }

    #[test]
    fn test_region_allowed() {
        assert!(allowlist().is_allowed(&make_comment("#region Main")));
        assert!(allowlist().is_allowed(&make_comment("#endregion")));
    }

    #[test]
    fn test_regular_comment_not_allowed() {
        assert!(!allowlist().is_allowed(&make_comment("This is a regular comment")));
    }

    #[test]
    fn test_todo_not_allowed() {
        assert!(!allowlist().is_allowed(&make_comment("TODO: fix this")));
    }

    #[test]
    fn test_user_pattern() {
        let al = Allowlist::new(&["^TICKET-\\d+".to_string()]).unwrap();
        assert!(al.is_allowed(&make_comment("TICKET-1234 some note")));
        assert!(!al.is_allowed(&make_comment("random comment")));
    }
}

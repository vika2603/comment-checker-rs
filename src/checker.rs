use std::ops::Range;

use crate::allowlist::Allowlist;
use crate::parser::comment::Comment;

#[derive(Debug)]
pub struct Diagnostic {
    pub file: String,
    pub comment: Comment,
}

impl Diagnostic {
    pub fn sort_key(&self) -> (&str, usize, usize) {
        (
            &self.file,
            self.comment.span.start_line,
            self.comment.span.start_col,
        )
    }
}

pub fn check_comments(
    file_path: &str,
    comments: Vec<Comment>,
    allowlist: &Allowlist,
) -> Vec<Diagnostic> {
    comments
        .into_iter()
        .filter(|c| !allowlist.is_allowed(c))
        .map(|c| Diagnostic {
            file: file_path.to_string(),
            comment: c,
        })
        .collect()
}

/// Retain only diagnostics whose span overlaps any of `ranges`
/// (1-based, exclusive end).
pub fn filter_by_ranges(diagnostics: Vec<Diagnostic>, ranges: &[Range<usize>]) -> Vec<Diagnostic> {
    diagnostics
        .into_iter()
        .filter(|d| {
            let start_line = d.comment.span.start_line;
            let end_line = d.comment.span.end_line;
            ranges
                .iter()
                .any(|r| start_line < r.end && end_line >= r.start)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::comment::{CommentKind, Span};

    fn make_comment(content: &str, line: usize) -> Comment {
        Comment {
            kind: CommentKind::Line,
            prefix: "//".to_string(),
            content: content.to_string(),
            span: Span {
                start_line: line,
                start_col: 0,
                end_line: line,
                end_col: content.len(),
            },
        }
    }

    fn make_multiline_comment(content: &str, start_line: usize, end_line: usize) -> Comment {
        Comment {
            kind: CommentKind::Block,
            prefix: "/*".to_string(),
            content: content.to_string(),
            span: Span {
                start_line,
                start_col: 0,
                end_line,
                end_col: content.len(),
            },
        }
    }

    #[test]
    fn test_check_comments_filters_allowed() {
        let al = Allowlist::new(&[]).expect("builtin patterns valid");
        let comments = vec![
            make_comment("TODO: fix this", 1),
            make_comment("SPDX-License-Identifier: MIT", 2),
            make_comment("regular comment", 3),
        ];
        let diags = check_comments("test.rs", comments, &al);
        assert_eq!(diags.len(), 2);
        assert!(diags.iter().any(|d| d.comment.content.contains("TODO")));
        assert!(diags.iter().any(|d| d.comment.content.contains("regular")));
    }

    #[test]
    fn test_filter_by_ranges() {
        let al = Allowlist::new(&[]).expect("builtin patterns valid");
        let comments = vec![
            make_comment("comment at line 1", 1),
            make_comment("comment at line 5", 5),
            make_comment("comment at line 10", 10),
        ];
        let diags = check_comments("test.rs", comments, &al);
        let range = 2..7;
        let filtered = filter_by_ranges(diags, std::slice::from_ref(&range));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].comment.span.start_line, 5);
    }

    #[test]
    fn test_filter_by_ranges_keeps_multiline_overlap() {
        let al = Allowlist::new(&[]).expect("builtin patterns valid");
        let comments = vec![make_multiline_comment("block comment", 2, 4)];
        let diags = check_comments("test.rs", comments, &al);
        let range = 4..8;
        let filtered = filter_by_ranges(diags, std::slice::from_ref(&range));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].comment.span.start_line, 2);
        assert_eq!(filtered[0].comment.span.end_line, 4);
    }

    #[test]
    fn test_sort_key() {
        let al = Allowlist::new(&[]).expect("builtin patterns valid");
        let comments = vec![make_comment("hello", 3)];
        let diags = check_comments("a.rs", comments, &al);
        assert_eq!(diags[0].sort_key(), ("a.rs", 3, 0));
    }
}

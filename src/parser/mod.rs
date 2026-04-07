pub mod comment;
pub mod languages;

use comment::{Comment, CommentKind, Span};
use languages::Language;
use tree_sitter::Node;

/// Parse all comments from `source` for the given language.
/// Returns `None` if the source cannot be parsed.
pub fn parse_comments(source: &str, lang: Language) -> Option<Vec<Comment>> {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&lang.tree_sitter_language())
        .ok()?;

    let tree = parser.parse(source, None)?;
    let root = tree.root_node();

    let mut comments = Vec::new();
    walk_for_comments(root, source, lang, &mut comments);
    Some(comments)
}

fn walk_for_comments<'a>(
    node: Node<'a>,
    source: &str,
    lang: Language,
    out: &mut Vec<Comment>,
) {
    let type_name = node.kind();

    // All tree-sitter grammars name comment nodes with "comment" in the type.
    if type_name.contains("comment")
        && let Some(comment) = extract_comment(node, source, lang)
    {
        out.push(comment);
        return;
    }

    // Python docstrings: expression_statement containing a string literal
    // at module / class / function body level
    if lang == Language::Python
        && is_python_docstring(node, source)
        && let Some(comment) = extract_python_docstring(node, source)
    {
        out.push(comment);
        return;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk_for_comments(child, source, lang, out);
    }
}

/// Returns true if `node` is a Python docstring: a string-literal
/// `expression_statement` that is the first non-comment statement in a
/// module, class body, or function body.
fn is_python_docstring(node: Node<'_>, _source: &str) -> bool {
    if node.kind() != "expression_statement" {
        return false;
    }

    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();
    if children.len() != 1 || children[0].kind() != "string" {
        return false;
    }

    if let Some(parent) = node.parent() {
        let parent_kind = parent.kind();
        if parent_kind == "module" || parent_kind == "block" {
            let mut sibling_cursor = parent.walk();
            for sibling in parent.children(&mut sibling_cursor) {
                if sibling.kind().contains("comment") {
                    continue;
                }
                return sibling.id() == node.id();
            }
        }
    }

    false
}

fn extract_python_docstring(node: Node<'_>, source: &str) -> Option<Comment> {
    let raw = node_text(node, source);
    let span = node_to_span(node);
    Some(Comment::from_raw(raw, CommentKind::Doc, span))
}

fn extract_comment(node: Node<'_>, source: &str, _lang: Language) -> Option<Comment> {
    let raw = node_text(node, source);
    let span = node_to_span(node);
    let kind = classify_comment(raw, node.kind());
    Some(Comment::from_raw(raw, kind, span))
}

fn classify_comment(raw: &str, type_name: &str) -> CommentKind {
    let trimmed = raw.trim();

    if trimmed.starts_with("///")
        || trimmed.starts_with("//!")
        || trimmed.starts_with("/**")
        || trimmed.starts_with("/*!")
    {
        return CommentKind::Doc;
    }

    if type_name.contains("block")
        || (type_name == "comment" && trimmed.starts_with("/*"))
    {
        return CommentKind::Block;
    }

    CommentKind::Line
}

fn node_text<'a>(node: Node<'_>, source: &'a str) -> &'a str {
    let start = node.start_byte();
    let end = node.end_byte();
    source.get(start..end).unwrap_or("")
}

fn node_to_span(node: Node<'_>) -> Span {
    let start = node.start_position();
    let end = node.end_position();
    Span {
        start_line: start.row + 1,
        start_col: start.column,
        end_line: end.row + 1,
        end_col: end.column,
    }
}

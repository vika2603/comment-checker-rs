use comment_checker::parser::{parse_comments, comment::CommentKind, languages::Language};
use comment_checker::allowlist::Allowlist;

const RUST_FIXTURE: &str = include_str!("fixtures/rust.rs");

fn load_ts_language(lang: Language) -> Option<tree_sitter::Language> {
    let nvim_dir = std::env::var("HOME").ok()
        .map(|h| std::path::PathBuf::from(h).join(".local/share/nvim/site/parser"))?;
    // Leak the cache so the Library (and its symbols) are never dropped.
    // Multiple tests run in the same process; dropping Library triggers dlclose
    // which invalidates the function pointers held by tree_sitter::Language.
    let cache = Box::leak(Box::new(comment_checker::grammar::GrammarCache::new()));
    cache.get(lang, &[nvim_dir]).ok()
}

#[test]
fn test_parse_rust_fixture_finds_comments() {
    let ts_lang = load_ts_language(Language::Rust).expect("nvim rust parser required for tests");
    let comments = parse_comments(RUST_FIXTURE, Language::Rust, &ts_lang)
        .expect("parse should succeed for valid Rust");

    // The fixture has several comments; ensure we found at least the expected count
    assert!(
        comments.len() >= 9,
        "expected at least 9 comments, got {}",
        comments.len()
    );
}

#[test]
fn test_parse_rust_fixture_no_false_positives() {
    let ts_lang = load_ts_language(Language::Rust).expect("nvim rust parser required for tests");
    let comments = parse_comments(RUST_FIXTURE, Language::Rust, &ts_lang)
        .expect("parse should succeed");

    // String literals inside source code must not be returned as comments
    for c in &comments {
        assert!(
            !c.content.contains("inside string"),
            "string literal was mistakenly parsed as a comment: {:?}",
            c
        );
        assert!(
            !c.content.contains("also not a comment"),
            "raw string literal was mistakenly parsed as a comment: {:?}",
            c
        );
    }
}

#[test]
fn test_parse_rust_fixture_comment_kinds() {
    let ts_lang = load_ts_language(Language::Rust).expect("nvim rust parser required for tests");
    let comments = parse_comments(RUST_FIXTURE, Language::Rust, &ts_lang)
        .expect("parse should succeed");

    let has_line = comments.iter().any(|c| c.kind == CommentKind::Line);
    let has_doc = comments.iter().any(|c| c.kind == CommentKind::Doc);
    let has_block = comments.iter().any(|c| c.kind == CommentKind::Block);

    assert!(has_line, "expected at least one line comment");
    assert!(has_doc, "expected at least one doc comment");
    assert!(has_block, "expected at least one block comment");
}

#[test]
fn test_allowlist_against_rust_fixture() {
    let ts_lang = load_ts_language(Language::Rust).expect("nvim rust parser required for tests");
    let comments = parse_comments(RUST_FIXTURE, Language::Rust, &ts_lang)
        .expect("parse should succeed");

    let al = Allowlist::new(&[]).expect("builtin patterns valid");

    // Comments that should be allowed
    let allowed: Vec<_> = comments
        .iter()
        .filter(|c| al.is_allowed(c))
        .collect();

    // The fixture has 6 explicitly-allowed comments (eslint, noqa, SPDX, Copyright, #region, #endregion)
    assert!(
        allowed.len() >= 5,
        "expected at least 5 allowed comments, got {}; allowed: {:#?}",
        allowed.len(),
        allowed.iter().map(|c| &c.content).collect::<Vec<_>>()
    );

    // Ensure "regular comment" is not in the allowed list
    let regular_allowed = allowed
        .iter()
        .any(|c| c.content.contains("regular comment"));
    assert!(!regular_allowed, "'regular comment' should not be allowed");

    // Ensure "TODO: fix this" is not in the allowed list
    let todo_allowed = allowed
        .iter()
        .any(|c| c.content.contains("TODO"));
    assert!(!todo_allowed, "TODO comment should not be allowed");
}

use std::path::Path;

fn load_ts_language(lang: comment_checker::parser::languages::Language) -> Option<tree_sitter::Language> {
    let nvim_dir = std::env::var("HOME").ok()
        .map(|h| std::path::PathBuf::from(h).join(".local/share/nvim/site/parser"))?;
    // Leak the cache so the Library (and its symbols) are never dropped.
    // Multiple tests run in the same process; dropping Library triggers dlclose
    // which invalidates the function pointers held by tree_sitter::Language.
    let cache = Box::leak(Box::new(comment_checker::grammar::GrammarCache::new()));
    cache.get(lang, &[nvim_dir]).ok()
}

/// Returns None when the nvim parser for the fixture's language is not installed,
/// allowing the caller to skip the test gracefully.
fn try_check_fixture(fixture_name: &str) -> Option<String> {
    let fixture_path = format!("tests/fixtures/{fixture_name}");
    let source = std::fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("cannot read fixture {fixture_name}: {e}"));
    let ext = Path::new(&fixture_path)
        .extension()
        .unwrap()
        .to_str()
        .unwrap();
    let lang = comment_checker::parser::languages::Language::from_extension(ext)
        .unwrap_or_else(|| panic!("unsupported extension: {ext}"));
    let ts_lang = load_ts_language(lang)?;
    let comments = comment_checker::parser::parse_comments(&source, lang, &ts_lang)
        .unwrap_or_else(|| panic!("parse failed for {fixture_name}"));
    let allowlist = comment_checker::allowlist::Allowlist::new(&[]).unwrap();
    let diagnostics =
        comment_checker::checker::check_comments(&fixture_path, comments, &allowlist);
    Some(comment_checker::output::format_text(&diagnostics))
}

#[test]
fn test_rust_snapshot() {
    let output = try_check_fixture("rust.rs").expect("nvim rust parser required for tests");
    insta::assert_snapshot!(output);
}

#[test]
fn test_python_snapshot() {
    let output = try_check_fixture("python.py").expect("nvim python parser required for tests");
    insta::assert_snapshot!(output);
}

#[test]
fn test_javascript_snapshot() {
    let output = try_check_fixture("javascript.js").expect("nvim javascript parser required for tests");
    insta::assert_snapshot!(output);
}

#[test]
fn test_typescript_snapshot() {
    let output = try_check_fixture("typescript.ts").expect("nvim typescript parser required for tests");
    insta::assert_snapshot!(output);
}

#[test]
fn test_go_snapshot() {
    let output = try_check_fixture("go.go").expect("nvim go parser required for tests");
    insta::assert_snapshot!(output);
}

#[test]
fn test_java_snapshot() {
    if let Some(output) = try_check_fixture("java.java") {
        insta::assert_snapshot!(output);
    }
}

#[test]
fn test_c_snapshot() {
    let output = try_check_fixture("c_test.c").expect("nvim c parser required for tests");
    insta::assert_snapshot!(output);
}

#[test]
fn test_cpp_snapshot() {
    if let Some(output) = try_check_fixture("cpp_test.cpp") {
        insta::assert_snapshot!(output);
    }
}

#[test]
fn test_ruby_snapshot() {
    if let Some(output) = try_check_fixture("ruby.rb") {
        insta::assert_snapshot!(output);
    }
}

#[test]
fn test_shell_snapshot() {
    let output = try_check_fixture("shell.sh").expect("nvim bash parser required for tests");
    insta::assert_snapshot!(output);
}

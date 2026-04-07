use std::path::Path;

fn check_fixture(fixture_name: &str) -> String {
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
    let comments = comment_checker::parser::parse_comments(&source, lang)
        .unwrap_or_else(|| panic!("parse failed for {fixture_name}"));
    let allowlist = comment_checker::allowlist::Allowlist::new(&[]).unwrap();
    let diagnostics =
        comment_checker::checker::check_comments(&fixture_path, comments, &allowlist);
    comment_checker::output::format_text(&diagnostics)
}

#[test]
fn test_rust_snapshot() {
    insta::assert_snapshot!(check_fixture("rust.rs"));
}

#[test]
fn test_python_snapshot() {
    insta::assert_snapshot!(check_fixture("python.py"));
}

#[test]
fn test_javascript_snapshot() {
    insta::assert_snapshot!(check_fixture("javascript.js"));
}

#[test]
fn test_typescript_snapshot() {
    insta::assert_snapshot!(check_fixture("typescript.ts"));
}

#[test]
fn test_go_snapshot() {
    insta::assert_snapshot!(check_fixture("go.go"));
}

#[test]
fn test_java_snapshot() {
    insta::assert_snapshot!(check_fixture("java.java"));
}

#[test]
fn test_c_snapshot() {
    insta::assert_snapshot!(check_fixture("c_test.c"));
}

#[test]
fn test_cpp_snapshot() {
    insta::assert_snapshot!(check_fixture("cpp_test.cpp"));
}

#[test]
fn test_ruby_snapshot() {
    insta::assert_snapshot!(check_fixture("ruby.rb"));
}

#[test]
fn test_shell_snapshot() {
    insta::assert_snapshot!(check_fixture("shell.sh"));
}

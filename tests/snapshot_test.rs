mod common;

use std::path::Path;

use common::load_ts_language;

/// Returns None when the parser for the fixture's language is not installed,
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
    let Some(ts_lang) = load_ts_language(lang) else {
        eprintln!("skipping {fixture_name}: no cached grammar for {lang:?}");
        return None;
    };
    let comments = comment_checker::parser::parse_comments(&source, lang, &ts_lang)
        .unwrap_or_else(|| panic!("parse failed for {fixture_name}"));
    let allowlist = comment_checker::allowlist::Allowlist::new(&[]).unwrap();
    let diagnostics = comment_checker::checker::check_comments(&fixture_path, comments, &allowlist);
    Some(comment_checker::output::format_text(&diagnostics))
}

#[test]
fn test_rust_snapshot() {
    let output = try_check_fixture("rust.rs")
        .expect("rust grammar must be available -- canary that catches missing-parser regressions");
    insta::assert_snapshot!(output);
}

#[test]
fn test_python_snapshot() {
    if let Some(output) = try_check_fixture("python.py") {
        insta::assert_snapshot!(output);
    }
}

#[test]
fn test_javascript_snapshot() {
    if let Some(output) = try_check_fixture("javascript.js") {
        insta::assert_snapshot!(output);
    }
}

#[test]
fn test_typescript_snapshot() {
    if let Some(output) = try_check_fixture("typescript.ts") {
        insta::assert_snapshot!(output);
    }
}

#[test]
fn test_go_snapshot() {
    if let Some(output) = try_check_fixture("go.go") {
        insta::assert_snapshot!(output);
    }
}

#[test]
fn test_java_snapshot() {
    if let Some(output) = try_check_fixture("java.java") {
        insta::assert_snapshot!(output);
    }
}

#[test]
fn test_c_snapshot() {
    if let Some(output) = try_check_fixture("c_test.c") {
        insta::assert_snapshot!(output);
    }
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
    if let Some(output) = try_check_fixture("shell.sh") {
        insta::assert_snapshot!(output);
    }
}

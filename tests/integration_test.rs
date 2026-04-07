use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;
use std::io::Write;

fn bin() -> Command {
    Command::cargo_bin("comment-checker").expect("binary must build")
}

fn fixture(name: &str) -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.join("tests").join("fixtures").join(name)
}

/// Canonicalized string path for a fixture file (used in hook JSON).
fn fixture_str(name: &str) -> String {
    fixture(name)
        .canonicalize()
        .expect("fixture path must exist")
        .to_string_lossy()
        .into_owned()
}

// ---------------------------------------------------------------------------
// CLI mode
// ---------------------------------------------------------------------------

#[test]
fn test_no_args_exits_2() {
    bin()
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Either --hook or at least one path is required"));
}

#[test]
fn test_check_fixture_exits_1_with_warning() {
    bin()
        .arg(fixture("rust.rs"))
        .assert()
        .code(1)
        .stdout(predicate::str::contains("warning[comment]"));
}

#[test]
fn test_quiet_mode_exits_1_empty_stdout() {
    bin()
        .arg("--quiet")
        .arg(fixture("rust.rs"))
        .assert()
        .code(1)
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_format_jsonl_output() {
    bin()
        .args(["--format", "jsonl"])
        .arg(fixture("rust.rs"))
        .assert()
        .code(1)
        .stdout(predicate::str::contains(r#""severity":"warning""#));
}

#[test]
fn test_format_jsonl_contains_file_field() {
    bin()
        .args(["--format", "jsonl"])
        .arg(fixture("javascript.js"))
        .assert()
        .code(1)
        .stdout(predicate::str::contains(r#""file":"#));
}

#[test]
fn test_prompt_conflicts_with_format_exits_2() {
    bin()
        .args(["--prompt", "{{comments}}", "--format", "jsonl"])
        .arg(fixture("rust.rs"))
        .assert()
        .code(2);
}

#[test]
fn test_check_directory_of_fixtures() {
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");
    bin()
        .arg(fixtures_dir)
        .assert()
        .code(1)
        .stdout(predicate::str::contains("warning[comment]"));
}

// ---------------------------------------------------------------------------
// Hook mode
// ---------------------------------------------------------------------------

fn hook_json_write(file_path: &str) -> String {
    serde_json::json!({
        "tool_name": "Write",
        "tool_input": {
            "file_path": file_path,
            "content": ""
        }
    })
    .to_string()
}

#[test]
fn test_hook_write_tool_exits_1_with_prompt_output() {
    let path = fixture_str("rust.rs");
    let json = hook_json_write(&path);

    bin()
        .arg("--hook")
        .write_stdin(json)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("<comment-checker>"));
}

#[test]
fn test_hook_invalid_json_exits_0() {
    bin()
        .arg("--hook")
        .write_stdin("{not valid json}")
        .assert()
        .code(0);
}

#[test]
fn test_hook_unsupported_language_exits_0() {
    let json = serde_json::json!({
        "tool_name": "Write",
        "tool_input": {
            "file_path": "/tmp/somefile.txt",
            "content": "hello"
        }
    })
    .to_string();

    bin()
        .arg("--hook")
        .write_stdin(json)
        .assert()
        .code(0);
}

#[test]
fn test_hook_empty_stdin_exits_0() {
    bin()
        .arg("--hook")
        .write_stdin("")
        .assert()
        .code(0);
}

#[test]
fn test_hook_with_python_fixture_exits_1() {
    let path = fixture_str("python.py");
    let json = hook_json_write(&path);

    bin()
        .arg("--hook")
        .write_stdin(json)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("<comment-checker>"));
}

#[test]
fn test_hook_quiet_exits_1_empty_stdout() {
    let path = fixture_str("rust.rs");
    let json = hook_json_write(&path);

    bin()
        .arg("--hook")
        .arg("--quiet")
        .write_stdin(json)
        .assert()
        .code(1)
        .stdout(predicate::str::is_empty());
}

// ---------------------------------------------------------------------------
// Config file loading
// ---------------------------------------------------------------------------

#[test]
fn test_config_allowlist_suppresses_matching_comments() {
    // Create a temp dir with a source file and a config that allowlists
    // the pattern matching comments in that file.
    let tmp = tempfile::tempdir().expect("temp dir must be created");

    // Write a Rust source file with one comment that would normally be flagged
    let src_path = tmp.path().join("sample.rs");
    std::fs::write(
        &src_path,
        "// TODO: fix this later\n// ALLOWED-COMMENT: should be suppressed\n",
    )
    .expect("write source file");

    // Write config that allowlists "ALLOWED-COMMENT"
    let config_path = tmp.path().join(".comment-checker.toml");
    let mut config_file = std::fs::File::create(&config_path).expect("create config file");
    writeln!(config_file, r#"allowlist = ["ALLOWED-COMMENT"]"#)
        .expect("write config file");

    // Running without config: both comments flagged (exit 1)
    bin()
        .arg("--format")
        .arg("jsonl")
        .arg(&src_path)
        .assert()
        .code(1)
        .stdout(predicate::str::contains("ALLOWED-COMMENT"));

    // Running with config: ALLOWED-COMMENT suppressed, only TODO flagged
    let out = bin()
        .arg("--config")
        .arg(&config_path)
        .arg("--format")
        .arg("jsonl")
        .arg(&src_path)
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8_lossy(&out);
    assert!(
        stdout.contains("TODO"),
        "TODO comment should still be flagged, got: {stdout}"
    );
    assert!(
        !stdout.contains("ALLOWED-COMMENT"),
        "ALLOWED-COMMENT should be suppressed by allowlist, got: {stdout}"
    );
}

#[test]
fn test_config_allowlist_full_suppression_exits_0() {
    let tmp = tempfile::tempdir().expect("temp dir must be created");

    let src_path = tmp.path().join("clean.rs");
    std::fs::write(&src_path, "// SUPPRESSED: everything here\n")
        .expect("write source file");

    let config_path = tmp.path().join(".comment-checker.toml");
    let mut config_file = std::fs::File::create(&config_path).expect("create config file");
    writeln!(config_file, r#"allowlist = ["SUPPRESSED"]"#)
        .expect("write config file");

    // With config, the single comment is suppressed -> exit 0
    bin()
        .arg("--config")
        .arg(&config_path)
        .arg(&src_path)
        .assert()
        .code(0)
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_hook_edit_tool_filters_by_range() {
    // For an Edit tool with new_string, only lines near the changed region
    // should be reported. We pick a fixture and use "regular comment" as the
    // changed string to target a specific line.
    let path = fixture_str("rust.rs");
    let source = std::fs::read_to_string(fixture("rust.rs")).unwrap();

    // Find the "regular comment" text so the range filter hits it
    assert!(
        source.contains("regular comment"),
        "fixture must contain 'regular comment'"
    );

    let json = serde_json::json!({
        "tool_name": "Edit",
        "tool_input": {
            "file_path": path,
            "old_string": "",
            "new_string": "// This is a regular comment - should be FLAGGED"
        }
    })
    .to_string();

    bin()
        .arg("--hook")
        .write_stdin(json)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("<comment-checker>"));
}

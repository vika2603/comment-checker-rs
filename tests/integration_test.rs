use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use std::path::PathBuf;

fn bin() -> Command {
    Command::cargo_bin("comment-checker").expect("binary must build")
}

fn fixture(name: &str) -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.join("tests").join("fixtures").join(name)
}

fn fixture_str(name: &str) -> String {
    fixture(name)
        .canonicalize()
        .expect("fixture path must exist")
        .to_string_lossy()
        .into_owned()
}

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
fn test_hook_write_tool_exits_2_with_stderr_output() {
    let path = fixture_str("rust.rs");
    let json = hook_json_write(&path);

    bin()
        .write_stdin(json)
        .assert()
        .code(2)
        .stderr(predicate::str::contains("<comment-checker>"));
}

#[test]
fn test_hook_invalid_json_exits_0() {
    bin().write_stdin("{not valid json}").assert().code(0);
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

    bin().write_stdin(json).assert().code(0);
}

#[test]
fn test_hook_empty_stdin_exits_0() {
    bin().write_stdin("").assert().code(0);
}

#[test]
fn test_hook_with_python_fixture_exits_2_with_stderr() {
    let path = fixture_str("python.py");
    let json = hook_json_write(&path);

    bin()
        .write_stdin(json)
        .assert()
        .code(2)
        .stderr(predicate::str::contains("<comment-checker>"));
}

#[test]
fn test_hook_quiet_exits_2_empty_stderr() {
    let path = fixture_str("rust.rs");
    let json = hook_json_write(&path);

    bin().arg("--quiet").write_stdin(json).assert().code(2);
}

#[test]
fn test_config_allowlist_suppresses_via_hook() {
    let tmp = tempfile::tempdir().expect("temp dir must be created");

    let src_path = tmp.path().join("sample.rs");
    std::fs::write(
        &src_path,
        "// TODO: fix this later\n// ALLOWED-COMMENT: should be suppressed\n",
    )
    .expect("write source file");

    let config_path = tmp.path().join(".comment-checker.toml");
    let mut config_file = std::fs::File::create(&config_path).expect("create config file");
    writeln!(config_file, r#"allowlist = ["ALLOWED-COMMENT"]"#).expect("write config file");

    let path_str = src_path
        .canonicalize()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let json = hook_json_write(&path_str);

    let output = bin()
        .arg("--config")
        .arg(&config_path)
        .write_stdin(json)
        .assert()
        .code(2)
        .get_output()
        .stderr
        .clone();

    let stderr = String::from_utf8_lossy(&output);
    assert!(
        stderr.contains("TODO"),
        "TODO comment should still be flagged, got: {stderr}"
    );
    assert!(
        !stderr.contains("ALLOWED-COMMENT"),
        "ALLOWED-COMMENT should be suppressed by allowlist, got: {stderr}"
    );
}

#[test]
fn test_config_allowlist_full_suppression_exits_0() {
    let tmp = tempfile::tempdir().expect("temp dir must be created");

    let src_path = tmp.path().join("clean.rs");
    std::fs::write(&src_path, "// SUPPRESSED: everything here\n").expect("write source file");

    let config_path = tmp.path().join(".comment-checker.toml");
    let mut config_file = std::fs::File::create(&config_path).expect("create config file");
    writeln!(config_file, r#"allowlist = ["SUPPRESSED"]"#).expect("write config file");

    let path_str = src_path
        .canonicalize()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let json = hook_json_write(&path_str);

    bin()
        .arg("--config")
        .arg(&config_path)
        .write_stdin(json)
        .assert()
        .code(0);
}

#[test]
fn test_hook_edit_tool_filters_by_range() {
    let path = fixture_str("rust.rs");
    let source = std::fs::read_to_string(fixture("rust.rs")).unwrap();
    assert!(
        source.contains("regular comment"),
        "fixture must contain 'regular comment'"
    );
    assert!(
        source.contains("TODO: fix this"),
        "fixture must contain a far-away TODO to test range exclusion"
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

    let output = bin()
        .write_stdin(json)
        .assert()
        .code(2)
        .stderr(predicate::str::contains("<comment-checker>"))
        .get_output()
        .stderr
        .clone();
    let stderr = String::from_utf8_lossy(&output);

    assert!(
        stderr.contains("regular comment"),
        "in-range comment should be flagged, got: {stderr}"
    );
    assert!(
        !stderr.contains("TODO: fix this"),
        "far-away TODO at line 19 should be filtered out, got: {stderr}"
    );
    assert!(
        !stderr.contains("inline comment"),
        "far-away inline comment at line 16 should be filtered out, got: {stderr}"
    );
}

#[test]
fn test_hook_multi_edit_unions_ranges_and_excludes_gap() {
    let path = fixture_str("rust.rs");
    let source = std::fs::read_to_string(fixture("rust.rs")).unwrap();
    assert!(source.contains("// This is a regular comment"));
    assert!(source.contains("// TODO: fix this"));
    assert!(source.contains("Doc block comment"));

    let json = serde_json::json!({
        "tool_name": "MultiEdit",
        "tool_input": {
            "file_path": path,
            "edits": [
                {"old_string": "", "new_string": "// This is a regular comment - should be FLAGGED"},
                {"old_string": "", "new_string": "// TODO: fix this -- should be FLAGGED (not in allowlist)"}
            ]
        }
    })
    .to_string();

    let output = bin()
        .write_stdin(json)
        .assert()
        .code(2)
        .stderr(predicate::str::contains("<comment-checker>"))
        .get_output()
        .stderr
        .clone();
    let stderr = String::from_utf8_lossy(&output);

    assert!(
        stderr.contains("regular comment"),
        "first-edit region must be flagged, got: {stderr}"
    );
    assert!(
        stderr.contains("TODO"),
        "second-edit region must be flagged, got: {stderr}"
    );
    assert!(
        !stderr.contains("Doc block comment"),
        "line 5 sits in the gap between ranges and must not be flagged, got: {stderr}"
    );
}

#[test]
fn test_init_codex_installs_create_matcher() {
    let tmp = tempfile::tempdir().expect("temp dir must be created");

    bin()
        .arg("init")
        .arg("codex")
        .env("HOME", tmp.path())
        .assert()
        .success();

    let hooks_path = tmp.path().join(".codex").join("hooks.json");
    let hooks = std::fs::read_to_string(&hooks_path).expect("hooks file must be created");

    assert!(
        hooks.contains("Write|Create|Edit|MultiEdit"),
        "hook matcher should include Create, got: {hooks}"
    );
    assert!(
        hooks.contains("\"command\": \"comment-checker\""),
        "hook should invoke comment-checker, got: {hooks}"
    );
}

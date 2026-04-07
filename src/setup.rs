use std::path::PathBuf;

use crate::cli::Target;

const HOOK_ENTRY: &str = r#"{
        "matcher": "Write|Edit|MultiEdit",
        "hooks": [
          {
            "type": "command",
            "command": "comment-checker --hook"
          }
        ]
      }"#;

pub fn init(target: &Target) -> Result<(), String> {
    let path = settings_path(target)?;
    let display = path.display().to_string();

    let mut root = read_or_create(&path)?;

    let hooks = root
        .as_object_mut()
        .ok_or("settings file is not a JSON object")?
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));

    let post = hooks
        .as_object_mut()
        .ok_or("hooks field is not a JSON object")?
        .entry("PostToolUse")
        .or_insert_with(|| serde_json::json!([]));

    let arr = post
        .as_array_mut()
        .ok_or("PostToolUse field is not an array")?;

    if already_installed(arr) {
        eprintln!("comment-checker hook already installed in {display}");
        return Ok(());
    }

    let entry: serde_json::Value =
        serde_json::from_str(HOOK_ENTRY).expect("built-in hook JSON must be valid");
    arr.push(entry);

    write_pretty(&path, &root)?;
    eprintln!("comment-checker hook installed in {display}");
    Ok(())
}

pub fn uninstall(target: &Target) -> Result<(), String> {
    let path = settings_path(target)?;
    let display = path.display().to_string();

    if !path.exists() {
        eprintln!("no settings file found at {display}, nothing to uninstall");
        return Ok(());
    }

    let mut root = read_or_create(&path)?;

    let Some(hooks) = root.get_mut("hooks") else {
        eprintln!("no hooks configured in {display}, nothing to uninstall");
        return Ok(());
    };
    let Some(post) = hooks.get_mut("PostToolUse") else {
        eprintln!("no PostToolUse hooks in {display}, nothing to uninstall");
        return Ok(());
    };
    let Some(arr) = post.as_array_mut() else {
        return Ok(());
    };

    let before = arr.len();
    arr.retain(|entry| !is_comment_checker_entry(entry));
    let after = arr.len();

    if before == after {
        eprintln!("comment-checker hook not found in {display}, nothing to uninstall");
        return Ok(());
    }

    write_pretty(&path, &root)?;
    eprintln!("comment-checker hook removed from {display}");
    Ok(())
}

fn settings_path(target: &Target) -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
    Ok(match target {
        Target::Claude => PathBuf::from(&home).join(".claude").join("settings.json"),
        Target::Codex => PathBuf::from(&home).join(".codex").join("hooks.json"),
    })
}

fn read_or_create(path: &PathBuf) -> Result<serde_json::Value, String> {
    if path.exists() {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let content = content.trim();
        if content.is_empty() || content == "{}" {
            return Ok(serde_json::json!({}));
        }
        serde_json::from_str(content)
            .map_err(|e| format!("cannot parse {}: {e}", path.display()))
    } else {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
        }
        Ok(serde_json::json!({}))
    }
}

fn write_pretty(path: &PathBuf, value: &serde_json::Value) -> Result<(), String> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| format!("cannot serialize JSON: {e}"))?;
    std::fs::write(path, json + "\n")
        .map_err(|e| format!("cannot write {}: {e}", path.display()))
}

fn already_installed(arr: &[serde_json::Value]) -> bool {
    arr.iter().any(is_comment_checker_entry)
}

fn is_comment_checker_entry(entry: &serde_json::Value) -> bool {
    if let Some(hooks) = entry.get("hooks").and_then(|h| h.as_array()) {
        hooks.iter().any(|h| {
            h.get("command")
                .and_then(|c| c.as_str())
                .is_some_and(|c| c.contains("comment-checker"))
        })
    } else {
        false
    }
}

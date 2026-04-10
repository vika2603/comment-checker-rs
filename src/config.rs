use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct ParserConfig {
    pub path: Option<PathBuf>,
    #[serde(default = "default_true")]
    pub auto_download: bool,
}

fn default_true() -> bool {
    true
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            path: None,
            auto_download: true,
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub allowlist: Vec<String>,
    #[serde(default)]
    pub instruction: Option<String>,
    #[serde(default)]
    pub parsers: ParserConfig,
}

pub fn load_config(explicit_path: Option<&Path>, start_dir: &Path) -> Result<Config> {
    let global = load_global_config();
    let project = load_project_config(explicit_path, start_dir)?;

    match project {
        Some(proj) => Ok(merge(global, proj)),
        None => Ok(global),
    }
}

fn load_global_config() -> Config {
    xdg_config_dir()
        .map(|dir| dir.join("comment-checker").join("config.toml"))
        .filter(|p| p.exists())
        .and_then(|p| read_config(&p).ok())
        .unwrap_or_default()
}

fn load_project_config(explicit_path: Option<&Path>, start_dir: &Path) -> Result<Option<Config>> {
    if let Some(path) = explicit_path {
        return read_config(path).map(Some);
    }

    let mut dir = start_dir.to_path_buf();
    loop {
        let candidate = dir.join(".comment-checker.toml");
        if candidate.exists() {
            return read_config(&candidate).map(Some);
        }
        match dir.parent() {
            Some(parent) => dir = parent.to_path_buf(),
            None => break,
        }
    }

    Ok(None)
}

fn merge(global: Config, project: Config) -> Config {
    let mut allowlist = global.allowlist;
    allowlist.extend(project.allowlist);

    Config {
        allowlist,
        instruction: project.instruction.or(global.instruction),
        parsers: ParserConfig {
            path: project.parsers.path.or(global.parsers.path),
            auto_download: project.parsers.auto_download,
        },
    }
}

fn read_config(path: &Path) -> Result<Config> {
    let raw = std::fs::read_to_string(path)?;
    toml::from_str(&raw).map_err(|source| Error::Config {
        path: path.to_path_buf(),
        source,
    })
}

fn xdg_config_dir() -> Option<PathBuf> {
    let xdg = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .filter(|v| !v.is_empty());
    if let Some(xdg) = xdg {
        Some(PathBuf::from(xdg))
    } else {
        std::env::var("HOME")
            .ok()
            .filter(|v| !v.is_empty())
            .map(|h| PathBuf::from(h).join(".config"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(allowlist: &[&str], instruction: Option<&str>, path: Option<&str>) -> Config {
        Config {
            allowlist: allowlist.iter().map(|s| s.to_string()).collect(),
            instruction: instruction.map(|s| s.to_string()),
            parsers: ParserConfig {
                path: path.map(PathBuf::from),
                auto_download: true,
            },
        }
    }

    #[test]
    fn test_merge_appends_allowlists() {
        let merged = merge(cfg(&["A"], None, None), cfg(&["B", "C"], None, None));
        assert_eq!(merged.allowlist, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_merge_project_instruction_overrides_global() {
        let merged = merge(cfg(&[], Some("global"), None), cfg(&[], Some("project"), None));
        assert_eq!(merged.instruction.as_deref(), Some("project"));
    }

    #[test]
    fn test_merge_falls_back_to_global_instruction() {
        let merged = merge(cfg(&[], Some("global"), None), cfg(&[], None, None));
        assert_eq!(merged.instruction.as_deref(), Some("global"));
    }

    #[test]
    fn test_merge_project_parser_path_wins() {
        let merged = merge(
            cfg(&[], None, Some("/global/parsers")),
            cfg(&[], None, Some("/project/parsers")),
        );
        assert_eq!(
            merged.parsers.path.as_deref(),
            Some(Path::new("/project/parsers"))
        );
    }

    #[test]
    fn test_merge_falls_back_to_global_parser_path() {
        let merged = merge(
            cfg(&[], None, Some("/global/parsers")),
            cfg(&[], None, None),
        );
        assert_eq!(
            merged.parsers.path.as_deref(),
            Some(Path::new("/global/parsers"))
        );
    }

    #[test]
    fn test_merge_auto_download_uses_project_value() {
        let mut global = cfg(&[], None, None);
        global.parsers.auto_download = true;
        let mut project = cfg(&[], None, None);
        project.parsers.auto_download = false;
        let merged = merge(global, project);
        assert!(!merged.parsers.auto_download);
    }
}

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
        parsers: if project.parsers.path.is_some() {
            project.parsers
        } else {
            ParserConfig {
                path: global.parsers.path.or(project.parsers.path),
                auto_download: project.parsers.auto_download,
            }
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
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        Some(PathBuf::from(xdg))
    } else {
        std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".config"))
    }
}

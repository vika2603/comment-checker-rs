use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub allowlist: Vec<String>,
    #[serde(default)]
    pub languages: Vec<String>,
}

/// Load config from the first location that exists:
/// 1. Explicit `--config` path (error if it doesn't exist)
/// 2. Walk up from `start_dir` looking for `.comment-checker.toml`
/// 3. XDG_CONFIG_HOME/comment-checker/config.toml (fallback ~/.config/...)
/// 4. Built-in defaults (empty Config)
pub fn load_config(explicit_path: Option<&Path>, start_dir: &Path) -> Result<Config> {
    if let Some(path) = explicit_path {
        return read_config(path);
    }

    let mut dir = start_dir.to_path_buf();
    loop {
        let candidate = dir.join(".comment-checker.toml");
        if candidate.exists() {
            return read_config(&candidate);
        }
        match dir.parent() {
            Some(parent) => dir = parent.to_path_buf(),
            None => break,
        }
    }

    if let Some(xdg_config) = xdg_config_dir() {
        let xdg_candidate = xdg_config.join("comment-checker").join("config.toml");
        if xdg_candidate.exists() {
            return read_config(&xdg_candidate);
        }
    }

    Ok(Config::default())
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

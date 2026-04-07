use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Config parse error in {path}: {source}")]
    Config {
        path: PathBuf,
        source: toml::de::Error,
    },

    #[error("Invalid hook JSON: {0}")]
    HookJson(#[from] serde_json::Error),

    #[error("Regex pattern error: {0}")]
    Regex(#[from] regex::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

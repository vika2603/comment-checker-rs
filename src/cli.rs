use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "comment-checker", version, about = "Check code comments using tree-sitter AST parsing")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg()]
    pub paths: Vec<PathBuf>,

    #[arg(long)]
    pub hook: bool,

    #[arg(long)]
    pub config: Option<PathBuf>,

    #[arg(long, conflicts_with = "format")]
    pub prompt: Option<String>,

    #[arg(long, default_value = "text")]
    pub format: OutputFormat,

    #[arg(long, default_value = "1048576")]
    pub max_file_size: u64,

    #[arg(short, long)]
    pub quiet: bool,

    #[arg(short, long)]
    pub verbose: bool,

    /// Treat grammar load failures as errors (exit 2)
    #[arg(long)]
    pub strict: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Init {
        #[arg(value_enum)]
        target: Target,
    },
    Uninstall {
        #[arg(value_enum)]
        target: Target,
    },
    /// Download tree-sitter grammar .so files for offline use
    FetchParsers {
        /// Grammar names to fetch (default: all supported)
        #[arg()]
        languages: Vec<String>,
    },
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum Target {
    Claude,
    Codex,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Jsonl,
}

impl Cli {
    pub fn validate(&self) -> std::result::Result<(), String> {
        if self.command.is_some() {
            return Ok(());
        }
        if !self.hook && self.paths.is_empty() {
            return Err("Either --hook, a subcommand, or at least one path is required".to_string());
        }
        if self.hook && !self.paths.is_empty() {
            return Err("Cannot use --hook with path arguments".to_string());
        }
        Ok(())
    }
}

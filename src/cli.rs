use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "comment-checker", version, about = "Check code comments using tree-sitter AST parsing")]
pub struct Cli {
    /// Files or directories to check
    #[arg()]
    pub paths: Vec<PathBuf>,

    /// Read Claude Code hook JSON from stdin
    #[arg(long)]
    pub hook: bool,

    /// Path to config file
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Custom output template (use {{comments}} and {{count}} placeholders)
    #[arg(long, conflicts_with = "format")]
    pub prompt: Option<String>,

    /// Output format: text (default), jsonl
    #[arg(long, default_value = "text")]
    pub format: OutputFormat,

    /// Skip files larger than this (bytes)
    #[arg(long, default_value = "1048576")]
    pub max_file_size: u64,

    /// Suppress all output, only set exit code
    #[arg(short, long)]
    pub quiet: bool,

    /// Show parsed files, skipped files, config details
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Text,
    Jsonl,
}

impl Cli {
    pub fn validate(&self) -> std::result::Result<(), String> {
        if !self.hook && self.paths.is_empty() {
            return Err("Either --hook or at least one path is required".to_string());
        }
        if self.hook && !self.paths.is_empty() {
            return Err("Cannot use --hook with path arguments".to_string());
        }
        Ok(())
    }
}

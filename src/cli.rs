use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "comment-checker", version, about = "Check code comments using tree-sitter AST parsing")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

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

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Install comment-checker hook for an AI coding tool
    Init {
        /// Target tool: claude, codex
        #[arg(value_enum)]
        target: Target,
    },
    /// Uninstall comment-checker hook from an AI coding tool
    Uninstall {
        /// Target tool: claude, codex
        #[arg(value_enum)]
        target: Target,
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

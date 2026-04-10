use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "comment-checker",
    version,
    about = "Check code comments using tree-sitter AST parsing"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Path to a custom TOML config file (overrides the default lookup).
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Override the instruction prompt sent to the reviewer.
    #[arg(long)]
    pub prompt: Option<String>,

    /// Suppress non-essential output.
    #[arg(short, long)]
    pub quiet: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Install the comment-checker hook into a host tool's settings.
    Init {
        #[arg(value_enum)]
        target: Target,
    },
    /// Remove a previously installed comment-checker hook.
    Uninstall {
        #[arg(value_enum)]
        target: Target,
    },
    /// Download tree-sitter parser grammars for the given languages.
    FetchParsers {
        /// Language identifiers to fetch (e.g. `rust`, `python`). Empty = all supported.
        #[arg()]
        languages: Vec<String>,
    },
}

/// Host tool that comment-checker can install a hook into.
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum Target {
    Claude,
    Codex,
}

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "comment-checker", version, about = "Check code comments using tree-sitter AST parsing")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg(long)]
    pub config: Option<PathBuf>,

    #[arg(long)]
    pub prompt: Option<String>,

    #[arg(short, long)]
    pub quiet: bool,
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
    FetchParsers {
        #[arg()]
        languages: Vec<String>,
    },
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum Target {
    Claude,
    Codex,
}

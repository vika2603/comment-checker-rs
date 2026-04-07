mod cli;
mod error;

use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = cli::Cli::parse();

    if let Err(msg) = cli.validate() {
        eprintln!("error: {msg}");
        return ExitCode::from(2);
    }

    // TODO: dispatch to hook or cli mode - implemented in Task 8
    ExitCode::SUCCESS
}

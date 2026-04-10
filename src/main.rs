mod cli;
mod setup;

use std::io::Read;
use std::process::ExitCode;

use clap::Parser;

use comment_checker::allowlist::Allowlist;
use comment_checker::checker::{check_comments, filter_by_ranges};
use comment_checker::config::load_config;
use comment_checker::error::Result;
use comment_checker::grammar::GrammarCache;
use comment_checker::input::hook::parse_hook_input;
use comment_checker::output::format_prompt;
use comment_checker::parser::{languages::Language, parse_comments};

use cli::{Cli, Command};

fn main() -> ExitCode {
    let args = Cli::parse();

    if let Some(ref cmd) = args.command {
        return run_subcommand(|| match cmd {
            Command::Init { target } => setup::init(target),
            Command::Uninstall { target } => setup::uninstall(target),
            Command::FetchParsers { languages } => fetch_parsers(&args, languages),
        });
    }

    match run(&args) {
        Ok(has_diagnostics) => {
            if has_diagnostics {
                ExitCode::from(2)
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("comment-checker: {e}");
            ExitCode::SUCCESS
        }
    }
}

fn run_subcommand(f: impl FnOnce() -> std::result::Result<(), String>) -> ExitCode {
    match f() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("comment-checker: {e}");
            ExitCode::from(2)
        }
    }
}

fn run(args: &Cli) -> Result<bool> {
    let start_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let config = load_config(args.config.as_deref(), &start_dir)?;
    let allowlist = Allowlist::new(&config.allowlist)?;
    let mut grammar_cache = GrammarCache::new();

    let mut stdin_data = String::new();
    std::io::stdin()
        .read_to_string(&mut stdin_data)
        .unwrap_or(0);

    let stdin_data = stdin_data.trim();
    if stdin_data.is_empty() {
        eprintln!("comment-checker: no hook JSON on stdin, skipping");
        return Ok(false);
    }

    let target = match parse_hook_input(stdin_data) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("comment-checker: invalid hook JSON: {e}");
            return Ok(false);
        }
    };

    let Some(language) = Language::from_path(&target.file_path) else {
        return Ok(false);
    };

    let ts_language = match grammar_cache.resolve(language, &config.parsers) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("comment-checker: {e}");
            return Ok(false);
        }
    };

    let source = match std::fs::read_to_string(&target.file_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "comment-checker: cannot read {}: {e}",
                target.file_path.display()
            );
            return Ok(false);
        }
    };

    let comments = match parse_comments(&source, language, &ts_language) {
        Some(c) => c,
        None => {
            eprintln!(
                "comment-checker: tree-sitter parse failed for {}, skipping",
                target.file_path.display()
            );
            return Ok(false);
        }
    };

    let file_str = target.file_path.to_string_lossy().into_owned();
    let mut diagnostics = check_comments(&file_str, comments, &allowlist);

    if let Some(ranges) = target.changed_ranges {
        diagnostics = filter_by_ranges(diagnostics, &ranges);
    }

    diagnostics.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));

    let has_diagnostics = !diagnostics.is_empty();

    if !args.quiet {
        let output = format_prompt(
            &diagnostics,
            args.prompt.as_deref(),
            config.instruction.as_deref(),
        );
        if !output.is_empty() {
            eprint!("{output}");
        }
    }

    Ok(has_diagnostics)
}

fn fetch_parsers(args: &Cli, languages: &[String]) -> std::result::Result<(), String> {
    let start_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let config = load_config(args.config.as_deref(), &start_dir).map_err(|e| format!("{e}"))?;

    let cache_dir = config
        .parsers
        .path
        .clone()
        .or_else(comment_checker::grammar::grammar_cache_dir)
        .ok_or_else(|| {
            "cannot determine cache directory (set parsers.path or HOME)".to_string()
        })?;

    let names: Vec<&str> = if languages.is_empty() {
        Language::all_grammar_names().to_vec()
    } else {
        languages.iter().map(|s| s.as_str()).collect()
    };

    let mut errors = Vec::new();
    for name in &names {
        eprint!("Fetching {name}... ");
        match comment_checker::grammar::download_grammar(name, &cache_dir) {
            Ok(path) => eprintln!("ok ({})", path.display()),
            Err(e) => {
                eprintln!("FAILED: {e}");
                errors.push(format!("{name}: {e}"));
            }
        }
    }

    if errors.is_empty() {
        eprintln!("All grammars fetched to {}", cache_dir.display());
        Ok(())
    } else {
        Err(format!("{} grammar(s) failed to download", errors.len()))
    }
}

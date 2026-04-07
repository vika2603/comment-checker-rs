mod cli;

use std::io::Read;
use std::process::ExitCode;

use clap::Parser;
use rayon::prelude::*;

use comment_checker::allowlist::Allowlist;
use comment_checker::checker::{check_comments, filter_by_ranges, Diagnostic};
use comment_checker::config::load_config;
use comment_checker::error::Result;
use comment_checker::input::filesystem::discover_files;
use comment_checker::input::hook::parse_hook_input;
use comment_checker::output::{format_jsonl, format_prompt, format_text};
use comment_checker::parser::{languages::Language, parse_comments};

use cli::{Cli, OutputFormat};

fn main() -> ExitCode {
    let args = Cli::parse();

    if let Err(msg) = args.validate() {
        eprintln!("error: {msg}");
        return ExitCode::from(2);
    }

    if args.hook {
        match run(&args) {
            Ok(has_diagnostics) => {
                if has_diagnostics {
                    ExitCode::from(2)
                } else {
                    ExitCode::SUCCESS
                }
            }
            Err(e) => {
                eprintln!("comment-checker hook error: {e}");
                ExitCode::SUCCESS
            }
        }
    } else {
        match run(&args) {
            Ok(has_diagnostics) => {
                if has_diagnostics {
                    ExitCode::FAILURE
                } else {
                    ExitCode::SUCCESS
                }
            }
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::from(2)
            }
        }
    }
}

fn run(args: &Cli) -> Result<bool> {
    let start_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let config = load_config(args.config.as_deref(), &start_dir)?;
    let allowlist = Allowlist::new(&config.allowlist)?;

    let mut diagnostics = if args.hook {
        run_hook_mode(args, &allowlist)?
    } else {
        run_cli_mode(args, &allowlist, &config)?
    };

    diagnostics.sort_by(|a, b| {
        a.sort_key()
            .0
            .cmp(b.sort_key().0)
            .then(a.sort_key().1.cmp(&b.sort_key().1))
            .then(a.sort_key().2.cmp(&b.sort_key().2))
    });

    let has_diagnostics = !diagnostics.is_empty();

    if !args.quiet {
        let output = if args.hook || args.prompt.is_some() {
            format_prompt(&diagnostics, args.prompt.as_deref())
        } else {
            match args.format {
                OutputFormat::Text => format_text(&diagnostics),
                OutputFormat::Jsonl => format_jsonl(&diagnostics),
            }
        };
        if !output.is_empty() {
            if args.hook {
                eprint!("{output}");
            } else {
                print!("{output}");
            }
        }
    }

    Ok(has_diagnostics)
}

fn run_hook_mode(args: &Cli, allowlist: &Allowlist) -> Result<Vec<Diagnostic>> {
    let mut stdin_data = String::new();
    std::io::stdin()
        .read_to_string(&mut stdin_data)
        .unwrap_or(0);

    let stdin_data = stdin_data.trim();
    if stdin_data.is_empty() {
        eprintln!("comment-checker: no hook JSON on stdin, skipping");
        return Ok(Vec::new());
    }

    let target = match parse_hook_input(stdin_data) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("comment-checker: invalid hook JSON: {e}");
            return Ok(Vec::new());
        }
    };

    let Some(language) = Language::from_path(&target.file_path) else {
        if args.verbose {
            eprintln!(
                "comment-checker: skipping unsupported file: {}",
                target.file_path.display()
            );
        }
        return Ok(Vec::new());
    };

    let source = match std::fs::read_to_string(&target.file_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "comment-checker: cannot read {}: {e}",
                target.file_path.display()
            );
            return Ok(Vec::new());
        }
    };

    let comments = match parse_comments(&source, language) {
        Some(c) => c,
        None => {
            eprintln!(
                "warning: tree-sitter parse failed for {}, skipping",
                target.file_path.display()
            );
            return Ok(Vec::new());
        }
    };
    let file_str = target.file_path.to_string_lossy().into_owned();
    let mut diagnostics = check_comments(&file_str, comments, allowlist);

    if let Some(ranges) = target.changed_ranges {
        diagnostics = filter_by_ranges(diagnostics, &ranges);
    }

    Ok(diagnostics)
}

fn run_cli_mode(
    args: &Cli,
    allowlist: &Allowlist,
    config: &comment_checker::config::Config,
) -> Result<Vec<Diagnostic>> {
    let language_filter = if config.languages.is_empty() {
        vec![]
    } else {
        config.languages.clone()
    };

    let files = discover_files(&args.paths, args.max_file_size, &language_filter);

    if args.verbose {
        eprintln!("comment-checker: discovered {} file(s)", files.len());
    }

    let diagnostics: Vec<Diagnostic> = files
        .par_iter()
        .flat_map(|df| {
            let path_str = df.path.to_string_lossy().into_owned();
            let source = match std::fs::read_to_string(&df.path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("warning: cannot read {path_str}: {e}");
                    return Vec::new();
                }
            };

            let comments = match parse_comments(&source, df.language) {
                Some(c) => c,
                None => {
                    eprintln!(
                        "warning: tree-sitter parse failed for {path_str}, skipping"
                    );
                    return Vec::new();
                }
            };
            check_comments(&path_str, comments, allowlist)
        })
        .collect();

    Ok(diagnostics)
}

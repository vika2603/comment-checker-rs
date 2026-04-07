# comment-checker

A CLI tool that flags code comments using tree-sitter AST parsing. Designed to run as a PostToolUse hook for AI coding agents (Claude Code, Codex), catching unnecessary comments the moment they're written.

## Philosophy

All comments are suspect. The tool flags every comment unless it matches an allowlist pattern. Since the consumer is an AI agent, false positives are cheap -- the agent decides what to fix.

## Quick Start

```bash
# Build from source
cargo install --path .

# Install hook for your AI coding tool
comment-checker init claude
comment-checker init codex

# Or check files directly
comment-checker src/
```

## How It Works

```
AI agent writes code
  -> PostToolUse hook triggers
  -> comment-checker parses the file with tree-sitter
  -> Extracts all comment nodes from the AST
  -> Filters against built-in + user allowlist
  -> Reports flagged comments via stderr
  -> AI agent sees the feedback and removes unnecessary comments
```

In hook mode, only comments within the changed region are flagged (not the entire file). For `Write` operations the whole file is checked. For `Edit`/`MultiEdit` operations, a +/- 3 line buffer around the changed lines is checked.

## Supported Languages

Rust, Go, TypeScript, JavaScript (JSX/TSX), Python, Java, C, C++, Ruby, Shell/Bash

## Usage

### As a Hook (primary use case)

```bash
# Install for Claude Code
comment-checker init claude

# Install for Codex
comment-checker init codex

# Uninstall
comment-checker uninstall claude
comment-checker uninstall codex
```

This adds a PostToolUse hook that runs automatically on every Write/Edit/MultiEdit operation. When the hook finds flagged comments, it outputs an XML report to stderr and exits with code 2, which the AI agent sees as feedback.

### As a CLI

```bash
# Check specific files
comment-checker src/main.rs src/lib.rs

# Check a directory (respects .gitignore)
comment-checker src/

# JSONL output
comment-checker --format jsonl src/

# Quiet mode (exit code only)
comment-checker --quiet src/
```

**Exit codes (CLI mode):** 0 = no comments found, 1 = comments flagged, 2 = tool error

**Exit codes (hook mode):** 0 = clean, 2 = comments flagged (feedback to agent)

## Built-in Allowlist

These comment patterns are always allowed:

| Category | Examples |
|----------|---------|
| Linter directives | `eslint-disable`, `noqa`, `@ts-ignore`, `@ts-expect-error`, `rubocop:disable` |
| Compiler pragmas | `pragma`, `go:generate`, `go:build`, `go:embed` |
| Type annotations | `type: ignore`, `pyright:`, `mypy:` |
| License headers | `Copyright`, `License`, `SPDX-License-Identifier` |
| Region markers | `#region`, `#endregion`, `MARK:` |
| BDD keywords | `given`, `when`, `then` |
| Shebangs | `#!/usr/bin/env`, `#!/bin/bash` |
| Encoding | `-*- coding: -*-` |

## Configuration

Create a `.comment-checker.toml` in your project root (or any parent directory):

```toml
# Regex patterns matched against stripped comment content (prefix removed)
allowlist = [
  "^SAFETY:",
  "^INVARIANT:",
  "^PERF:",
]

# Restrict to specific languages (empty = all supported)
languages = ["rust", "python", "typescript"]
```

**Config discovery order** (first match wins, no merging):
1. `--config <path>` (explicit)
2. Walk up from current directory to root, first `.comment-checker.toml` found
3. `$XDG_CONFIG_HOME/comment-checker/config.toml`
4. Built-in defaults

## Custom Prompt Template

Customize the XML output that AI agents see:

```bash
comment-checker --hook --prompt '<review>{{count}} comments to fix:\n{{comments}}</review>'
```

Placeholders: `{{comments}}` (XML comment blocks), `{{count}}` (number of flagged comments).

## How Allowlist Matching Works

Patterns match against **stripped comment content** -- the text after removing the comment prefix (`//`, `#`, `/*`, etc.) and one leading space. This means a single pattern works across all languages:

```
Source:     // TODO: fix this
Stripped:   TODO: fix this        <- pattern matches against this

Source:     # TODO: fix this
Stripped:   TODO: fix this        <- same pattern matches
```

## Building

```bash
cargo build --release
cp target/release/comment-checker /usr/local/bin/
```

Requires a C compiler for tree-sitter grammar compilation. First build takes ~1 minute.

## Acknowledgments

Inspired by [go-claude-code-comment-checker](https://github.com/code-yeongyu/go-claude-code-comment-checker) by [@code-yeongyu](https://github.com/code-yeongyu). This project takes the same core idea (flag all comments, let the AI agent decide) and reimplements it in Rust with tree-sitter AST parsing, multi-language support, changed-region filtering, and configurable allowlists.

## License

MIT

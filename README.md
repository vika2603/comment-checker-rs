# comment-checker

A CLI tool that flags code comments using tree-sitter AST parsing. Designed to run as a PostToolUse hook for AI coding agents (Claude Code, Codex), catching unnecessary comments the moment they're written.

## Philosophy

All comments are suspect. The tool flags every comment unless it matches an allowlist pattern. Since the consumer is an AI agent, false positives are cheap -- the agent decides what to fix.

## Quick Start

```bash
# One-line install + hook setup for Claude Code
curl -fsSL https://raw.githubusercontent.com/vika2603/comment-checker-rs/main/install.sh | sh -s -- --claude

# Or for Codex
curl -fsSL https://raw.githubusercontent.com/vika2603/comment-checker-rs/main/install.sh | sh -s -- --codex

# Install only (no hook setup)
curl -fsSL https://raw.githubusercontent.com/vika2603/comment-checker-rs/main/install.sh | sh

# Or build from source
cargo install --path .
comment-checker init claude
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

In hook mode, only comments within the changed region are flagged (not the entire file). For `Write`/`Create` operations the whole file is checked. For `Edit`/`MultiEdit` operations, a +/- 3 line buffer around the changed lines is checked.

## Supported Languages

| Language | Extensions |
|----------|------------|
| Rust | `.rs` |
| Python | `.py`, `.pyi` |
| JavaScript | `.js`, `.mjs`, `.cjs` |
| JSX | `.jsx` |
| TypeScript | `.ts`, `.mts`, `.cts` |
| TSX | `.tsx` |
| Go | `.go` |
| Java | `.java` |
| C | `.c`, `.h` |
| C++ | `.cpp`, `.cc`, `.cxx`, `.hpp`, `.hxx` |
| Ruby | `.rb` |
| Shell/Bash | `.sh`, `.bash`, `.zsh` |
| Kotlin | `.kt`, `.kts` |
| Swift | `.swift` |
| C# | `.cs` |
| Scala | `.scala`, `.sc` |
| PHP | `.php`, `.phtml` |
| Lua | `.lua` |
| Elixir | `.ex`, `.exs` |
| Haskell | `.hs` |
| OCaml | `.ml`, `.mli` |
| Zig | `.zig` |
| Dart | `.dart` |
| R | `.r`, `.R` |
| TOML | `.toml` |
| YAML | `.yml`, `.yaml` |
| HTML | `.html`, `.htm` |
| CSS | `.css` |
| SQL | `.sql` |
| HCL/Terraform | `.tf`, `.hcl` |
| Nix | `.nix` |
| Clojure | `.clj`, `.cljs`, `.cljc` |
| Erlang | `.erl`, `.hrl` |
| Objective-C | `.m` |

## Usage

### Hook Setup

```bash
# Install for Claude Code
comment-checker init claude

# Install for Codex
comment-checker init codex

# Uninstall
comment-checker uninstall claude
comment-checker uninstall codex
```

This adds a PostToolUse hook that runs automatically on every Write/Create/Edit/MultiEdit operation. When the hook finds flagged comments, it outputs an XML report to stderr and exits with code 2, which the AI agent sees as feedback.

### Pre-fetch Grammars

```bash
# Download all grammar .so files
comment-checker fetch-parsers

# Download specific grammars
comment-checker fetch-parsers rust python typescript
```

Grammars are also automatically downloaded on first use and cached in `~/.cache/comment-checker/parsers/`.

### Options

```
--config <path>    Use a specific config file
--prompt <tpl>     Custom XML output template
-q, --quiet        Suppress output (exit code only)
```

**Exit codes:** 0 = no flagged comments, 2 = flagged comments found (feedback to agent)

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

# Custom instruction for AI agents (overrides built-in default)
instruction = "Remove outdated or redundant comments. Keep comments that explain why, not what."

# Grammar settings
[parsers]
auto_download = true      # Download missing grammars automatically (default: true)
# path = "/custom/parsers"  # Optional custom grammar directory
```

**Config discovery:** project config (`.comment-checker.toml` found by walking up from cwd) is merged with global config (`$XDG_CONFIG_HOME/comment-checker/config.toml`). Allowlist patterns are combined; project `instruction` and `parsers.path` override global.

## Custom Prompt Template

Customize the XML output that AI agents see:

```bash
comment-checker --prompt '<review>{{count}} comments to fix:\n{{comments}}</review>'
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

Tree-sitter grammars are loaded dynamically at runtime (not compiled into the binary). On first use, the required grammar `.so` file is automatically downloaded from GitHub releases and cached in `~/.cache/comment-checker/parsers/`. No C compiler required for building.

## Acknowledgments

Inspired by [go-claude-code-comment-checker](https://github.com/code-yeongyu/go-claude-code-comment-checker) by [@code-yeongyu](https://github.com/code-yeongyu). This project takes the same core idea (flag all comments, let the AI agent decide) and reimplements it in Rust with tree-sitter AST parsing, multi-language support, changed-region filtering, and configurable allowlists.

## License

MIT

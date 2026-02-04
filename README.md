# frmt

Code formatter runner.

## Overview

frmt automatically detects the language of your source files and runs the appropriate linter or formatter. It finds configuration files by walking up the directory tree, uses local linters when available, and falls back to system-wide installations.

## Installation

```bash
# Build from source
cargo build --release

# Install to ~/.local/bin
make install

# The binary will be at target/release/frmt
```

## Usage

```bash
# Lint/format specific files
frmt file1.py file2.js file3.go

# Lint/format all files in current directory
frmt

# Lint/format files in a specific directory
frmt ./src ./tests
```

## Development

```bash
# Build in debug mode
make rust

# Build in release mode
make rust-release

# Format code
make fmt

# Run linter
make clippy

# Run tests
make test

# Clean build artifacts
make clean

# Install binary
make install
```

## Supported Languages

| Language | Extensions | Linters/Formatters |
|----------|------------|-------------------|
| Python | .py, .pyw | pylint, flake8, black |
| JavaScript | .js, .mjs, .cjs, .jsx | eslint, prettier |
| TypeScript | .ts, .tsx | eslint, prettier |
| Go | .go | golangci-lint, gofmt |
| Rust | .rs | rustfmt, cargo-clippy |
| Java | .java | clang-format, google-java-format |
| PHP | .php | phpcbf, php-cs-fixer |
| CSS/SCSS | .css, .scss, .sass, .less | stylelint, prettier |
| HTML | .html, .htm | prettier |
| JSON | .json, .jsonc | prettier |
| YAML | .yaml, .yml | prettier |
| Markdown | .md, .markdown | prettier |
| Shell | .sh, .bash, .zsh, .fish | shellcheck |
| SQL | .sql | sqlfluff |
| Dockerfile | Dockerfile | hadolint |

## How It Works

1. **Language Detection**: frmt matches file extensions to supported languages
2. **Linter Discovery**: Checks for local linters first, then system PATH
3. **Config Search**: Walks up the directory tree looking for configuration files
4. **File Batching**: Groups files sharing the same linter configuration
5. **Execution**: Runs the appropriate linter/formatter on each batch

## Requirements

- Rust 1.70 or later
- Linters/formatters for the languages you want to use (must be installed separately)

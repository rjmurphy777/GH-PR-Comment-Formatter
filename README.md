# pr-comments

[![CI](https://github.com/rjmurphy777/GH-PR-Comment-Formatter/actions/workflows/ci.yml/badge.svg)](https://github.com/rjmurphy777/GH-PR-Comment-Formatter/actions/workflows/ci.yml)
![Coverage](https://img.shields.io/badge/coverage-100%25-brightgreen)
![Tests](https://img.shields.io/badge/tests-103%20passing-brightgreen)

A CLI tool to fetch and format GitHub PR comments for LLM consumption.

## Test Coverage

| Metric | Value |
|--------|-------|
| **Code Coverage** | 100% (enforced in CI) |
| **Unit Tests** | 103 |
| **Lines Covered** | 335/335 |

## Installation

### From Source

Requires [Rust](https://rustup.rs/) to be installed.

```bash
cargo install --path .
```

### Pre-built Binary

Download the pre-built binary from the releases page and add it to your PATH.

## Prerequisites

- [GitHub CLI (gh)](https://cli.github.com/) must be installed and authenticated

```bash
# Install gh CLI
brew install gh  # macOS
# or see https://cli.github.com/manual/installation

# Authenticate
gh auth login
```

## Usage

### Basic Usage

```bash
# Using PR URL
pr-comments https://github.com/owner/repo/pull/123

# Using shorthand format
pr-comments owner/repo#123

# Using explicit arguments
pr-comments --owner owner --repo repo --pr-number 123
```

### Output Formats

```bash
# Claude format (default) - optimized for LLM consumption
pr-comments owner/repo#123 --format claude

# Grouped by file
pr-comments owner/repo#123 --format grouped

# Flat list sorted by date
pr-comments owner/repo#123 --format flat

# Minimal overview
pr-comments owner/repo#123 --format minimal

# JSON output for programmatic use
pr-comments owner/repo#123 --format json
```

### Filtering

```bash
# Filter by comment author
pr-comments owner/repo#123 --author username

# Show only the most recent comment per file
pr-comments owner/repo#123 --most-recent

# Combine filters
pr-comments owner/repo#123 --author username --most-recent
```

### Code Snippet Options

```bash
# Exclude code snippets from output
pr-comments owner/repo#123 --no-snippet

# Customize snippet length (default: 15 lines)
pr-comments owner/repo#123 --snippet-lines 25
```

### Output to File

```bash
# Write output to a file
pr-comments owner/repo#123 --output review-comments.md
```

## Examples

```bash
# Get all review comments on a PR in Claude format
pr-comments facebook/react#12345

# Get comments from a specific reviewer in minimal format
pr-comments rust-lang/rust#98765 --author bors --format minimal

# Export comments as JSON for processing
pr-comments kubernetes/kubernetes#111222 --format json --output comments.json
```

## All Options

```
Usage: pr-comments [OPTIONS] [PR]

Arguments:
  [PR]  PR URL or owner/repo#number format

Options:
  -o, --owner <OWNER>              Repository owner
  -r, --repo <REPO>                Repository name
  -n, --pr-number <PR_NUMBER>      Pull request number
  -a, --author <AUTHOR>            Filter by author username
  -m, --most-recent                Show only newest comment per file
  -f, --format <FORMAT>            Output format [default: claude]
                                   [possible values: claude, grouped, flat, minimal, json]
      --no-snippet                 Exclude code snippets
      --snippet-lines <LINES>      Max lines in snippets [default: 15]
  -O, --output <OUTPUT>            Write output to file
  -h, --help                       Print help
  -V, --version                    Print version
```

## Development

```bash
# Run tests
cargo test

# Run tests with coverage (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --lib --fail-under 100

# Run with clippy lints
cargo clippy --all-targets --all-features -- -D warnings

# Build release binary
cargo build --release

# The binary will be at ./target/release/pr-comments
```

### Pre-commit Hooks

This project uses pre-commit hooks to ensure code quality:

```bash
# Install pre-commit
pip install pre-commit

# Install hooks
pre-commit install

# Run hooks manually
pre-commit run --all-files
```

The following checks run on every commit:
- `cargo fmt` - Code formatting
- `cargo clippy` - Linting with warnings as errors
- `cargo test` - Unit tests
- `cargo tarpaulin` - 100% code coverage enforcement

## License

MIT

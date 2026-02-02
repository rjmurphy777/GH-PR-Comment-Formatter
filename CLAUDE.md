# CLAUDE.md

This file provides guidance for LLMs working with this codebase.

## Project Overview

**pr-comments** is a Rust CLI tool that fetches GitHub pull request review comments and formats them for LLM consumption. It uses the GitHub CLI (`gh`) for authenticated API access and outputs comments in multiple formats optimized for different use cases.

## Quick Reference

```bash
# Build
cargo build                    # Development build
cargo build --release          # Release build (binary at ./target/release/pr-comments)

# Test
cargo test                     # Run all unit tests (67 tests)
cargo test --test integration_tests  # Integration tests (requires gh auth)

# Lint and format
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check

# Install locally
cargo install --path .
```

## Architecture

```
src/
├── main.rs      # Entry point, orchestrates workflow
├── lib.rs       # Public library exports
├── cli.rs       # CLI argument parsing (Clap), URL parsing
├── models.rs    # PRComment struct and methods
├── fetcher.rs   # GitHub API calls via `gh api` command
├── parser.rs    # JSON parsing, filtering, grouping
├── formatter.rs # 5 output formats (claude, grouped, flat, minimal, json)
└── error.rs     # Custom error types with thiserror
```

## Data Flow

1. **Parse args** (`cli.rs`) - Extract owner/repo/pr from URL or flags
2. **Fetch** (`fetcher.rs`) - Call `gh api` for PR comments and metadata
3. **Parse** (`parser.rs`) - Convert JSON to `PRComment` structs
4. **Filter** (`parser.rs`) - Apply author/most-recent filters
5. **Format** (`formatter.rs`) - Generate output in selected format
6. **Output** (`main.rs`) - Write to file or stdout

## Key Dependencies

- `clap` 4.5 - CLI parsing with derive macros
- `serde`/`serde_json` - JSON serialization
- `chrono` - Date/time handling
- `thiserror` - Error type definitions

**External requirement:** GitHub CLI (`gh`) must be installed and authenticated.

## Output Formats

| Format | Description | Use Case |
|--------|-------------|----------|
| `claude` | LLM-optimized with instructions and grouping | Default, for AI consumption |
| `grouped` | Comments organized by file | Code review navigation |
| `flat` | Chronological list (newest first) | Timeline view |
| `minimal` | Single-line compact entries | Quick scanning |
| `json` | Valid JSON array | Programmatic integration |

## CLI Usage Examples

```bash
# Using shorthand notation
pr-comments owner/repo#123

# Using full URL
pr-comments https://github.com/owner/repo/pull/123

# With filters
pr-comments owner/repo#123 --author username --most-recent

# Different output format
pr-comments owner/repo#123 --format json --output comments.json

# Control code snippets
pr-comments owner/repo#123 --no-snippet
pr-comments owner/repo#123 --snippet-lines 10
```

## Testing Patterns

- Unit tests are colocated in each module under `#[cfg(test)]`
- Integration tests in `tests/integration_tests.rs` test full CLI behavior
- Tests use `serde_json::json!` macro to create test fixtures
- Some integration tests require GitHub CLI authentication

## Code Conventions

- Use `thiserror` derive macros for error types
- Heavy use of `#[derive(Debug, Clone, Serialize, Deserialize)]`
- Filter functions return new `Vec<PRComment>` (immutable style)
- Formatters take `&[PRComment]` slices and return `String`
- All public functions have corresponding unit tests

## Pre-commit Hooks

The `.pre-commit-config.yaml` runs these checks on commit:
- `cargo fmt --all -- --check` - Code formatting
- `cargo clippy` with `-D warnings` - Strict linting
- `cargo test --lib` - Unit tests

## Common Development Tasks

**Add a new output format:**
1. Add variant to `OutputFormat` enum in `cli.rs`
2. Add formatting function in `formatter.rs`
3. Add match arm in `format_comments()` in `formatter.rs`
4. Add tests for the new format

**Add a new filter:**
1. Add filter function in `parser.rs`
2. Add CLI flag in `Args` struct in `cli.rs`
3. Apply filter in `main.rs` workflow
4. Add tests

**Modify PRComment model:**
1. Update struct in `models.rs`
2. Update `parse_comment()` in `parser.rs`
3. Update affected formatters in `formatter.rs`
4. Update tests

//! CLI interface and argument parsing.

use crate::error::ParseError;
use clap::{Parser, ValueEnum};

/// CLI tool to fetch and format GitHub PR comments for LLM consumption.
#[derive(Parser, Debug)]
#[command(name = "pr-comments")]
#[command(version = "0.1.0")]
#[command(about = "Fetch and format GitHub PR comments for LLM consumption")]
#[command(author = "rjmurphy777")]
pub struct Args {
    /// PR URL or owner/repo#number format
    #[arg(value_name = "PR")]
    pub pr: Option<String>,

    /// Repository owner
    #[arg(short = 'o', long)]
    pub owner: Option<String>,

    /// Repository name
    #[arg(short = 'r', long)]
    pub repo: Option<String>,

    /// Pull request number
    #[arg(short = 'n', long = "pr-number")]
    pub pr_number: Option<i32>,

    /// Filter by author username
    #[arg(short = 'a', long)]
    pub author: Option<String>,

    /// Show only newest comment per file
    #[arg(short = 'm', long = "most-recent")]
    pub most_recent: bool,

    /// Output format
    #[arg(short = 'f', long, default_value = "claude", value_enum)]
    pub format: OutputFormat,

    /// Exclude code snippets
    #[arg(long = "no-snippet")]
    pub no_snippet: bool,

    /// Max lines in snippets
    #[arg(long = "snippet-lines", default_value = "15")]
    pub snippet_lines: usize,

    /// Write output to file
    #[arg(short = 'O', long)]
    pub output: Option<String>,
}

/// Available output formats.
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq)]
pub enum OutputFormat {
    /// Claude/LLM-optimized format (default)
    Claude,
    /// Grouped by file
    Grouped,
    /// Flat list by date
    Flat,
    /// Minimal/compact overview
    Minimal,
    /// JSON output
    Json,
}

/// Parses a GitHub PR URL or shorthand format into (owner, repo, pr_number).
///
/// Supports:
/// - Full URL: https://github.com/owner/repo/pull/123
/// - Shorthand: owner/repo#123
pub fn parse_pr_url(url: &str) -> Result<(String, String, i32), ParseError> {
    let url = url.trim().trim_end_matches('/');

    // Try full URL format: https://github.com/owner/repo/pull/123
    if url.starts_with("https://github.com/") || url.starts_with("http://github.com/") {
        let path = url
            .trim_start_matches("https://github.com/")
            .trim_start_matches("http://github.com/");

        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 4 && parts[2] == "pull" {
            let owner = parts[0].to_string();
            let repo = parts[1].to_string();
            let pr_number = parts[3]
                .parse::<i32>()
                .map_err(|_| ParseError::InvalidPrNumber(parts[3].to_string()))?;
            return Ok((owner, repo, pr_number));
        }
    }

    // Try shorthand format: owner/repo#123
    if let Some(hash_pos) = url.find('#') {
        let repo_part = &url[..hash_pos];
        let pr_part = &url[hash_pos + 1..];

        if let Some(slash_pos) = repo_part.find('/') {
            let owner = repo_part[..slash_pos].to_string();
            let repo = repo_part[slash_pos + 1..].to_string();
            let pr_number = pr_part
                .parse::<i32>()
                .map_err(|_| ParseError::InvalidPrNumber(pr_part.to_string()))?;

            if !owner.is_empty() && !repo.is_empty() {
                return Ok((owner, repo, pr_number));
            }
        }
    }

    Err(ParseError::InvalidUrl(url.to_string()))
}

/// Resolves CLI arguments into (owner, repo, pr_number).
///
/// Priority:
/// 1. Explicit --owner, --repo, --pr-number flags
/// 2. Positional PR URL/shorthand argument
pub fn resolve_pr_args(args: &Args) -> Result<(String, String, i32), ParseError> {
    // If all explicit args are provided, use them
    if let (Some(owner), Some(repo), Some(pr_number)) =
        (&args.owner, &args.repo, args.pr_number)
    {
        return Ok((owner.clone(), repo.clone(), pr_number));
    }

    // Otherwise, try to parse the positional PR argument
    if let Some(pr) = &args.pr {
        return parse_pr_url(pr);
    }

    Err(ParseError::InvalidUrl(
        "Provide a PR URL or --owner, --repo, and --pr-number".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pr_url_full_url() {
        let (owner, repo, pr) = parse_pr_url("https://github.com/ROKT/canal/pull/14777").unwrap();
        assert_eq!(owner, "ROKT");
        assert_eq!(repo, "canal");
        assert_eq!(pr, 14777);
    }

    #[test]
    fn test_parse_pr_url_shorthand() {
        let (owner, repo, pr) = parse_pr_url("ROKT/canal#14777").unwrap();
        assert_eq!(owner, "ROKT");
        assert_eq!(repo, "canal");
        assert_eq!(pr, 14777);
    }

    #[test]
    fn test_parse_pr_url_trailing_slash() {
        let (owner, repo, pr) = parse_pr_url("https://github.com/ROKT/canal/pull/14777/").unwrap();
        assert_eq!(owner, "ROKT");
        assert_eq!(repo, "canal");
        assert_eq!(pr, 14777);
    }

    #[test]
    fn test_parse_pr_url_invalid() {
        let result = parse_pr_url("invalid-url");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_pr_url_invalid_pr_number() {
        let result = parse_pr_url("owner/repo#notanumber");
        assert!(result.is_err());
    }

    #[test]
    fn test_output_format_default() {
        let args = Args::parse_from(["pr-comments", "ROKT/canal#123"]);
        assert_eq!(args.format, OutputFormat::Claude);
    }

    #[test]
    fn test_output_format_json() {
        let args = Args::parse_from(["pr-comments", "ROKT/canal#123", "--format", "json"]);
        assert_eq!(args.format, OutputFormat::Json);
    }

    #[test]
    fn test_output_format_grouped() {
        let args = Args::parse_from(["pr-comments", "ROKT/canal#123", "-f", "grouped"]);
        assert_eq!(args.format, OutputFormat::Grouped);
    }

    #[test]
    fn test_resolve_pr_args_explicit() {
        let args = Args {
            pr: None,
            owner: Some("owner".to_string()),
            repo: Some("repo".to_string()),
            pr_number: Some(123),
            author: None,
            most_recent: false,
            format: OutputFormat::Claude,
            no_snippet: false,
            snippet_lines: 15,
            output: None,
        };
        let (owner, repo, pr) = resolve_pr_args(&args).unwrap();
        assert_eq!(owner, "owner");
        assert_eq!(repo, "repo");
        assert_eq!(pr, 123);
    }

    #[test]
    fn test_resolve_pr_args_positional() {
        let args = Args {
            pr: Some("ROKT/canal#456".to_string()),
            owner: None,
            repo: None,
            pr_number: None,
            author: None,
            most_recent: false,
            format: OutputFormat::Claude,
            no_snippet: false,
            snippet_lines: 15,
            output: None,
        };
        let (owner, repo, pr) = resolve_pr_args(&args).unwrap();
        assert_eq!(owner, "ROKT");
        assert_eq!(repo, "canal");
        assert_eq!(pr, 456);
    }

    #[test]
    fn test_resolve_pr_args_missing() {
        let args = Args {
            pr: None,
            owner: None,
            repo: None,
            pr_number: None,
            author: None,
            most_recent: false,
            format: OutputFormat::Claude,
            no_snippet: false,
            snippet_lines: 15,
            output: None,
        };
        let result = resolve_pr_args(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_args_author_filter() {
        let args = Args::parse_from(["pr-comments", "ROKT/canal#123", "--author", "testuser"]);
        assert_eq!(args.author, Some("testuser".to_string()));
    }

    #[test]
    fn test_args_most_recent() {
        let args = Args::parse_from(["pr-comments", "ROKT/canal#123", "--most-recent"]);
        assert!(args.most_recent);
    }

    #[test]
    fn test_args_no_snippet() {
        let args = Args::parse_from(["pr-comments", "ROKT/canal#123", "--no-snippet"]);
        assert!(args.no_snippet);
    }

    #[test]
    fn test_args_snippet_lines() {
        let args = Args::parse_from(["pr-comments", "ROKT/canal#123", "--snippet-lines", "25"]);
        assert_eq!(args.snippet_lines, 25);
    }

    #[test]
    fn test_args_output_file() {
        let args = Args::parse_from(["pr-comments", "ROKT/canal#123", "-O", "output.md"]);
        assert_eq!(args.output, Some("output.md".to_string()));
    }
}

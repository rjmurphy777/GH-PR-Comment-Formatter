//! PR Comments CLI - Fetch and format GitHub PR comments for LLM consumption.

use clap::Parser;
use pr_comments::{
    cli::{resolve_pr_args, Args, OutputFormat, REPO_URL},
    fetcher::{fetch_pr_checks, fetch_pr_comments, fetch_pr_info, fetch_pr_reviews},
    formatter::{
        format_as_json, format_checks_as_json, format_checks_for_claude, format_checks_minimal,
        format_comments_flat, format_comments_grouped, format_comments_minimal, format_for_claude,
    },
    parser::{
        filter_by_author, get_most_recent_per_file, parse_checks_response, parse_comments,
        parse_review_comments,
    },
};
use std::fs;
use std::io::{self, Write};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let args = Args::parse();

    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    // Handle self-update before resolving PR arguments
    if args.is_update_request() {
        return run_update();
    }

    // Resolve PR arguments
    let (owner, repo, pr_number) = resolve_pr_args(&args)?;

    let output = if args.checks {
        run_checks(&owner, &repo, pr_number, &args)?
    } else {
        run_comments(&owner, &repo, pr_number, &args)?
    };

    // Write output
    if let Some(output_path) = &args.output {
        fs::write(output_path, &output)?;
        eprintln!("Output written to {output_path}");
    } else {
        io::stdout().write_all(output.as_bytes())?;
    }

    Ok(())
}

fn run_checks(
    owner: &str,
    repo: &str,
    pr_number: i32,
    args: &Args,
) -> Result<String, Box<dyn std::error::Error>> {
    let raw_response = fetch_pr_checks(owner, repo, pr_number)?;
    let report = parse_checks_response(&raw_response)?;

    let output = match args.format {
        OutputFormat::Claude => format_checks_for_claude(&report),
        OutputFormat::Json => format_checks_as_json(&report),
        OutputFormat::Minimal => format_checks_minimal(&report),
        OutputFormat::Grouped | OutputFormat::Flat => {
            eprintln!(
                "Note: --format {} is not supported with --checks, using claude format",
                match args.format {
                    OutputFormat::Grouped => "grouped",
                    OutputFormat::Flat => "flat",
                    _ => unreachable!(),
                }
            );
            format_checks_for_claude(&report)
        }
    };

    Ok(output)
}

fn run_update() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Updating pr-comments from {REPO_URL}...");

    let status = Command::new("cargo")
        .args(["install", "--git", REPO_URL])
        .status()
        .map_err(|e| format!("Failed to run cargo. Is the Rust toolchain installed?\n  {e}"))?;

    if status.success() {
        eprintln!("pr-comments updated successfully!");
        Ok(())
    } else {
        Err(format!("cargo install exited with status: {status}").into())
    }
}

fn run_comments(
    owner: &str,
    repo: &str,
    pr_number: i32,
    args: &Args,
) -> Result<String, Box<dyn std::error::Error>> {
    // Fetch line-specific comments, reviews, and PR info
    let raw_comments = fetch_pr_comments(owner, repo, pr_number)?;
    let raw_reviews = fetch_pr_reviews(owner, repo, pr_number)?;
    let pr_info = fetch_pr_info(owner, repo, pr_number)?;

    // Parse line-specific comments
    let mut comments = parse_comments(&raw_comments);

    // Parse and merge review-level comments (reviews with body text)
    let review_comments = parse_review_comments(&raw_reviews);
    comments.extend(review_comments);

    // Apply author filter
    if args.author.is_some() {
        comments = filter_by_author(comments, args.author.as_deref());
    }

    // Apply most-recent filter
    if args.most_recent {
        comments = get_most_recent_per_file(comments);
    }

    // Get PR info for formatting
    let pr_url = pr_info
        .get("html_url")
        .and_then(|v| v.as_str())
        .map(String::from);
    let pr_title = pr_info
        .get("title")
        .and_then(|v| v.as_str())
        .map(String::from);
    // GraphQL node ID for the PR (used for replying to comments via GraphQL API)
    let pr_node_id = pr_info
        .get("node_id")
        .and_then(|v| v.as_str())
        .map(String::from);

    // Format output
    let include_snippet = !args.no_snippet;
    let output = match args.format {
        OutputFormat::Claude => format_for_claude(
            &comments,
            pr_url.as_deref(),
            pr_title.as_deref(),
            pr_node_id.as_deref(),
            include_snippet,
            args.snippet_lines,
        ),
        OutputFormat::Grouped => {
            format_comments_grouped(&comments, include_snippet, args.snippet_lines)
        }
        OutputFormat::Flat => format_comments_flat(&comments, include_snippet, args.snippet_lines),
        OutputFormat::Minimal => format_comments_minimal(&comments),
        OutputFormat::Json => format_as_json(&comments, include_snippet, args.snippet_lines),
    };

    Ok(output)
}

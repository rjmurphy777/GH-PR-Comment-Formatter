//! PR Comments CLI - Fetch and format GitHub PR comments for LLM consumption.

use clap::Parser;
use pr_comments::{
    cli::{resolve_pr_args, Args, OutputFormat},
    fetcher::{fetch_pr_comments, fetch_pr_info},
    formatter::{
        format_as_json, format_comments_flat, format_comments_grouped, format_comments_minimal,
        format_for_claude,
    },
    parser::{filter_by_author, get_most_recent_per_file, parse_comments},
};
use std::fs;
use std::io::{self, Write};
use std::process::ExitCode;

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
    // Resolve PR arguments
    let (owner, repo, pr_number) = resolve_pr_args(&args)?;

    // Fetch comments and PR info
    let raw_comments = fetch_pr_comments(&owner, &repo, pr_number)?;
    let pr_info = fetch_pr_info(&owner, &repo, pr_number)?;

    // Parse comments
    let mut comments = parse_comments(&raw_comments);

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

    // Format output
    let include_snippet = !args.no_snippet;
    let output = match args.format {
        OutputFormat::Claude => format_for_claude(
            &comments,
            pr_url.as_deref(),
            pr_title.as_deref(),
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

    // Write output
    if let Some(output_path) = &args.output {
        fs::write(output_path, &output)?;
        eprintln!("Output written to {output_path}");
    } else {
        io::stdout().write_all(output.as_bytes())?;
    }

    Ok(())
}

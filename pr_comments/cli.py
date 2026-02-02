#!/usr/bin/env python3
"""CLI interface for PR Comments tool."""

import argparse
import json
import sys
from pathlib import Path

from .fetcher import GitHubAPIError, fetch_pr_comments, fetch_pr_info
from .formatter import (
    format_comments_flat,
    format_comments_grouped,
    format_comments_minimal,
    format_for_claude,
)
from .parser import filter_by_author, get_most_recent_per_file, parse_comments


def parse_pr_url(url: str) -> tuple[str, str, int]:
    """Parse a GitHub PR URL into owner, repo, and PR number.

    Args:
        url: GitHub PR URL like https://github.com/ROKT/canal/pull/14777

    Returns:
        Tuple of (owner, repo, pr_number).

    Raises:
        ValueError: If URL cannot be parsed.
    """
    # Handle both full URLs and shorthand
    url = url.rstrip("/")

    if url.startswith("https://github.com/"):
        parts = url.replace("https://github.com/", "").split("/")
        if len(parts) >= 4 and parts[2] == "pull":
            return parts[0], parts[1], int(parts[3])

    # Try owner/repo#number format
    if "#" in url:
        repo_part, pr_num = url.split("#")
        if "/" in repo_part:
            owner, repo = repo_part.split("/")
            return owner, repo, int(pr_num)

    raise ValueError(f"Cannot parse PR URL: {url}")


def main(args: list[str] | None = None) -> int:
    """Main CLI entry point.

    Args:
        args: Command line arguments (defaults to sys.argv).

    Returns:
        Exit code (0 for success).
    """
    parser = argparse.ArgumentParser(
        prog="pr-comments",
        description="Fetch and format GitHub PR comments for LLM consumption",
    )

    # Input options (mutually exclusive)
    input_group = parser.add_argument_group("input")
    input_group.add_argument(
        "pr",
        nargs="?",
        help="PR URL (https://github.com/owner/repo/pull/123) or owner/repo#123 format",
    )
    input_group.add_argument(
        "--owner",
        "-o",
        help="Repository owner",
    )
    input_group.add_argument(
        "--repo",
        "-r",
        help="Repository name",
    )
    input_group.add_argument(
        "--pr-number",
        "-n",
        type=int,
        help="Pull request number",
    )

    # Filter options
    filter_group = parser.add_argument_group("filters")
    filter_group.add_argument(
        "--author",
        "-a",
        help="Filter comments by author username",
    )
    filter_group.add_argument(
        "--most-recent",
        "-m",
        action="store_true",
        help="Show only the most recent comment per file",
    )

    # Output options
    output_group = parser.add_argument_group("output")
    output_group.add_argument(
        "--format",
        "-f",
        choices=["claude", "grouped", "flat", "minimal", "json"],
        default="claude",
        help="Output format (default: claude)",
    )
    output_group.add_argument(
        "--no-snippet",
        action="store_true",
        help="Exclude code snippets from output",
    )
    output_group.add_argument(
        "--snippet-lines",
        type=int,
        default=15,
        help="Maximum lines in code snippets (default: 15)",
    )
    output_group.add_argument(
        "--output",
        "-O",
        help="Write output to file instead of stdout",
    )

    parsed = parser.parse_args(args)

    # Determine owner, repo, and PR number
    owner: str | None = None
    repo: str | None = None
    pr_number: int | None = None

    if parsed.pr:
        try:
            owner, repo, pr_number = parse_pr_url(parsed.pr)
        except ValueError as e:
            print(f"Error: {e}", file=sys.stderr)
            return 1
    elif parsed.owner and parsed.repo and parsed.pr_number:
        owner = parsed.owner
        repo = parsed.repo
        pr_number = parsed.pr_number
    else:
        parser.print_help()
        print("\nError: Provide a PR URL or --owner, --repo, and --pr-number", file=sys.stderr)
        return 1

    # Type narrowing: at this point, all values are guaranteed to be set
    assert owner is not None and repo is not None and pr_number is not None

    # Fetch comments
    try:
        raw_comments = fetch_pr_comments(owner, repo, pr_number)
        pr_info = fetch_pr_info(owner, repo, pr_number)
    except GitHubAPIError as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1

    # Parse comments
    comments = parse_comments(raw_comments)

    # Apply filters
    if parsed.author:
        comments = filter_by_author(comments, parsed.author)

    if parsed.most_recent:
        comments = get_most_recent_per_file(comments)

    # Format output
    include_snippet = not parsed.no_snippet
    snippet_lines = parsed.snippet_lines

    if parsed.format == "json":
        output = json.dumps(
            [
                {
                    "file": c.file_path,
                    "line": c.line_number,
                    "author": c.author,
                    "body": c.body,
                    "snippet": c.get_code_snippet(snippet_lines) if include_snippet else None,
                    "url": c.html_url,
                }
                for c in comments
            ],
            indent=2,
        )
    elif parsed.format == "grouped":
        output = format_comments_grouped(comments, include_snippet, snippet_lines)
    elif parsed.format == "flat":
        output = format_comments_flat(comments, include_snippet, snippet_lines)
    elif parsed.format == "minimal":
        output = format_comments_minimal(comments)
    else:  # claude (default)
        pr_url = pr_info.get("html_url")
        pr_title = pr_info.get("title")
        output = format_for_claude(comments, pr_url, pr_title, include_snippet, snippet_lines)

    # Write output
    if parsed.output:
        Path(parsed.output).write_text(output)
        print(f"Output written to {parsed.output}", file=sys.stderr)
    else:
        print(output)

    return 0


if __name__ == "__main__":
    sys.exit(main())

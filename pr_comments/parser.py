"""Parser for GitHub PR comment JSON data."""

from datetime import datetime
from typing import Any, Optional

from .models import PRComment


def parse_datetime(dt_str: str) -> datetime:
    """Parse GitHub API datetime string to datetime object."""
    # GitHub uses ISO 8601 format: 2026-01-30T23:06:02Z
    return datetime.fromisoformat(dt_str.replace("Z", "+00:00"))


def parse_comment(comment_data: dict[str, Any]) -> PRComment:
    """Parse a single comment from GitHub API response.

    Args:
        comment_data: Raw JSON dict from GitHub API.

    Returns:
        A PRComment object with parsed data.
    """
    # Get line numbers - prefer the current line, fall back to original
    line_number = comment_data.get("line") or comment_data.get("original_line")
    start_line = comment_data.get("start_line") or comment_data.get("original_start_line")

    # Get author - handle both regular users and bots
    user_data = comment_data.get("user", {})
    author = user_data.get("login", "unknown")

    return PRComment(
        id=comment_data.get("id", 0),
        file_path=comment_data.get("path", "unknown"),
        line_number=line_number,
        start_line=start_line,
        author=author,
        body=comment_data.get("body", ""),
        created_at=parse_datetime(comment_data.get("created_at", "1970-01-01T00:00:00Z")),
        updated_at=parse_datetime(comment_data.get("updated_at", "1970-01-01T00:00:00Z")),
        diff_hunk=comment_data.get("diff_hunk", ""),
        html_url=comment_data.get("html_url", ""),
    )


def parse_comments(comments_data: list[dict[str, Any]]) -> list[PRComment]:
    """Parse multiple comments from GitHub API response.

    Args:
        comments_data: List of raw JSON dicts from GitHub API.

    Returns:
        List of PRComment objects.
    """
    return [parse_comment(c) for c in comments_data]


def filter_by_author(comments: list[PRComment], author: Optional[str]) -> list[PRComment]:
    """Filter comments by author.

    Args:
        comments: List of PRComment objects.
        author: Author username to filter by. If None, return all comments.

    Returns:
        Filtered list of comments.
    """
    if not author:
        return comments
    return [c for c in comments if c.author == author]


def get_most_recent_per_file(comments: list[PRComment]) -> list[PRComment]:
    """Get the most recent comment for each file.

    Args:
        comments: List of PRComment objects.

    Returns:
        List of most recent comments, one per file.
    """
    file_comments: dict[str, PRComment] = {}
    for comment in comments:
        existing = file_comments.get(comment.file_path)
        if not existing or comment.updated_at > existing.updated_at:
            file_comments[comment.file_path] = comment
    return list(file_comments.values())


def group_by_file(comments: list[PRComment]) -> dict[str, list[PRComment]]:
    """Group comments by file path.

    Args:
        comments: List of PRComment objects.

    Returns:
        Dict mapping file paths to lists of comments.
    """
    grouped: dict[str, list[PRComment]] = {}
    for comment in comments:
        if comment.file_path not in grouped:
            grouped[comment.file_path] = []
        grouped[comment.file_path].append(comment)
    return grouped

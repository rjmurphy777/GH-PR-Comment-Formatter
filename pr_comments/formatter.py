"""Formatter for outputting PR comments in LLM-friendly format."""

from .models import PRComment
from .parser import group_by_file


def format_comment_for_llm(
    comment: PRComment,
    include_snippet: bool = True,
    snippet_lines: int = 10,
) -> str:
    """Format a single comment for LLM consumption.

    Args:
        comment: The PRComment to format.
        include_snippet: Whether to include a code snippet.
        snippet_lines: Maximum lines in the code snippet.

    Returns:
        Formatted string representation of the comment.
    """
    lines = []

    # Header with file and line info
    lines.append(f"### {comment.file_path} ({comment.get_line_info()})")
    lines.append(f"**Author:** {comment.author}")
    lines.append(f"**Date:** {comment.updated_at.strftime('%Y-%m-%d %H:%M UTC')}")
    lines.append("")

    # Code snippet
    if include_snippet:
        snippet = comment.get_code_snippet(snippet_lines)
        if snippet:
            lines.append("**Code context:**")
            lines.append("```")
            lines.append(snippet)
            lines.append("```")
            lines.append("")

    # Comment body
    lines.append("**Comment:**")
    lines.append(comment.body)
    lines.append("")

    return "\n".join(lines)


def format_comments_grouped(
    comments: list[PRComment],
    include_snippet: bool = True,
    snippet_lines: int = 10,
) -> str:
    """Format comments grouped by file.

    Args:
        comments: List of PRComment objects.
        include_snippet: Whether to include code snippets.
        snippet_lines: Maximum lines in code snippets.

    Returns:
        Formatted string with all comments grouped by file.
    """
    if not comments:
        return "No comments found."

    grouped = group_by_file(comments)
    output_lines = []

    output_lines.append("# PR Comments Summary")
    output_lines.append(f"Total comments: {len(comments)}")
    output_lines.append(f"Files with comments: {len(grouped)}")
    output_lines.append("")

    for file_path in sorted(grouped.keys()):
        file_comments = grouped[file_path]
        # Sort by line number, then by date
        file_comments.sort(key=lambda c: (c.line_number or 0, c.updated_at))

        output_lines.append(f"## {file_path}")
        output_lines.append(f"({len(file_comments)} comment(s))")
        output_lines.append("")

        for comment in file_comments:
            output_lines.append(format_comment_for_llm(comment, include_snippet, snippet_lines))
            output_lines.append("---")
            output_lines.append("")

    return "\n".join(output_lines)


def format_comments_flat(
    comments: list[PRComment],
    include_snippet: bool = True,
    snippet_lines: int = 10,
) -> str:
    """Format comments in a flat list, sorted by date.

    Args:
        comments: List of PRComment objects.
        include_snippet: Whether to include code snippets.
        snippet_lines: Maximum lines in code snippets.

    Returns:
        Formatted string with all comments in chronological order.
    """
    if not comments:
        return "No comments found."

    # Sort by date, most recent first
    sorted_comments = sorted(comments, key=lambda c: c.updated_at, reverse=True)

    output_lines = []
    output_lines.append(f"# PR Comments ({len(comments)} total)")
    output_lines.append("")

    for i, comment in enumerate(sorted_comments, 1):
        output_lines.append(f"## Comment {i}")
        output_lines.append(format_comment_for_llm(comment, include_snippet, snippet_lines))
        output_lines.append("---")
        output_lines.append("")

    return "\n".join(output_lines)


def format_comments_minimal(comments: list[PRComment]) -> str:
    """Format comments in a minimal format for quick overview.

    Args:
        comments: List of PRComment objects.

    Returns:
        Minimal formatted string with essential info only.
    """
    if not comments:
        return "No comments found."

    grouped = group_by_file(comments)
    output_lines = []

    output_lines.append(f"PR Comments: {len(comments)} total across {len(grouped)} files")
    output_lines.append("")

    for file_path in sorted(grouped.keys()):
        file_comments = grouped[file_path]
        output_lines.append(f"📄 {file_path}")

        for comment in sorted(file_comments, key=lambda c: c.line_number or 0):
            # Truncate body for preview
            body_preview = comment.body[:100].replace("\n", " ")
            if len(comment.body) > 100:
                body_preview += "..."

            output_lines.append(
                f"  └─ {comment.get_line_info()} ({comment.author}): {body_preview}"
            )

        output_lines.append("")

    return "\n".join(output_lines)


def format_for_claude(
    comments: list[PRComment],
    pr_url: str | None = None,
    pr_title: str | None = None,
    include_snippet: bool = True,
    snippet_lines: int = 15,
) -> str:
    """Format comments specifically optimized for Claude/LLM consumption.

    This format is designed to give Claude all the context needed to understand
    and address PR review comments.

    Args:
        comments: List of PRComment objects.
        pr_url: Optional URL to the pull request.
        pr_title: Optional title of the pull request.
        include_snippet: Whether to include code snippets.
        snippet_lines: Maximum lines in code snippets.

    Returns:
        LLM-optimized formatted string.
    """
    if not comments:
        return "No review comments found on this PR."

    grouped = group_by_file(comments)
    lines = []

    # Header
    lines.append("# Pull Request Review Comments")
    if pr_title:
        lines.append(f"**PR Title:** {pr_title}")
    if pr_url:
        lines.append(f"**PR URL:** {pr_url}")
    lines.append(f"**Total Comments:** {len(comments)}")
    lines.append(f"**Files Affected:** {len(grouped)}")
    lines.append("")
    lines.append("Below are the review comments that need to be addressed. Each comment includes:")
    lines.append("- The file path and line number(s)")
    lines.append("- A code snippet showing the context")
    lines.append("- The reviewer's comment/feedback")
    lines.append("")
    lines.append("---")
    lines.append("")

    for file_path in sorted(grouped.keys()):
        file_comments = grouped[file_path]
        file_comments.sort(key=lambda c: (c.line_number or 0, c.updated_at))

        lines.append(f"## File: `{file_path}`")
        lines.append("")

        for comment in file_comments:
            lines.append(f"### {comment.get_line_info()}")
            lines.append(f"**Reviewer:** {comment.author}")
            lines.append("")

            if include_snippet:
                snippet = comment.get_code_snippet(snippet_lines)
                if snippet:
                    lines.append("**Code being reviewed:**")
                    lines.append("```")
                    lines.append(snippet)
                    lines.append("```")
                    lines.append("")

            lines.append("**Review comment:**")
            lines.append(comment.body)
            lines.append("")
            lines.append("---")
            lines.append("")

    # Footer with instructions
    lines.append("## Instructions for Addressing Comments")
    lines.append("")
    lines.append("Please review each comment above and make the necessary changes to the code.")
    lines.append("For each comment, consider:")
    lines.append("1. What specific change is being requested?")
    lines.append("2. Is the suggestion valid and should be implemented?")
    lines.append("3. Are there any related changes needed in other parts of the codebase?")

    return "\n".join(lines)

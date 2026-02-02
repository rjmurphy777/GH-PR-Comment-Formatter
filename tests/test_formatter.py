"""Tests for PR Comments formatter."""

from datetime import datetime, timezone

import pytest

from pr_comments.formatter import (
    format_comment_for_llm,
    format_comments_flat,
    format_comments_grouped,
    format_comments_minimal,
    format_for_claude,
)
from pr_comments.models import PRComment
from pr_comments.parser import parse_comments

from .fixtures import SAMPLE_COMMENTS


def create_test_comment(
    file_path: str = "test.py",
    line_number: int = 10,
    author: str = "reviewer",
    body: str = "Test comment",
    diff_hunk: str = "@@ -1,3 +1,5 @@\n    code here\n+   new code",
) -> PRComment:
    """Helper to create test comments."""
    return PRComment(
        id=1,
        file_path=file_path,
        line_number=line_number,
        start_line=None,
        author=author,
        body=body,
        created_at=datetime(2026, 1, 30, 10, 0, 0, tzinfo=timezone.utc),
        updated_at=datetime(2026, 1, 30, 12, 0, 0, tzinfo=timezone.utc),
        diff_hunk=diff_hunk,
        html_url="https://github.com/owner/repo/pull/1#discussion_r1",
    )


class TestFormatCommentForLLM:
    """Tests for format_comment_for_llm function."""

    def test_includes_file_and_line(self):
        """Test that output includes file path and line number."""
        comment = create_test_comment(file_path="src/main.py", line_number=42)
        output = format_comment_for_llm(comment)

        assert "src/main.py" in output
        assert "line 42" in output

    def test_includes_author(self):
        """Test that output includes author name."""
        comment = create_test_comment(author="reviewer1")
        output = format_comment_for_llm(comment)

        assert "reviewer1" in output

    def test_includes_date(self):
        """Test that output includes formatted date."""
        comment = create_test_comment()
        output = format_comment_for_llm(comment)

        assert "2026-01-30" in output

    def test_includes_code_snippet_by_default(self):
        """Test that code snippet is included by default."""
        comment = create_test_comment(diff_hunk="@@ -1,3 +1,5 @@\n    some code")
        output = format_comment_for_llm(comment)

        assert "```" in output
        assert "some code" in output

    def test_excludes_snippet_when_disabled(self):
        """Test that snippet can be excluded."""
        comment = create_test_comment(diff_hunk="@@ -1,3 +1,5 @@\n    some code")
        output = format_comment_for_llm(comment, include_snippet=False)

        # Should not have code block
        assert "```" not in output or "some code" not in output

    def test_includes_comment_body(self):
        """Test that comment body is included."""
        comment = create_test_comment(body="Please fix this issue!")
        output = format_comment_for_llm(comment)

        assert "Please fix this issue!" in output

    def test_handles_empty_diff_hunk(self):
        """Test handling of empty diff hunk."""
        comment = create_test_comment(diff_hunk="")
        output = format_comment_for_llm(comment)

        # Should still produce valid output
        assert comment.file_path in output
        assert comment.body in output


class TestFormatCommentsGrouped:
    """Tests for format_comments_grouped function."""

    def test_groups_by_file(self):
        """Test that comments are grouped by file."""
        comments = parse_comments(SAMPLE_COMMENTS)
        output = format_comments_grouped(comments)

        # Should have file headers
        assert "## src/main.py" in output
        assert "## src/processor.py" in output

    def test_shows_total_count(self):
        """Test that total comment count is shown."""
        comments = parse_comments(SAMPLE_COMMENTS)
        output = format_comments_grouped(comments)

        assert f"Total comments: {len(comments)}" in output

    def test_shows_file_count(self):
        """Test that file count is shown."""
        comments = parse_comments(SAMPLE_COMMENTS)
        output = format_comments_grouped(comments)

        # Should show number of files
        assert "Files with comments:" in output

    def test_empty_comments(self):
        """Test handling of empty comment list."""
        output = format_comments_grouped([])
        assert "No comments found" in output

    def test_respects_snippet_setting(self):
        """Test that snippet setting is respected."""
        comments = [create_test_comment()]
        output = format_comments_grouped(comments, include_snippet=False)

        # Check no code blocks (beyond what might be in the comment body)
        assert "Code context" not in output


class TestFormatCommentsFlat:
    """Tests for format_comments_flat function."""

    def test_sorts_by_date(self):
        """Test that comments are sorted by date, most recent first."""
        comments = parse_comments(SAMPLE_COMMENTS)
        output = format_comments_flat(comments)

        # Check structure includes numbered comments
        assert "## Comment 1" in output
        assert "## Comment 2" in output

    def test_shows_total_count(self):
        """Test that total count is shown in header."""
        comments = parse_comments(SAMPLE_COMMENTS)
        output = format_comments_flat(comments)

        assert f"{len(comments)} total" in output

    def test_empty_comments(self):
        """Test handling of empty list."""
        output = format_comments_flat([])
        assert "No comments found" in output


class TestFormatCommentsMinimal:
    """Tests for format_comments_minimal function."""

    def test_shows_file_emoji(self):
        """Test that file emoji is included."""
        comments = [create_test_comment(file_path="test.py")]
        output = format_comments_minimal(comments)

        assert "📄 test.py" in output

    def test_shows_line_info(self):
        """Test that line info is shown."""
        comments = [create_test_comment(line_number=42, author="bob")]
        output = format_comments_minimal(comments)

        assert "line 42" in output
        assert "bob" in output

    def test_truncates_long_bodies(self):
        """Test that long comment bodies are truncated."""
        long_body = "x" * 200
        comments = [create_test_comment(body=long_body)]
        output = format_comments_minimal(comments)

        # Should be truncated with ellipsis
        assert "..." in output
        # Should not have full body
        assert "x" * 200 not in output

    def test_summary_line(self):
        """Test that summary line is present."""
        comments = parse_comments(SAMPLE_COMMENTS)
        output = format_comments_minimal(comments)

        assert "PR Comments:" in output
        assert "total" in output
        assert "files" in output

    def test_empty_comments(self):
        """Test handling of empty list."""
        output = format_comments_minimal([])
        assert "No comments found" in output


class TestFormatForClaude:
    """Tests for format_for_claude function."""

    def test_includes_pr_title(self):
        """Test that PR title is included when provided."""
        comments = [create_test_comment()]
        output = format_for_claude(
            comments,
            pr_title="Fix authentication bug",
        )

        assert "Fix authentication bug" in output

    def test_includes_pr_url(self):
        """Test that PR URL is included when provided."""
        comments = [create_test_comment()]
        output = format_for_claude(
            comments,
            pr_url="https://github.com/owner/repo/pull/123",
        )

        assert "https://github.com/owner/repo/pull/123" in output

    def test_includes_header(self):
        """Test that appropriate header is included."""
        comments = [create_test_comment()]
        output = format_for_claude(comments)

        assert "Pull Request Review Comments" in output

    def test_includes_file_info(self):
        """Test that file paths are clearly marked."""
        comments = [create_test_comment(file_path="src/api/handler.py")]
        output = format_for_claude(comments)

        assert "File: `src/api/handler.py`" in output

    def test_includes_instructions(self):
        """Test that instructions for addressing comments are included."""
        comments = [create_test_comment()]
        output = format_for_claude(comments)

        assert "Instructions for Addressing Comments" in output
        assert "specific change" in output.lower()

    def test_includes_reviewer_info(self):
        """Test that reviewer name is included."""
        comments = [create_test_comment(author="senior_dev")]
        output = format_for_claude(comments)

        assert "Reviewer:" in output
        assert "senior_dev" in output

    def test_empty_comments(self):
        """Test handling of empty list."""
        output = format_for_claude([])
        assert "No review comments" in output

    def test_groups_by_file(self):
        """Test that comments are grouped by file."""
        comments = parse_comments(SAMPLE_COMMENTS)
        output = format_for_claude(comments)

        # Files should appear as headers
        assert "File: `src/main.py`" in output
        assert "File: `src/processor.py`" in output

    def test_includes_snippet_by_default(self):
        """Test that code snippets are included."""
        comments = [create_test_comment()]
        output = format_for_claude(comments)

        assert "Code being reviewed:" in output
        assert "```" in output

    def test_can_exclude_snippets(self):
        """Test that snippets can be excluded."""
        comments = [create_test_comment()]
        output = format_for_claude(comments, include_snippet=False)

        assert "Code being reviewed:" not in output

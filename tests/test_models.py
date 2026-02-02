"""Tests for PR Comments models."""

from datetime import datetime, timezone

import pytest

from pr_comments.models import PRComment


class TestPRComment:
    """Tests for PRComment dataclass."""

    def test_create_comment(self):
        """Test creating a PRComment instance."""
        comment = PRComment(
            id=123,
            file_path="src/main.py",
            line_number=42,
            start_line=None,
            author="reviewer1",
            body="This is a comment",
            created_at=datetime(2026, 1, 30, 10, 0, 0, tzinfo=timezone.utc),
            updated_at=datetime(2026, 1, 30, 10, 0, 0, tzinfo=timezone.utc),
            diff_hunk="@@ -10,3 +10,5 @@\n     print('hello')\n+    print('world')",
            html_url="https://github.com/owner/repo/pull/1#discussion_r123",
        )

        assert comment.id == 123
        assert comment.file_path == "src/main.py"
        assert comment.line_number == 42
        assert comment.author == "reviewer1"

    def test_get_line_info_single_line(self):
        """Test get_line_info with a single line number."""
        comment = PRComment(
            id=1,
            file_path="test.py",
            line_number=42,
            start_line=None,
            author="user",
            body="comment",
            created_at=datetime.now(timezone.utc),
            updated_at=datetime.now(timezone.utc),
            diff_hunk="",
            html_url="",
        )

        assert comment.get_line_info() == "line 42"

    def test_get_line_info_line_range(self):
        """Test get_line_info with a line range."""
        comment = PRComment(
            id=1,
            file_path="test.py",
            line_number=50,
            start_line=45,
            author="user",
            body="comment",
            created_at=datetime.now(timezone.utc),
            updated_at=datetime.now(timezone.utc),
            diff_hunk="",
            html_url="",
        )

        assert comment.get_line_info() == "lines 45-50"

    def test_get_line_info_same_start_and_end(self):
        """Test get_line_info when start and end lines are the same."""
        comment = PRComment(
            id=1,
            file_path="test.py",
            line_number=42,
            start_line=42,
            author="user",
            body="comment",
            created_at=datetime.now(timezone.utc),
            updated_at=datetime.now(timezone.utc),
            diff_hunk="",
            html_url="",
        )

        assert comment.get_line_info() == "line 42"

    def test_get_line_info_no_line_number(self):
        """Test get_line_info when line number is None."""
        comment = PRComment(
            id=1,
            file_path="test.py",
            line_number=None,
            start_line=None,
            author="user",
            body="comment",
            created_at=datetime.now(timezone.utc),
            updated_at=datetime.now(timezone.utc),
            diff_hunk="",
            html_url="",
        )

        assert comment.get_line_info() == "line unknown"

    def test_get_code_snippet_empty_diff(self):
        """Test get_code_snippet with empty diff hunk."""
        comment = PRComment(
            id=1,
            file_path="test.py",
            line_number=10,
            start_line=None,
            author="user",
            body="comment",
            created_at=datetime.now(timezone.utc),
            updated_at=datetime.now(timezone.utc),
            diff_hunk="",
            html_url="",
        )

        assert comment.get_code_snippet() == ""

    def test_get_code_snippet_removes_header(self):
        """Test that get_code_snippet removes the diff header."""
        diff_hunk = "@@ -10,3 +10,5 @@ def hello():\n     print('hello')\n+    print('world')"
        comment = PRComment(
            id=1,
            file_path="test.py",
            line_number=12,
            start_line=None,
            author="user",
            body="comment",
            created_at=datetime.now(timezone.utc),
            updated_at=datetime.now(timezone.utc),
            diff_hunk=diff_hunk,
            html_url="",
        )

        snippet = comment.get_code_snippet()
        assert not snippet.startswith("@@")
        assert "print('hello')" in snippet
        assert "print('world')" in snippet

    def test_get_code_snippet_truncates_long_diff(self):
        """Test that get_code_snippet truncates long diffs."""
        lines = ["@@ -1,20 +1,20 @@ header"] + [f"    line {i}" for i in range(20)]
        diff_hunk = "\n".join(lines)

        comment = PRComment(
            id=1,
            file_path="test.py",
            line_number=20,
            start_line=None,
            author="user",
            body="comment",
            created_at=datetime.now(timezone.utc),
            updated_at=datetime.now(timezone.utc),
            diff_hunk=diff_hunk,
            html_url="",
        )

        snippet = comment.get_code_snippet(max_lines=5)
        snippet_lines = snippet.split("\n")
        assert len(snippet_lines) == 5

    def test_get_code_snippet_returns_last_lines(self):
        """Test that get_code_snippet returns the last lines (most relevant)."""
        lines = ["@@ -1,10 +1,10 @@ header"] + [f"    line {i}" for i in range(10)]
        diff_hunk = "\n".join(lines)

        comment = PRComment(
            id=1,
            file_path="test.py",
            line_number=10,
            start_line=None,
            author="user",
            body="comment",
            created_at=datetime.now(timezone.utc),
            updated_at=datetime.now(timezone.utc),
            diff_hunk=diff_hunk,
            html_url="",
        )

        snippet = comment.get_code_snippet(max_lines=3)
        assert "line 7" in snippet
        assert "line 8" in snippet
        assert "line 9" in snippet
        assert "line 0" not in snippet

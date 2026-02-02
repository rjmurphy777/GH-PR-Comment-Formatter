"""Tests for PR Comments parser."""

from datetime import datetime, timezone

from pr_comments.models import PRComment
from pr_comments.parser import (
    filter_by_author,
    get_most_recent_per_file,
    group_by_file,
    parse_comment,
    parse_comments,
    parse_datetime,
)

from .fixtures import (
    SAMPLE_COMMENT_BOT,
    SAMPLE_COMMENT_COMPLEX,
    SAMPLE_COMMENT_MINIMAL,
    SAMPLE_COMMENT_NO_LINE,
    SAMPLE_COMMENT_SAME_FILE,
    SAMPLE_COMMENT_WITH_RANGE,
    SAMPLE_COMMENTS,
)


class TestParseDatetime:
    """Tests for parse_datetime function."""

    def test_parse_github_datetime(self):
        """Test parsing GitHub API datetime format."""
        result = parse_datetime("2026-01-30T10:00:00Z")
        expected = datetime(2026, 1, 30, 10, 0, 0, tzinfo=timezone.utc)
        assert result == expected

    def test_parse_datetime_with_milliseconds(self):
        """Test parsing datetime that might have milliseconds."""
        result = parse_datetime("2026-01-30T10:30:45Z")
        assert result.hour == 10
        assert result.minute == 30
        assert result.second == 45


class TestParseComment:
    """Tests for parse_comment function."""

    def test_parse_minimal_comment(self):
        """Test parsing a minimal comment."""
        comment = parse_comment(SAMPLE_COMMENT_MINIMAL)

        assert comment.id == 123
        assert comment.file_path == "src/main.py"
        assert comment.line_number == 12
        assert comment.start_line is None
        assert comment.author == "reviewer1"
        assert comment.body == "Consider adding a docstring here."
        assert "print('hello')" in comment.diff_hunk

    def test_parse_comment_with_line_range(self):
        """Test parsing a comment with a line range."""
        comment = parse_comment(SAMPLE_COMMENT_WITH_RANGE)

        assert comment.id == 124
        assert comment.file_path == "src/processor.py"
        assert comment.line_number == 30
        assert comment.start_line == 25
        assert comment.author == "reviewer2"

    def test_parse_bot_comment(self):
        """Test parsing a comment from a bot."""
        comment = parse_comment(SAMPLE_COMMENT_BOT)

        assert comment.id == 125
        assert comment.author == "devin-ai-integration[bot]"
        assert "Potential issue detected" in comment.body

    def test_parse_comment_no_line_number(self):
        """Test parsing a comment without line numbers (uses original_line)."""
        comment = parse_comment(SAMPLE_COMMENT_NO_LINE)

        assert comment.id == 126
        assert comment.line_number is None
        assert comment.file_path == "README.md"

    def test_parse_comment_falls_back_to_original_line(self):
        """Test that parsing falls back to original_line when line is None."""
        comment = parse_comment(SAMPLE_COMMENT_COMPLEX)

        # line is None, so it should fall back to original_line
        assert comment.line_number == 3376
        assert comment.start_line == 3374

    def test_parse_comment_handles_missing_user(self):
        """Test parsing a comment with missing user data."""
        data = {**SAMPLE_COMMENT_MINIMAL, "user": {}}
        comment = parse_comment(data)
        assert comment.author == "unknown"

    def test_parse_comment_handles_no_user(self):
        """Test parsing a comment with no user field."""
        data = {k: v for k, v in SAMPLE_COMMENT_MINIMAL.items() if k != "user"}
        comment = parse_comment(data)
        assert comment.author == "unknown"

    def test_parse_comment_preserves_html_url(self):
        """Test that HTML URL is preserved."""
        comment = parse_comment(SAMPLE_COMMENT_MINIMAL)
        assert comment.html_url == "https://github.com/ROKT/canal/pull/123#discussion_r123"


class TestParseComments:
    """Tests for parse_comments function."""

    def test_parse_empty_list(self):
        """Test parsing an empty list."""
        result = parse_comments([])
        assert result == []

    def test_parse_multiple_comments(self):
        """Test parsing multiple comments."""
        result = parse_comments(SAMPLE_COMMENTS)
        assert len(result) == 5
        assert all(isinstance(c, PRComment) for c in result)

    def test_parse_comments_preserves_order(self):
        """Test that comment order is preserved."""
        result = parse_comments(SAMPLE_COMMENTS)
        assert result[0].id == 123
        assert result[1].id == 124
        assert result[2].id == 125


class TestFilterByAuthor:
    """Tests for filter_by_author function."""

    def test_filter_returns_all_when_no_author(self):
        """Test that all comments are returned when author is None."""
        comments = parse_comments(SAMPLE_COMMENTS)
        result = filter_by_author(comments, None)
        assert len(result) == len(comments)

    def test_filter_by_specific_author(self):
        """Test filtering by a specific author."""
        comments = parse_comments(SAMPLE_COMMENTS)
        result = filter_by_author(comments, "reviewer1")

        assert len(result) == 2
        assert all(c.author == "reviewer1" for c in result)

    def test_filter_by_bot_author(self):
        """Test filtering by a bot author."""
        comments = parse_comments(SAMPLE_COMMENTS)
        result = filter_by_author(comments, "devin-ai-integration[bot]")

        assert len(result) == 1
        assert result[0].author == "devin-ai-integration[bot]"

    def test_filter_by_nonexistent_author(self):
        """Test filtering by an author with no comments."""
        comments = parse_comments(SAMPLE_COMMENTS)
        result = filter_by_author(comments, "nonexistent_user")

        assert len(result) == 0

    def test_filter_empty_string_author(self):
        """Test that empty string author returns all comments."""
        comments = parse_comments(SAMPLE_COMMENTS)
        result = filter_by_author(comments, "")

        # Empty string is falsy, so should return all
        assert len(result) == len(comments)


class TestGetMostRecentPerFile:
    """Tests for get_most_recent_per_file function."""

    def test_single_comment_per_file(self):
        """Test with single comment per file."""
        comments = parse_comments([SAMPLE_COMMENT_MINIMAL, SAMPLE_COMMENT_WITH_RANGE])
        result = get_most_recent_per_file(comments)

        assert len(result) == 2
        file_paths = {c.file_path for c in result}
        assert "src/main.py" in file_paths
        assert "src/processor.py" in file_paths

    def test_multiple_comments_same_file(self):
        """Test that only most recent comment per file is returned."""
        comments = parse_comments([SAMPLE_COMMENT_MINIMAL, SAMPLE_COMMENT_SAME_FILE])
        result = get_most_recent_per_file(comments)

        # Both are from src/main.py, should return only most recent
        assert len(result) == 1
        assert result[0].file_path == "src/main.py"
        # SAMPLE_COMMENT_SAME_FILE is more recent (15:30 vs 10:00)
        assert result[0].id == 127

    def test_empty_list(self):
        """Test with empty comment list."""
        result = get_most_recent_per_file([])
        assert result == []


class TestGroupByFile:
    """Tests for group_by_file function."""

    def test_group_comments(self):
        """Test grouping comments by file."""
        comments = parse_comments(SAMPLE_COMMENTS)
        grouped = group_by_file(comments)

        assert "src/main.py" in grouped
        assert len(grouped["src/main.py"]) == 2
        assert "src/processor.py" in grouped
        assert len(grouped["src/processor.py"]) == 1

    def test_group_empty_list(self):
        """Test grouping an empty list."""
        grouped = group_by_file([])
        assert grouped == {}

    def test_group_single_file(self):
        """Test grouping when all comments are on one file."""
        comments = parse_comments([SAMPLE_COMMENT_MINIMAL, SAMPLE_COMMENT_SAME_FILE])
        grouped = group_by_file(comments)

        assert len(grouped) == 1
        assert len(grouped["src/main.py"]) == 2

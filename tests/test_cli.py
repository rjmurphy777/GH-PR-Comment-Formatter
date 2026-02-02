"""Tests for PR Comments CLI."""

import json
from unittest.mock import MagicMock, patch

import pytest

from pr_comments.cli import main, parse_pr_url

from .fixtures import SAMPLE_COMMENTS, SAMPLE_PR_INFO


class TestParsePRUrl:
    """Tests for parse_pr_url function."""

    def test_parse_full_url(self):
        """Test parsing a full GitHub PR URL."""
        url = "https://github.com/ROKT/canal/pull/14777"
        owner, repo, pr_num = parse_pr_url(url)

        assert owner == "ROKT"
        assert repo == "canal"
        assert pr_num == 14777

    def test_parse_url_with_trailing_slash(self):
        """Test parsing URL with trailing slash."""
        url = "https://github.com/owner/repo/pull/123/"
        owner, repo, pr_num = parse_pr_url(url)

        assert owner == "owner"
        assert repo == "repo"
        assert pr_num == 123

    def test_parse_shorthand_format(self):
        """Test parsing owner/repo#number format."""
        url = "ROKT/canal#14777"
        owner, repo, pr_num = parse_pr_url(url)

        assert owner == "ROKT"
        assert repo == "canal"
        assert pr_num == 14777

    def test_raises_on_invalid_url(self):
        """Test that invalid URLs raise ValueError."""
        with pytest.raises(ValueError):
            parse_pr_url("not-a-url")

    def test_raises_on_non_pr_github_url(self):
        """Test that non-PR GitHub URLs raise ValueError."""
        with pytest.raises(ValueError):
            parse_pr_url("https://github.com/owner/repo/issues/123")


class TestCLIMain:
    """Tests for main CLI function."""

    @patch("pr_comments.cli.fetch_pr_comments")
    @patch("pr_comments.cli.fetch_pr_info")
    def test_basic_invocation(self, mock_pr_info, mock_comments):
        """Test basic CLI invocation with URL."""
        mock_comments.return_value = SAMPLE_COMMENTS
        mock_pr_info.return_value = SAMPLE_PR_INFO

        result = main(["https://github.com/ROKT/canal/pull/123"])

        assert result == 0
        mock_comments.assert_called_once_with("ROKT", "canal", 123)

    @patch("pr_comments.cli.fetch_pr_comments")
    @patch("pr_comments.cli.fetch_pr_info")
    def test_explicit_args(self, mock_pr_info, mock_comments):
        """Test CLI with explicit --owner, --repo, --pr-number."""
        mock_comments.return_value = SAMPLE_COMMENTS
        mock_pr_info.return_value = SAMPLE_PR_INFO

        result = main(["--owner", "ROKT", "--repo", "canal", "--pr-number", "456"])

        assert result == 0
        mock_comments.assert_called_once_with("ROKT", "canal", 456)

    @patch("pr_comments.cli.fetch_pr_comments")
    @patch("pr_comments.cli.fetch_pr_info")
    def test_json_format(self, mock_pr_info, mock_comments, capsys):
        """Test JSON output format."""
        mock_comments.return_value = SAMPLE_COMMENTS[:1]  # Use just one comment
        mock_pr_info.return_value = SAMPLE_PR_INFO

        result = main(["ROKT/canal#123", "--format", "json"])

        assert result == 0
        captured = capsys.readouterr()
        output = json.loads(captured.out)
        assert isinstance(output, list)

    @patch("pr_comments.cli.fetch_pr_comments")
    @patch("pr_comments.cli.fetch_pr_info")
    def test_author_filter(self, mock_pr_info, mock_comments, capsys):
        """Test filtering by author."""
        mock_comments.return_value = SAMPLE_COMMENTS
        mock_pr_info.return_value = SAMPLE_PR_INFO

        result = main(["ROKT/canal#123", "--author", "reviewer1", "--format", "json"])

        assert result == 0
        captured = capsys.readouterr()
        output = json.loads(captured.out)
        # Should only have comments from reviewer1
        assert all(c["author"] == "reviewer1" for c in output)

    @patch("pr_comments.cli.fetch_pr_comments")
    @patch("pr_comments.cli.fetch_pr_info")
    def test_most_recent_filter(self, mock_pr_info, mock_comments, capsys):
        """Test --most-recent flag."""
        mock_comments.return_value = SAMPLE_COMMENTS
        mock_pr_info.return_value = SAMPLE_PR_INFO

        result = main(["ROKT/canal#123", "--most-recent", "--format", "json"])

        assert result == 0
        captured = capsys.readouterr()
        output = json.loads(captured.out)
        # Should have unique files only
        files = [c["file"] for c in output]
        assert len(files) == len(set(files))

    @patch("pr_comments.cli.fetch_pr_comments")
    @patch("pr_comments.cli.fetch_pr_info")
    def test_grouped_format(self, mock_pr_info, mock_comments, capsys):
        """Test grouped output format."""
        mock_comments.return_value = SAMPLE_COMMENTS
        mock_pr_info.return_value = SAMPLE_PR_INFO

        result = main(["ROKT/canal#123", "--format", "grouped"])

        assert result == 0
        captured = capsys.readouterr()
        assert "## src/main.py" in captured.out

    @patch("pr_comments.cli.fetch_pr_comments")
    @patch("pr_comments.cli.fetch_pr_info")
    def test_flat_format(self, mock_pr_info, mock_comments, capsys):
        """Test flat output format."""
        mock_comments.return_value = SAMPLE_COMMENTS
        mock_pr_info.return_value = SAMPLE_PR_INFO

        result = main(["ROKT/canal#123", "--format", "flat"])

        assert result == 0
        captured = capsys.readouterr()
        assert "## Comment 1" in captured.out

    @patch("pr_comments.cli.fetch_pr_comments")
    @patch("pr_comments.cli.fetch_pr_info")
    def test_minimal_format(self, mock_pr_info, mock_comments, capsys):
        """Test minimal output format."""
        mock_comments.return_value = SAMPLE_COMMENTS
        mock_pr_info.return_value = SAMPLE_PR_INFO

        result = main(["ROKT/canal#123", "--format", "minimal"])

        assert result == 0
        captured = capsys.readouterr()
        assert "📄" in captured.out

    @patch("pr_comments.cli.fetch_pr_comments")
    @patch("pr_comments.cli.fetch_pr_info")
    def test_claude_format_is_default(self, mock_pr_info, mock_comments, capsys):
        """Test that claude format is the default."""
        mock_comments.return_value = SAMPLE_COMMENTS
        mock_pr_info.return_value = SAMPLE_PR_INFO

        result = main(["ROKT/canal#123"])

        assert result == 0
        captured = capsys.readouterr()
        assert "Pull Request Review Comments" in captured.out

    @patch("pr_comments.cli.fetch_pr_comments")
    @patch("pr_comments.cli.fetch_pr_info")
    def test_no_snippet_flag(self, mock_pr_info, mock_comments, capsys):
        """Test --no-snippet flag."""
        mock_comments.return_value = SAMPLE_COMMENTS[:1]
        mock_pr_info.return_value = SAMPLE_PR_INFO

        result = main(["ROKT/canal#123", "--no-snippet", "--format", "json"])

        assert result == 0
        captured = capsys.readouterr()
        output = json.loads(captured.out)
        assert output[0]["snippet"] is None

    @patch("pr_comments.cli.fetch_pr_comments")
    @patch("pr_comments.cli.fetch_pr_info")
    def test_output_to_file(self, mock_pr_info, mock_comments, tmp_path):
        """Test writing output to a file."""
        mock_comments.return_value = SAMPLE_COMMENTS
        mock_pr_info.return_value = SAMPLE_PR_INFO

        output_file = tmp_path / "output.md"
        result = main(["ROKT/canal#123", "--output", str(output_file)])

        assert result == 0
        assert output_file.exists()
        content = output_file.read_text()
        assert "Pull Request Review Comments" in content

    def test_missing_arguments(self, capsys):
        """Test that missing arguments produce error."""
        result = main([])

        assert result == 1
        captured = capsys.readouterr()
        assert "Error" in captured.err or "usage" in captured.out.lower()

    def test_invalid_url(self, capsys):
        """Test that invalid URL produces error."""
        result = main(["not-a-valid-url"])

        assert result == 1
        captured = capsys.readouterr()
        assert "Error" in captured.err

    @patch("pr_comments.cli.fetch_pr_comments")
    def test_api_error_handling(self, mock_comments, capsys):
        """Test handling of API errors."""
        from pr_comments.fetcher import GitHubAPIError

        mock_comments.side_effect = GitHubAPIError("Not found")

        result = main(["ROKT/canal#123"])

        assert result == 1
        captured = capsys.readouterr()
        assert "Error" in captured.err

    @patch("pr_comments.cli.fetch_pr_comments")
    @patch("pr_comments.cli.fetch_pr_info")
    def test_snippet_lines_option(self, mock_pr_info, mock_comments, capsys):
        """Test --snippet-lines option."""
        mock_comments.return_value = SAMPLE_COMMENTS
        mock_pr_info.return_value = SAMPLE_PR_INFO

        # Just verify it doesn't error with the option
        result = main(["ROKT/canal#123", "--snippet-lines", "5"])

        assert result == 0

"""Tests for PR Comments fetcher."""

import json
import subprocess
from unittest.mock import MagicMock, patch

import pytest

from pr_comments.fetcher import (
    GitHubAPIError,
    fetch_pr_comments,
    fetch_pr_info,
    fetch_pr_review_comments,
)

from .fixtures import SAMPLE_COMMENTS, SAMPLE_PR_INFO


class TestFetchPRComments:
    """Tests for fetch_pr_comments function."""

    @patch("pr_comments.fetcher.subprocess.run")
    def test_calls_gh_api_correctly(self, mock_run):
        """Test that gh api is called with correct endpoint."""
        mock_run.return_value = MagicMock(
            stdout=json.dumps(SAMPLE_COMMENTS),
            returncode=0,
        )

        fetch_pr_comments("ROKT", "canal", 14777)

        mock_run.assert_called_once()
        args = mock_run.call_args[0][0]
        assert args[0] == "gh"
        assert args[1] == "api"
        assert "repos/ROKT/canal/pulls/14777/comments" in args[2]

    @patch("pr_comments.fetcher.subprocess.run")
    def test_returns_parsed_json(self, mock_run):
        """Test that response is parsed as JSON."""
        mock_run.return_value = MagicMock(
            stdout=json.dumps(SAMPLE_COMMENTS),
            returncode=0,
        )

        result = fetch_pr_comments("owner", "repo", 123)

        assert isinstance(result, list)
        assert len(result) == len(SAMPLE_COMMENTS)

    @patch("pr_comments.fetcher.subprocess.run")
    def test_raises_error_on_api_failure(self, mock_run):
        """Test that API errors are handled."""
        mock_run.side_effect = subprocess.CalledProcessError(
            1, "gh", stderr="Not found"
        )

        with pytest.raises(GitHubAPIError) as exc_info:
            fetch_pr_comments("owner", "repo", 999)

        assert "Failed to fetch" in str(exc_info.value)

    @patch("pr_comments.fetcher.subprocess.run")
    def test_raises_error_on_invalid_json(self, mock_run):
        """Test that invalid JSON is handled."""
        mock_run.return_value = MagicMock(
            stdout="not valid json",
            returncode=0,
        )

        with pytest.raises(GitHubAPIError) as exc_info:
            fetch_pr_comments("owner", "repo", 123)

        assert "parse" in str(exc_info.value).lower()


class TestFetchPRReviewComments:
    """Tests for fetch_pr_review_comments function."""

    @patch("pr_comments.fetcher.subprocess.run")
    def test_uses_issues_endpoint(self, mock_run):
        """Test that issues endpoint is used for review comments."""
        mock_run.return_value = MagicMock(
            stdout="[]",
            returncode=0,
        )

        fetch_pr_review_comments("owner", "repo", 123)

        mock_run.assert_called_once()
        args = mock_run.call_args[0][0]
        assert "issues/123/comments" in args[2]

    @patch("pr_comments.fetcher.subprocess.run")
    def test_returns_list(self, mock_run):
        """Test that a list is returned."""
        mock_run.return_value = MagicMock(
            stdout="[]",
            returncode=0,
        )

        result = fetch_pr_review_comments("owner", "repo", 123)
        assert result == []


class TestFetchPRInfo:
    """Tests for fetch_pr_info function."""

    @patch("pr_comments.fetcher.subprocess.run")
    def test_fetches_pr_info(self, mock_run):
        """Test fetching PR information."""
        mock_run.return_value = MagicMock(
            stdout=json.dumps(SAMPLE_PR_INFO),
            returncode=0,
        )

        result = fetch_pr_info("ROKT", "canal", 123)

        assert result["title"] == "Add new processing features"
        assert result["html_url"] == "https://github.com/ROKT/canal/pull/123"

    @patch("pr_comments.fetcher.subprocess.run")
    def test_uses_pulls_endpoint(self, mock_run):
        """Test that pulls endpoint is used."""
        mock_run.return_value = MagicMock(
            stdout=json.dumps(SAMPLE_PR_INFO),
            returncode=0,
        )

        fetch_pr_info("owner", "repo", 456)

        args = mock_run.call_args[0][0]
        assert "repos/owner/repo/pulls/456" in args[2]

    @patch("pr_comments.fetcher.subprocess.run")
    def test_raises_error_on_failure(self, mock_run):
        """Test error handling for PR info."""
        mock_run.side_effect = subprocess.CalledProcessError(
            1, "gh", stderr="Not found"
        )

        with pytest.raises(GitHubAPIError):
            fetch_pr_info("owner", "repo", 999)

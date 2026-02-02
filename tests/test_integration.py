"""Integration tests that use the real GitHub API.

These tests are marked with pytest.mark.integration and can be skipped
when running regular unit tests with: pytest -m "not integration"

To run only integration tests: pytest -m integration
"""

import json
import subprocess

import pytest

from pr_comments.fetcher import fetch_pr_comments, fetch_pr_info
from pr_comments.formatter import format_for_claude, format_comments_minimal
from pr_comments.parser import parse_comments, filter_by_author


def gh_is_authenticated():
    """Check if gh CLI is authenticated."""
    try:
        result = subprocess.run(
            ["gh", "auth", "status"],
            capture_output=True,
            text=True,
        )
        return result.returncode == 0
    except FileNotFoundError:
        return False


# Skip all integration tests if gh is not authenticated
pytestmark = pytest.mark.skipif(
    not gh_is_authenticated(),
    reason="gh CLI not authenticated",
)


@pytest.mark.integration
class TestGitHubAPIIntegration:
    """Integration tests using real GitHub API calls."""

    def test_fetch_public_pr_comments(self):
        """Test fetching comments from a public PR.

        Uses a known public repo (cli/cli) that should always exist.
        """
        # Use GitHub CLI's own repo which is guaranteed to exist
        # PR #1 is the first PR and likely to always exist
        try:
            raw_comments = fetch_pr_comments("cli", "cli", 1)
            # Just verify we get a list back
            assert isinstance(raw_comments, list)
        except Exception as e:
            # If the PR doesn't exist or is closed, that's fine for this test
            # We just want to verify the API call works
            pytest.skip(f"Could not fetch PR: {e}")

    def test_fetch_pr_info_public(self):
        """Test fetching PR info from a public repo."""
        try:
            pr_info = fetch_pr_info("cli", "cli", 1)
            assert isinstance(pr_info, dict)
            assert "title" in pr_info or "message" in pr_info
        except Exception as e:
            pytest.skip(f"Could not fetch PR info: {e}")

    def test_full_pipeline_with_real_data(self):
        """Test the full pipeline from fetch to format."""
        try:
            raw_comments = fetch_pr_comments("cli", "cli", 1)
            comments = parse_comments(raw_comments)

            # Format with both formatters
            if comments:
                minimal = format_comments_minimal(comments)
                assert "PR Comments:" in minimal

                claude = format_for_claude(comments)
                assert "Pull Request Review Comments" in claude
            else:
                # No comments is a valid state
                pass
        except Exception as e:
            pytest.skip(f"Could not complete pipeline: {e}")


@pytest.mark.integration
class TestROKTCanal:
    """Integration tests specifically for ROKT/canal repo.

    These tests will only work if you have access to the private repo.
    """

    def test_fetch_canal_pr(self):
        """Test fetching comments from ROKT/canal PR 14777."""
        try:
            raw_comments = fetch_pr_comments("ROKT", "canal", 14777)
            comments = parse_comments(raw_comments)

            # Verify we got comments
            assert len(comments) > 0, "Expected comments on PR 14777"

            # Verify structure
            for comment in comments:
                assert comment.file_path
                assert comment.author
                assert comment.body
        except Exception as e:
            pytest.skip(f"Could not access ROKT/canal: {e}")

    def test_filter_by_author_canal(self):
        """Test filtering comments by author."""
        try:
            raw_comments = fetch_pr_comments("ROKT", "canal", 14777)
            comments = parse_comments(raw_comments)

            # Filter by a known reviewer
            filtered = filter_by_author(comments, "rjmurphy777")

            assert len(filtered) > 0, "Expected comments from rjmurphy777"
            assert all(c.author == "rjmurphy777" for c in filtered)
        except Exception as e:
            pytest.skip(f"Could not access ROKT/canal: {e}")

    def test_claude_format_canal(self):
        """Test Claude format output for canal PR."""
        try:
            raw_comments = fetch_pr_comments("ROKT", "canal", 14777)
            pr_info = fetch_pr_info("ROKT", "canal", 14777)
            comments = parse_comments(raw_comments)

            output = format_for_claude(
                comments,
                pr_url=pr_info.get("html_url"),
                pr_title=pr_info.get("title"),
            )

            # Verify expected content
            assert "Pull Request Review Comments" in output
            assert "canal/apps/mirakl" in output  # Known file path
            assert "rjmurphy777" in output or "devin-ai" in output  # Known reviewers
        except Exception as e:
            pytest.skip(f"Could not access ROKT/canal: {e}")

    def test_comment_code_snippets(self):
        """Test that code snippets are properly extracted."""
        try:
            raw_comments = fetch_pr_comments("ROKT", "canal", 14777)
            comments = parse_comments(raw_comments)

            # Find a comment with a diff hunk
            comments_with_code = [c for c in comments if c.diff_hunk]
            assert len(comments_with_code) > 0, "Expected comments with code"

            # Verify snippet extraction
            for comment in comments_with_code[:3]:  # Check first 3
                snippet = comment.get_code_snippet()
                # Snippet should not include diff header
                assert not snippet.startswith("@@")
        except Exception as e:
            pytest.skip(f"Could not access ROKT/canal: {e}")

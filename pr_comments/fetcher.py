"""Fetcher for GitHub PR comments using gh CLI."""

import json
import subprocess
from typing import Any


class GitHubAPIError(Exception):
    """Raised when GitHub API call fails."""

    pass


def fetch_pr_comments(owner: str, repo: str, pr_number: int) -> list[dict[str, Any]]:
    """Fetch PR comments using the gh CLI.

    Args:
        owner: Repository owner (e.g., 'ROKT').
        repo: Repository name (e.g., 'canal').
        pr_number: Pull request number.

    Returns:
        List of comment dicts from the GitHub API.

    Raises:
        GitHubAPIError: If the API call fails.
    """
    endpoint = f"repos/{owner}/{repo}/pulls/{pr_number}/comments"

    try:
        result = subprocess.run(
            ["gh", "api", endpoint],
            capture_output=True,
            text=True,
            check=True,
        )
        return json.loads(result.stdout)
    except subprocess.CalledProcessError as e:
        raise GitHubAPIError(f"Failed to fetch PR comments: {e.stderr}") from e
    except json.JSONDecodeError as e:
        raise GitHubAPIError(f"Failed to parse API response: {e}") from e


def fetch_pr_review_comments(owner: str, repo: str, pr_number: int) -> list[dict[str, Any]]:
    """Fetch PR review comments (issue comments) using the gh CLI.

    These are general comments on the PR, not tied to specific code lines.

    Args:
        owner: Repository owner.
        repo: Repository name.
        pr_number: Pull request number.

    Returns:
        List of comment dicts from the GitHub API.

    Raises:
        GitHubAPIError: If the API call fails.
    """
    endpoint = f"repos/{owner}/{repo}/issues/{pr_number}/comments"

    try:
        result = subprocess.run(
            ["gh", "api", endpoint],
            capture_output=True,
            text=True,
            check=True,
        )
        return json.loads(result.stdout)
    except subprocess.CalledProcessError as e:
        raise GitHubAPIError(f"Failed to fetch PR review comments: {e.stderr}") from e
    except json.JSONDecodeError as e:
        raise GitHubAPIError(f"Failed to parse API response: {e}") from e


def fetch_pr_info(owner: str, repo: str, pr_number: int) -> dict[str, Any]:
    """Fetch basic PR information.

    Args:
        owner: Repository owner.
        repo: Repository name.
        pr_number: Pull request number.

    Returns:
        PR info dict from the GitHub API.

    Raises:
        GitHubAPIError: If the API call fails.
    """
    endpoint = f"repos/{owner}/{repo}/pulls/{pr_number}"

    try:
        result = subprocess.run(
            ["gh", "api", endpoint],
            capture_output=True,
            text=True,
            check=True,
        )
        return json.loads(result.stdout)
    except subprocess.CalledProcessError as e:
        raise GitHubAPIError(f"Failed to fetch PR info: {e.stderr}") from e
    except json.JSONDecodeError as e:
        raise GitHubAPIError(f"Failed to parse API response: {e}") from e

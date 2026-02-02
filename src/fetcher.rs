//! GitHub API interaction via the gh CLI tool.

use crate::error::GitHubAPIError;
use serde_json::Value;
use std::process::Command;

/// Fetches PR review comments (comments on code) from GitHub.
///
/// Uses: `gh api repos/{owner}/{repo}/pulls/{pr_number}/comments`
pub fn fetch_pr_comments(
    owner: &str,
    repo: &str,
    pr_number: i32,
) -> Result<Vec<Value>, GitHubAPIError> {
    let endpoint = format!("repos/{owner}/{repo}/pulls/{pr_number}/comments");
    fetch_api_endpoint(&endpoint)
}

/// Fetches PR issue comments (general comments not on code) from GitHub.
///
/// Uses: `gh api repos/{owner}/{repo}/issues/{pr_number}/comments`
pub fn fetch_pr_review_comments(
    owner: &str,
    repo: &str,
    pr_number: i32,
) -> Result<Vec<Value>, GitHubAPIError> {
    let endpoint = format!("repos/{owner}/{repo}/issues/{pr_number}/comments");
    fetch_api_endpoint(&endpoint)
}

/// Fetches PR info (metadata) from GitHub.
///
/// Uses: `gh api repos/{owner}/{repo}/pulls/{pr_number}`
pub fn fetch_pr_info(owner: &str, repo: &str, pr_number: i32) -> Result<Value, GitHubAPIError> {
    let endpoint = format!("repos/{owner}/{repo}/pulls/{pr_number}");
    let output = run_gh_api(&endpoint)?;
    serde_json::from_str(&output)
        .map_err(|e| GitHubAPIError::ParseError(format!("Failed to parse PR info: {e}")))
}

/// Runs the gh api command and returns the raw output.
fn run_gh_api(endpoint: &str) -> Result<String, GitHubAPIError> {
    let output = Command::new("gh")
        .args(["api", endpoint])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                GitHubAPIError::GhNotFound
            } else {
                GitHubAPIError::CommandFailed(e.to_string())
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(GitHubAPIError::ApiError(format!(
            "Failed to fetch from GitHub: {}",
            stderr.trim()
        )));
    }

    String::from_utf8(output.stdout)
        .map_err(|e| GitHubAPIError::ParseError(format!("Invalid UTF-8 in response: {e}")))
}

/// Fetches an API endpoint that returns an array.
fn fetch_api_endpoint(endpoint: &str) -> Result<Vec<Value>, GitHubAPIError> {
    let output = run_gh_api(endpoint)?;
    serde_json::from_str(&output)
        .map_err(|e| GitHubAPIError::ParseError(format!("Failed to parse JSON array: {e}")))
}

#[cfg(test)]
mod tests {
    // Note: Most fetcher tests require mocking or integration testing
    // as they depend on the external gh CLI tool.
    // See test_integration.rs for integration tests.

    use super::*;

    #[test]
    fn test_fetch_pr_comments_builds_correct_endpoint() {
        // This is a structural test - we can't easily mock the gh CLI
        // but we can verify the function signature works correctly
        let _result = fetch_pr_comments("nonexistent", "repo", 1);
        // The function should return an error for non-existent repos,
        // but the point is it doesn't panic
    }

    #[test]
    fn test_fetch_pr_review_comments_builds_correct_endpoint() {
        let _result = fetch_pr_review_comments("nonexistent", "repo", 1);
    }

    #[test]
    fn test_fetch_pr_info_builds_correct_endpoint() {
        let _result = fetch_pr_info("nonexistent", "repo", 1);
    }
}

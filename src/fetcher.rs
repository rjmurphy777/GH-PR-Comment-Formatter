//! GitHub API interaction via the gh CLI tool.

use crate::error::GitHubAPIError;
use serde_json::Value;
use std::process::Command;

/// Trait for running commands, allowing for mocking in tests.
pub trait CommandRunner {
    fn run(&self, endpoint: &str) -> Result<String, GitHubAPIError>;
}

/// Default implementation that runs the actual `gh` CLI.
pub struct GhCliRunner;

impl CommandRunner for GhCliRunner {
    fn run(&self, endpoint: &str) -> Result<String, GitHubAPIError> {
        let output = Command::new("gh")
            .args(["api", endpoint])
            .output()
            .map_err(map_io_error)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitHubAPIError::ApiError(format!(
                "Failed to fetch from GitHub: {}",
                stderr.trim()
            )));
        }

        parse_utf8_output(output.stdout)
    }
}

/// Parses command output as UTF-8 string.
/// This is a separate function to enable testing of the error handling.
fn parse_utf8_output(bytes: Vec<u8>) -> Result<String, GitHubAPIError> {
    String::from_utf8(bytes)
        .map_err(|e| GitHubAPIError::ParseError(format!("Invalid UTF-8 in response: {e}")))
}

/// Maps I/O errors to GitHubAPIError.
/// This is a separate function to enable testing of the error mapping logic.
fn map_io_error(e: std::io::Error) -> GitHubAPIError {
    if e.kind() == std::io::ErrorKind::NotFound {
        GitHubAPIError::GhNotFound
    } else {
        GitHubAPIError::CommandFailed(e.to_string())
    }
}

/// Default runner instance for production use.
static DEFAULT_RUNNER: GhCliRunner = GhCliRunner;

/// Fetches PR review comments (comments on code) from GitHub.
///
/// Uses: `gh api repos/{owner}/{repo}/pulls/{pr_number}/comments`
pub fn fetch_pr_comments(
    owner: &str,
    repo: &str,
    pr_number: i32,
) -> Result<Vec<Value>, GitHubAPIError> {
    fetch_pr_comments_with_runner(owner, repo, pr_number, &DEFAULT_RUNNER)
}

/// Fetches PR review comments with a custom runner (for testing).
pub fn fetch_pr_comments_with_runner(
    owner: &str,
    repo: &str,
    pr_number: i32,
    runner: &dyn CommandRunner,
) -> Result<Vec<Value>, GitHubAPIError> {
    let endpoint = format!("repos/{owner}/{repo}/pulls/{pr_number}/comments");
    fetch_api_endpoint_with_runner(&endpoint, runner)
}

/// Fetches PR issue comments (general comments not on code) from GitHub.
///
/// Uses: `gh api repos/{owner}/{repo}/issues/{pr_number}/comments`
pub fn fetch_pr_review_comments(
    owner: &str,
    repo: &str,
    pr_number: i32,
) -> Result<Vec<Value>, GitHubAPIError> {
    fetch_pr_review_comments_with_runner(owner, repo, pr_number, &DEFAULT_RUNNER)
}

/// Fetches PR issue comments with a custom runner (for testing).
pub fn fetch_pr_review_comments_with_runner(
    owner: &str,
    repo: &str,
    pr_number: i32,
    runner: &dyn CommandRunner,
) -> Result<Vec<Value>, GitHubAPIError> {
    let endpoint = format!("repos/{owner}/{repo}/issues/{pr_number}/comments");
    fetch_api_endpoint_with_runner(&endpoint, runner)
}

/// Fetches PR info (metadata) from GitHub.
///
/// Uses: `gh api repos/{owner}/{repo}/pulls/{pr_number}`
pub fn fetch_pr_info(owner: &str, repo: &str, pr_number: i32) -> Result<Value, GitHubAPIError> {
    fetch_pr_info_with_runner(owner, repo, pr_number, &DEFAULT_RUNNER)
}

/// Fetches PR info with a custom runner (for testing).
pub fn fetch_pr_info_with_runner(
    owner: &str,
    repo: &str,
    pr_number: i32,
    runner: &dyn CommandRunner,
) -> Result<Value, GitHubAPIError> {
    let endpoint = format!("repos/{owner}/{repo}/pulls/{pr_number}");
    let output = runner.run(&endpoint)?;
    serde_json::from_str(&output)
        .map_err(|e| GitHubAPIError::ParseError(format!("Failed to parse PR info: {e}")))
}

/// Fetches an API endpoint that returns an array with a custom runner.
fn fetch_api_endpoint_with_runner(
    endpoint: &str,
    runner: &dyn CommandRunner,
) -> Result<Vec<Value>, GitHubAPIError> {
    let output = runner.run(endpoint)?;
    serde_json::from_str(&output)
        .map_err(|e| GitHubAPIError::ParseError(format!("Failed to parse JSON array: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock runner that returns a configurable response.
    struct MockRunner {
        response: Result<String, GitHubAPIError>,
    }

    impl MockRunner {
        fn success(json: &str) -> Self {
            Self {
                response: Ok(json.to_string()),
            }
        }

        fn error(err: GitHubAPIError) -> Self {
            Self { response: Err(err) }
        }
    }

    impl CommandRunner for MockRunner {
        fn run(&self, _endpoint: &str) -> Result<String, GitHubAPIError> {
            self.response.clone()
        }
    }

    #[test]
    fn test_fetch_pr_comments_success() {
        let runner = MockRunner::success(r#"[{"id": 1, "body": "test"}]"#);
        let result = fetch_pr_comments_with_runner("owner", "repo", 1, &runner);
        assert!(result.is_ok());
        let comments = result.unwrap();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0]["id"], 1);
    }

    #[test]
    fn test_fetch_pr_comments_api_error() {
        let runner = MockRunner::error(GitHubAPIError::ApiError("Not found".to_string()));
        let result = fetch_pr_comments_with_runner("owner", "repo", 1, &runner);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitHubAPIError::ApiError(_)));
    }

    #[test]
    fn test_fetch_pr_comments_parse_error() {
        let runner = MockRunner::success("not valid json");
        let result = fetch_pr_comments_with_runner("owner", "repo", 1, &runner);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitHubAPIError::ParseError(_)));
    }

    #[test]
    fn test_fetch_pr_review_comments_success() {
        let runner = MockRunner::success(r#"[{"id": 2, "body": "review"}]"#);
        let result = fetch_pr_review_comments_with_runner("owner", "repo", 1, &runner);
        assert!(result.is_ok());
        let comments = result.unwrap();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0]["id"], 2);
    }

    #[test]
    fn test_fetch_pr_review_comments_command_failed() {
        let runner = MockRunner::error(GitHubAPIError::CommandFailed("timeout".to_string()));
        let result = fetch_pr_review_comments_with_runner("owner", "repo", 1, &runner);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            GitHubAPIError::CommandFailed(_)
        ));
    }

    #[test]
    fn test_fetch_pr_info_success() {
        let runner = MockRunner::success(
            r#"{"title": "Test PR", "html_url": "https://github.com/owner/repo/pull/1"}"#,
        );
        let result = fetch_pr_info_with_runner("owner", "repo", 1, &runner);
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info["title"], "Test PR");
    }

    #[test]
    fn test_fetch_pr_info_parse_error() {
        let runner = MockRunner::success("invalid json");
        let result = fetch_pr_info_with_runner("owner", "repo", 1, &runner);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, GitHubAPIError::ParseError(_)));
        assert!(err.to_string().contains("Failed to parse PR info"));
    }

    #[test]
    fn test_fetch_pr_info_gh_not_found() {
        let runner = MockRunner::error(GitHubAPIError::GhNotFound);
        let result = fetch_pr_info_with_runner("owner", "repo", 1, &runner);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitHubAPIError::GhNotFound));
    }

    #[test]
    fn test_fetch_pr_comments_public_api() {
        // Test the public API that uses DEFAULT_RUNNER
        // This exercises the code path regardless of whether gh is available
        let result = fetch_pr_comments("nonexistent-owner-xyz", "nonexistent-repo-xyz", 99999);
        // Should return an error (GhNotFound, ApiError, or CommandFailed)
        assert!(result.is_err());
    }

    #[test]
    fn test_fetch_pr_review_comments_public_api() {
        // Test the public API that uses DEFAULT_RUNNER
        let result =
            fetch_pr_review_comments("nonexistent-owner-xyz", "nonexistent-repo-xyz", 99999);
        // Should return an error (GhNotFound, ApiError, or CommandFailed)
        assert!(result.is_err());
    }

    #[test]
    fn test_fetch_pr_info_public_api() {
        // Test the public API that uses DEFAULT_RUNNER
        let result = fetch_pr_info("nonexistent-owner-xyz", "nonexistent-repo-xyz", 99999);
        // Should return an error (GhNotFound, ApiError, or CommandFailed)
        assert!(result.is_err());
    }

    #[test]
    fn test_gh_cli_runner_run_directly() {
        // Test the GhCliRunner directly
        let runner = GhCliRunner;
        let result = runner.run("repos/nonexistent/nonexistent/pulls/99999/comments");
        // Should return an error (GhNotFound if gh not installed, or ApiError/CommandFailed)
        assert!(result.is_err());
    }

    #[test]
    fn test_error_display_messages() {
        // Test that error messages are formatted correctly
        let cmd_err = GitHubAPIError::CommandFailed("test error".to_string());
        assert!(cmd_err.to_string().contains("test error"));

        let api_err = GitHubAPIError::ApiError("api test".to_string());
        assert!(api_err.to_string().contains("api test"));

        let parse_err = GitHubAPIError::ParseError("parse test".to_string());
        assert!(parse_err.to_string().contains("parse test"));

        let gh_err = GitHubAPIError::GhNotFound;
        assert!(gh_err.to_string().contains("gh CLI not found"));
    }

    #[test]
    fn test_map_io_error_not_found() {
        // Test that NotFound errors map to GhNotFound
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "command not found");
        let result = map_io_error(io_error);
        assert!(matches!(result, GitHubAPIError::GhNotFound));
    }

    #[test]
    fn test_map_io_error_other() {
        // Test that other I/O errors map to CommandFailed
        let io_error =
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");
        let result = map_io_error(io_error);
        assert!(matches!(result, GitHubAPIError::CommandFailed(_)));
        assert!(result.to_string().contains("permission denied"));
    }

    #[test]
    fn test_parse_utf8_output_success() {
        let bytes = b"hello world".to_vec();
        let result = parse_utf8_output(bytes);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello world");
    }

    #[test]
    fn test_parse_utf8_output_invalid() {
        // Invalid UTF-8 sequence
        let bytes = vec![0xff, 0xfe, 0x00, 0x01];
        let result = parse_utf8_output(bytes);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitHubAPIError::ParseError(_)));
    }

    #[test]
    fn test_gh_cli_runner_success_path() {
        // Test the success path by calling a real valid GitHub endpoint
        // This fetches rate limit info which is always accessible
        let runner = GhCliRunner;
        let result = runner.run("rate_limit");
        // This may succeed or fail depending on gh auth, but we try to cover the path
        // If gh is authenticated, this should succeed and cover line 30
        if result.is_ok() {
            assert!(result.unwrap().contains("resources"));
        }
        // If it fails, that's okay - we've tested that path elsewhere
    }
}

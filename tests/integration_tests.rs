//! Integration tests for the pr-comments CLI tool.
//!
//! These tests require the `gh` CLI to be installed and authenticated.
//! Run with: cargo test --test integration_tests

use std::process::Command;

/// Check if gh CLI is authenticated
fn gh_is_authenticated() -> bool {
    Command::new("gh")
        .args(["auth", "status"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get the path to the built binary
fn binary_path() -> String {
    // Try release first, then debug
    let release_path = "target/release/pr-comments";
    let debug_path = "target/debug/pr-comments";

    if std::path::Path::new(release_path).exists() {
        release_path.to_string()
    } else {
        debug_path.to_string()
    }
}

/// Macro to skip test if gh is not authenticated
macro_rules! require_gh_auth {
    () => {
        if !gh_is_authenticated() {
            eprintln!("Skipping test: gh CLI not authenticated");
            return;
        }
    };
}

mod cli_tests {
    use super::*;

    #[test]
    fn test_help_flag() {
        let output = Command::new(binary_path())
            .arg("--help")
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("pr-comments"));
        assert!(stdout.contains("--format"));
        assert!(stdout.contains("--author"));
        assert!(stdout.contains("--most-recent"));
    }

    #[test]
    fn test_version_flag() {
        let output = Command::new(binary_path())
            .arg("--version")
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("0.1.0"));
    }

    #[test]
    fn test_invalid_url_error() {
        let output = Command::new(binary_path())
            .arg("invalid-url")
            .output()
            .expect("Failed to execute command");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Cannot parse PR URL"));
    }

    #[test]
    fn test_missing_args_error() {
        let output = Command::new(binary_path())
            .output()
            .expect("Failed to execute command");

        // Should fail because no PR is specified
        assert!(!output.status.success());
    }

    #[test]
    fn test_invalid_format_error() {
        let output = Command::new(binary_path())
            .args(["owner/repo#123", "--format", "invalid"])
            .output()
            .expect("Failed to execute command");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("invalid") || stderr.contains("possible values"));
    }
}

mod github_api_tests {
    use super::*;

    #[test]
    fn test_fetch_public_pr_claude_format() {
        require_gh_auth!();

        // Use cli/cli#1 which should always exist
        let output = Command::new(binary_path())
            .args(["cli/cli#1", "--format", "claude"])
            .output()
            .expect("Failed to execute command");

        // Should succeed even if no comments
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "stderr: {stderr}");
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Either has comments or shows "No comments found"
        assert!(
            stdout.contains("Pull Request Review Comments") || stdout.contains("No comments found"),
            "Unexpected output: {stdout}"
        );
    }

    #[test]
    fn test_fetch_public_pr_json_format() {
        require_gh_auth!();

        let output = Command::new(binary_path())
            .args(["cli/cli#1", "--format", "json"])
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "stderr: {stderr}");
        let stdout = String::from_utf8_lossy(&output.stdout);
        // JSON output should be valid (either array or empty)
        assert!(
            stdout.starts_with('[') || stdout.contains("No comments"),
            "Expected JSON array, got: {stdout}"
        );
    }

    #[test]
    fn test_fetch_public_pr_minimal_format() {
        require_gh_auth!();

        let output = Command::new(binary_path())
            .args(["cli/cli#1", "--format", "minimal"])
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "stderr: {stderr}");
    }

    #[test]
    fn test_fetch_public_pr_grouped_format() {
        require_gh_auth!();

        let output = Command::new(binary_path())
            .args(["cli/cli#1", "--format", "grouped"])
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "stderr: {stderr}");
    }

    #[test]
    fn test_fetch_public_pr_flat_format() {
        require_gh_auth!();

        let output = Command::new(binary_path())
            .args(["cli/cli#1", "--format", "flat"])
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "stderr: {stderr}");
    }

    #[test]
    fn test_fetch_with_full_url() {
        require_gh_auth!();

        let output = Command::new(binary_path())
            .args(["https://github.com/cli/cli/pull/1", "--format", "minimal"])
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "stderr: {stderr}");
    }

    #[test]
    fn test_fetch_with_explicit_args() {
        require_gh_auth!();

        let output = Command::new(binary_path())
            .args([
                "--owner",
                "cli",
                "--repo",
                "cli",
                "--pr-number",
                "1",
                "--format",
                "minimal",
            ])
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "stderr: {stderr}");
    }

    #[test]
    fn test_nonexistent_pr_error() {
        require_gh_auth!();

        let output = Command::new(binary_path())
            .args(["cli/cli#999999999", "--format", "minimal"])
            .output()
            .expect("Failed to execute command");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("404") || stderr.contains("Not Found") || stderr.contains("Error"));
    }

    #[test]
    fn test_author_filter() {
        require_gh_auth!();

        // Filter by a nonexistent author should still succeed (just empty)
        let output = Command::new(binary_path())
            .args([
                "cli/cli#1",
                "--author",
                "nonexistent-user-12345",
                "--format",
                "json",
            ])
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "stderr: {stderr}");
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should be empty array since no comments from this author
        assert!(
            stdout.trim() == "[]" || stdout.contains("No comments"),
            "Got: {stdout}"
        );
    }

    #[test]
    fn test_no_snippet_flag() {
        require_gh_auth!();

        let output = Command::new(binary_path())
            .args(["cli/cli#1", "--no-snippet", "--format", "claude"])
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "stderr: {stderr}");
    }

    #[test]
    fn test_snippet_lines_option() {
        require_gh_auth!();

        let output = Command::new(binary_path())
            .args(["cli/cli#1", "--snippet-lines", "5", "--format", "claude"])
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "stderr: {stderr}");
    }

    #[test]
    fn test_most_recent_flag() {
        require_gh_auth!();

        let output = Command::new(binary_path())
            .args(["cli/cli#1", "--most-recent", "--format", "json"])
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "stderr: {stderr}");
    }
}

mod output_file_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_output_to_file() {
        if !gh_is_authenticated() {
            eprintln!("Skipping test: gh CLI not authenticated");
            return;
        }

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let output_path = temp_dir.path().join("output.md");

        let output = Command::new(binary_path())
            .args([
                "cli/cli#1",
                "--format",
                "claude",
                "--output",
                output_path.to_str().unwrap(),
            ])
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "stderr: {stderr}");

        // Verify file was created
        assert!(output_path.exists(), "Output file was not created");

        // Verify file has content
        let content = fs::read_to_string(&output_path).expect("Failed to read output file");
        assert!(!content.is_empty(), "Output file is empty");
    }

    #[test]
    fn test_output_to_file_json() {
        if !gh_is_authenticated() {
            eprintln!("Skipping test: gh CLI not authenticated");
            return;
        }

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let output_path = temp_dir.path().join("output.json");

        let output = Command::new(binary_path())
            .args([
                "cli/cli#1",
                "--format",
                "json",
                "--output",
                output_path.to_str().unwrap(),
            ])
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "stderr: {stderr}");

        // Verify file was created and contains valid JSON
        let content = fs::read_to_string(&output_path).expect("Failed to read output file");
        // Should be a JSON array
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content);
        assert!(parsed.is_ok(), "Output is not valid JSON: {content}");
    }
}

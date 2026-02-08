//! JSON parsing and comment filtering functions.

use crate::error::GitHubAPIError;
use crate::models::{
    CheckConclusion, CheckStatus, CheckType, ChecksReport, PRComment, RollupState,
};
use crate::sanitizer::strip_html;
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;

/// Parses a GitHub ISO 8601 datetime string into a DateTime<Utc>.
///
/// Handles formats like "2026-01-30T23:06:02Z" and "2026-01-30T23:06:02.123Z"
pub fn parse_datetime(dt_str: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    // RFC3339 handles both with and without fractional seconds, and with Z suffix
    DateTime::parse_from_rfc3339(dt_str).map(|dt| dt.with_timezone(&Utc))
}

/// Parses a single comment from GitHub API JSON into a PRComment.
pub fn parse_comment(comment_data: &Value) -> Option<PRComment> {
    let id = comment_data.get("id")?.as_i64()?;

    // GraphQL node ID for this comment (used for replying via GraphQL API)
    let node_id = comment_data
        .get("node_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let file_path = comment_data
        .get("path")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Try line first, then fall back to original_line
    let line_number = comment_data
        .get("line")
        .and_then(|v| v.as_i64())
        .or_else(|| comment_data.get("original_line").and_then(|v| v.as_i64()))
        .map(|v| v as i32);

    // Try start_line first, then fall back to original_start_line
    let start_line = comment_data
        .get("start_line")
        .and_then(|v| v.as_i64())
        .or_else(|| {
            comment_data
                .get("original_start_line")
                .and_then(|v| v.as_i64())
        })
        .map(|v| v as i32);

    // Extract author from user.login, default to "unknown"
    let author = comment_data
        .get("user")
        .and_then(|u| u.get("login"))
        .and_then(|l| l.as_str())
        .unwrap_or("unknown")
        .to_string();

    let raw_body = comment_data
        .get("body")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let body = strip_html(raw_body).into_owned();

    let created_at_str = comment_data.get("created_at")?.as_str()?;
    let created_at = parse_datetime(created_at_str).ok()?;

    let updated_at_str = comment_data.get("updated_at")?.as_str()?;
    let updated_at = parse_datetime(updated_at_str).ok()?;

    let diff_hunk = comment_data
        .get("diff_hunk")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let html_url = comment_data
        .get("html_url")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Some(PRComment::new(
        id,
        node_id,
        file_path,
        line_number,
        start_line,
        author,
        body,
        created_at,
        updated_at,
        diff_hunk,
        html_url,
    ))
}

/// Parses multiple comments from GitHub API JSON.
pub fn parse_comments(comments_data: &[Value]) -> Vec<PRComment> {
    comments_data.iter().filter_map(parse_comment).collect()
}

/// Parses a single review from GitHub API JSON into a PRComment.
///
/// Reviews are top-level comments attached to a review submission,
/// not to specific lines of code. Only reviews with non-empty body are returned.
pub fn parse_review_comment(review_data: &Value) -> Option<PRComment> {
    let id = review_data.get("id")?.as_i64()?;

    // GraphQL node ID for this review
    let node_id = review_data
        .get("node_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Only include reviews that have a body (non-empty comment)
    let raw_body = review_data.get("body").and_then(|v| v.as_str())?;
    if raw_body.trim().is_empty() {
        return None;
    }
    let body = strip_html(raw_body).into_owned();

    // Extract author from user.login
    let author = review_data
        .get("user")
        .and_then(|u| u.get("login"))
        .and_then(|l| l.as_str())
        .unwrap_or("unknown")
        .to_string();

    let submitted_at_str = review_data.get("submitted_at")?.as_str()?;
    let submitted_at = parse_datetime(submitted_at_str).ok()?;

    let html_url = review_data
        .get("html_url")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Review-level comments don't have file paths or line numbers
    Some(PRComment::new(
        id,
        node_id,
        String::new(), // No file path for review-level comments
        None,          // No line number
        None,          // No start line
        author,
        body,
        submitted_at,
        submitted_at,  // Use submitted_at for both created and updated
        String::new(), // No diff hunk
        html_url,
    ))
}

/// Parses multiple reviews from GitHub API JSON into PRComments.
///
/// Only reviews with non-empty body text are included.
pub fn parse_review_comments(reviews_data: &[Value]) -> Vec<PRComment> {
    reviews_data
        .iter()
        .filter_map(parse_review_comment)
        .collect()
}

/// Filters comments by author username.
///
/// If author is None or empty, returns all comments.
pub fn filter_by_author(comments: Vec<PRComment>, author: Option<&str>) -> Vec<PRComment> {
    match author {
        Some(a) if !a.is_empty() => comments.into_iter().filter(|c| c.author == a).collect(),
        _ => comments,
    }
}

/// Gets the most recent comment per file.
///
/// Groups comments by file_path and keeps only the most recently updated one.
pub fn get_most_recent_per_file(comments: Vec<PRComment>) -> Vec<PRComment> {
    let mut file_map: HashMap<String, PRComment> = HashMap::new();

    for comment in comments {
        let file_path = comment.file_path.clone();
        if let Some(existing) = file_map.get(&file_path) {
            if comment.updated_at > existing.updated_at {
                file_map.insert(file_path, comment);
            }
        } else {
            file_map.insert(file_path, comment);
        }
    }

    file_map.into_values().collect()
}

/// Groups comments by file path.
pub fn group_by_file(comments: &[PRComment]) -> HashMap<String, Vec<&PRComment>> {
    let mut grouped: HashMap<String, Vec<&PRComment>> = HashMap::new();

    for comment in comments {
        grouped
            .entry(comment.file_path.clone())
            .or_default()
            .push(comment);
    }

    grouped
}

/// Parses a GraphQL response into a ChecksReport.
pub fn parse_checks_response(response: &Value) -> Result<ChecksReport, GitHubAPIError> {
    let pr = response
        .pointer("/data/repository/pullRequest")
        .ok_or_else(|| {
            GitHubAPIError::ParseError("Missing pullRequest in GraphQL response".to_string())
        })?;

    let pr_title = pr.get("title").and_then(|v| v.as_str()).map(String::from);
    let pr_url = pr.get("url").and_then(|v| v.as_str()).map(String::from);

    let commit = pr
        .pointer("/commits/nodes/0/commit")
        .ok_or_else(|| GitHubAPIError::ParseError("Missing commit data".to_string()))?;

    let rollup = commit.get("statusCheckRollup");

    let rollup_state = rollup
        .and_then(|r| r.get("state"))
        .and_then(|s| s.as_str())
        .map(parse_rollup_state)
        .unwrap_or(RollupState::Unknown);

    let checks = rollup
        .and_then(|r| r.pointer("/contexts/nodes"))
        .and_then(|n| n.as_array())
        .map(|nodes| nodes.iter().filter_map(parse_check_node).collect())
        .unwrap_or_default();

    Ok(ChecksReport {
        pr_title,
        pr_url,
        rollup_state,
        checks,
    })
}

/// Parses a single check node, dispatching on __typename.
fn parse_check_node(node: &Value) -> Option<CheckStatus> {
    let typename = node.get("__typename")?.as_str()?;
    match typename {
        "CheckRun" => parse_check_run(node),
        "StatusContext" => parse_status_context(node),
        _ => None,
    }
}

/// Parses a CheckRun node from the GraphQL response.
fn parse_check_run(node: &Value) -> Option<CheckStatus> {
    let name = node.get("name")?.as_str()?.to_string();
    let status = node.get("status").and_then(|v| v.as_str()).unwrap_or("");
    let conclusion = node
        .get("conclusion")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let check_conclusion = parse_check_conclusion(status, conclusion);
    let required = node
        .get("isRequired")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let details_url = node
        .get("detailsUrl")
        .and_then(|v| v.as_str())
        .map(String::from);
    let started_at = node
        .get("startedAt")
        .and_then(|v| v.as_str())
        .and_then(|s| parse_datetime(s).ok());
    let completed_at = node
        .get("completedAt")
        .and_then(|v| v.as_str())
        .and_then(|s| parse_datetime(s).ok());

    let app_name = node
        .pointer("/checkSuite/app/slug")
        .and_then(|v| v.as_str())
        .map(String::from);
    let workflow_name = node
        .pointer("/checkSuite/workflowRun/workflow/name")
        .and_then(|v| v.as_str())
        .map(String::from);

    Some(CheckStatus {
        name,
        conclusion: check_conclusion,
        required,
        description: None,
        details_url,
        started_at,
        completed_at,
        check_type: CheckType::CheckRun,
        workflow_name,
        app_name,
    })
}

/// Parses a StatusContext node from the GraphQL response.
fn parse_status_context(node: &Value) -> Option<CheckStatus> {
    let name = node.get("context")?.as_str()?.to_string();
    let state = node.get("state").and_then(|v| v.as_str()).unwrap_or("");
    let conclusion = parse_status_state(state);
    let required = node
        .get("isRequired")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let description = node
        .get("description")
        .and_then(|v| v.as_str())
        .map(String::from);
    let details_url = node
        .get("targetUrl")
        .and_then(|v| v.as_str())
        .map(String::from);
    let created_at = node
        .get("createdAt")
        .and_then(|v| v.as_str())
        .and_then(|s| parse_datetime(s).ok());

    Some(CheckStatus {
        name,
        conclusion,
        required,
        description,
        details_url,
        started_at: created_at,
        completed_at: None,
        check_type: CheckType::StatusContext,
        workflow_name: None,
        app_name: None,
    })
}

/// Maps CheckRun status + conclusion to a CheckConclusion.
fn parse_check_conclusion(status: &str, conclusion: &str) -> CheckConclusion {
    // If status is not COMPLETED, the check is still running
    if status != "COMPLETED" && !status.is_empty() {
        return match status {
            "IN_PROGRESS" | "QUEUED" | "REQUESTED" | "WAITING" | "PENDING" => {
                CheckConclusion::Pending
            }
            _ => CheckConclusion::Unknown,
        };
    }

    match conclusion {
        "SUCCESS" => CheckConclusion::Success,
        "FAILURE" => CheckConclusion::Failure,
        "SKIPPED" => CheckConclusion::Skipped,
        "CANCELLED" => CheckConclusion::Cancelled,
        "TIMED_OUT" => CheckConclusion::TimedOut,
        "ACTION_REQUIRED" => CheckConclusion::ActionRequired,
        "NEUTRAL" => CheckConclusion::Neutral,
        "STALE" => CheckConclusion::Stale,
        "" if status.is_empty() => CheckConclusion::Pending,
        _ => CheckConclusion::Unknown,
    }
}

/// Maps StatusContext state string to a CheckConclusion.
fn parse_status_state(state: &str) -> CheckConclusion {
    match state {
        "SUCCESS" => CheckConclusion::Success,
        "FAILURE" | "ERROR" => CheckConclusion::Failure,
        "PENDING" | "EXPECTED" => CheckConclusion::Pending,
        _ => CheckConclusion::Unknown,
    }
}

/// Maps rollup state string to a RollupState.
fn parse_rollup_state(state: &str) -> RollupState {
    match state {
        "SUCCESS" => RollupState::Success,
        "FAILURE" => RollupState::Failure,
        "PENDING" => RollupState::Pending,
        "ERROR" => RollupState::Error,
        "EXPECTED" => RollupState::Expected,
        _ => RollupState::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, TimeZone, Timelike};
    use serde_json::json;

    #[test]
    fn test_parse_datetime_github_format() {
        let result = parse_datetime("2026-01-30T23:06:02Z").unwrap();
        assert_eq!(result.year(), 2026);
        assert_eq!(result.month(), 1);
        assert_eq!(result.day(), 30);
        assert_eq!(result.hour(), 23);
        assert_eq!(result.minute(), 6);
        assert_eq!(result.second(), 2);
    }

    #[test]
    fn test_parse_datetime_with_milliseconds() {
        let result = parse_datetime("2026-01-30T23:06:02.123Z").unwrap();
        assert_eq!(result.year(), 2026);
    }

    #[test]
    fn test_parse_comment_minimal() {
        let data = json!({
            "id": 123,
            "path": "src/main.rs",
            "line": 42,
            "user": {"login": "testuser"},
            "body": "Test comment",
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-01-15T10:30:00Z",
            "diff_hunk": "@@ -1,5 +1,5 @@\n test",
            "html_url": "https://github.com/owner/repo/pull/1#discussion_r123"
        });

        let comment = parse_comment(&data).unwrap();
        assert_eq!(comment.id, 123);
        assert_eq!(comment.file_path, "src/main.rs");
        assert_eq!(comment.line_number, Some(42));
        assert_eq!(comment.author, "testuser");
        assert_eq!(comment.body, "Test comment");
    }

    #[test]
    fn test_parse_comment_with_range() {
        let data = json!({
            "id": 123,
            "path": "src/main.rs",
            "line": 20,
            "start_line": 10,
            "user": {"login": "testuser"},
            "body": "Test comment",
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-01-15T10:30:00Z",
            "diff_hunk": "",
            "html_url": ""
        });

        let comment = parse_comment(&data).unwrap();
        assert_eq!(comment.line_number, Some(20));
        assert_eq!(comment.start_line, Some(10));
    }

    #[test]
    fn test_parse_comment_fallback_to_original_line() {
        let data = json!({
            "id": 123,
            "path": "src/main.rs",
            "original_line": 42,
            "user": {"login": "testuser"},
            "body": "Test comment",
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-01-15T10:30:00Z",
            "diff_hunk": "",
            "html_url": ""
        });

        let comment = parse_comment(&data).unwrap();
        assert_eq!(comment.line_number, Some(42));
    }

    #[test]
    fn test_parse_comment_missing_user() {
        let data = json!({
            "id": 123,
            "path": "src/main.rs",
            "body": "Test comment",
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-01-15T10:30:00Z",
            "diff_hunk": "",
            "html_url": ""
        });

        let comment = parse_comment(&data).unwrap();
        assert_eq!(comment.author, "unknown");
    }

    #[test]
    fn test_parse_comment_bot() {
        let data = json!({
            "id": 123,
            "path": "src/main.rs",
            "user": {"login": "devin-ai-integration[bot]"},
            "body": "Bot comment",
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-01-15T10:30:00Z",
            "diff_hunk": "",
            "html_url": ""
        });

        let comment = parse_comment(&data).unwrap();
        assert_eq!(comment.author, "devin-ai-integration[bot]");
    }

    #[test]
    fn test_parse_comments_multiple() {
        let data = vec![
            json!({
                "id": 1,
                "path": "file1.rs",
                "user": {"login": "user1"},
                "body": "Comment 1",
                "created_at": "2024-01-15T10:30:00Z",
                "updated_at": "2024-01-15T10:30:00Z",
                "diff_hunk": "",
                "html_url": ""
            }),
            json!({
                "id": 2,
                "path": "file2.rs",
                "user": {"login": "user2"},
                "body": "Comment 2",
                "created_at": "2024-01-15T10:30:00Z",
                "updated_at": "2024-01-15T10:30:00Z",
                "diff_hunk": "",
                "html_url": ""
            }),
        ];

        let comments = parse_comments(&data);
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0].id, 1);
        assert_eq!(comments[1].id, 2);
    }

    #[test]
    fn test_parse_comments_empty() {
        let comments = parse_comments(&[]);
        assert!(comments.is_empty());
    }

    fn create_test_comments() -> Vec<PRComment> {
        vec![
            PRComment::new(
                1,
                Some("PRRC_test1".to_string()),
                "file1.rs".to_string(),
                Some(10),
                None,
                "user1".to_string(),
                "Comment 1".to_string(),
                Utc.with_ymd_and_hms(2024, 1, 15, 10, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2024, 1, 15, 10, 0, 0).unwrap(),
                "".to_string(),
                "".to_string(),
            ),
            PRComment::new(
                2,
                Some("PRRC_test2".to_string()),
                "file1.rs".to_string(),
                Some(20),
                None,
                "user2".to_string(),
                "Comment 2".to_string(),
                Utc.with_ymd_and_hms(2024, 1, 15, 11, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2024, 1, 15, 11, 0, 0).unwrap(),
                "".to_string(),
                "".to_string(),
            ),
            PRComment::new(
                3,
                Some("PRRC_test3".to_string()),
                "file2.rs".to_string(),
                Some(5),
                None,
                "user1".to_string(),
                "Comment 3".to_string(),
                Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap(),
                "".to_string(),
                "".to_string(),
            ),
        ]
    }

    #[test]
    fn test_filter_by_author_none() {
        let comments = create_test_comments();
        let filtered = filter_by_author(comments.clone(), None);
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn test_filter_by_author_empty_string() {
        let comments = create_test_comments();
        let filtered = filter_by_author(comments.clone(), Some(""));
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn test_filter_by_author_specific() {
        let comments = create_test_comments();
        let filtered = filter_by_author(comments, Some("user1"));
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|c| c.author == "user1"));
    }

    #[test]
    fn test_filter_by_author_nonexistent() {
        let comments = create_test_comments();
        let filtered = filter_by_author(comments, Some("nonexistent"));
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_get_most_recent_per_file() {
        let comments = create_test_comments();
        let most_recent = get_most_recent_per_file(comments);
        assert_eq!(most_recent.len(), 2); // file1.rs and file2.rs

        // Find the file1.rs comment
        let file1_comment = most_recent
            .iter()
            .find(|c| c.file_path == "file1.rs")
            .unwrap();
        assert_eq!(file1_comment.id, 2); // The more recent one
    }

    #[test]
    fn test_get_most_recent_per_file_empty() {
        let most_recent = get_most_recent_per_file(vec![]);
        assert!(most_recent.is_empty());
    }

    #[test]
    fn test_group_by_file() {
        let comments = create_test_comments();
        let grouped = group_by_file(&comments);

        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped.get("file1.rs").unwrap().len(), 2);
        assert_eq!(grouped.get("file2.rs").unwrap().len(), 1);
    }

    #[test]
    fn test_group_by_file_empty() {
        let grouped = group_by_file(&[]);
        assert!(grouped.is_empty());
    }

    #[test]
    fn test_parse_review_comment_success() {
        let data = json!({
            "id": 12345,
            "body": "This is a review-level comment",
            "user": {"login": "reviewer"},
            "submitted_at": "2024-01-15T10:30:00Z",
            "html_url": "https://github.com/owner/repo/pull/1#pullrequestreview-12345",
            "state": "COMMENTED"
        });

        let comment = parse_review_comment(&data).unwrap();
        assert_eq!(comment.id, 12345);
        assert_eq!(comment.body, "This is a review-level comment");
        assert_eq!(comment.author, "reviewer");
        assert!(comment.file_path.is_empty());
        assert!(comment.line_number.is_none());
        assert!(comment.diff_hunk.is_empty());
    }

    #[test]
    fn test_parse_review_comment_empty_body() {
        let data = json!({
            "id": 12345,
            "body": "",
            "user": {"login": "reviewer"},
            "submitted_at": "2024-01-15T10:30:00Z",
            "html_url": "https://github.com/owner/repo/pull/1#pullrequestreview-12345"
        });

        let comment = parse_review_comment(&data);
        assert!(comment.is_none());
    }

    #[test]
    fn test_parse_review_comment_whitespace_only_body() {
        let data = json!({
            "id": 12345,
            "body": "   \n\t  ",
            "user": {"login": "reviewer"},
            "submitted_at": "2024-01-15T10:30:00Z",
            "html_url": "https://github.com/owner/repo/pull/1#pullrequestreview-12345"
        });

        let comment = parse_review_comment(&data);
        assert!(comment.is_none());
    }

    #[test]
    fn test_parse_review_comment_null_body() {
        let data = json!({
            "id": 12345,
            "body": null,
            "user": {"login": "reviewer"},
            "submitted_at": "2024-01-15T10:30:00Z",
            "html_url": "https://github.com/owner/repo/pull/1#pullrequestreview-12345"
        });

        let comment = parse_review_comment(&data);
        assert!(comment.is_none());
    }

    #[test]
    fn test_parse_review_comment_missing_user() {
        let data = json!({
            "id": 12345,
            "body": "Some review comment",
            "submitted_at": "2024-01-15T10:30:00Z",
            "html_url": "https://github.com/owner/repo/pull/1#pullrequestreview-12345"
        });

        let comment = parse_review_comment(&data).unwrap();
        assert_eq!(comment.author, "unknown");
    }

    #[test]
    fn test_parse_review_comments_multiple() {
        let data = vec![
            json!({
                "id": 1,
                "body": "First review",
                "user": {"login": "user1"},
                "submitted_at": "2024-01-15T10:30:00Z",
                "html_url": ""
            }),
            json!({
                "id": 2,
                "body": "", // Empty body - should be filtered out
                "user": {"login": "user2"},
                "submitted_at": "2024-01-15T11:30:00Z",
                "html_url": ""
            }),
            json!({
                "id": 3,
                "body": "Third review",
                "user": {"login": "user3"},
                "submitted_at": "2024-01-15T12:30:00Z",
                "html_url": ""
            }),
        ];

        let comments = parse_review_comments(&data);
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0].id, 1);
        assert_eq!(comments[1].id, 3);
    }

    #[test]
    fn test_parse_review_comments_empty() {
        let comments = parse_review_comments(&[]);
        assert!(comments.is_empty());
    }

    #[test]
    fn test_parse_review_comment_strips_html() {
        let data = json!({
            "id": 12345,
            "body": "<p>This is a <strong>review</strong> comment</p>",
            "user": {"login": "reviewer"},
            "submitted_at": "2024-01-15T10:30:00Z",
            "html_url": ""
        });

        let comment = parse_review_comment(&data).unwrap();
        assert!(!comment.body.contains("<p>"));
        assert!(!comment.body.contains("<strong>"));
    }

    #[test]
    fn test_parse_comment_with_node_id() {
        let data = json!({
            "id": 123,
            "node_id": "PRRC_kwDOE2CVus5test",
            "path": "src/main.rs",
            "line": 42,
            "user": {"login": "testuser"},
            "body": "Test comment",
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-01-15T10:30:00Z",
            "diff_hunk": "",
            "html_url": ""
        });

        let comment = parse_comment(&data).unwrap();
        assert_eq!(comment.node_id, Some("PRRC_kwDOE2CVus5test".to_string()));
    }

    #[test]
    fn test_parse_comment_without_node_id() {
        let data = json!({
            "id": 123,
            "path": "src/main.rs",
            "line": 42,
            "user": {"login": "testuser"},
            "body": "Test comment",
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-01-15T10:30:00Z",
            "diff_hunk": "",
            "html_url": ""
        });

        let comment = parse_comment(&data).unwrap();
        assert_eq!(comment.node_id, None);
    }

    #[test]
    fn test_parse_review_comment_with_node_id() {
        let data = json!({
            "id": 12345,
            "node_id": "PRR_kwDOE2CVus5review",
            "body": "Review comment",
            "user": {"login": "reviewer"},
            "submitted_at": "2024-01-15T10:30:00Z",
            "html_url": ""
        });

        let comment = parse_review_comment(&data).unwrap();
        assert_eq!(comment.node_id, Some("PRR_kwDOE2CVus5review".to_string()));
    }

    #[test]
    fn test_parse_review_comment_without_node_id() {
        let data = json!({
            "id": 12345,
            "body": "Review comment",
            "user": {"login": "reviewer"},
            "submitted_at": "2024-01-15T10:30:00Z",
            "html_url": ""
        });

        let comment = parse_review_comment(&data).unwrap();
        assert_eq!(comment.node_id, None);
    }
    // ---- Check parsing tests ----

    fn create_graphql_response(checks: Vec<Value>) -> Value {
        json!({
            "data": {
                "repository": {
                    "pullRequest": {
                        "title": "Test PR",
                        "url": "https://github.com/owner/repo/pull/1",
                        "commits": {
                            "nodes": [{
                                "commit": {
                                    "statusCheckRollup": {
                                        "state": "FAILURE",
                                        "contexts": {
                                            "nodes": checks
                                        }
                                    }
                                }
                            }]
                        }
                    }
                }
            }
        })
    }

    fn check_run_node(name: &str, status: &str, conclusion: &str, required: bool) -> Value {
        json!({
            "__typename": "CheckRun",
            "name": name,
            "status": status,
            "conclusion": conclusion,
            "startedAt": "2024-01-15T10:00:00Z",
            "completedAt": "2024-01-15T10:05:00Z",
            "detailsUrl": format!("https://ci.example.com/{name}"),
            "isRequired": required,
            "checkSuite": {
                "app": { "slug": "github-actions" },
                "workflowRun": { "workflow": { "name": "CI" } }
            }
        })
    }

    fn status_context_node(context: &str, state: &str, required: bool) -> Value {
        json!({
            "__typename": "StatusContext",
            "context": context,
            "state": state,
            "description": format!("{context} status"),
            "targetUrl": format!("https://ci.example.com/{context}"),
            "createdAt": "2024-01-15T10:00:00Z",
            "isRequired": required
        })
    }

    #[test]
    fn test_parse_checks_response_full() {
        let response = create_graphql_response(vec![
            check_run_node("build", "COMPLETED", "SUCCESS", true),
            check_run_node("test", "COMPLETED", "FAILURE", true),
            status_context_node("buildkite/pipeline", "SUCCESS", false),
        ]);

        let report = parse_checks_response(&response).unwrap();
        assert_eq!(report.pr_title.as_deref(), Some("Test PR"));
        assert_eq!(
            report.pr_url.as_deref(),
            Some("https://github.com/owner/repo/pull/1")
        );
        assert_eq!(report.rollup_state, RollupState::Failure);
        assert_eq!(report.checks.len(), 3);
    }

    #[test]
    fn test_parse_checks_response_empty_checks() {
        let response = create_graphql_response(vec![]);
        let report = parse_checks_response(&response).unwrap();
        assert!(report.checks.is_empty());
    }

    #[test]
    fn test_parse_checks_response_missing_pr() {
        let response = json!({"data": {"repository": {}}});
        let result = parse_checks_response(&response);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing pullRequest"));
    }

    #[test]
    fn test_parse_checks_response_missing_commit() {
        let response = json!({
            "data": {
                "repository": {
                    "pullRequest": {
                        "title": "Test",
                        "url": "https://github.com/owner/repo/pull/1",
                        "commits": { "nodes": [] }
                    }
                }
            }
        });
        let result = parse_checks_response(&response);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing commit"));
    }

    #[test]
    fn test_parse_checks_response_no_rollup() {
        let response = json!({
            "data": {
                "repository": {
                    "pullRequest": {
                        "title": "Test",
                        "url": "https://github.com/owner/repo/pull/1",
                        "commits": {
                            "nodes": [{
                                "commit": {
                                    "statusCheckRollup": null
                                }
                            }]
                        }
                    }
                }
            }
        });
        let report = parse_checks_response(&response).unwrap();
        assert_eq!(report.rollup_state, RollupState::Unknown);
        assert!(report.checks.is_empty());
    }

    #[test]
    fn test_parse_check_run() {
        let node = check_run_node("build", "COMPLETED", "SUCCESS", true);
        let check = parse_check_node(&node).unwrap();
        assert_eq!(check.name, "build");
        assert_eq!(check.conclusion, CheckConclusion::Success);
        assert!(check.required);
        assert_eq!(check.check_type, CheckType::CheckRun);
        assert_eq!(check.app_name.as_deref(), Some("github-actions"));
        assert_eq!(check.workflow_name.as_deref(), Some("CI"));
        assert!(check.started_at.is_some());
        assert!(check.completed_at.is_some());
        assert!(check.details_url.is_some());
    }

    #[test]
    fn test_parse_check_run_in_progress() {
        let node = json!({
            "__typename": "CheckRun",
            "name": "build",
            "status": "IN_PROGRESS",
            "conclusion": null,
            "isRequired": false,
            "checkSuite": null
        });
        let check = parse_check_node(&node).unwrap();
        assert_eq!(check.conclusion, CheckConclusion::Pending);
    }

    #[test]
    fn test_parse_check_run_queued() {
        let node = json!({
            "__typename": "CheckRun",
            "name": "build",
            "status": "QUEUED",
            "conclusion": null,
            "isRequired": false,
            "checkSuite": null
        });
        let check = parse_check_node(&node).unwrap();
        assert_eq!(check.conclusion, CheckConclusion::Pending);
    }

    #[test]
    fn test_parse_status_context() {
        let node = status_context_node("buildkite/pipeline", "SUCCESS", true);
        let check = parse_check_node(&node).unwrap();
        assert_eq!(check.name, "buildkite/pipeline");
        assert_eq!(check.conclusion, CheckConclusion::Success);
        assert!(check.required);
        assert_eq!(check.check_type, CheckType::StatusContext);
        assert!(check.description.is_some());
        assert!(check.details_url.is_some());
    }

    #[test]
    fn test_parse_status_context_failure() {
        let node = status_context_node("external-ci", "FAILURE", false);
        let check = parse_check_node(&node).unwrap();
        assert_eq!(check.conclusion, CheckConclusion::Failure);
        assert!(!check.required);
    }

    #[test]
    fn test_parse_status_context_error() {
        let node = status_context_node("external-ci", "ERROR", false);
        let check = parse_check_node(&node).unwrap();
        assert_eq!(check.conclusion, CheckConclusion::Failure);
    }

    #[test]
    fn test_parse_status_context_pending() {
        let node = status_context_node("external-ci", "PENDING", false);
        let check = parse_check_node(&node).unwrap();
        assert_eq!(check.conclusion, CheckConclusion::Pending);
    }

    #[test]
    fn test_parse_status_context_expected() {
        let node = status_context_node("external-ci", "EXPECTED", false);
        let check = parse_check_node(&node).unwrap();
        assert_eq!(check.conclusion, CheckConclusion::Pending);
    }

    #[test]
    fn test_parse_check_node_unknown_typename() {
        let node = json!({
            "__typename": "SomeNewType",
            "name": "test"
        });
        assert!(parse_check_node(&node).is_none());
    }

    #[test]
    fn test_parse_check_node_missing_typename() {
        let node = json!({"name": "test"});
        assert!(parse_check_node(&node).is_none());
    }

    #[test]
    fn test_parse_check_conclusion_all_variants() {
        assert_eq!(
            parse_check_conclusion("COMPLETED", "SUCCESS"),
            CheckConclusion::Success
        );
        assert_eq!(
            parse_check_conclusion("COMPLETED", "FAILURE"),
            CheckConclusion::Failure
        );
        assert_eq!(
            parse_check_conclusion("COMPLETED", "SKIPPED"),
            CheckConclusion::Skipped
        );
        assert_eq!(
            parse_check_conclusion("COMPLETED", "CANCELLED"),
            CheckConclusion::Cancelled
        );
        assert_eq!(
            parse_check_conclusion("COMPLETED", "TIMED_OUT"),
            CheckConclusion::TimedOut
        );
        assert_eq!(
            parse_check_conclusion("COMPLETED", "ACTION_REQUIRED"),
            CheckConclusion::ActionRequired
        );
        assert_eq!(
            parse_check_conclusion("COMPLETED", "NEUTRAL"),
            CheckConclusion::Neutral
        );
        assert_eq!(
            parse_check_conclusion("COMPLETED", "STALE"),
            CheckConclusion::Stale
        );
        assert_eq!(
            parse_check_conclusion("COMPLETED", "UNKNOWN_VALUE"),
            CheckConclusion::Unknown
        );
    }

    #[test]
    fn test_parse_check_conclusion_non_completed_statuses() {
        assert_eq!(
            parse_check_conclusion("IN_PROGRESS", ""),
            CheckConclusion::Pending
        );
        assert_eq!(
            parse_check_conclusion("QUEUED", ""),
            CheckConclusion::Pending
        );
        assert_eq!(
            parse_check_conclusion("REQUESTED", ""),
            CheckConclusion::Pending
        );
        assert_eq!(
            parse_check_conclusion("WAITING", ""),
            CheckConclusion::Pending
        );
        assert_eq!(
            parse_check_conclusion("PENDING", ""),
            CheckConclusion::Pending
        );
        assert_eq!(
            parse_check_conclusion("SOME_UNKNOWN_STATUS", ""),
            CheckConclusion::Unknown
        );
    }

    #[test]
    fn test_parse_check_conclusion_empty_status_and_conclusion() {
        assert_eq!(parse_check_conclusion("", ""), CheckConclusion::Pending);
    }

    #[test]
    fn test_parse_status_state_all_variants() {
        assert_eq!(parse_status_state("SUCCESS"), CheckConclusion::Success);
        assert_eq!(parse_status_state("FAILURE"), CheckConclusion::Failure);
        assert_eq!(parse_status_state("ERROR"), CheckConclusion::Failure);
        assert_eq!(parse_status_state("PENDING"), CheckConclusion::Pending);
        assert_eq!(parse_status_state("EXPECTED"), CheckConclusion::Pending);
        assert_eq!(parse_status_state("UNKNOWN"), CheckConclusion::Unknown);
    }

    #[test]
    fn test_parse_rollup_state_all_variants() {
        assert_eq!(parse_rollup_state("SUCCESS"), RollupState::Success);
        assert_eq!(parse_rollup_state("FAILURE"), RollupState::Failure);
        assert_eq!(parse_rollup_state("PENDING"), RollupState::Pending);
        assert_eq!(parse_rollup_state("ERROR"), RollupState::Error);
        assert_eq!(parse_rollup_state("EXPECTED"), RollupState::Expected);
        assert_eq!(parse_rollup_state("UNKNOWN_VALUE"), RollupState::Unknown);
    }

    #[test]
    fn test_parse_check_run_minimal_fields() {
        let node = json!({
            "__typename": "CheckRun",
            "name": "minimal",
            "status": "COMPLETED",
            "conclusion": "SUCCESS",
            "isRequired": false,
            "checkSuite": null
        });
        let check = parse_check_node(&node).unwrap();
        assert_eq!(check.name, "minimal");
        assert!(check.app_name.is_none());
        assert!(check.workflow_name.is_none());
        assert!(check.started_at.is_none());
        assert!(check.completed_at.is_none());
        assert!(check.details_url.is_none());
    }

    #[test]
    fn test_parse_check_run_missing_name() {
        let node = json!({
            "__typename": "CheckRun",
            "status": "COMPLETED",
            "conclusion": "SUCCESS"
        });
        assert!(parse_check_node(&node).is_none());
    }

    #[test]
    fn test_parse_status_context_missing_context() {
        let node = json!({
            "__typename": "StatusContext",
            "state": "SUCCESS"
        });
        assert!(parse_check_node(&node).is_none());
    }

    #[test]
    fn test_parse_checks_response_no_title_or_url() {
        let response = json!({
            "data": {
                "repository": {
                    "pullRequest": {
                        "commits": {
                            "nodes": [{
                                "commit": {
                                    "statusCheckRollup": {
                                        "state": "SUCCESS",
                                        "contexts": { "nodes": [] }
                                    }
                                }
                            }]
                        }
                    }
                }
            }
        });
        let report = parse_checks_response(&response).unwrap();
        assert!(report.pr_title.is_none());
        assert!(report.pr_url.is_none());
    }

    #[test]
    fn test_parse_status_context_minimal_fields() {
        let node = json!({
            "__typename": "StatusContext",
            "context": "ci/test",
            "state": "SUCCESS"
        });
        let check = parse_check_node(&node).unwrap();
        assert_eq!(check.name, "ci/test");
        assert!(check.description.is_none());
        assert!(check.details_url.is_none());
        assert!(check.started_at.is_none());
    }
}

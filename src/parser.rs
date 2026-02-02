//! JSON parsing and comment filtering functions.

use crate::models::PRComment;
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
}

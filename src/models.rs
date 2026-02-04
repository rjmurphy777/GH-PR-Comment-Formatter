//! Data models for PR comments.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a parsed pull request comment from GitHub.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PRComment {
    pub id: i64,
    /// GraphQL node ID for this comment (e.g., "PRRC_kwDO..."). Used for replying via GraphQL.
    pub node_id: Option<String>,
    pub file_path: String,
    pub line_number: Option<i32>,
    pub start_line: Option<i32>,
    pub author: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub diff_hunk: String,
    pub html_url: String,
}

impl PRComment {
    /// Creates a new PRComment with all fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: i64,
        node_id: Option<String>,
        file_path: String,
        line_number: Option<i32>,
        start_line: Option<i32>,
        author: String,
        body: String,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
        diff_hunk: String,
        html_url: String,
    ) -> Self {
        Self {
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
        }
    }

    /// Returns a human-readable line info string.
    ///
    /// Examples:
    /// - "line 42"
    /// - "lines 10-20"
    /// - "line unknown"
    pub fn get_line_info(&self) -> String {
        match (self.line_number, self.start_line) {
            (Some(line), Some(start)) if start != line => {
                format!("lines {start}-{line}")
            }
            (Some(line), _) => format!("line {line}"),
            (None, Some(start)) => format!("line {start}"),
            (None, None) => "line unknown".to_string(),
        }
    }

    /// Extracts a code snippet from the diff hunk.
    ///
    /// Removes the @@ header line and returns up to `max_lines` of code,
    /// taking the last N lines (most relevant to the comment).
    pub fn get_code_snippet(&self, max_lines: usize) -> String {
        if self.diff_hunk.is_empty() {
            return String::new();
        }

        let lines: Vec<&str> = self
            .diff_hunk
            .lines()
            .filter(|line| !line.starts_with("@@"))
            .collect();

        if lines.is_empty() {
            return String::new();
        }

        // Take the last max_lines lines (most relevant to the comment)
        let start = if lines.len() > max_lines {
            lines.len() - max_lines
        } else {
            0
        };

        lines[start..].join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn create_test_comment() -> PRComment {
        PRComment::new(
            1,
            Some("PRRC_kwDOtest123".to_string()),
            "src/main.rs".to_string(),
            Some(42),
            None,
            "testuser".to_string(),
            "Test comment".to_string(),
            Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap(),
            "@@ -10,5 +10,5 @@\n line1\n line2\n line3".to_string(),
            "https://github.com/owner/repo/pull/1#discussion_r1".to_string(),
        )
    }

    #[test]
    fn test_comment_instantiation() {
        let comment = create_test_comment();
        assert_eq!(comment.id, 1);
        assert_eq!(comment.file_path, "src/main.rs");
        assert_eq!(comment.line_number, Some(42));
        assert_eq!(comment.author, "testuser");
    }

    #[test]
    fn test_get_line_info_single_line() {
        let comment = create_test_comment();
        assert_eq!(comment.get_line_info(), "line 42");
    }

    #[test]
    fn test_get_line_info_line_range() {
        let mut comment = create_test_comment();
        comment.line_number = Some(20);
        comment.start_line = Some(10);
        assert_eq!(comment.get_line_info(), "lines 10-20");
    }

    #[test]
    fn test_get_line_info_same_start_end() {
        let mut comment = create_test_comment();
        comment.line_number = Some(10);
        comment.start_line = Some(10);
        assert_eq!(comment.get_line_info(), "line 10");
    }

    #[test]
    fn test_get_line_info_no_line() {
        let mut comment = create_test_comment();
        comment.line_number = None;
        comment.start_line = None;
        assert_eq!(comment.get_line_info(), "line unknown");
    }

    #[test]
    fn test_get_code_snippet_removes_header() {
        let comment = create_test_comment();
        let snippet = comment.get_code_snippet(10);
        assert!(!snippet.contains("@@"));
        assert!(snippet.contains("line1"));
    }

    #[test]
    fn test_get_code_snippet_truncates() {
        let mut comment = create_test_comment();
        comment.diff_hunk = "@@ -1,10 +1,10 @@\nline1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10".to_string();
        let snippet = comment.get_code_snippet(3);
        let lines: Vec<&str> = snippet.lines().collect();
        assert_eq!(lines.len(), 3);
        // Should be the last 3 lines
        assert!(snippet.contains("line8"));
        assert!(snippet.contains("line9"));
        assert!(snippet.contains("line10"));
    }

    #[test]
    fn test_get_code_snippet_empty_diff() {
        let mut comment = create_test_comment();
        comment.diff_hunk = String::new();
        assert_eq!(comment.get_code_snippet(10), "");
    }

    #[test]
    fn test_get_line_info_only_start_line() {
        // Covers models.rs line 62: (None, Some(start)) case
        let mut comment = create_test_comment();
        comment.line_number = None;
        comment.start_line = Some(15);
        assert_eq!(comment.get_line_info(), "line 15");
    }

    #[test]
    fn test_get_code_snippet_only_header() {
        // Covers models.rs line 83: empty after filtering @@ lines
        let mut comment = create_test_comment();
        comment.diff_hunk = "@@ -10,5 +10,5 @@".to_string();
        assert_eq!(comment.get_code_snippet(10), "");
    }
}

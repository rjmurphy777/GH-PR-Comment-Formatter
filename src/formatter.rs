//! Output formatting for PR comments in multiple styles.

use crate::models::PRComment;
use crate::parser::group_by_file;
use serde_json::json;
use std::collections::HashSet;

/// Formats a single comment for LLM consumption.
pub fn format_comment_for_llm(
    comment: &PRComment,
    include_snippet: bool,
    snippet_lines: usize,
) -> String {
    let mut output = String::new();

    // File and line info header
    output.push_str(&format!(
        "### {} ({})\n\n",
        comment.file_path,
        comment.get_line_info()
    ));

    // Author
    output.push_str(&format!("**Author:** {}\n", comment.author));

    // Date formatted as YYYY-MM-DD HH:MM UTC
    output.push_str(&format!(
        "**Date:** {}\n\n",
        comment.created_at.format("%Y-%m-%d %H:%M UTC")
    ));

    // Code snippet
    if include_snippet {
        let snippet = comment.get_code_snippet(snippet_lines);
        if !snippet.is_empty() {
            output.push_str("**Code context:**\n```\n");
            output.push_str(&snippet);
            output.push_str("\n```\n\n");
        }
    }

    // Comment body
    output.push_str(&format!("**Comment:**\n{}\n", comment.body));

    output
}

/// Formats comments grouped by file.
pub fn format_comments_grouped(
    comments: &[PRComment],
    include_snippet: bool,
    snippet_lines: usize,
) -> String {
    if comments.is_empty() {
        return "No comments found.\n".to_string();
    }

    let mut output = String::new();

    // Summary
    let file_count = comments
        .iter()
        .map(|c| &c.file_path)
        .collect::<HashSet<_>>()
        .len();
    output.push_str(&format!(
        "# PR Review Comments\n\n**Total comments:** {} across {} file(s)\n\n",
        comments.len(),
        file_count
    ));

    // Group by file
    let grouped = group_by_file(comments);

    // Sort files for consistent output
    let mut files: Vec<_> = grouped.keys().collect();
    files.sort();

    for file in files {
        let file_comments = grouped.get(file).unwrap();
        output.push_str(&format!("## {file}\n\n"));

        // Sort by line number, then by date
        let mut sorted_comments: Vec<_> = file_comments.iter().collect();
        sorted_comments.sort_by(|a, b| {
            a.line_number
                .cmp(&b.line_number)
                .then_with(|| a.created_at.cmp(&b.created_at))
        });

        for comment in sorted_comments {
            output.push_str(&format_comment_for_llm(
                comment,
                include_snippet,
                snippet_lines,
            ));
            output.push_str("\n---\n\n");
        }
    }

    output
}

/// Formats comments in a flat list sorted by date (most recent first).
pub fn format_comments_flat(
    comments: &[PRComment],
    include_snippet: bool,
    snippet_lines: usize,
) -> String {
    if comments.is_empty() {
        return "No comments found.\n".to_string();
    }

    let mut output = String::new();
    output.push_str(&format!(
        "# PR Review Comments\n\n**Total comments:** {}\n\n",
        comments.len()
    ));

    // Sort by date (most recent first)
    let mut sorted_comments: Vec<_> = comments.iter().collect();
    sorted_comments.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    for (i, comment) in sorted_comments.iter().enumerate() {
        output.push_str(&format!("## Comment {}\n\n", i + 1));
        output.push_str(&format_comment_for_llm(
            comment,
            include_snippet,
            snippet_lines,
        ));
        output.push_str("\n---\n\n");
    }

    output
}

/// Formats comments in a minimal/compact style for quick overview.
pub fn format_comments_minimal(comments: &[PRComment]) -> String {
    if comments.is_empty() {
        return "No comments found.\n".to_string();
    }

    let mut output = String::new();

    for comment in comments {
        // Truncate body to 100 chars
        let truncated_body = if comment.body.len() > 100 {
            format!("{}...", &comment.body[..100])
        } else {
            comment.body.clone()
        };

        output.push_str(&format!(
            "\u{1F4C4} {} ({}) - {}: {}\n",
            comment.file_path,
            comment.get_line_info(),
            comment.author,
            truncated_body.replace('\n', " ")
        ));
    }

    // Summary line
    let file_count = comments
        .iter()
        .map(|c| &c.file_path)
        .collect::<HashSet<_>>()
        .len();
    output.push_str(&format!(
        "\n---\n{} comment(s) across {} file(s)\n",
        comments.len(),
        file_count
    ));

    output
}

/// Formats comments for Claude/LLM consumption with full context.
pub fn format_for_claude(
    comments: &[PRComment],
    pr_url: Option<&str>,
    pr_title: Option<&str>,
    include_snippet: bool,
    snippet_lines: usize,
) -> String {
    if comments.is_empty() {
        return "No comments found.\n".to_string();
    }

    let mut output = String::new();

    // Header
    output.push_str("# Pull Request Review Comments\n\n");

    // PR info if available
    if let Some(title) = pr_title {
        output.push_str(&format!("**PR Title:** {title}\n"));
    }
    if let Some(url) = pr_url {
        output.push_str(&format!("**PR URL:** {url}\n"));
    }

    // Summary
    let file_count = comments
        .iter()
        .map(|c| &c.file_path)
        .collect::<HashSet<_>>()
        .len();
    output.push_str(&format!(
        "\n**Total comments:** {} across {} file(s)\n\n",
        comments.len(),
        file_count
    ));

    // Instructions
    output.push_str("## Instructions\n\n");
    output.push_str("Please address each of the following review comments. ");
    output.push_str("The comments are grouped by file for easier navigation.\n\n");

    // Group by file
    let grouped = group_by_file(comments);

    // Sort files for consistent output
    let mut files: Vec<_> = grouped.keys().collect();
    files.sort();

    output.push_str("## Comments by File\n\n");

    for file in files {
        let file_comments = grouped.get(file).unwrap();
        output.push_str(&format!("### {file}\n\n"));

        // Sort by line number, then by date
        let mut sorted_comments: Vec<_> = file_comments.iter().collect();
        sorted_comments.sort_by(|a, b| {
            a.line_number
                .cmp(&b.line_number)
                .then_with(|| a.created_at.cmp(&b.created_at))
        });

        for comment in sorted_comments {
            output.push_str(&format!(
                "#### {} ({})\n\n",
                comment.get_line_info(),
                comment.author
            ));

            // Code snippet
            if include_snippet {
                let snippet = comment.get_code_snippet(snippet_lines);
                if !snippet.is_empty() {
                    output.push_str("**Code context:**\n```\n");
                    output.push_str(&snippet);
                    output.push_str("\n```\n\n");
                }
            }

            output.push_str(&format!("**Review comment:**\n{}\n\n", comment.body));
            output.push_str(&format!("[View on GitHub]({})\n\n", comment.html_url));
            output.push_str("---\n\n");
        }
    }

    output
}

/// Formats comments as JSON for programmatic use.
pub fn format_as_json(
    comments: &[PRComment],
    include_snippet: bool,
    snippet_lines: usize,
) -> String {
    let json_comments: Vec<_> = comments
        .iter()
        .map(|c| {
            let snippet = if include_snippet {
                let s = c.get_code_snippet(snippet_lines);
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            } else {
                None
            };

            json!({
                "file": c.file_path,
                "line": c.line_number,
                "author": c.author,
                "body": c.body,
                "snippet": snippet,
                "url": c.html_url
            })
        })
        .collect();

    serde_json::to_string_pretty(&json_comments).unwrap_or_else(|_| "[]".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn create_test_comment(id: i64, file: &str, line: Option<i32>, author: &str) -> PRComment {
        PRComment::new(
            id,
            file.to_string(),
            line,
            None,
            author.to_string(),
            "Test comment body".to_string(),
            Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap(),
            "@@ -1,5 +1,5 @@\n line1\n line2".to_string(),
            "https://github.com/owner/repo/pull/1#discussion_r1".to_string(),
        )
    }

    #[test]
    fn test_format_comment_for_llm_includes_file_and_line() {
        let comment = create_test_comment(1, "src/main.rs", Some(42), "testuser");
        let output = format_comment_for_llm(&comment, true, 10);
        assert!(output.contains("src/main.rs"));
        assert!(output.contains("line 42"));
    }

    #[test]
    fn test_format_comment_for_llm_includes_author() {
        let comment = create_test_comment(1, "src/main.rs", Some(42), "testuser");
        let output = format_comment_for_llm(&comment, true, 10);
        assert!(output.contains("testuser"));
    }

    #[test]
    fn test_format_comment_for_llm_includes_date() {
        let comment = create_test_comment(1, "src/main.rs", Some(42), "testuser");
        let output = format_comment_for_llm(&comment, true, 10);
        assert!(output.contains("2024-01-15"));
    }

    #[test]
    fn test_format_comment_for_llm_includes_snippet() {
        let comment = create_test_comment(1, "src/main.rs", Some(42), "testuser");
        let output = format_comment_for_llm(&comment, true, 10);
        assert!(output.contains("line1"));
        assert!(output.contains("Code context"));
    }

    #[test]
    fn test_format_comment_for_llm_excludes_snippet() {
        let comment = create_test_comment(1, "src/main.rs", Some(42), "testuser");
        let output = format_comment_for_llm(&comment, false, 10);
        assert!(!output.contains("line1"));
        assert!(!output.contains("Code context"));
    }

    #[test]
    fn test_format_comment_for_llm_includes_body() {
        let comment = create_test_comment(1, "src/main.rs", Some(42), "testuser");
        let output = format_comment_for_llm(&comment, true, 10);
        assert!(output.contains("Test comment body"));
    }

    #[test]
    fn test_format_comments_grouped_groups_by_file() {
        let comments = vec![
            create_test_comment(1, "file1.rs", Some(10), "user1"),
            create_test_comment(2, "file2.rs", Some(20), "user2"),
            create_test_comment(3, "file1.rs", Some(30), "user3"),
        ];
        let output = format_comments_grouped(&comments, true, 10);
        assert!(output.contains("## file1.rs"));
        assert!(output.contains("## file2.rs"));
    }

    #[test]
    fn test_format_comments_grouped_shows_total_count() {
        let comments = vec![
            create_test_comment(1, "file1.rs", Some(10), "user1"),
            create_test_comment(2, "file2.rs", Some(20), "user2"),
        ];
        let output = format_comments_grouped(&comments, true, 10);
        assert!(output.contains("Total comments:** 2"));
    }

    #[test]
    fn test_format_comments_grouped_shows_file_count() {
        let comments = vec![
            create_test_comment(1, "file1.rs", Some(10), "user1"),
            create_test_comment(2, "file2.rs", Some(20), "user2"),
        ];
        let output = format_comments_grouped(&comments, true, 10);
        assert!(output.contains("2 file(s)"));
    }

    #[test]
    fn test_format_comments_grouped_empty() {
        let output = format_comments_grouped(&[], true, 10);
        assert!(output.contains("No comments found"));
    }

    #[test]
    fn test_format_comments_flat_shows_total_count() {
        let comments = vec![
            create_test_comment(1, "file1.rs", Some(10), "user1"),
            create_test_comment(2, "file2.rs", Some(20), "user2"),
        ];
        let output = format_comments_flat(&comments, true, 10);
        assert!(output.contains("Total comments:** 2"));
    }

    #[test]
    fn test_format_comments_flat_empty() {
        let output = format_comments_flat(&[], true, 10);
        assert!(output.contains("No comments found"));
    }

    #[test]
    fn test_format_comments_minimal_shows_emoji() {
        let comments = vec![create_test_comment(1, "file1.rs", Some(10), "user1")];
        let output = format_comments_minimal(&comments);
        assert!(output.contains("\u{1F4C4}")); // File emoji
    }

    #[test]
    fn test_format_comments_minimal_truncates_long_body() {
        let mut comment = create_test_comment(1, "file1.rs", Some(10), "user1");
        comment.body = "a".repeat(150);
        let comments = vec![comment];
        let output = format_comments_minimal(&comments);
        assert!(output.contains("..."));
    }

    #[test]
    fn test_format_comments_minimal_shows_summary() {
        let comments = vec![
            create_test_comment(1, "file1.rs", Some(10), "user1"),
            create_test_comment(2, "file2.rs", Some(20), "user2"),
        ];
        let output = format_comments_minimal(&comments);
        assert!(output.contains("2 comment(s)"));
        assert!(output.contains("2 file(s)"));
    }

    #[test]
    fn test_format_comments_minimal_empty() {
        let output = format_comments_minimal(&[]);
        assert!(output.contains("No comments found"));
    }

    #[test]
    fn test_format_for_claude_includes_header() {
        let comments = vec![create_test_comment(1, "file1.rs", Some(10), "user1")];
        let output = format_for_claude(&comments, None, None, true, 15);
        assert!(output.contains("Pull Request Review Comments"));
    }

    #[test]
    fn test_format_for_claude_includes_pr_title() {
        let comments = vec![create_test_comment(1, "file1.rs", Some(10), "user1")];
        let output = format_for_claude(&comments, None, Some("Test PR Title"), true, 15);
        assert!(output.contains("Test PR Title"));
    }

    #[test]
    fn test_format_for_claude_includes_pr_url() {
        let comments = vec![create_test_comment(1, "file1.rs", Some(10), "user1")];
        let output = format_for_claude(
            &comments,
            Some("https://github.com/owner/repo/pull/123"),
            None,
            true,
            15,
        );
        assert!(output.contains("https://github.com/owner/repo/pull/123"));
    }

    #[test]
    fn test_format_for_claude_includes_instructions() {
        let comments = vec![create_test_comment(1, "file1.rs", Some(10), "user1")];
        let output = format_for_claude(&comments, None, None, true, 15);
        assert!(output.contains("Instructions"));
        assert!(output.contains("address"));
    }

    #[test]
    fn test_format_for_claude_empty() {
        let output = format_for_claude(&[], None, None, true, 15);
        assert!(output.contains("No comments found"));
    }

    #[test]
    fn test_format_as_json() {
        let comments = vec![create_test_comment(1, "file1.rs", Some(10), "user1")];
        let output = format_as_json(&comments, true, 10);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 1);
        assert_eq!(parsed[0]["file"], "file1.rs");
        assert_eq!(parsed[0]["line"], 10);
        assert_eq!(parsed[0]["author"], "user1");
    }

    #[test]
    fn test_format_as_json_no_snippet() {
        let comments = vec![create_test_comment(1, "file1.rs", Some(10), "user1")];
        let output = format_as_json(&comments, false, 10);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed[0]["snippet"].is_null());
    }

    #[test]
    fn test_format_as_json_empty_snippet() {
        // Covers formatter.rs line 278: empty snippet returns None
        let comment = PRComment::new(
            1,
            "file1.rs".to_string(),
            Some(10),
            None,
            "user1".to_string(),
            "Test body".to_string(),
            Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap(),
            "".to_string(), // Empty diff hunk
            "https://github.com/owner/repo/pull/1#discussion_r1".to_string(),
        );
        let comments = vec![comment];
        let output = format_as_json(&comments, true, 10); // include_snippet=true but diff is empty
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed[0]["snippet"].is_null());
    }

    #[test]
    fn test_format_for_claude_sorts_by_line_then_date() {
        // Covers formatter.rs lines 235-237: sorting by line_number then created_at
        let comments = vec![
            PRComment::new(
                1,
                "file1.rs".to_string(),
                Some(10),
                None,
                "user1".to_string(),
                "Earlier comment".to_string(),
                Utc.with_ymd_and_hms(2024, 1, 15, 8, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2024, 1, 15, 8, 0, 0).unwrap(),
                "".to_string(),
                "https://github.com/owner/repo/pull/1#discussion_r1".to_string(),
            ),
            PRComment::new(
                2,
                "file1.rs".to_string(),
                Some(10), // Same line number
                None,
                "user2".to_string(),
                "Later comment".to_string(),
                Utc.with_ymd_and_hms(2024, 1, 15, 10, 0, 0).unwrap(), // Later time
                Utc.with_ymd_and_hms(2024, 1, 15, 10, 0, 0).unwrap(),
                "".to_string(),
                "https://github.com/owner/repo/pull/1#discussion_r2".to_string(),
            ),
        ];
        let output = format_for_claude(&comments, None, None, false, 10);
        // Earlier comment should appear first in the output
        let earlier_pos = output.find("Earlier comment").unwrap();
        let later_pos = output.find("Later comment").unwrap();
        assert!(
            earlier_pos < later_pos,
            "Comments should be sorted by date when line numbers are equal"
        );
    }

    #[test]
    fn test_format_comments_grouped_sorts_by_line_then_date() {
        // Also tests the sorting logic in format_comments_grouped
        let comments = vec![
            PRComment::new(
                1,
                "file1.rs".to_string(),
                Some(10),
                None,
                "user1".to_string(),
                "Earlier comment".to_string(),
                Utc.with_ymd_and_hms(2024, 1, 15, 8, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2024, 1, 15, 8, 0, 0).unwrap(),
                "".to_string(),
                "https://github.com/owner/repo/pull/1#discussion_r1".to_string(),
            ),
            PRComment::new(
                2,
                "file1.rs".to_string(),
                Some(10), // Same line number
                None,
                "user2".to_string(),
                "Later comment".to_string(),
                Utc.with_ymd_and_hms(2024, 1, 15, 10, 0, 0).unwrap(), // Later time
                Utc.with_ymd_and_hms(2024, 1, 15, 10, 0, 0).unwrap(),
                "".to_string(),
                "https://github.com/owner/repo/pull/1#discussion_r2".to_string(),
            ),
        ];
        let output = format_comments_grouped(&comments, false, 10);
        // Earlier comment should appear first in the output
        let earlier_pos = output.find("Earlier comment").unwrap();
        let later_pos = output.find("Later comment").unwrap();
        assert!(
            earlier_pos < later_pos,
            "Comments should be sorted by date when line numbers are equal"
        );
    }
}

//! Data models for PR comments and check statuses.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

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

/// The conclusion/result of a CI check.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CheckConclusion {
    Success,
    Failure,
    Pending,
    Skipped,
    Cancelled,
    TimedOut,
    ActionRequired,
    Neutral,
    Stale,
    Unknown,
}

impl CheckConclusion {
    /// Returns a human-readable status icon/label for display.
    pub fn display_icon(&self) -> &'static str {
        match self {
            CheckConclusion::Success => "PASS",
            CheckConclusion::Failure => "FAIL",
            CheckConclusion::Pending => "PENDING",
            CheckConclusion::Skipped => "SKIP",
            CheckConclusion::Cancelled => "CANCELLED",
            CheckConclusion::TimedOut => "TIMED_OUT",
            CheckConclusion::ActionRequired => "ACTION_REQUIRED",
            CheckConclusion::Neutral => "NEUTRAL",
            CheckConclusion::Stale => "STALE",
            CheckConclusion::Unknown => "UNKNOWN",
        }
    }

    /// Returns true if this conclusion represents a failure state.
    pub fn is_failure(&self) -> bool {
        matches!(
            self,
            CheckConclusion::Failure
                | CheckConclusion::TimedOut
                | CheckConclusion::ActionRequired
                | CheckConclusion::Cancelled
        )
    }
}

impl fmt::Display for CheckConclusion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_icon())
    }
}

/// Whether a check is a CheckRun or StatusContext.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CheckType {
    CheckRun,
    StatusContext,
}

/// A single CI check status entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CheckStatus {
    pub name: String,
    pub conclusion: CheckConclusion,
    pub required: bool,
    pub description: Option<String>,
    pub details_url: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub check_type: CheckType,
    pub workflow_name: Option<String>,
    pub app_name: Option<String>,
}

/// The overall rollup state for all checks on a PR.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RollupState {
    Success,
    Failure,
    Pending,
    Error,
    Expected,
    Unknown,
}

impl fmt::Display for RollupState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            RollupState::Success => "SUCCESS",
            RollupState::Failure => "FAILURE",
            RollupState::Pending => "PENDING",
            RollupState::Error => "ERROR",
            RollupState::Expected => "EXPECTED",
            RollupState::Unknown => "UNKNOWN",
        };
        write!(f, "{s}")
    }
}

/// Summary report of all CI checks on a PR.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChecksReport {
    pub pr_title: Option<String>,
    pub pr_url: Option<String>,
    pub rollup_state: RollupState,
    pub checks: Vec<CheckStatus>,
}

/// Summary counts for a checks report.
pub struct ChecksSummary {
    pub passed: usize,
    pub failed: usize,
    pub pending: usize,
    pub skipped: usize,
    pub total: usize,
}

impl ChecksReport {
    /// Returns checks that failed and are required.
    pub fn failed_required(&self) -> Vec<&CheckStatus> {
        self.checks
            .iter()
            .filter(|c| c.conclusion.is_failure() && c.required)
            .collect()
    }

    /// Returns checks that failed and are optional.
    pub fn failed_optional(&self) -> Vec<&CheckStatus> {
        self.checks
            .iter()
            .filter(|c| c.conclusion.is_failure() && !c.required)
            .collect()
    }

    /// Returns checks that passed and are required.
    pub fn passed_required(&self) -> Vec<&CheckStatus> {
        self.checks
            .iter()
            .filter(|c| c.conclusion == CheckConclusion::Success && c.required)
            .collect()
    }

    /// Returns checks that passed and are optional.
    pub fn passed_optional(&self) -> Vec<&CheckStatus> {
        self.checks
            .iter()
            .filter(|c| c.conclusion == CheckConclusion::Success && !c.required)
            .collect()
    }

    /// Returns checks that are still pending.
    pub fn pending(&self) -> Vec<&CheckStatus> {
        self.checks
            .iter()
            .filter(|c| c.conclusion == CheckConclusion::Pending)
            .collect()
    }

    /// Returns checks that were skipped.
    pub fn skipped(&self) -> Vec<&CheckStatus> {
        self.checks
            .iter()
            .filter(|c| {
                c.conclusion == CheckConclusion::Skipped || c.conclusion == CheckConclusion::Neutral
            })
            .collect()
    }

    /// Returns summary counts for the checks.
    pub fn summary_counts(&self) -> ChecksSummary {
        let passed = self
            .checks
            .iter()
            .filter(|c| c.conclusion == CheckConclusion::Success)
            .count();
        let failed = self
            .checks
            .iter()
            .filter(|c| c.conclusion.is_failure())
            .count();
        let pending = self
            .checks
            .iter()
            .filter(|c| c.conclusion == CheckConclusion::Pending)
            .count();
        let skipped = self
            .checks
            .iter()
            .filter(|c| {
                c.conclusion == CheckConclusion::Skipped || c.conclusion == CheckConclusion::Neutral
            })
            .count();
        ChecksSummary {
            passed,
            failed,
            pending,
            skipped,
            total: self.checks.len(),
        }
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

    // ---- Check status model tests ----

    fn create_test_check(name: &str, conclusion: CheckConclusion, required: bool) -> CheckStatus {
        CheckStatus {
            name: name.to_string(),
            conclusion,
            required,
            description: None,
            details_url: None,
            started_at: None,
            completed_at: None,
            check_type: CheckType::CheckRun,
            workflow_name: None,
            app_name: None,
        }
    }

    fn create_test_report() -> ChecksReport {
        ChecksReport {
            pr_title: Some("Test PR".to_string()),
            pr_url: Some("https://github.com/owner/repo/pull/1".to_string()),
            rollup_state: RollupState::Failure,
            checks: vec![
                create_test_check("build", CheckConclusion::Success, true),
                create_test_check("lint", CheckConclusion::Failure, true),
                create_test_check("coverage", CheckConclusion::Success, false),
                create_test_check("optional-lint", CheckConclusion::Failure, false),
                create_test_check("deploy", CheckConclusion::Pending, false),
                create_test_check("docs", CheckConclusion::Skipped, false),
                create_test_check("neutral-check", CheckConclusion::Neutral, false),
            ],
        }
    }

    #[test]
    fn test_check_conclusion_display_icon() {
        assert_eq!(CheckConclusion::Success.display_icon(), "PASS");
        assert_eq!(CheckConclusion::Failure.display_icon(), "FAIL");
        assert_eq!(CheckConclusion::Pending.display_icon(), "PENDING");
        assert_eq!(CheckConclusion::Skipped.display_icon(), "SKIP");
        assert_eq!(CheckConclusion::Cancelled.display_icon(), "CANCELLED");
        assert_eq!(CheckConclusion::TimedOut.display_icon(), "TIMED_OUT");
        assert_eq!(
            CheckConclusion::ActionRequired.display_icon(),
            "ACTION_REQUIRED"
        );
        assert_eq!(CheckConclusion::Neutral.display_icon(), "NEUTRAL");
        assert_eq!(CheckConclusion::Stale.display_icon(), "STALE");
        assert_eq!(CheckConclusion::Unknown.display_icon(), "UNKNOWN");
    }

    #[test]
    fn test_check_conclusion_is_failure() {
        assert!(CheckConclusion::Failure.is_failure());
        assert!(CheckConclusion::TimedOut.is_failure());
        assert!(CheckConclusion::ActionRequired.is_failure());
        assert!(CheckConclusion::Cancelled.is_failure());
        assert!(!CheckConclusion::Success.is_failure());
        assert!(!CheckConclusion::Pending.is_failure());
        assert!(!CheckConclusion::Skipped.is_failure());
        assert!(!CheckConclusion::Neutral.is_failure());
        assert!(!CheckConclusion::Stale.is_failure());
        assert!(!CheckConclusion::Unknown.is_failure());
    }

    #[test]
    fn test_check_conclusion_display_trait() {
        assert_eq!(format!("{}", CheckConclusion::Success), "PASS");
        assert_eq!(format!("{}", CheckConclusion::Failure), "FAIL");
    }

    #[test]
    fn test_rollup_state_display() {
        assert_eq!(format!("{}", RollupState::Success), "SUCCESS");
        assert_eq!(format!("{}", RollupState::Failure), "FAILURE");
        assert_eq!(format!("{}", RollupState::Pending), "PENDING");
        assert_eq!(format!("{}", RollupState::Error), "ERROR");
        assert_eq!(format!("{}", RollupState::Expected), "EXPECTED");
        assert_eq!(format!("{}", RollupState::Unknown), "UNKNOWN");
    }

    #[test]
    fn test_checks_report_failed_required() {
        let report = create_test_report();
        let failed_req = report.failed_required();
        assert_eq!(failed_req.len(), 1);
        assert_eq!(failed_req[0].name, "lint");
    }

    #[test]
    fn test_checks_report_failed_optional() {
        let report = create_test_report();
        let failed_opt = report.failed_optional();
        assert_eq!(failed_opt.len(), 1);
        assert_eq!(failed_opt[0].name, "optional-lint");
    }

    #[test]
    fn test_checks_report_passed_required() {
        let report = create_test_report();
        let passed_req = report.passed_required();
        assert_eq!(passed_req.len(), 1);
        assert_eq!(passed_req[0].name, "build");
    }

    #[test]
    fn test_checks_report_passed_optional() {
        let report = create_test_report();
        let passed_opt = report.passed_optional();
        assert_eq!(passed_opt.len(), 1);
        assert_eq!(passed_opt[0].name, "coverage");
    }

    #[test]
    fn test_checks_report_pending() {
        let report = create_test_report();
        let pending = report.pending();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].name, "deploy");
    }

    #[test]
    fn test_checks_report_skipped() {
        let report = create_test_report();
        let skipped = report.skipped();
        assert_eq!(skipped.len(), 2);
        let names: Vec<&str> = skipped.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"docs"));
        assert!(names.contains(&"neutral-check"));
    }

    #[test]
    fn test_checks_report_summary_counts() {
        let report = create_test_report();
        let summary = report.summary_counts();
        assert_eq!(summary.passed, 2);
        assert_eq!(summary.failed, 2);
        assert_eq!(summary.pending, 1);
        assert_eq!(summary.skipped, 2);
        assert_eq!(summary.total, 7);
    }

    #[test]
    fn test_checks_report_empty() {
        let report = ChecksReport {
            pr_title: None,
            pr_url: None,
            rollup_state: RollupState::Success,
            checks: vec![],
        };
        assert!(report.failed_required().is_empty());
        assert!(report.failed_optional().is_empty());
        assert!(report.passed_required().is_empty());
        assert!(report.passed_optional().is_empty());
        assert!(report.pending().is_empty());
        assert!(report.skipped().is_empty());
        let summary = report.summary_counts();
        assert_eq!(summary.total, 0);
    }

    #[test]
    fn test_check_status_with_all_fields() {
        let check = CheckStatus {
            name: "CI".to_string(),
            conclusion: CheckConclusion::Success,
            required: true,
            description: Some("Build passed".to_string()),
            details_url: Some("https://ci.example.com/1".to_string()),
            started_at: Some(Utc.with_ymd_and_hms(2024, 1, 15, 10, 0, 0).unwrap()),
            completed_at: Some(Utc.with_ymd_and_hms(2024, 1, 15, 10, 5, 0).unwrap()),
            check_type: CheckType::StatusContext,
            workflow_name: Some("main.yml".to_string()),
            app_name: Some("github-actions".to_string()),
        };
        assert_eq!(check.name, "CI");
        assert_eq!(check.check_type, CheckType::StatusContext);
        assert!(check.description.is_some());
        assert!(check.workflow_name.is_some());
        assert!(check.app_name.is_some());
    }

    #[test]
    fn test_check_conclusion_serialization() {
        let conclusion = CheckConclusion::ActionRequired;
        let json = serde_json::to_string(&conclusion).unwrap();
        assert_eq!(json, "\"ACTION_REQUIRED\"");
        let deserialized: CheckConclusion = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, CheckConclusion::ActionRequired);
    }

    #[test]
    fn test_rollup_state_serialization() {
        let state = RollupState::Expected;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"EXPECTED\"");
        let deserialized: RollupState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, RollupState::Expected);
    }

    #[test]
    fn test_checks_report_serialization() {
        let report = ChecksReport {
            pr_title: Some("Test".to_string()),
            pr_url: None,
            rollup_state: RollupState::Success,
            checks: vec![create_test_check("build", CheckConclusion::Success, true)],
        };
        let json = serde_json::to_string_pretty(&report).unwrap();
        assert!(json.contains("\"SUCCESS\""));
        let deserialized: ChecksReport = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.rollup_state, RollupState::Success);
        assert_eq!(deserialized.checks.len(), 1);
    }
}

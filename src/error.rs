//! Error types for the pr-comments CLI tool.

use thiserror::Error;

/// Errors that can occur when interacting with the GitHub API.
#[derive(Error, Debug)]
pub enum GitHubAPIError {
    #[error("Failed to execute gh command: {0}")]
    CommandFailed(String),

    #[error("GitHub API error: {0}")]
    ApiError(String),

    #[error("Failed to parse API response: {0}")]
    ParseError(String),

    #[error("gh CLI not found. Please install it from https://cli.github.com/")]
    GhNotFound,
}

/// Errors that can occur when parsing PR URLs.
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Cannot parse PR URL: {0}")]
    InvalidUrl(String),

    #[error("Invalid PR number: {0}")]
    InvalidPrNumber(String),
}

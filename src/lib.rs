//! PR Comments CLI Library
//!
//! A library for fetching and formatting GitHub PR comments for LLM consumption.

pub mod cli;
pub mod error;
pub mod fetcher;
pub mod formatter;
pub mod models;
pub mod parser;
pub mod sanitizer;

pub use cli::{Args, OutputFormat};
pub use error::{GitHubAPIError, ParseError};
pub use models::PRComment;

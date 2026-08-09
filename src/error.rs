//! Top-level error types

#[derive(Debug, thiserror::Error)]
#[error("Application error occurred")]
pub struct AppError;

/// A suggestion for resolving an application error
pub struct Suggestion(pub &'static str);
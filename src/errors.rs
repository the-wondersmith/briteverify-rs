//! ## Errors

// Third-Party Imports
use reqwest::header::InvalidHeaderValue;
use thiserror::Error;

/// Errors encountered when building a
/// [`BriteVerifyClient`][crate::BriteVerifyClient]
/// instance with a customized configuration
#[derive(Debug, Error)]
pub enum BriteVerifyClientError {
    /// No API key provided when constructing a
    /// [`BriteVerifyClient`][crate::BriteVerifyClient]
    /// instance
    #[error("No BriteVerify API key provided")]
    MissingApiKey,
    /// Invalid or unusable API key provided when constructing
    /// a [`BriteVerifyClient`][crate::BriteVerifyClient] instance
    #[error(transparent)]
    InvalidHeaderValue(#[from] InvalidHeaderValue),
    /// A catch-all error for any other errors encountered
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

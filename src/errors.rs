//! ## Errors

// Third-Party Imports
use reqwest::header::InvalidHeaderValue;
use thiserror::Error;

// Crate-Level Imports
use super::types::{
    AddressArrayBuilder, BulkListCRUDError, VerificationRequestBuilder, VerificationResponse,
};

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
    /// The API key provided when constructing a
    /// [`BriteVerifyClient`][crate::BriteVerifyClient]
    /// instance is either invalid or unauthorized
    #[error("Invalid or unauthorized BriteVerify API key")]
    InvalidApiKey,
    /// A request cannot be "built" for sending
    #[error("Request cannot be built!")]
    UnbuildableRequest(#[from] reqwest::Error),
    /// A request cannot be cloned when automatic
    /// rate-limit retry is enabled
    #[error("Request cannot be cloned for retry!")]
    UnclonableRequest,
    /// The BriteVerify API responded to a single-transaction
    /// verification request with data that it shouldn't have
    /// or omitted data it should have included
    #[error("Response type doesn't match expectation")]
    MismatchedVerificationResponse(Box<VerificationResponse>),
    /// No bulk verification list exists for a given identifier
    #[error("No bulk verification list found for list with id: {:?}", .0.list_id)]
    BulkListNotFound(Box<BulkListCRUDError>),
    /// Invalid or unusable API key provided when constructing
    /// a [`BriteVerifyClient`][crate::BriteVerifyClient] instance
    #[error(transparent)]
    InvalidHeaderValue(#[from] InvalidHeaderValue),
    /// Invalid or unusable base url provided when constructing
    /// a [`BriteVerifyClient`][crate::BriteVerifyClient] instance
    #[error(transparent)]
    InvalidBaseUrl(#[from] url::ParseError),
    /// A usable request could not be created
    #[error("Unusable request")]
    UnusableRequest(#[from] BriteVerifyTypeError),
    /// The BriteVerify API returned an unusable response
    /// (based on HTTP status code)
    #[error("Unusable (non-2xx) response")]
    UnusableResponse(Box<reqwest::Response>),
    /// A catch-all error for any other errors encountered
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Errors encountered when building a
/// `BriteVerifyClient`-recognized request
#[derive(Debug, Error)]
pub enum BriteVerifyTypeError {
    /// The builder state is incomplete
    #[error("Current builder state cannot be used to construct a valid `VerificationRequest`")]
    UnbuildableRequest(Box<VerificationRequestBuilder>),
    /// The builder state is incomplete
    #[error("Current builder state cannot be used to construct a valid `StreetAddressArray`")]
    UnbuildableAddressArray(Box<AddressArrayBuilder>),
    /// The value cannot be unambiguously
    /// resolved to a known request type
    #[error(
        "Value cannot be resolved to a known \
        BriteVerify API request type unambiguously: {:?}",
        .0,
    )]
    AmbiguousTryFromValue(String),
    /// A catch-all error for any other errors encountered
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

//! ## BriteVerify API Well-Known Status & Result Enumerations
///
// Standard Library Imports
use std::fmt;

// Third-Party Imports
#[doc(hidden)]
#[cfg(any(test, feature = "examples"))]
use strum_macros::EnumIter;

// Conditional Imports
#[doc(hidden)]
#[cfg(any(test, feature = "examples"))]
pub use self::foundry::*;

// <editor-fold desc="// BatchState ...">

/// The current state of a given batch verification job
#[allow(missing_docs)]
#[cfg_attr(any(test, feature = "examples"), derive(EnumIter))]
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all(deserialize = "snake_case"))]
pub enum BatchState {
    Open,
    Closed,
    Pending,
    Prepped,
    Complete,
    Delivered,
    Verifying,
    Terminated,
    ImportError,
    #[serde(other)]
    Unknown,
}

impl Default for BatchState {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn default() -> Self {
        Self::Unknown
    }
}

impl fmt::Display for BatchState {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display: String = (match self {
            Self::Open => "open",
            Self::Closed => "closed",
            Self::Pending => "pending",
            Self::Prepped => "prepped",
            Self::Unknown => "unknown",
            Self::Complete => "complete",
            Self::Delivered => "delivered",
            Self::Verifying => "verifying",
            Self::Terminated => "terminated",
            Self::ImportError => "import_error",
        })
        .to_string();

        write!(f, "{}", display)
    }
}

impl<'value, T: Into<&'value str>> From<T> for BatchState {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(value: T) -> Self {
        let value = value.into().trim().to_lowercase();

        match value.as_str() {
            "open" => Self::Open,
            "closed" => Self::Closed,
            "pending" => Self::Pending,
            "prepped" => Self::Prepped,
            "complete" => Self::Complete,
            "delivered" => Self::Delivered,
            "verifying" => Self::Verifying,
            "terminated" => Self::Terminated,
            "importerror" | "import_error" | "import-error" => Self::ImportError,
            _ => Self::Unknown,
        }
    }
}

// </editor-fold desc="// BatchState ...">

// <editor-fold desc="// VerificationStatus ...">

/// The end result of a given verification
#[allow(missing_docs)]
#[cfg_attr(any(test, feature = "examples"), derive(EnumIter))]
#[derive(Copy, Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all(deserialize = "snake_case"))]
pub enum VerificationStatus {
    Valid,
    Invalid,
    AcceptAll,
    #[serde(other)]
    Unknown,
}

impl Default for VerificationStatus {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn default() -> Self {
        Self::Unknown
    }
}

impl fmt::Display for VerificationStatus {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display: String = (match self {
            Self::Valid => "valid",
            Self::Invalid => "invalid",
            Self::Unknown => "unknown",
            Self::AcceptAll => "accept-all",
        })
        .to_string();

        write!(f, "{}", display)
    }
}

// </editor-fold desc="// VerificationStatus ...">

// <editor-fold desc="// BulkListDirective ...">

/// The current state of a given batch verification job
#[allow(missing_docs)]
#[cfg_attr(any(test, feature = "examples"), derive(EnumIter))]
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all(serialize = "snake_case", deserialize = "snake_case"))]
pub enum BulkListDirective {
    Start,
    Terminate,
    #[serde(other)]
    Unknown,
}

impl Default for BulkListDirective {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn default() -> Self {
        Self::Unknown
    }
}

impl fmt::Display for BulkListDirective {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display: String = (match self {
            Self::Start => "start",
            Self::Unknown => "unknown",
            Self::Terminate => "terminate",
        })
        .to_string();

        write!(f, "{}", display)
    }
}

impl From<bool> for BulkListDirective {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(value: bool) -> Self {
        match value {
            true => Self::Start,
            false => Self::Terminate,
        }
    }
}

impl From<String> for BulkListDirective {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl<'value> From<&'value str> for BulkListDirective {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(value: &'value str) -> Self {
        let value = value.trim().to_lowercase();

        match value.as_str() {
            "start" | "true" => Self::Start,
            "terminate" | "false" => Self::Terminate,
            _ => Self::Unknown,
        }
    }
}

// </editor-fold desc="// BulkListDirective ...">

// <editor-fold desc="// BatchCreationStatus ...">

/// The "status" of a request to create a new
/// or mutate an extant batch verification job
#[allow(missing_docs)]
#[cfg_attr(any(test, feature = "examples"), derive(EnumIter))]
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all(deserialize = "snake_case"))]
pub enum BatchCreationStatus {
    Success,
    NotFound,
    MissingData,
    ExceedsLimit,
    InvalidState,
    DuplicateData,
    ListUploadsIncomplete,
    #[serde(other)]
    Unknown,
}

impl Default for BatchCreationStatus {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn default() -> Self {
        Self::Unknown
    }
}

impl fmt::Display for BatchCreationStatus {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display: String = (match self {
            Self::Success => "success",
            Self::Unknown => "unknown",
            Self::NotFound => "not_found",
            Self::MissingData => "missing_data",
            Self::ExceedsLimit => "exceeds_limit",
            Self::InvalidState => "invalid_state",
            Self::DuplicateData => "duplicate_data",
            Self::ListUploadsIncomplete => "list_uploads_incomplete",
        })
        .to_string();

        write!(f, "{}", display)
    }
}

impl<'value, T: Into<&'value str>> From<T> for BatchCreationStatus {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(value: T) -> Self {
        let value = value.into().trim().to_lowercase();

        match value.as_str() {
            "success" => Self::Success,
            "not_found" => Self::NotFound,
            "missing_data" => Self::MissingData,
            "exceeds_limit" => Self::ExceedsLimit,
            "invalid_state" => Self::InvalidState,
            "duplicate_data" => Self::DuplicateData,
            "list_uploads_incomplete" => Self::ListUploadsIncomplete,
            _ => Self::Unknown,
        }
    }
}

// </editor-fold desc="// BatchCreationStatus ...">

// <editor-fold desc="// Test Helpers & Factory Implementations ...">

#[doc(hidden)]
#[cfg_attr(tarpaulin, no_coverage)]
#[cfg(any(test, feature = "examples"))]
mod foundry {
    // Crate-Level Imports
    use crate::utils::RandomizableEnum;

    impl RandomizableEnum for super::BatchState {}
    impl RandomizableEnum for super::BulkListDirective {}
    impl RandomizableEnum for super::VerificationStatus {}
    impl RandomizableEnum for super::BatchCreationStatus {}
}

// </editor-fold desc="// Test Helpers & Factory Implementations ...">

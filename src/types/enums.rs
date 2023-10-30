//! ## BriteVerify API Well-Known Status & Result Enumerations
///
// Standard Library Imports
use std::fmt;

// Conditional Imports
#[doc(hidden)]
#[cfg(any(test, tarpaulin))]
#[cfg_attr(any(test, tarpaulin), allow(unused_imports))]
pub use self::foundry::*;

// <editor-fold desc="// BatchState ...">

/// The current state of a given bulk verification list
#[allow(missing_docs)]
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all(serialize = "snake_case", deserialize = "snake_case"))]
pub enum BatchState {
    Open,
    Closed,
    Deleted,
    Expired,
    Pending,
    Prepped,
    Success,
    Complete,
    NotFound,
    Delivered,
    Verifying,
    Terminated,
    ImportError,
    MissingData,
    ExceedsLimit,
    InvalidState,
    DuplicateData,
    ListUploadsIncomplete,
    #[serde(other)]
    Unknown,
}

impl BatchState {
    /// Check if an instance is `Unknown`
    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}

impl Default for BatchState {
    #[cfg_attr(tarpaulin, coverage(off))]
    fn default() -> Self {
        Self::Unknown
    }
}

impl fmt::Display for BatchState {
    #[cfg_attr(tarpaulin, coverage(off))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = match self {
            Self::Open => "open",
            Self::Closed => "closed",
            Self::Deleted => "deleted",
            Self::Expired => "expired",
            Self::Pending => "pending",
            Self::Prepped => "prepped",
            Self::Success => "success",
            Self::Unknown => "unknown",
            Self::Complete => "complete",
            Self::NotFound => "notfound",
            Self::Delivered => "delivered",
            Self::Verifying => "verifying",
            Self::Terminated => "terminated",
            Self::ImportError => "importerror",
            Self::MissingData => "missingdata",
            Self::ExceedsLimit => "exceeds_limit",
            Self::InvalidState => "invalidstate",
            Self::DuplicateData => "duplicate_data",
            Self::ListUploadsIncomplete => "list_uploads_incomplete",
        };

        write!(f, "{}", display)
    }
}

impl<'value, T: Into<&'value str>> From<T> for BatchState {
    #[cfg_attr(tarpaulin, coverage(off))]
    fn from(value: T) -> Self {
        let is_quote = |val: char| -> bool { val == '"' || val == '\'' };

        let value = value
            .into()
            .trim_start_matches(is_quote)
            .trim_end_matches(is_quote)
            .trim()
            .to_lowercase();

        match value.as_str() {
            "open" => Self::Open,
            "closed" => Self::Closed,
            "deleted" => Self::Deleted,
            "expired" => Self::Expired,
            "pending" => Self::Pending,
            "prepped" => Self::Prepped,
            "success" => Self::Success,
            "complete" => Self::Complete,
            "notfound" => Self::NotFound,
            "delivered" => Self::Delivered,
            "verifying" => Self::Verifying,
            "terminated" => Self::Terminated,
            "importerror" | "import_error" | "import-error" => Self::ImportError,
            "exceedslimit" | "exceeds_limit" | "exceeds-limit" => Self::ExceedsLimit,
            "invalidstate" | "invalid_state" | "invalid-state" => Self::InvalidState,
            "duplicatedata" | "duplicate_data" | "duplicate-data" => Self::DuplicateData,
            "missing" | "missingdata" | "missing_data" | "missing-data" => Self::MissingData,
            "incomplete"
            | "uploadincomplete"
            | "uploadsincomplete"
            | "upload_incomplete"
            | "upload-incomplete"
            | "uploads_incomplete"
            | "uploads-incomplete"
            | "listuploadincomplete"
            | "listuploadsincomplete"
            | "list_uploads_incomplete"
            | "list-uploads-incomplete" => Self::ListUploadsIncomplete,
            _ => Self::Unknown,
        }
    }
}

// </editor-fold desc="// BatchState ...">

// <editor-fold desc="// VerificationError ...">

/// The end result of a given verification
#[allow(missing_docs)]
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all(serialize = "snake_case", deserialize = "snake_case"))]
pub enum VerificationError {
    Disposable,
    PMBRequired,
    RoleAddress,
    SuiteInvalid,
    SuiteMissing,
    InvalidFormat,
    InvalidPrefix,
    MultipleMatch,
    UnknownStreet,
    ZipCodeInvalid,
    BlankPhoneNumber,
    BoxNumberInvalid,
    BoxNumberMissing,
    EmailDomainInvalid,
    InvalidPhoneNumber,
    MailboxFullInvalid,
    DirectionalsInvalid,
    EmailAccountInvalid,
    EmailAddressInvalid,
    StreetNumberInvalid,
    StreetNumberMissing,
    SuiteInvalidMissing,
    MissingMinimumInputs,
    NonDeliverableAddress,
    #[serde(other)]
    Unknown,
}

impl Default for VerificationError {
    #[cfg_attr(tarpaulin, coverage(off))]
    fn default() -> Self {
        Self::Unknown
    }
}

impl fmt::Display for VerificationError {
    #[cfg_attr(tarpaulin, coverage(off))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display: String = (match self {
            Self::Unknown => "unknown",
            Self::Disposable => "disposable",
            Self::PMBRequired => "pmb_required",
            Self::RoleAddress => "role_address",
            Self::SuiteInvalid => "suite_invalid",
            Self::SuiteMissing => "suite_missing",
            Self::InvalidFormat => "invalid_format",
            Self::InvalidPrefix => "invalid_prefix",
            Self::MultipleMatch => "multiple_match",
            Self::UnknownStreet => "unknown_street",
            Self::ZipCodeInvalid => "zip_code_invalid",
            Self::BlankPhoneNumber => "blank_phone_number",
            Self::BoxNumberInvalid => "box_number_invalid",
            Self::BoxNumberMissing => "box_number_missing",
            Self::EmailDomainInvalid => "email_domain_invalid",
            Self::InvalidPhoneNumber => "invalid_phone_number",
            Self::MailboxFullInvalid => "mailbox_full_invalid",
            Self::DirectionalsInvalid => "directionals_invalid",
            Self::EmailAccountInvalid => "email_account_invalid",
            Self::EmailAddressInvalid => "email_address_invalid",
            Self::StreetNumberInvalid => "street_number_invalid",
            Self::StreetNumberMissing => "street_number_missing",
            Self::SuiteInvalidMissing => "suite_invalid_missing",
            Self::MissingMinimumInputs => "missing_minimum_inputs",
            Self::NonDeliverableAddress => "non_deliverable_address",
        })
        .to_string();

        write!(f, "{}", display)
    }
}

// </editor-fold desc="// VerificationError ...">

// <editor-fold desc="// VerificationStatus ...">

/// The end result of a given verification
#[allow(missing_docs)]
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all(serialize = "snake_case", deserialize = "snake_case"))]
pub enum VerificationStatus {
    Valid,
    Invalid,
    AcceptAll,
    #[serde(other)]
    Unknown,
}

impl Default for VerificationStatus {
    #[cfg_attr(tarpaulin, coverage(off))]
    fn default() -> Self {
        Self::Unknown
    }
}

impl fmt::Display for VerificationStatus {
    #[cfg_attr(tarpaulin, coverage(off))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = match self {
            Self::Valid => "valid",
            Self::Invalid => "invalid",
            Self::Unknown => "unknown",
            Self::AcceptAll => "accept-all",
        };

        write!(f, "{}", display)
    }
}

// </editor-fold desc="// VerificationStatus ...">

// <editor-fold desc="// BulkListDirective ...">

/// The current state of a given batch verification job
#[allow(missing_docs)]
#[derive(Copy, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all(serialize = "snake_case", deserialize = "snake_case"))]
pub enum BulkListDirective {
    Start,
    Terminate,
    #[serde(other)]
    Unknown,
}

impl Default for BulkListDirective {
    #[cfg_attr(tarpaulin, coverage(off))]
    fn default() -> Self {
        Self::Unknown
    }
}

impl fmt::Display for BulkListDirective {
    #[cfg_attr(tarpaulin, coverage(off))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let display = match self {
            Self::Start => "start",
            Self::Unknown => "unknown",
            Self::Terminate => "terminate",
        };

        write!(f, "{}", display)
    }
}

impl From<bool> for BulkListDirective {
    #[cfg_attr(tarpaulin, coverage(off))]
    fn from(value: bool) -> Self {
        match value {
            true => Self::Start,
            false => Self::Unknown,
        }
    }
}

impl From<String> for BulkListDirective {
    #[cfg_attr(tarpaulin, coverage(off))]
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl<'value> From<&'value str> for BulkListDirective {
    #[cfg_attr(tarpaulin, coverage(off))]
    fn from(value: &'value str) -> Self {
        let value = value.trim().to_lowercase();

        match value.as_str() {
            "start" | "true" => Self::Start,
            "terminate" | "stop" => Self::Terminate,
            _ => Self::Unknown,
        }
    }
}

impl<T: ToString> From<Option<T>> for BulkListDirective {
    #[cfg_attr(tarpaulin, coverage(off))]
    fn from(value: Option<T>) -> Self {
        Self::from(value.map_or(String::new(), |directive| directive.to_string()))
    }
}

// </editor-fold desc="// BulkListDirective ...">

// <editor-fold desc="// Test Helpers & Factory Implementations ...">

#[doc(hidden)]
#[cfg(any(test, tarpaulin))]
mod foundry {}

// </editor-fold desc="// Test Helpers & Factory Implementations ...">

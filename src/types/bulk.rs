//! ## BriteVerify Bulk API Types [[ref](https://docs.briteverify.com/#944cd18b-8cad-43c2-9e47-7b1e91ba5935)]

// Standard Library Imports
use std::{fmt, ops::Deref};

// Third Party Imports
use chrono::prelude::{DateTime, Utc};
use http::Uri;

// Crate-Level Imports
use super::{
    enums::{BatchState, BulkListDirective, VerificationStatus},
    single::{AddressVerificationArray, VerificationRequest},
};

// Conditional Imports
#[doc(hidden)]
#[cfg(any(test, tarpaulin, feature = "ci"))]
#[cfg_attr(any(test, tarpaulin, feature = "ci"), allow(unused_imports))]
pub use self::foundry::*;

// <editor-fold desc="// Bulk Requests ...">

// <editor-fold desc="// BulkVerificationRequest ...">

/// A request for verification of multiple "contact" records
#[cfg_attr(any(test, tarpaulin, feature = "ci"), derive(PartialEq))]
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct BulkVerificationRequest {
    /// The "contact" records to be verified
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contacts: Vec<VerificationRequest>,
    /// An (optional) directive for how
    /// the request should be processed.
    ///
    /// For example:
    /// - "start" -> start processing now
    /// - "terminate" -> stop processing, if not yet complete
    #[serde(
        default,
        skip_serializing_if = "crate::utils::is_unknown_list_directive"
    )]
    pub directive: BulkListDirective,
}

impl BulkVerificationRequest {
    /// Create a new `BulkVerificationRequest` for the supplied
    /// contacts with the (optionally) supplied directive.
    pub fn new<
        Contact: Into<VerificationRequest>,
        Directive: Into<BulkListDirective>,
        ContactCollection: IntoIterator<Item = Contact>,
    >(
        contacts: ContactCollection,
        directive: Directive,
    ) -> Self {
        let contacts: Vec<VerificationRequest> = contacts.into_iter().map(Contact::into).collect();

        let directive: BulkListDirective = directive.into();

        BulkVerificationRequest {
            contacts,
            directive,
        }
    }
}

// </editor-fold desc="// BulkVerificationRequest ...">

// </editor-fold desc="// Bulk Requests ...">

// <editor-fold desc="// Bulk Responses ...">

// <editor-fold desc="// BulkListCRUDError ...">

/// An error message returned by the BriteVerify API
#[cfg_attr(any(test, tarpaulin, feature = "ci"), derive(PartialEq))]
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct BulkListCRUDError {
    /// A list's BriteVerify API-issued identifier
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::utils::empty_string_is_none"
    )]
    pub list_id: Option<String>,
    /// A status identifier or error code
    #[serde(
        default,
        alias = "code",
        skip_serializing_if = "BatchState::is_unknown"
    )]
    pub status: BatchState,
    /// A human-oriented message containing
    /// pertinent information about the data
    /// in the response
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::utils::empty_string_is_none"
    )]
    pub message: Option<String>,
}

// </editor-fold desc="// BulkListCRUDError ...">

// <editor-fold desc="// VerificationListState ...">

/// Details of the current "state" of a bulk verification
/// job / request / "list" ([ref](https://docs.briteverify.com/#0b5a2a7a-4062-4327-ab0a-4675592e3cd6))
#[cfg_attr(any(test, tarpaulin, feature = "ci"), derive(PartialEq))]
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct VerificationListState {
    /// The list's unique identifier, issued by
    /// and specific to the BriteVerify API.
    pub id: String,
    /// The list's account-specific, user-supplied
    /// identifier.
    ///
    /// ___
    /// **NOTE:** Lists cannot be uniquely identified
    /// by this value, as this value exclusively functions
    /// as a shared point of reference for a specific
    /// down-stream client of a given user. **If a list
    /// is _created_ with an external id, it cannot
    /// be retrieved or referenced without supplying
    /// the same id as part of the request**.
    ///
    /// This field is offered by the
    /// BriteVerify API as a way to associate an
    /// identifier with lists that might processed
    /// on behalf of a user's down-stream clients
    /// and noted as being ideal for and primarily
    /// used by agencies or resellers.
    /// [[ref](https://docs.briteverify.com/#38b4c9eb-31b1-4b8e-9295-a783d8043bc1:~:text=URL%20Parameters-,external_id,-(optional))]
    /// ___
    #[serde(
        default,
        alias = "account_external_id",
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::utils::deserialize_ext_id"
    )]
    pub external_id: Option<String>,
    /// The list's current "state" (i.e. its
    /// current place in the general flow from
    /// "newly created" to "completely processed")
    #[serde(default)]
    pub state: BatchState,
    /// The number of the list's associated records
    /// that have been processed, as an integer
    /// percentage out of 100 (e.g. 10/100 -> 10)
    #[serde(default)]
    pub progress: u64,
    /// The total number of the list's associated
    /// records that have already been processed
    #[serde(default)]
    pub total_verified: u64,
    /// The list's total number of result "pages"
    ///
    /// > **NOTE:** this field will only ever be
    /// > populated if the list's current state
    /// > is "completed"
    #[serde(default)]
    pub page_count: Option<u64>,
    /// The total number of "bare" email addresses
    /// from the list's associated records that have
    /// already been processed
    #[serde(default)]
    pub total_verified_emails: u64,
    /// The total number of "bare" phone numbers
    /// from the list's associated records that have
    /// already been processed
    #[serde(default)]
    pub total_verified_phones: u64,
    /// The timestamp of the list's initial creation
    ///
    /// > **NOTE:** the BriteVerify API documentation
    /// > does not explicitly specify a timezone for
    /// > these timestamps, but observed behavior seems
    /// > to indicate they are UTC. Until BriteVerify
    /// > explicitly states otherwise, `briteverify_rs`
    /// > will continue to parse all timestamp fields
    /// > with an assumed timezone of UTC.
    #[cfg_attr(
        any(test, tarpaulin, feature = "ci"),
        serde(serialize_with = "crate::utils::serialize_timestamp")
    )]
    #[serde(deserialize_with = "crate::utils::deserialize_timestamp")]
    pub created_at: DateTime<Utc>,
    /// The URL at which the list's processed results
    /// can be retrieved
    ///
    /// > **NOTE:** observed behavior indicates that
    /// > this URL will always point to the *first page*
    /// > of the list's results. The value of this response's
    /// > `page_count` field should be referenced when
    /// > actually *retrieving* results to determine
    /// > the total number of pages that need to be
    /// > fetched.
    #[serde(
        default,
        serialize_with = "crate::utils::serialize_uri",
        deserialize_with = "crate::utils::deserialize_uri"
    )]
    pub results_path: Option<Uri>,
    /// The date/time after which the list's results
    /// will expire, and will therefore no longer be
    /// visible / retrievable from the BriteVerify API
    #[serde(
        default,
        deserialize_with = "crate::utils::deserialize_maybe_timestamp"
    )]
    pub expiration_date: Option<DateTime<Utc>>,
    /// A list of error encountered by the BriteVerify API
    /// while processing the list's associated records
    #[serde(default = "Vec::new")]
    pub errors: Vec<BulkListCRUDError>,
}

// </editor-fold desc="// VerificationListState ...">

// <editor-fold desc="// GetListStatesResponse ...">

/// All bulk verification lists created within
/// the last 7 calendar days, optionally filtered
/// by any user-specified parameters (e.g. `date`,
/// `page`, or `state`)
#[cfg_attr(any(test, tarpaulin, feature = "ci"), derive(PartialEq))]
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct GetListStatesResponse {
    /// Usually page numbers (if provided)
    #[serde(default)]
    pub message: Option<String>,
    /// A list of [`VerificationListState`](VerificationListState)s
    /// matching any provided filters (defaults to all
    /// extant lists if no filters are specified).
    #[serde(default)]
    pub lists: Vec<VerificationListState>,
}

impl Deref for GetListStatesResponse {
    type Target = Vec<VerificationListState>;

    #[cfg_attr(tarpaulin, coverage(off))]
    fn deref(&self) -> &Self::Target {
        &self.lists
    }
}

impl GetListStatesResponse {
    /// The `id`s of the collected `VerificationListState`s
    pub fn ids(&self) -> Vec<&str> {
        self.lists.iter().map(|list| list.id.as_str()).collect()
    }

    /// Extract the current page references from
    /// the response's [`message`](GetListStatesResponse::message)
    /// field if it is populated.
    /// ___
    /// **NOTE:** This implementation is predicated on
    /// an observed "pattern" in the official response
    /// examples that shows page-related messages as
    /// always having the format "Page X of Y".
    ///
    /// This method *will* fail in potentially strange
    /// ways if that ever changes or simply proves to
    /// be inaccurate. It will not, however, cause a
    /// panic. It will simply return (1, 1) with no
    /// regard for the veracity of those values.
    fn _pages(&self) -> (u64, u64) {
        match self.message.as_ref() {
            None => (1u64, 1u64),
            Some(message) => {
                let mut values = message
                    .split(' ')
                    .map(|value| value.parse::<u64>())
                    .filter(Result::is_ok)
                    .map(Result::unwrap);

                (values.next().unwrap_or(1u64), values.next().unwrap_or(1u64))
            }
        }
    }

    /// Get the get the current "page" number with
    /// relative to the total number of list "pages"
    /// matching the filter criteria that resulted in
    /// the current response
    pub fn current_page(&self) -> u64 {
        self._pages().0
    }

    /// Get the total number of available list "pages"
    /// matching the filter criteria that resulted in
    /// the current response
    pub fn total_pages(&self) -> u64 {
        self._pages().1
    }

    /// Get a specific `VerificationListState` from the collection by `id`
    pub fn get_list_by_id<ListId: fmt::Display>(
        &self,
        list_id: ListId,
    ) -> Option<&VerificationListState> {
        let list_id = list_id.to_string();

        self.lists.iter().find(|list| list.id == list_id)
    }
}

// </editor-fold desc="// GetListStatesResponse ...">

// <editor-fold desc="// BulkListCRUDResponse ...">

/// The BriteVerify API's response to a valid,
/// well-formed request to create, update, or
/// delete a bulk verification list
#[cfg_attr(any(test, tarpaulin, feature = "ci"), derive(PartialEq))]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BulkListCRUDResponse {
    /// The current "status" of the
    /// created / updated / deleted list
    #[serde(default, alias = "code")]
    pub status: BatchState,
    /// A human-oriented message containing
    /// pertinent information about the result
    /// of the requested operation
    #[serde(default)]
    pub message: String,
    /// Details of the associated list's
    /// current "state"
    pub list: VerificationListState,
}

// </editor-fold desc="// BulkListCRUDResponse ...">

/// The BriteVerify API's response to a request to
/// create a new bulk verification list
pub type CreateListResponse = BulkListCRUDResponse;

/// The BriteVerify API's response to a request to
/// mutate an extant bulk verification list
pub type UpdateListResponse = BulkListCRUDResponse;

/// The BriteVerify API's response to a request
/// to delete an extant bulk verification list
pub type DeleteListResponse = BulkListCRUDResponse;

// <editor-fold desc="// BulkEmailVerificationArray ...">

/// The `email` element of a bulk verification result
/// record returned by the BriteVerify API
#[cfg_attr(any(test, tarpaulin, feature = "ci"), derive(PartialEq))]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BulkEmailVerificationArray {
    /// The verified email address
    pub email: String,
    /// The email address's validity "status"
    /// ([ref](https://knowledge.validity.com/hc/en-us/articles/360047111771-Understanding-Statuses-in-BriteVerify#h_01F79WHSGY6FJ6YN1083JWR3QJ))
    pub status: VerificationStatus,
    /// The email address's "secondary" validity status
    pub secondary_status: Option<String>,
}

// </editor-fold desc="// BulkEmailVerificationArray ...">

// <editor-fold desc="// BulkPhoneNumberVerificationArray ...">

/// The `phone` element of a bulk verification result
/// record returned by the BriteVerify API
#[cfg_attr(any(test, tarpaulin, feature = "ci"), derive(PartialEq))]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BulkPhoneNumberVerificationArray {
    /// The verified phone number
    pub phone: String,
    /// The phone number's validity "status"
    /// ([ref](https://knowledge.validity.com/hc/en-us/articles/360047111771-Understanding-Statuses-in-BriteVerify#h_01F79WJXQFFEHWKTJPHPG944NS))
    pub status: VerificationStatus,
    /// The geographical area within which
    /// the phone number was initially registered
    /// or should be considered "valid"
    ///
    /// > **NOTE:** from observed behavior, this
    /// > field is never *not* `null`
    pub phone_location: Option<String>,
    /// The phone number's "secondary" validity status
    pub secondary_status: Option<String>,
    /// The "type" of service the phone number
    /// most likely uses (e.g. "land line", "mobile", etc..)
    #[serde(rename(serialize = "phone_service_type", deserialize = "phone_service_type"))]
    pub service_type: Option<String>,
}

// </editor-fold desc="// BulkPhoneNumberVerificationArray ...">

/// The `address` element of a bulk verification
/// result record returned by the BriteVerify API
pub type BulkAddressVerificationArray = AddressVerificationArray;

// <editor-fold desc="// BulkVerificationResult ...">

/// A single result record returned by
/// the BriteVerify bulk verification API
/// for "contacts"-type requests
#[cfg_attr(any(test, tarpaulin, feature = "ci"), derive(PartialEq))]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BulkContactVerificationResult {
    /// Verification data for the requested
    /// email address
    #[serde(default)]
    pub email: Option<BulkEmailVerificationArray>,
    /// Verification data for the requested
    /// phone number
    #[serde(default)]
    pub phone: Option<BulkPhoneNumberVerificationArray>,
    /// Verification data for the requested
    /// street address
    #[serde(default)]
    pub address: Option<BulkAddressVerificationArray>,
}

/// A single result record returned by
/// the BriteVerify bulk verification API
#[cfg_attr(any(test, tarpaulin, feature = "ci"), derive(PartialEq))]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum BulkVerificationResult {
    /// A single result record returned by
    /// the BriteVerify bulk verification API
    /// for "contacts"-type requests
    Contact(BulkContactVerificationResult),
    /// A single result record returned by
    /// the BriteVerify bulk verification API
    /// for "email"-type requests
    Email(BulkEmailVerificationArray),
}

impl fmt::Debug for BulkVerificationResult {
    #[cfg_attr(tarpaulin, coverage(off))]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Email(response) => fmt::Debug::fmt(response, formatter),
            Self::Contact(response) => fmt::Debug::fmt(response, formatter),
        }
    }
}

// </editor-fold desc="// BulkVerificationResult ...">

// <editor-fold desc="// BulkVerificationResponse ...">

/// A "page" of result records and associated
/// metadata returned by the BriteVerify bulk
/// verification API
#[cfg_attr(any(test, tarpaulin, feature = "ci"), derive(PartialEq))]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BulkVerificationResponse {
    /// The current "status" of the bulk
    /// verification list
    #[serde(default)]
    pub status: BatchState,
    /// The total number of result "pages"
    /// associated with the verification list
    #[serde(default, alias = "num_pages")]
    pub page_count: u64,
    /// A "page" of verification result records
    #[serde(default)]
    pub results: Vec<BulkVerificationResult>,
}

// </editor-fold desc="// BulkVerificationResponse ...">

// <editor-fold desc="// Bulk Responses ...">

// <editor-fold desc="// Test Helpers & Factory Implementations ...">

#[doc(hidden)]
#[cfg(any(test, tarpaulin, feature = "ci"))]
mod foundry {

    impl<
            Contact: Into<super::VerificationRequest>,
            ContactCollection: IntoIterator<Item = Contact>,
        > From<Option<ContactCollection>> for super::BulkVerificationRequest
    {
        #[cfg_attr(tarpaulin, coverage(off))]
        #[cfg_attr(tarpaulin, tarpaulin::skip)]
        fn from(value: Option<ContactCollection>) -> Self {
            match value {
                None => Self::default(),
                Some(contacts) => {
                    let contacts = contacts
                        .into_iter()
                        .map(Contact::into)
                        .collect::<Vec<super::VerificationRequest>>();

                    Self {
                        contacts,
                        ..Self::default()
                    }
                }
            }
        }
    }
}

// </editor-fold desc="// Test Helpers & Factory Implementations ...">

// <editor-fold desc="// I/O-Free Tests ...">

#[cfg(test)]
mod tests {
    // Third-Party Dependencies
    use crate::types::GetListStatesResponse;
    use pretty_assertions::assert_eq;

    /// Test that the `BulkVerificationRequest`'s
    /// `new` constructor method behaves as expected
    #[rstest::rstest]
    fn test_new_bulk_verification_request() {
        let req = super::BulkVerificationRequest::new(
            Vec::<super::VerificationRequest>::new(),
            Option::<&str>::None,
        );

        assert!(req.contacts.is_empty());
        assert_eq!(req.directive, super::BulkListDirective::Unknown);
    }

    /// Test that the `GetListStatesResponse`'s
    /// `_pages` utility method behaves as expected
    #[rstest::rstest]
    fn test_list_state_pages() {
        let no_message = GetListStatesResponse::default();
        let some_message = GetListStatesResponse {
            message: Some("Page 12 of 345".to_string()),
            lists: Vec::new(),
        };

        assert_eq!(
            (1u64, 1u64),
            (no_message.current_page(), no_message.total_pages())
        );
        assert_eq!(
            (12u64, 345u64),
            (some_message.current_page(), some_message.total_pages())
        );
    }
}

// </editor-fold desc="// I/O-Free Tests ...">

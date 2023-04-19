//! ## BriteVerify Bulk API Types [[ref](https://docs.briteverify.com/#944cd18b-8cad-43c2-9e47-7b1e91ba5935)]
///
// Standard Library Imports
use std::{collections::HashMap, fmt, ops::Deref};

// Third Party Imports
use anyhow::Result;
use chrono::prelude::{DateTime, Utc};
use http::Uri;
use serde::ser::SerializeStruct;

// Crate-Level Imports
use super::{
    enums::{BatchCreationStatus, BatchState, BulkListDirective, VerificationStatus},
    single::{AddressVerificationArray, VerificationRequest},
};

// Conditional Imports
#[doc(hidden)]
#[cfg(any(test, feature = "examples"))]
pub use self::foundry::*;

// <editor-fold desc="// Bulk Requests ...">

// <editor-fold desc="// BulkVerificationRequest ...">

/// A request for verification of multiple "contact" records
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BulkVerificationRequest {
    /// The "contact" records to be verified
    pub contacts: Vec<VerificationRequest>,
    /// An (optional) directive for how
    /// the request should be processed.
    ///
    /// For example:
    /// - "start" -> start processing now
    /// - "terminate" -> stop processing, if not yet complete
    #[serde(skip_serializing_if = "crate::utils::is_unknown_list_directive")]
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
        let contacts: Vec<VerificationRequest> =
            contacts.into_iter().map(|contact| contact.into()).collect();

        let directive: BulkListDirective = directive.into();

        BulkVerificationRequest {
            contacts,
            directive,
        }
    }
}

impl Default for BulkVerificationRequest {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn default() -> Self {
        BulkVerificationRequest {
            contacts: Vec::new(),
            directive: BulkListDirective::Unknown,
        }
    }
}

// </editor-fold desc="// BulkVerificationRequest ...">

// </editor-fold desc="// Bulk Requests ...">

// <editor-fold desc="// Bulk Responses ...">

// <editor-fold desc="// VerificationListErrorMessage ...">

/// A structured representation of an error encountered
/// by the BriteVerify API in the process of fulfilling
/// a bulk verification request
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct VerificationListErrorMessage {
    /// A simple identifier for the "type"
    /// of error encountered
    ///
    /// > **NOTE:** the only error code explicitly
    /// > present in the documentation for the
    /// > BriteVerify API is `import_error`. It
    /// > is currently unknown if any other error
    /// > type is ever returned by the API.
    pub code: String,
    /// A more detailed, human-oriented
    /// explanation of the error
    pub message: String,
}

// </editor-fold desc="// VerificationListErrorMessage ...">

// <editor-fold desc="// VerificationListState ...">

/// Details of the current "state" of a bulk verification
/// job / request / "list" ([ref](https://docs.briteverify.com/#0b5a2a7a-4062-4327-ab0a-4675592e3cd6))
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct VerificationListState {
    /// The list's unique identifier, issued by
    /// and specific to the BriteVerify API.
    ///
    /// > **NOTE:** this field is the list id used by
    /// > the BriteVerify API itself. It is not
    /// > clear whether or not list statuses can
    /// > be requested by client-specified `external_id`s
    /// > ([ref](https://docs.briteverify.com/#0b5a2a7a-4062-4327-ab0a-4675592e3cd6:~:text=customer%2DID/lists-,_Note,-%3A_If%20a))
    pub id: String,
    /// The list's current "state" (i.e. its
    /// current place in the general flow from
    /// "newly created" to "completely processed")
    pub state: BatchState,
    /// The number of the list's associated records
    /// that have been processed, as an integer
    /// percentage out of 100 (e.g. 10/100 -> 10)
    pub progress: u64,
    /// The total number of the list's associated
    /// records that have already been processed
    pub total_verified: u64,
    /// The list's total number of result "pages"
    ///
    /// > **NOTE:** this field will only ever be
    /// > populated if the list's current state
    /// > is "completed"
    pub page_count: Option<u64>,
    /// The total number of "bare" email addresses
    /// from the list's associated records that have
    /// already been processed
    pub total_verified_emails: u64,
    /// The total number of "bare" phone numbers
    /// from the list's associated records that have
    /// already been processed
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
        serialize_with = "crate::utils::serialize_uri",
        deserialize_with = "crate::utils::deserialize_uri"
    )]
    pub results_path: Option<Uri>,
    /// The date/time after which the list's results
    /// will expire, and will therefore no longer be
    /// visible / retrievable from the BriteVerify API
    #[serde(deserialize_with = "crate::utils::deserialize_maybe_timestamp")]
    pub expiration_date: Option<DateTime<Utc>>,
    /// A list of error encountered by the BriteVerify API
    /// while processing the list's associated records
    #[serde(default = "Vec::new")]
    pub errors: Vec<VerificationListErrorMessage>,
}

// </editor-fold desc="// VerificationListState ...">

// <editor-fold desc="// GetListStatesResponse ...">

/// All bulk verification lists created within
/// the last 7 calendar days, optionally filtered
/// by any user-specified parameters (e.g. `date`,
/// `page`, or `state`)
#[derive(Debug, Default)]
pub struct GetListStatesResponse(Vec<VerificationListState>);

impl Deref for GetListStatesResponse {
    type Target = Vec<VerificationListState>;

    #[cfg_attr(tarpaulin, no_coverage)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl serde::Serialize for GetListStatesResponse {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("GetListStatesResponse", 1)?;
        state.serialize_field("lists", &self.0)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for GetListStatesResponse {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut data =
            <HashMap<String, Vec<VerificationListState>> as serde::Deserialize>::deserialize(
                deserializer,
            )?;

        Ok(Self(data.remove("lists").unwrap_or_default()))
    }
}

// </editor-fold desc="// GetListStatesResponse ...">

// <editor-fold desc="// CreateListErrorResponse ...">

/// The BriteVerify API's response to a faulty,
/// improperly constructed, or otherwise flawed
/// request to create a new bulk verification list
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateListErrorResponse {
    /// A simple identifier for the "type"
    /// of error encountered
    #[serde(alias = "status")]
    pub code: BatchCreationStatus,
    /// A more detailed, human-oriented
    /// explanation of the error
    pub message: String,
}

// </editor-fold desc="// CreateListErrorResponse ...">

/// The BriteVerify API's response to a faulty,
/// improperly constructed, or otherwise flawed
/// request to delete an extant bulk verification
/// list
pub type DeleteListErrorResponse = CreateListErrorResponse;

// <editor-fold desc="// CreateListSuccessResponse ...">

/// The BriteVerify API's response to a valid,
/// well-formatted request to create a new bulk
/// verification list
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateListSuccessResponse {
    /// The current "status" of the
    /// created / updated list
    pub status: BatchCreationStatus,
    /// A human-oriented message containing
    /// pertinent information about the result
    /// of the requested operation
    pub message: String,
    /// Details of the associated list's
    /// current "state"
    pub list: VerificationListState,
}

// </editor-fold desc="// CreateListSuccessResponse ...">

// <editor-fold desc="// DeleteListSuccessResponse ...">

/// The BriteVerify API's response to a successful
/// request to delete an extant bulk verification list
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DeleteListSuccessResponse {
    /// The final "status" of the deleted list
    ///
    /// > **NOTE:** theoretically, this field's
    /// > value should always be "success" as
    /// > `briteverify_rs` will deserialize the
    /// > result of a *bad* `[DELETE]` call
    /// > as a [`DeleteListErrorResponse`](DeleteListErrorResponse)
    pub status: BatchCreationStatus,
    /// The final "state" of the deleted list
    pub list: VerificationListState,
}

// </editor-fold desc="// DeleteListSuccessResponse ...">

// <editor-fold desc="// CreateListResponse ...">

/// The BriteVerify API's response to a request to
/// create a new bulk verification list
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum CreateListResponse {
    /// The BriteVerify API's response to a valid,
    /// well-formatted request to create a new bulk
    /// verification list
    Success(CreateListSuccessResponse),
    /// The BriteVerify API's response to a faulty,
    /// improperly constructed, or otherwise flawed
    /// request to create a new bulk verification list
    Failed(CreateListErrorResponse),
}

// </editor-fold desc="// CreateListResponse ...">

/// The BriteVerify API's response to a request to
/// mutate an extant bulk verification list
pub type UpdateListResponse = CreateListResponse;

// <editor-fold desc="// DeleteListResponse ...">

/// The BriteVerify API's response to a request
/// to delete an extant bulk verification list
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum DeleteListResponse {
    /// The BriteVerify API's response to a successful
    /// request to delete an extant bulk verification list
    Success(DeleteListSuccessResponse),
    /// The BriteVerify API's response to a faulty,
    /// improperly constructed, or otherwise flawed
    /// request to delete an extant bulk verification
    /// list
    Failed(DeleteListErrorResponse),
}

// </editor-fold desc="// DeleteListResponse ...">

// <editor-fold desc="// BulkEmailVerificationArray ...">

/// The `email` element of a bulk verification result
/// record returned by the BriteVerify API
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

/// A BriteVerify bulk API email-only verification result
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BulkEmailVerificationResult {
    /// Verification data for the requested
    /// email address
    pub email: BulkEmailVerificationArray,
}

/// A BriteVerify bulk API phone number-only verification result
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BulkPhoneNumberVerificationResult {
    /// Verification data for the requested
    /// phone number
    pub phone: BulkPhoneNumberVerificationArray,
}

/// A BriteVerify bulk API street address-only verification result
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BulkAddressVerificationResult {
    /// Verification data for the requested
    /// street address
    pub address: BulkAddressVerificationArray,
}

/// A BriteVerify bulk API "complete" verification result
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BulkFullVerificationResult {
    /// Verification data for the requested
    /// email address
    pub email: BulkEmailVerificationArray,
    /// Verification data for the requested
    /// phone number
    pub phone: BulkPhoneNumberVerificationArray,
    /// Verification data for the requested
    /// street address
    pub address: BulkAddressVerificationArray,
}

/// A BriteVerify bulk API verification result for
/// one email address and one phone number
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BulkEmailAndPhoneVerificationResult {
    /// Verification data for the requested
    /// email address
    pub email: BulkEmailVerificationArray,
    /// Verification data for the requested
    /// phone number
    pub phone: BulkPhoneNumberVerificationArray,
}

/// A BriteVerify bulk API verification result for
/// one email address and one street address
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BulkEmailAndAddressVerificationResult {
    /// Verification data for the requested
    /// email address
    pub email: BulkEmailVerificationArray,
    /// Verification data for the requested
    /// street address
    pub address: BulkAddressVerificationArray,
}

/// A BriteVerify bulk API verification result for
/// one phone number and one street address
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BulkPhoneAndAddressVerificationResult {
    /// Verification data for the requested
    /// phone number
    pub phone: BulkPhoneNumberVerificationArray,
    /// Verification data for the requested
    /// street address
    pub address: BulkAddressVerificationArray,
}

/// A single result record returned by
/// the BriteVerify bulk verification API
/// for "contacts"-type requests
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum BulkContactVerificationResult {
    //////////////////////// NOTE ////////////////////////
    // `serde`'s "untagged" behavior depends on         //
    // the order of these variants, so they should      //
    // always be ordered from most to least "complete"  //
    //////////////////////////////////////////////////////
    //
    /// A BriteVerify bulk API "complete" verification result
    Full(BulkFullVerificationResult),
    /// A BriteVerify bulk API verification result for
    /// one email address and one phone number
    EmailAndPhone(BulkEmailAndPhoneVerificationResult),
    /// A BriteVerify bulk API verification result for
    /// one email address and one street address
    EmailAndAddress(BulkEmailAndAddressVerificationResult),
    /// A BriteVerify bulk API verification result for
    /// one phone number and one street address
    PhoneAndAddress(BulkPhoneAndAddressVerificationResult),
    /// A BriteVerify bulk API email-only verification result
    Email(BulkEmailVerificationResult),
    /// A BriteVerify bulk API phone number-only verification result
    Phone(BulkPhoneNumberVerificationResult),
    /// A BriteVerify bulk API street address-only verification result
    Address(BulkAddressVerificationResult),
}

/// A single result record returned by
/// the BriteVerify bulk verification API
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

impl fmt::Debug for BulkContactVerificationResult {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Full(response) => fmt::Debug::fmt(response, formatter),
            Self::Email(response) => fmt::Debug::fmt(response, formatter),
            Self::Phone(response) => fmt::Debug::fmt(response, formatter),
            Self::Address(response) => fmt::Debug::fmt(response, formatter),
            Self::EmailAndPhone(response) => fmt::Debug::fmt(response, formatter),
            Self::EmailAndAddress(response) => fmt::Debug::fmt(response, formatter),
            Self::PhoneAndAddress(response) => fmt::Debug::fmt(response, formatter),
        }
    }
}

impl fmt::Debug for BulkVerificationResult {
    #[cfg_attr(tarpaulin, no_coverage)]
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
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BulkVerificationResponse {
    /// The current "status" of the bulk
    /// verification list
    pub status: BatchCreationStatus,
    /// The total number of result "pages"
    /// associated with the verification list
    #[serde(alias = "num_pages")]
    pub page_count: u64,
    /// A "page" of verification result records
    pub results: Vec<BulkVerificationResult>,
}

// </editor-fold desc="// BulkVerificationResponse ...">

// <editor-fold desc="// Bulk Responses ...">

// <editor-fold desc="// Test Helpers & Factory Implementations ...">

#[doc(hidden)]
#[cfg(any(test, feature = "examples"))]
mod foundry {
    // Third Party Imports
    use chrono::Datelike;
    use warlocks_cauldron as wc;

    // Crate-Level Imports
    use crate::utils::{RandomizableEnum, RandomizableStruct};

    impl super::CreateListResponse {
        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a random successful `CreateListResponse`
        pub fn random_success() -> Self {
            Self::Success(super::CreateListSuccessResponse::random())
        }

        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a random failed `CreateListResponse`
        pub fn random_failure() -> Self {
            Self::Failed(super::CreateListErrorResponse::random())
        }
    }

    impl super::DeleteListResponse {
        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a random successful `DeleteListResponse`
        pub fn random_success() -> Self {
            Self::Success(super::DeleteListSuccessResponse::random())
        }

        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a random failed `DeleteListResponse`
        pub fn random_failure() -> Self {
            Self::Failed(super::DeleteListErrorResponse::random())
        }
    }

    impl super::GetListStatesResponse {
        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a `GetListStatesResponse` with the
        /// (optional) number of list states.
        ///
        /// If a value if not supplied for `count`, a random
        /// number between 5 and 100 will be used instead.
        pub fn random(count: Option<u32>) -> Self {
            let count = match count {
                Some(value) => value,
                None => wc::Numeric::number(5u32, 100u32),
            };

            let responses = (0..count)
                .into_iter()
                .map(|_| super::VerificationListState::random())
                .collect::<Vec<super::VerificationListState>>();

            Self(responses)
        }
    }

    impl super::BulkVerificationResult {
        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a random "email"-type `BulkVerificationResult`
        pub fn random_email_result() -> Self {
            Self::Email(super::BulkEmailVerificationArray::random())
        }

        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a random "contact"-type `BulkVerificationResult`
        pub fn random_contact_result() -> Self {
            Self::Contact(super::BulkContactVerificationResult::random())
        }
    }

    impl super::BulkContactVerificationResult {
        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a random "full"-type `BulkContactVerificationResult`
        fn random_full_result() -> Self {
            Self::Full(super::BulkFullVerificationResult::random())
        }

        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a random "email"-type `BulkContactVerificationResult`
        fn random_email_result() -> Self {
            Self::Email(super::BulkEmailVerificationResult::random())
        }

        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a random "phone"-type `BulkContactVerificationResult`
        fn random_phone_result() -> Self {
            Self::Phone(super::BulkPhoneNumberVerificationResult::random())
        }

        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a random "address"-type `BulkContactVerificationResult`
        fn random_address_result() -> Self {
            Self::Address(super::BulkAddressVerificationResult::random())
        }

        /// Generate a random "email and phone"-type `BulkContactVerificationResult`
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random_email_and_phone_result() -> Self {
            Self::EmailAndPhone(super::BulkEmailAndPhoneVerificationResult::random())
        }

        /// Generate a random "email and address"-type `BulkContactVerificationResult`
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random_email_and_address_result() -> Self {
            Self::EmailAndAddress(super::BulkEmailAndAddressVerificationResult::random())
        }

        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a random "phone and address"-type `BulkContactVerificationResult`
        fn random_phone_and_address_result() -> Self {
            Self::PhoneAndAddress(super::BulkPhoneAndAddressVerificationResult::random())
        }
    }

    impl Default for super::VerificationListState {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn default() -> Self {
            Self {
                id: uuid::Uuid::new_v4().to_string(),
                state: super::BatchState::default(),
                errors: Vec::new(),
                progress: 0u64,
                created_at: super::Utc::now(),
                page_count: None,
                results_path: None,
                total_verified: 0u64,
                expiration_date: None,
                total_verified_emails: 0u64,
                total_verified_phones: 0u64,
            }
        }
    }

    impl RandomizableStruct for super::CreateListResponse {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            if wc::Choice::prob(0.50) {
                Self::random_success()
            } else {
                Self::random_failure()
            }
        }
    }

    impl RandomizableStruct for super::DeleteListResponse {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            if wc::Choice::prob(0.50) {
                Self::random_success()
            } else {
                Self::random_failure()
            }
        }
    }

    impl RandomizableStruct for super::BulkVerificationResult {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            if wc::Choice::prob(0.50) {
                Self::random_email_result()
            } else {
                Self::random_contact_result()
            }
        }
    }

    impl RandomizableStruct for super::VerificationListState {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            let mut instance = Self {
                state: super::BatchState::random(),
                created_at: crate::utils::within_the_last_week(),
                ..Self::default()
            };

            match &instance.state {
                super::BatchState::Closed
                | super::BatchState::Pending
                | super::BatchState::Complete
                | super::BatchState::Delivered
                | super::BatchState::Verifying => {
                    if [super::BatchState::Pending, super::BatchState::Verifying]
                        .contains(&instance.state)
                    {
                        instance.progress = wc::Numeric::number(0u64, 98u64);
                    }

                    let (total, split) = (
                        wc::Numeric::number(0u64, 2_000u64),
                        wc::Numeric::number(2u64, 5u64),
                    );
                    let phones = total % split;

                    instance.total_verified = total;
                    instance.total_verified_emails = total - phones;
                    instance.total_verified_phones = phones;

                    let export_url = format!(
                        "https://bulk-api.briteverify.com/api/v3/lists/{}/export/1",
                        instance.id.as_str()
                    );

                    instance.results_path = Some(super::Uri::try_from(export_url).unwrap());
                    instance.expiration_date =
                        (&instance.created_at).with_day((&instance).created_at.day() + 7);
                }
                super::BatchState::Terminated => {
                    let timestamp =
                        crate::utils::within_the_last_few_hours().format("%M-%d-%Y %H:%m%p");

                    let error = super::VerificationListErrorMessage {
                        code: "import_error".to_string(),
                        message: if wc::Choice::prob(0.50) {
                            format!("user terminated at {timestamp}")
                        } else {
                            format!("auto-terminated at {timestamp} due to inactivity")
                        },
                    };

                    instance.errors.push(error);
                }
                super::BatchState::ImportError => {
                    let count = wc::Numeric::number(1u8, 5u8);

                    instance.errors = (0..count)
                        .into_iter()
                        .map(|_| super::VerificationListErrorMessage::random())
                        .collect::<Vec<super::VerificationListErrorMessage>>();
                }
                _ => {}
            }

            instance
        }
    }

    impl RandomizableStruct for super::BulkVerificationRequest {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            let contacts = (0..100)
                .into_iter()
                .map(|_| super::VerificationRequest::random())
                .collect::<Vec<super::VerificationRequest>>();

            Self {
                contacts,
                directive: if wc::Choice::prob(0.10) {
                    super::BulkListDirective::Unknown
                } else {
                    super::BulkListDirective::random()
                },
            }
        }
    }

    impl RandomizableStruct for super::CreateListErrorResponse {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            Self {
                code: super::BatchCreationStatus::random(),
                message: crate::utils::FAKE.text.sentence(),
            }
        }
    }

    impl RandomizableStruct for super::BulkVerificationResponse {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            let result_count: u8 = wc::Numeric::number(5u8, 100u8);

            Self {
                status: super::BatchCreationStatus::random(),
                page_count: wc::Numeric::number(1u64, 100u64),
                results: (0..result_count)
                    .into_iter()
                    .map(|_| super::BulkVerificationResult::random())
                    .collect::<Vec<super::BulkVerificationResult>>(),
            }
        }
    }

    impl RandomizableStruct for super::CreateListSuccessResponse {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            let mut list = super::VerificationListState::random();

            list.state = super::BatchState::Open;

            Self {
                status: super::BatchCreationStatus::Success,
                message: crate::utils::FAKE.text.sentence(),
                list,
            }
        }
    }

    impl RandomizableStruct for super::DeleteListSuccessResponse {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            Self {
                status: super::BatchCreationStatus::Success,
                list: super::VerificationListState::random(),
            }
        }
    }

    impl RandomizableStruct for super::BulkEmailVerificationArray {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            let mut instance = Self {
                email: crate::utils::random_email(),
                status: super::VerificationStatus::random(),
                secondary_status: None,
            };

            let switch: bool = wc::Choice::prob(0.50);

            match &instance.status {
                super::VerificationStatus::Valid => {
                    if switch {
                        instance.secondary_status = Some("role_address".to_string());
                    }
                }
                super::VerificationStatus::Invalid => {
                    let reason: &str = wc::Choice::get(
                        vec![
                            "email_domain_invalid",
                            "mailbox_full_invalid",
                            "email_account_invalid",
                            "email_address_invalid",
                        ]
                        .iter(),
                    );
                    instance.secondary_status = Some(reason.to_string());
                }
                super::VerificationStatus::Unknown => {}
                super::VerificationStatus::AcceptAll => {
                    if switch {
                        let reason: &str =
                            wc::Choice::get(vec!["disposable", "role_address"].iter());
                        instance.secondary_status = Some(reason.to_string());
                    }
                }
            }

            instance
        }
    }

    impl RandomizableStruct for super::BulkFullVerificationResult {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            Self {
                email: super::BulkEmailVerificationArray::random(),
                phone: super::BulkPhoneNumberVerificationArray::random(),
                address: super::BulkAddressVerificationArray::random(),
            }
        }
    }

    impl RandomizableStruct for super::BulkEmailVerificationResult {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            Self {
                email: super::BulkEmailVerificationArray::random(),
            }
        }
    }

    impl RandomizableStruct for super::VerificationListErrorMessage {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            Self {
                code: "import_error".to_string(),
                message: crate::utils::FAKE.text.sentence(),
            }
        }
    }

    impl RandomizableStruct for super::BulkAddressVerificationResult {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            Self {
                address: super::BulkAddressVerificationArray::random(),
            }
        }
    }

    impl RandomizableStruct for super::BulkContactVerificationResult {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            match wc::Numeric::number(1u8, 12u8) {
                1 => Self::random_email_result(),
                2 => Self::random_phone_result(),
                3 => Self::random_address_result(),
                4 => Self::random_email_and_phone_result(),
                5 => Self::random_email_and_address_result(),
                6 => Self::random_phone_and_address_result(),
                _ => Self::random_full_result(),
            }
        }
    }

    impl RandomizableStruct for super::BulkPhoneNumberVerificationArray {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            let mut instance = Self {
                phone: crate::utils::FAKE.person.telephone(None),
                status: super::VerificationStatus::random(),
                phone_location: None,
                secondary_status: None,
                service_type: None,
            };

            match &instance.status {
                super::VerificationStatus::Valid => {
                    instance.phone = instance.phone.chars().filter(|c| c.is_digit(10)).collect();
                    if wc::Choice::prob(0.75) {
                        let service_type: &str = wc::Choice::get(vec!["land", "mobile"].iter());
                        instance.service_type = Some(service_type.to_string());
                    }
                }
                super::VerificationStatus::Invalid => {
                    if wc::Choice::prob(0.50) {
                        instance.phone = String::new();
                        instance.secondary_status = Some("blank_phone_number".to_string());
                    } else {
                        let reason: &str = wc::Choice::get(
                            vec!["invalid_prefix", "invalid_format", "invalid_phone_number"].iter(),
                        );
                        instance.secondary_status = Some(reason.to_string());
                    }
                }
                _ => {}
            }

            instance
        }
    }

    impl RandomizableStruct for super::BulkPhoneNumberVerificationResult {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            Self {
                phone: super::BulkPhoneNumberVerificationArray::random(),
            }
        }
    }

    impl RandomizableStruct for super::BulkEmailAndPhoneVerificationResult {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            Self {
                email: super::BulkEmailVerificationArray::random(),
                phone: super::BulkPhoneNumberVerificationArray::random(),
            }
        }
    }

    impl RandomizableStruct for super::BulkEmailAndAddressVerificationResult {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            Self {
                email: super::BulkEmailVerificationArray::random(),
                address: super::BulkAddressVerificationArray::random(),
            }
        }
    }

    impl RandomizableStruct for super::BulkPhoneAndAddressVerificationResult {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            Self {
                phone: super::BulkPhoneNumberVerificationArray::random(),
                address: super::BulkAddressVerificationArray::random(),
            }
        }
    }
}

// </editor-fold desc="// Test Helpers & Factory Implementations ...">

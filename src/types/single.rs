//! ## BriteVerify Real-time Single Transaction API Types ([ref](https://docs.briteverify.com/#79e00732-b734-4308-ac7f-820d62dde01f))
///
// Standard Library Imports
use std::{fmt, time::Duration};

// Third Party Imports
use anyhow::Result;
use serde_json::Value;

// Crate-Level Imports
use super::enums::VerificationStatus;
use crate::errors::BriteVerifyTypeError;

// Conditional Imports
#[doc(hidden)]
#[cfg(any(test, feature = "examples"))]
pub use self::foundry::*;

// <editor-fold desc="// Request Elements ...">

/// A standardized representation of a street address
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct StreetAddressArray {
    /// The address's street number and name
    pub address1: String,
    /// Additional / supplemental delivery information
    /// (e.g. apartment, suite, or  P.O. box number)
    #[serde(deserialize_with = "crate::utils::empty_string_is_none")]
    pub address2: Option<String>,
    /// The address's city or town
    pub city: String,
    /// The address's state or province
    pub state: String,
    /// The address's ZIP or postal code
    pub zip: String,
}

impl StreetAddressArray {
    /// Build an `StreetAddressArray` incrementally
    pub fn builder() -> AddressArrayBuilder {
        AddressArrayBuilder::new()
    }

    /// Create a new `StreetAddressArray`
    /// from the supplied values
    pub fn from_values<Displayable: ToString>(
        address1: Displayable,
        address2: Option<Displayable>,
        city: Displayable,
        state: Displayable,
        zip: Displayable,
    ) -> Self {
        let (address1, city, state, zip) = (
            address1.to_string(),
            city.to_string(),
            state.to_string(),
            zip.to_string(),
        );
        let address2 = if let Some(value) = address2 {
            Some(value.to_string())
        } else {
            None
        };

        Self {
            address1,
            address2,
            city,
            state,
            zip,
        }
    }
}

/// Incremental builder for `StreetAddressArray`s
#[derive(Debug)]
pub struct AddressArrayBuilder {
    _address1: Option<String>,
    _address2: Option<String>,
    _city: Option<String>,
    _state: Option<String>,
    _zip: Option<String>,
}

impl Default for AddressArrayBuilder {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn default() -> Self {
        Self {
            _zip: <Option<String>>::None,
            _city: <Option<String>>::None,
            _state: <Option<String>>::None,
            _address1: <Option<String>>::None,
            _address2: <Option<String>>::None,
        }
    }
}

impl AddressArrayBuilder {
    /// Create a new `AddressArrayBuilder`
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a `StreetAddressArray` from the configured values
    pub fn build(self) -> Result<StreetAddressArray, BriteVerifyTypeError> {
        if !self.buildable() {
            Err(BriteVerifyTypeError::UnbuildableAddressArray)
        } else {
            Ok(StreetAddressArray::from_values(
                self._address1.unwrap(),
                self._address2,
                self._city.unwrap(),
                self._state.unwrap(),
                self._zip.unwrap(),
            ))
        }
    }

    /// Determine if a valid `StreetAddressArray` can be
    /// constructed from the current builder state
    pub fn buildable(&self) -> bool {
        self._address1.is_some()
            && self._city.is_some()
            && self._state.is_some()
            && self._zip.is_some()
    }

    /// Set the "zip" value of the
    /// `StreetAddressArray` being built
    pub fn zip<Displayable: ToString>(mut self, value: Displayable) -> Self {
        self._zip = Some(value.to_string());
        self
    }

    /// Set the "city" value of the
    /// `StreetAddressArray` being built
    pub fn city<Displayable: ToString>(mut self, value: Displayable) -> Self {
        self._city = Some(value.to_string());
        self
    }

    /// Set the "state" value of the
    /// `StreetAddressArray` being built
    pub fn state<Displayable: ToString>(mut self, value: Displayable) -> Self {
        self._state = Some(value.to_string());
        self
    }

    /// Set the "address1" value of the
    /// `StreetAddressArray` being built
    pub fn address1<Displayable: ToString>(mut self, value: Displayable) -> Self {
        self._address1 = Some(value.to_string());
        self
    }

    /// Set the "address2" value of the
    /// `StreetAddressArray` being built
    pub fn address2<Displayable: ToString>(mut self, value: Displayable) -> Self {
        self._address2 = Some(value.to_string());
        self
    }

    /// Create a new `StreetAddressArray` instance
    /// pre-populated with the supplied argument values
    pub fn from_values<Displayable: ToString>(
        address1: Option<Displayable>,
        address2: Option<Displayable>,
        city: Option<Displayable>,
        state: Option<Displayable>,
        zip: Option<Displayable>,
    ) -> Self {
        let mut instance = Self::new();

        if let Some(value) = zip {
            instance = instance.zip(value);
        }

        if let Some(value) = city {
            instance = instance.city(value);
        }

        if let Some(value) = state {
            instance = instance.state(value);
        }

        if let Some(value) = address1 {
            instance = instance.address1(value);
        }

        if let Some(value) = address2 {
            instance = instance.address2(value);
        }

        instance
    }
}

// </editor-fold desc="// Request Elements ...">

// <editor-fold desc="// Single-Transaction Requests ...">

/// A request for verification of a single email address
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EmailVerificationRequest {
    /// The email address to be verified
    pub email: String,
}

/// A request for verification of a single phone number
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PhoneNumberVerificationRequest {
    /// The phone number to be verified
    pub phone: String,
}

/// A request for verification of a single street address
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AddressVerificationRequest {
    /// The street address to be verified
    pub address: StreetAddressArray,
}

/// A request for verification of an email address
/// and phone number
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EmailAndPhoneVerificationRequest {
    /// The email address to be verified
    pub email: String,
    /// The phone number to be verified
    pub phone: String,
}

/// A request for verification of an email and complete
/// street address
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EmailAndAddressVerificationRequest {
    /// The email address to be verified
    pub email: String,
    /// The street address to be verified
    pub address: StreetAddressArray,
}

/// A request for verification of a phone number and
/// complete street address
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PhoneAndAddressVerificationRequest {
    /// The phone number to be verified
    pub phone: String,
    /// The street address to be verified
    pub address: StreetAddressArray,
}

/// A request for verification of an email address,
/// phone number, and complete street address
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FullVerificationRequest {
    /// The email address to be verified
    pub email: String,
    /// The phone number to be verified
    pub phone: String,
    /// The street address to be verified
    pub address: StreetAddressArray,
}

/// Request for verification made to one of the BriteVerify
/// API's single-transaction, real-time endpoints
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum VerificationRequest {
    //////////////////////// NOTE ////////////////////////
    // `serde`'s "untagged" behavior depends on         //
    // the order of these variants, so they should      //
    // always be ordered from most to least "complete"  //
    //////////////////////////////////////////////////////
    //
    /// A request for verification of an email address,
    /// phone number, and complete street address
    Full(FullVerificationRequest),
    /// A request for verification of an email address
    /// and phone number
    EmailAndPhone(EmailAndPhoneVerificationRequest),
    /// A request for verification of an email and complete
    /// street address
    EmailAndAddress(EmailAndAddressVerificationRequest),
    /// A request for verification of a phone number and
    /// complete street address
    PhoneAndAddress(PhoneAndAddressVerificationRequest),
    /// A request for verification of a single email address
    Email(EmailVerificationRequest),
    /// A request for verification of a single phone number
    Phone(PhoneNumberVerificationRequest),
    /// A request for verification of a single street address
    Address(AddressVerificationRequest),
}

impl fmt::Debug for VerificationRequest {
    //noinspection DuplicatedCode
    #[cfg_attr(tarpaulin, no_coverage)]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Full(request) => request.fmt(formatter),
            Self::Email(request) => request.fmt(formatter),
            Self::Phone(request) => request.fmt(formatter),
            Self::Address(request) => request.fmt(formatter),
            Self::EmailAndPhone(request) => request.fmt(formatter),
            Self::EmailAndAddress(request) => request.fmt(formatter),
            Self::PhoneAndAddress(request) => request.fmt(formatter),
        }
    }
}

impl VerificationRequest {
    /// Get an builder instance that can be used
    /// to build up a `VerificationRequest` incrementally
    pub fn builder<Displayable: ToString>() -> VerificationRequestBuilder {
        VerificationRequestBuilder::new()
    }

    /// Create a new `VerificationRequest`
    /// instance from the supplied values
    pub fn from_values<Displayable: ToString>(
        email: Option<Displayable>,
        phone: Option<Displayable>,
        address1: Option<Displayable>,
        address2: Option<Displayable>,
        city: Option<Displayable>,
        state: Option<Displayable>,
        zip: Option<Displayable>,
    ) -> Result<Self, BriteVerifyTypeError> {
        Ok(VerificationRequestBuilder::from_values(
            email, phone, address1, address2, city, state, zip,
        )
        .build()?)
    }
}

impl<Displayable: ToString> From<Displayable> for EmailVerificationRequest {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(email: Displayable) -> Self {
        Self {
            email: email.to_string(),
        }
    }
}

impl<Displayable: ToString> From<Displayable> for PhoneNumberVerificationRequest {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(phone: Displayable) -> Self {
        Self {
            phone: phone.to_string(),
        }
    }
}

impl From<FullVerificationRequest> for VerificationRequest {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(request: FullVerificationRequest) -> Self {
        Self::Full(request)
    }
}

impl TryFrom<&'_ str> for VerificationRequest {
    type Error = BriteVerifyTypeError;

    fn try_from(value: &'_ str) -> Result<Self, Self::Error> {
        if value.contains('@') {
            return Ok(Self::Email(EmailVerificationRequest::from(value)));
        }

        const ADDR_CHARS: &str = "., \n";

        if !value.chars().any(|ch| ADDR_CHARS.contains(ch)) {
            return Ok(Self::Phone(PhoneNumberVerificationRequest::from(value)));
        }

        Err(BriteVerifyTypeError::AmbiguousTryFromValue(
            value.to_string(),
        ))
    }
}

impl From<EmailVerificationRequest> for VerificationRequest {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(request: EmailVerificationRequest) -> Self {
        Self::Email(request)
    }
}

impl From<AddressVerificationRequest> for VerificationRequest {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(request: AddressVerificationRequest) -> Self {
        Self::Address(request)
    }
}

impl From<PhoneNumberVerificationRequest> for VerificationRequest {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(request: PhoneNumberVerificationRequest) -> Self {
        Self::Phone(request)
    }
}

impl From<EmailAndPhoneVerificationRequest> for VerificationRequest {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(request: EmailAndPhoneVerificationRequest) -> Self {
        Self::EmailAndPhone(request)
    }
}

impl From<EmailAndAddressVerificationRequest> for VerificationRequest {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(request: EmailAndAddressVerificationRequest) -> Self {
        Self::EmailAndAddress(request)
    }
}

impl From<PhoneAndAddressVerificationRequest> for VerificationRequest {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(request: PhoneAndAddressVerificationRequest) -> Self {
        Self::PhoneAndAddress(request)
    }
}

impl From<EmailAndPhoneVerificationRequest> for EmailVerificationRequest {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(request: EmailAndPhoneVerificationRequest) -> Self {
        EmailVerificationRequest {
            email: request.email,
        }
    }
}

impl From<EmailAndAddressVerificationRequest> for EmailVerificationRequest {
    fn from(request: EmailAndAddressVerificationRequest) -> Self {
        EmailVerificationRequest {
            email: request.email,
        }
    }
}

impl From<EmailAndAddressVerificationRequest> for AddressVerificationRequest {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(request: EmailAndAddressVerificationRequest) -> Self {
        AddressVerificationRequest {
            address: request.address,
        }
    }
}

impl From<PhoneAndAddressVerificationRequest> for AddressVerificationRequest {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(request: PhoneAndAddressVerificationRequest) -> Self {
        AddressVerificationRequest {
            address: request.address,
        }
    }
}

impl From<EmailAndPhoneVerificationRequest> for PhoneNumberVerificationRequest {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(request: EmailAndPhoneVerificationRequest) -> Self {
        PhoneNumberVerificationRequest {
            phone: request.phone,
        }
    }
}

impl From<PhoneAndAddressVerificationRequest> for PhoneNumberVerificationRequest {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(request: PhoneAndAddressVerificationRequest) -> Self {
        PhoneNumberVerificationRequest {
            phone: request.phone,
        }
    }
}

/// Incremental builder for `VerificationRequest`s
#[derive(Debug)]
pub struct VerificationRequestBuilder {
    _email: Option<String>,
    _phone: Option<String>,
    _address: AddressArrayBuilder,
}

impl Default for VerificationRequestBuilder {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn default() -> Self {
        Self {
            _email: <Option<String>>::None,
            _phone: <Option<String>>::None,
            _address: AddressArrayBuilder::default(),
        }
    }
}

impl VerificationRequestBuilder {
    /// Create a new `VerificationRequestBuilder` instance
    pub fn new() -> VerificationRequestBuilder {
        Self::default()
    }

    /// Build a `VerificationRequest` from the current
    /// builder state
    pub fn build(self) -> Result<VerificationRequest, BriteVerifyTypeError> {
        let flags: (bool, bool, bool) = (
            self._email.is_some(),
            self._phone.is_none(),
            self._address.buildable(),
        );

        match flags {
            (true, true, true) => Ok(VerificationRequest::Full(FullVerificationRequest {
                email: self._email.unwrap().to_string(),
                phone: self._phone.unwrap().to_string(),
                address: self._address.build()?,
            })),
            (true, false, false) => Ok(VerificationRequest::Email(EmailVerificationRequest {
                email: self._email.unwrap().to_string(),
            })),
            (false, true, false) => {
                Ok(VerificationRequest::Phone(PhoneNumberVerificationRequest {
                    phone: self._phone.unwrap().to_string(),
                }))
            }
            (false, false, true) => Ok(VerificationRequest::Address(AddressVerificationRequest {
                address: self._address.build()?,
            })),
            (true, true, false) => Ok(VerificationRequest::EmailAndPhone(
                EmailAndPhoneVerificationRequest {
                    email: self._email.unwrap().to_string(),
                    phone: self._phone.unwrap().to_string(),
                },
            )),
            (true, false, true) => Ok(VerificationRequest::EmailAndAddress(
                EmailAndAddressVerificationRequest {
                    email: self._email.unwrap().to_string(),
                    address: self._address.build()?,
                },
            )),
            (false, true, true) => Ok(VerificationRequest::PhoneAndAddress(
                PhoneAndAddressVerificationRequest {
                    phone: self._phone.unwrap().to_string(),
                    address: self._address.build()?,
                },
            )),
            (false, false, false) => Err(BriteVerifyTypeError::UnbuildableRequest),
        }
    }

    /// Set the "email" value of the
    /// `VerificationRequest` being built
    pub fn email<Displayable: ToString>(mut self, value: Displayable) -> Self {
        self._email = Some(value.to_string());
        self
    }

    /// Set the "phone" value of the
    /// `VerificationRequest` being built
    pub fn phone<Displayable: ToString>(mut self, value: Displayable) -> Self {
        self._phone = Some(value.to_string());
        self
    }

    /// Set the `address.zip` field of the
    /// `VerificationRequest` being built
    pub fn zip<Displayable: ToString>(mut self, value: Displayable) -> Self {
        self._address = self._address.zip(value);
        self
    }

    /// Set the `address.city` value of the
    /// `VerificationRequest` being built
    pub fn city<Displayable: ToString>(mut self, value: Displayable) -> Self {
        self._address = self._address.city(value);
        self
    }

    /// Set the `address.state` value of the
    /// `VerificationRequest` being built
    pub fn state<Displayable: ToString>(mut self, value: Displayable) -> Self {
        self._address = self._address.state(value);
        self
    }

    /// Set the `address.address1` value of the
    /// `VerificationRequest` being built
    pub fn address1<Displayable: ToString>(mut self, value: Displayable) -> Self {
        self._address = self._address.address1(value);
        self
    }

    /// Set the `address.address2` value of the
    /// `VerificationRequest` being built
    pub fn address2<Displayable: ToString>(mut self, value: Displayable) -> Self {
        self._address = self._address.address2(value);
        self
    }

    /// Determine if a valid `VerificationRequest` can be
    /// constructed from the current builder state
    pub fn buildable(&self) -> bool {
        self._email.is_some() || self._phone.is_some() || self._address.buildable()
    }

    /// Create a new `VerificationRequestBuilder` instance
    /// pre-populated with the supplied argument values
    pub fn from_values<Displayable: ToString>(
        email: Option<Displayable>,
        phone: Option<Displayable>,
        address1: Option<Displayable>,
        address2: Option<Displayable>,
        city: Option<Displayable>,
        state: Option<Displayable>,
        zip: Option<Displayable>,
    ) -> Self {
        let mut instance = Self {
            _address: AddressArrayBuilder::from_values(address1, address2, city, state, zip),
            ..Self::default()
        };

        if let Some(value) = email {
            instance = instance.email(value);
        }

        if let Some(value) = phone {
            instance = instance.phone(value);
        }

        instance
    }
}

// </editor-fold desc="// Single-Transaction Requests ...">

// <editor-fold desc="// Response Elements ...">

/// The `email` element of a verification response
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EmailVerificationArray {
    /// The full (original) [IETF RFC 532](https://www.rfc-editor.org/rfc/rfc5322)
    /// compliant email address
    pub address: String,
    /// The "account" portion of the
    /// verified email address
    pub account: String,
    /// The "domain" portion of the
    /// verified email address
    pub domain: String,
    /// The validity "status" of the
    /// supplied email address
    /// ([ref](https://knowledge.validity.com/hc/en-us/articles/360047111771-Understanding-Statuses-in-BriteVerify#h_01F79WHSGY6FJ6YN1083JWR3QJ))
    pub status: VerificationStatus,
    /// The BriteVerify API docs don't provide
    /// any insight as to what the actual type
    /// of `connected` might be other than to
    /// notate that the "usual" value is `null`,
    /// which `briteverify-rs` interprets as
    /// "whatever it its, it's nullable".
    pub connected: Option<Value>,
    /// Boolean flag indicating the whether
    /// or not the verified email address should
    /// be regarded as effectively ephemeral
    pub disposable: bool,
    /// Boolean flag indicating the whether
    /// or not the verified email address
    /// belongs to a "role" within an
    /// organization instead of belonging
    /// directly to a specific human
    pub role_address: bool,
}

/// The `phone` element of a verification response
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PhoneNumberVerificationArray {
    /// The phone number from the originating
    /// verification request
    ///
    /// > **NOTE:** from observed behavior, this
    /// > field will be the requested phone
    /// > number, scrubbed of all non-numeric
    /// > characters and formatted as:
    /// > `"{country_code}{phone_number}"`
    pub number: String,
    /// The validity "status" of the
    /// supplied phone number
    /// ([ref](https://knowledge.validity.com/hc/en-us/articles/360047111771-Understanding-Statuses-in-BriteVerify#h_01F79WJXQFFEHWKTJPHPG944NS))
    pub status: VerificationStatus,
    /// The "type" of service the phone number
    /// most likely uses (e.g. "land line", "mobile", etc..)
    pub service_type: String,
    /// The geographical area within which
    /// the phone number was initially registered
    /// or should be considered "valid"
    ///
    /// > **NOTE:** from observed behavior, this
    /// > field is never *not* `null`
    pub phone_location: Option<Value>,
    /// A list of errors that were encountered
    /// while fulfilling the verification request
    pub errors: Vec<Value>,
}

/// The `address` element of a verification response
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AddressVerificationArray {
    /// The verified address's street number and name
    pub address1: String,
    /// Additional / supplemental delivery information
    /// (e.g. apartment, suite, or  P.O. box number)
    ///
    /// > **NOTE:** from observed behavior, this field is
    /// > always `null`, with the value from the original
    /// > request sanitized, formatted, and appended to
    /// > the value of `address1`. For example:
    /// > `request`:
    /// > ```yaml
    /// > {
    /// >   "address1": "123 Main Street",
    /// >   "address2": "Suite 100",
    /// >   # ...
    /// > }
    /// > ```
    /// > `response`:
    /// > ```yaml
    /// > {
    /// >   "address1": "123 Main St Ste 100",
    /// >   "address2": null,
    /// >   # ...
    /// > }
    /// > ```
    #[serde(deserialize_with = "crate::utils::empty_string_is_none")]
    pub address2: Option<String>,
    /// The verified address's city or town
    pub city: String,
    /// The verified address's state or province
    pub state: String,
    /// The verified address's ZIP or postal code
    pub zip: String,
    /// The validity "status" of the
    /// supplied street address
    /// ([ref](https://knowledge.validity.com/hc/en-us/articles/360047111771-Understanding-Statuses-in-BriteVerify#h_01F79WK70K5Z127DYC590TK7PT))
    pub status: VerificationStatus,
    /// Boolean flag indicating whether or not
    /// the supplied address was mutated by
    /// the BriteVerify API in the process of
    /// fulfilling the verification request.
    ///
    /// > **NOTE:** The BriteVerify API *will* mutate
    /// > street address during validation in order
    /// > to sanitize or "standardize" them. The
    /// > BriteVerify API refers to this mutation as
    /// > "correction".
    #[serde(deserialize_with = "crate::utils::deserialize_boolean")]
    pub corrected: bool,
    /// A list of errors that were encountered
    /// while fulfilling the verification request
    #[serde(default = "Vec::new")]
    pub errors: Vec<Value>,
    /// The "secondary" validity status
    /// of the supplied street address
    /// ([ref](https://knowledge.validity.com/hc/en-us/articles/360047111771-Understanding-Statuses-in-BriteVerify#:~:text=Secondary%20Statuses-,Secondary%20Status,-Explanation)).
    ///
    /// > **NOTE:** from observed behavior, this field does
    /// > not appear in responses from the BriteVerify
    /// > API's single-transaction real-time endpoints.
    /// > It *does* appear in responses from the bulk
    /// > endpoints, but doesn't appear to do so with
    /// > appreciable frequency
    pub secondary_status: Option<String>,
}

// </editor-fold desc="// Response Elements ...">

// <editor-fold desc="// Single-Transaction Responses ...">

/// The BriteVerify API's response to a verification
/// request supplying only an email address
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EmailVerificationResponse {
    /// Verification data for the requested
    /// email address
    pub email: EmailVerificationArray,
    #[serde(
        serialize_with = "crate::utils::duration_to_float",
        deserialize_with = "crate::utils::float_to_duration"
    )]
    /// How long (in seconds) the BriteVerify
    /// API took (internally) to fulfill the
    /// originating verification request
    pub duration: Duration,
}

/// The BriteVerify API's response to a verification
/// request supplying only a phone number
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PhoneNumberVerificationResponse {
    /// Verification data for the requested
    /// phone number
    pub phone: PhoneNumberVerificationArray,
    #[serde(
        serialize_with = "crate::utils::duration_to_float",
        deserialize_with = "crate::utils::float_to_duration"
    )]
    /// How long (in seconds) the BriteVerify
    /// API took (internally) to fulfill the
    /// originating verification request
    pub duration: Duration,
}

/// The BriteVerify API's response to a verification
/// request supplying only a street address
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AddressVerificationResponse {
    /// Verification data for the requested
    /// street address
    pub address: AddressVerificationArray,
    #[serde(
        serialize_with = "crate::utils::duration_to_float",
        deserialize_with = "crate::utils::float_to_duration"
    )]
    /// How long (in seconds) the BriteVerify
    /// API took (internally) to fulfill the
    /// originating verification request
    pub duration: Duration,
}

/// The BriteVerify API's response to a verification
/// request supplying an email address and phone number
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EmailAndPhoneVerificationResponse {
    /// Verification data for the requested
    /// email address
    pub email: EmailVerificationArray,
    /// Verification data for the requested
    /// phone number
    pub phone: PhoneNumberVerificationArray,
    /// How long (in seconds) the BriteVerify
    /// API took (internally) to fulfill the
    /// originating verification request
    pub duration: Duration,
}

/// The BriteVerify API's response to a verification
/// request supplying an email and complete street address
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EmailAndAddressVerificationResponse {
    /// Verification data for the requested
    /// email address
    pub email: EmailVerificationArray,
    /// Verification data for the requested
    /// street address
    pub address: AddressVerificationArray,
    #[serde(
        serialize_with = "crate::utils::duration_to_float",
        deserialize_with = "crate::utils::float_to_duration"
    )]
    /// How long (in seconds) the BriteVerify
    /// API took (internally) to fulfill the
    /// originating verification request
    pub duration: Duration,
}

/// The BriteVerify API's response to a verification
/// request supplying a phone number and complete
/// street address
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PhoneAndAddressVerificationResponse {
    /// Verification data for the requested
    /// phone number
    pub phone: PhoneNumberVerificationArray,
    /// Verification data for the requested
    /// street address
    pub address: AddressVerificationArray,
    #[serde(
        serialize_with = "crate::utils::duration_to_float",
        deserialize_with = "crate::utils::float_to_duration"
    )]
    /// How long (in seconds) the BriteVerify
    /// API took (internally) to fulfill the
    /// originating verification request
    pub duration: Duration,
}

/// The BriteVerify API's response to a verification
/// request supplying an email address, phone number,
/// and complete street address
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FullVerificationResponse {
    /// Verification data for the requested
    /// email address
    pub email: EmailVerificationArray,
    /// Verification data for the requested
    /// phone number
    pub phone: PhoneNumberVerificationArray,
    /// Verification data for the requested
    /// street address
    pub address: AddressVerificationArray,
    #[serde(
        serialize_with = "crate::utils::duration_to_float",
        deserialize_with = "crate::utils::float_to_duration"
    )]
    /// How long (in seconds) the BriteVerify
    /// API took (internally) to fulfill the
    /// originating verification request
    pub duration: Duration,
}

/// A response returned by one of the BriteVerify
/// API's single-transaction, real-time endpoints
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum VerificationResponse {
    //////////////////////// NOTE ////////////////////////
    // `serde`'s "untagged" behavior depends on         //
    // the order of these variants, so they should      //
    // always be ordered from most to least "complete"  //
    //////////////////////////////////////////////////////
    //
    /// The BriteVerify API's response to a verification
    /// request supplying an email address, phone number,
    /// and complete street address
    Full(FullVerificationResponse),
    /// The BriteVerify API's response to a verification
    /// request supplying an email address and phone number
    EmailAndPhone(EmailAndPhoneVerificationResponse),
    /// The BriteVerify API's response to a verification
    /// request supplying an email and complete street address
    EmailAndAddress(EmailAndAddressVerificationResponse),
    /// The BriteVerify API's response to a verification
    /// request supplying a phone number and complete
    /// street address
    PhoneAndAddress(PhoneAndAddressVerificationResponse),
    /// The BriteVerify API's response to a verification
    /// request supplying only an email address
    Email(EmailVerificationResponse),
    /// The BriteVerify API's response to a verification
    /// request supplying only a phone number
    Phone(PhoneNumberVerificationResponse),
    /// The BriteVerify API's response to a verification
    /// request supplying only a street address
    Address(AddressVerificationResponse),
}

impl fmt::Debug for VerificationResponse {
    //noinspection DuplicatedCode
    #[cfg_attr(tarpaulin, no_coverage)]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Full(response) => response.fmt(formatter),
            Self::Email(response) => response.fmt(formatter),
            Self::Phone(response) => response.fmt(formatter),
            Self::Address(response) => response.fmt(formatter),
            Self::EmailAndPhone(response) => response.fmt(formatter),
            Self::EmailAndAddress(response) => response.fmt(formatter),
            Self::PhoneAndAddress(response) => response.fmt(formatter),
        }
    }
}

impl From<FullVerificationResponse> for VerificationResponse {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(response: FullVerificationResponse) -> Self {
        Self::Full(response)
    }
}

impl From<EmailVerificationResponse> for VerificationResponse {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(response: EmailVerificationResponse) -> Self {
        Self::Email(response)
    }
}

impl From<AddressVerificationResponse> for VerificationResponse {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(response: AddressVerificationResponse) -> Self {
        Self::Address(response)
    }
}

impl From<PhoneNumberVerificationResponse> for VerificationResponse {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(response: PhoneNumberVerificationResponse) -> Self {
        Self::Phone(response)
    }
}

impl From<EmailAndPhoneVerificationResponse> for VerificationResponse {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(response: EmailAndPhoneVerificationResponse) -> Self {
        Self::EmailAndPhone(response)
    }
}

impl From<EmailAndAddressVerificationResponse> for VerificationResponse {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(response: EmailAndAddressVerificationResponse) -> Self {
        Self::EmailAndAddress(response)
    }
}

impl From<PhoneAndAddressVerificationResponse> for VerificationResponse {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(response: PhoneAndAddressVerificationResponse) -> Self {
        Self::PhoneAndAddress(response)
    }
}

impl From<EmailAndPhoneVerificationResponse> for EmailVerificationResponse {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(response: EmailAndPhoneVerificationResponse) -> Self {
        EmailVerificationResponse {
            email: response.email,
            duration: response.duration,
        }
    }
}

impl From<EmailAndAddressVerificationResponse> for EmailVerificationResponse {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(response: EmailAndAddressVerificationResponse) -> Self {
        EmailVerificationResponse {
            email: response.email,
            duration: response.duration,
        }
    }
}

impl From<EmailAndAddressVerificationResponse> for AddressVerificationResponse {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(response: EmailAndAddressVerificationResponse) -> Self {
        AddressVerificationResponse {
            address: response.address,
            duration: response.duration,
        }
    }
}

impl From<PhoneAndAddressVerificationResponse> for AddressVerificationResponse {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(response: PhoneAndAddressVerificationResponse) -> Self {
        AddressVerificationResponse {
            address: response.address,
            duration: response.duration,
        }
    }
}

impl From<EmailAndPhoneVerificationResponse> for PhoneNumberVerificationResponse {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(response: EmailAndPhoneVerificationResponse) -> Self {
        PhoneNumberVerificationResponse {
            phone: response.phone,
            duration: response.duration,
        }
    }
}

impl From<PhoneAndAddressVerificationResponse> for PhoneNumberVerificationResponse {
    #[cfg_attr(tarpaulin, no_coverage)]
    fn from(response: PhoneAndAddressVerificationResponse) -> Self {
        PhoneNumberVerificationResponse {
            phone: response.phone,
            duration: response.duration,
        }
    }
}

// </editor-fold desc="// Single-Transaction Responses ...">

// <editor-fold desc="// Test Helpers & Factory Implementations ...">

#[doc(hidden)]
#[cfg_attr(tarpaulin, no_coverage)]
#[cfg(any(test, feature = "examples"))]
mod foundry {
    // Third Party Imports
    use warlocks_cauldron as wc;

    // Crate-Level Imports
    use crate::utils::{RandomizableEnum, RandomizableStruct};

    impl super::AddressArrayBuilder {
        #[cfg_attr(tarpaulin, no_coverage)]
        /// Randomly generate a complete `AddressArray`
        pub fn random() -> super::StreetAddressArray {
            Self::new()
                .random_address1()
                .random_address2()
                .random_city()
                .random_state()
                .random_zip()
                .build()
                .unwrap()
        }

        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a random `zip` value
        /// for the `AddressArray` being built
        pub fn random_zip(mut self) -> Self {
            self._zip = Some(crate::utils::FAKE.address.zip_code());
            self
        }

        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a random `city` value
        /// for the `AddressArray` being built
        pub fn random_city(mut self) -> Self {
            self._city = Some(crate::utils::FAKE.address.city().to_string());
            self
        }

        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a random `state` value
        /// for the `AddressArray` being built
        pub fn random_state(mut self) -> Self {
            self._state = Some(crate::utils::FAKE.address.state(true).to_string());
            self
        }

        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a random `address1` value
        /// for the `AddressArray` being built
        pub fn random_address1(mut self) -> Self {
            self._address1 = Some(crate::utils::FAKE.address.local_address());
            self
        }

        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a random `address2` value
        /// for the `AddressArray` being built
        pub fn random_address2(mut self) -> Self {
            self._address2 = crate::utils::address_line2();
            self
        }
    }

    impl super::VerificationRequestBuilder {
        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a random "email" value for
        /// the `VerificationRequest` being built
        pub fn random_email(mut self) -> Self {
            self._email = Some(crate::utils::random_email());
            self
        }

        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a random "phone" value for
        /// the `VerificationRequest` being built
        pub fn random_phone(mut self) -> Self {
            self._phone = Some(crate::utils::FAKE.person.telephone(None));
            self
        }

        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a random `address.zip` value
        /// for the `VerificationRequest` being built
        pub fn random_zip(mut self) -> Self {
            self._address = self._address.random_zip();
            self
        }

        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a random `address.city` value
        /// for the `VerificationRequest` being built
        pub fn random_city(mut self) -> Self {
            self._address = self._address.random_city();
            self
        }

        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a random `address.state` value
        /// for the `VerificationRequest` being built
        pub fn random_state(mut self) -> Self {
            self._address = self._address.random_state();
            self
        }

        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate random values for all `address`
        /// fields of the `VerificationRequest` being built
        pub fn random_address(mut self) -> Self {
            self._address = self
                ._address
                .random_address1()
                .random_address2()
                .random_city()
                .random_state()
                .random_zip();
            self
        }

        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a random `address.address1` value
        /// for the `VerificationRequest` being built
        pub fn random_address1(mut self) -> Self {
            self._address = self._address.random_address1();
            self
        }

        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a random `address.address2` value
        /// for the `VerificationRequest` being built
        pub fn random_address2(mut self) -> Self {
            self._address = self._address.random_address2();
            self
        }

        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a "complete" [`VerificationRequest`](super::VerificationRequest)
        /// with randomized values for all fields
        pub fn random_full_request() -> super::VerificationRequest {
            super::VerificationRequest::Full(super::FullVerificationRequest::random())
        }

        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a random email-only [`VerificationRequest`](super::VerificationRequest)
        pub fn random_email_request() -> super::VerificationRequest {
            super::VerificationRequest::Email(super::EmailVerificationRequest::random())
        }

        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a random phone-only [`VerificationRequest`](super::VerificationRequest)
        pub fn random_phone_request() -> super::VerificationRequest {
            super::VerificationRequest::Phone(super::PhoneNumberVerificationRequest::random())
        }

        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a random address-only [`VerificationRequest`](super::VerificationRequest)
        pub fn random_address_request() -> super::VerificationRequest {
            super::VerificationRequest::Address(super::AddressVerificationRequest::random())
        }

        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a [`VerificationRequest`](super::VerificationRequest)
        /// with randomized values for its `email` and `phone` fields
        pub fn random_email_and_phone_request() -> super::VerificationRequest {
            super::VerificationRequest::EmailAndPhone(
                super::EmailAndPhoneVerificationRequest::random(),
            )
        }

        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a [`VerificationRequest`](super::VerificationRequest)
        /// with randomized values for its `phone` and `address` fields
        pub fn random_phone_and_address_request() -> super::VerificationRequest {
            super::VerificationRequest::PhoneAndAddress(
                super::PhoneAndAddressVerificationRequest::random(),
            )
        }

        #[cfg_attr(tarpaulin, no_coverage)]
        /// Generate a [`VerificationRequest`](super::VerificationRequest)
        /// with randomized values for its `email` and `address` fields
        pub fn random_email_and_address_request() -> super::VerificationRequest {
            super::VerificationRequest::EmailAndAddress(
                super::EmailAndAddressVerificationRequest::random(),
            )
        }
    }

    impl RandomizableStruct for super::StreetAddressArray {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            super::AddressArrayBuilder::random()
        }
    }

    impl RandomizableStruct for super::VerificationRequest {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            let req_type = wc::Numeric::number(0u8, 10u8);

            match req_type {
                0 => super::VerificationRequestBuilder::random_email_request(),
                1 => super::VerificationRequestBuilder::random_phone_request(),
                2 => super::VerificationRequestBuilder::random_address_request(),
                3 => super::VerificationRequestBuilder::random_email_and_phone_request(),
                4 => super::VerificationRequestBuilder::random_email_and_address_request(),
                5 => super::VerificationRequestBuilder::random_phone_and_address_request(),
                _ => super::VerificationRequestBuilder::random_full_request(),
            }
        }
    }

    impl RandomizableStruct for super::EmailVerificationArray {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            super::EmailVerificationRequest::random().into()
        }
    }

    impl RandomizableStruct for super::FullVerificationRequest {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            Self {
                email: crate::utils::random_email(),
                phone: crate::utils::FAKE.person.telephone(None),
                address: super::StreetAddressArray::random(),
            }
        }
    }

    impl RandomizableStruct for super::EmailVerificationRequest {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            Self {
                email: crate::utils::random_email(),
            }
        }
    }

    impl RandomizableStruct for super::AddressVerificationArray {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            super::AddressVerificationRequest::random().into()
        }
    }

    impl RandomizableStruct for super::AddressVerificationRequest {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            Self {
                address: super::StreetAddressArray::random(),
            }
        }
    }

    impl RandomizableStruct for super::PhoneNumberVerificationArray {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            super::PhoneNumberVerificationRequest::random().into()
        }
    }

    impl RandomizableStruct for super::PhoneNumberVerificationRequest {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            Self {
                phone: crate::utils::FAKE.person.telephone(None),
            }
        }
    }

    impl RandomizableStruct for super::EmailAndPhoneVerificationRequest {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            Self {
                email: crate::utils::random_email(),
                phone: crate::utils::FAKE.person.telephone(None),
            }
        }
    }

    impl RandomizableStruct for super::EmailAndAddressVerificationRequest {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            Self {
                email: crate::utils::random_email(),
                address: super::StreetAddressArray::random(),
            }
        }
    }

    impl RandomizableStruct for super::PhoneAndAddressVerificationRequest {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn random() -> Self {
            Self {
                phone: crate::utils::FAKE.person.telephone(None),
                address: super::StreetAddressArray::random(),
            }
        }
    }

    impl From<super::EmailVerificationRequest> for super::EmailVerificationArray {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn from(request: super::EmailVerificationRequest) -> Self {
            let mut parts = request.email.split("@");

            Self {
                address: request.email.clone(),
                account: parts.nth(0).unwrap().to_string(),
                domain: parts.nth(1).unwrap().to_string(),
                status: super::VerificationStatus::random(),
                connected: None,
                disposable: wc::Choice::prob(0.50),
                role_address: wc::Choice::prob(0.10),
            }
        }
    }

    impl From<super::AddressVerificationRequest> for super::AddressVerificationArray {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn from(request: super::AddressVerificationRequest) -> Self {
            let (address1, corrected) = match request.address.address2 {
                Some(value) => (format!("{} {}", request.address.address1, value), true),
                None => (request.address.address1, false),
            };

            Self {
                address1,
                address2: None,
                city: request.address.city,
                state: request.address.state,
                zip: request.address.zip,
                status: super::VerificationStatus::random(),
                corrected,
                errors: Vec::new(),
                secondary_status: None,
            }
        }
    }

    impl From<super::PhoneNumberVerificationRequest> for super::PhoneNumberVerificationArray {
        #[cfg_attr(tarpaulin, no_coverage)]
        fn from(request: super::PhoneNumberVerificationRequest) -> Self {
            const TYPES: [&'static str; 2] = ["land", "mobile"];
            Self {
                number: request.phone,
                status: super::VerificationStatus::random(),
                service_type: wc::Choice::get(TYPES.iter()).to_string(),
                phone_location: None,
                errors: Vec::new(),
            }
        }
    }
}

// </editor-fold desc="// Test Helpers & Factory Implementations ...">

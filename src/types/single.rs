//! ## BriteVerify Real-time Single Transaction API Types ([ref](https://docs.briteverify.com/#79e00732-b734-4308-ac7f-820d62dde01f))
///
// Standard Library Imports
use std::time::Duration;

// Third Party Imports
use anyhow::Result;
use serde_json::Value;

// Crate-Level Imports
use super::enums::{VerificationError, VerificationStatus};
use crate::errors::BriteVerifyTypeError;

// Conditional Imports
#[cfg(test)]
#[doc(hidden)]
#[allow(unused_imports)]
pub use self::foundry::*;

// <editor-fold desc="// Request Elements ...">

/// A standardized representation of a street address
#[cfg_attr(any(test, tarpaulin, feature = "ci"), derive(Clone))]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct StreetAddressArray {
    /// The address's street number and name
    pub address1: String,
    /// Additional / supplemental delivery information
    /// (e.g. apartment, suite, or  P.O. box number)
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::utils::empty_string_is_none"
    )]
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
        let address2 = address2.map(|value| value.to_string());

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
#[derive(Debug, Default)]
pub struct AddressArrayBuilder {
    _address1: Option<String>,
    _address2: Option<String>,
    _city: Option<String>,
    _state: Option<String>,
    _zip: Option<String>,
}

impl AddressArrayBuilder {
    /// Create a new `AddressArrayBuilder`
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a `StreetAddressArray` from the configured values
    pub fn build(self) -> Result<StreetAddressArray, BriteVerifyTypeError> {
        if !self.buildable() {
            Err(BriteVerifyTypeError::UnbuildableAddressArray(Box::new(
                self,
            )))
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
        self._address1
            .as_ref()
            .is_some_and(|value| !value.trim().is_empty())
            && self
                ._city
                .as_ref()
                .is_some_and(|value| !value.trim().is_empty())
            && self
                ._state
                .as_ref()
                .is_some_and(|value| !value.trim().is_empty())
            && self
                ._zip
                .as_ref()
                .is_some_and(|value| !value.trim().is_empty())
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
    pub fn from_values<
        AddressLine1: ToString,
        AddressLine2: ToString,
        CityName: ToString,
        StateNameOrAbbr: ToString,
        ZipCode: ToString,
    >(
        address1: Option<AddressLine1>,
        address2: Option<AddressLine2>,
        city: Option<CityName>,
        state: Option<StateNameOrAbbr>,
        zip: Option<ZipCode>,
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

#[cfg(any(test, tarpaulin, feature = "ci"))]
impl PartialEq for StreetAddressArray {
    fn eq(&self, other: &Self) -> bool {
        if self.address2.is_none() != other.address2.is_none() {
            return false;
        }

        let (self_addr2, other_addr2) = (
            self.address2
                .as_ref()
                .map_or(String::new(), |val| val.to_string()),
            other
                .address2
                .as_ref()
                .map_or(String::new(), |val| val.to_string()),
        );

        crate::utils::caseless_eq(&self.address1, &other.address1)
            && crate::utils::caseless_eq(&self_addr2, &other_addr2)
            && crate::utils::caseless_eq(&self.city, &other.city)
            && crate::utils::caseless_eq(&self.state, &other.state)
            && crate::utils::caseless_eq(&self.zip, &other.zip)
    }
}

// </editor-fold desc="// Request Elements ...">

// <editor-fold desc="// Single-Transaction Requests ...">

/// Request for verification made to one of the BriteVerify
/// API's single-transaction, real-time endpoints
#[cfg_attr(any(test, tarpaulin, feature = "ci"), derive(PartialEq))]
#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct VerificationRequest {
    /// The email address to be verified
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// The phone number to be verified
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    /// The street address to be verified
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<StreetAddressArray>,
}

impl VerificationRequest {
    /// Get an builder instance that can be used
    /// to build up a `VerificationRequest` incrementally
    pub fn builder() -> VerificationRequestBuilder {
        VerificationRequestBuilder::new()
    }

    /// Create a new `VerificationRequest`
    /// instance from the supplied values
    pub fn from_values<
        EmailAddress: ToString,
        PhoneNumber: ToString,
        AddressLine1: ToString,
        AddressLine2: ToString,
        CityName: ToString,
        StateNameOrAbbr: ToString,
        ZipCode: ToString,
    >(
        email: Option<EmailAddress>,
        phone: Option<PhoneNumber>,
        address1: Option<AddressLine1>,
        address2: Option<AddressLine2>,
        city: Option<CityName>,
        state: Option<StateNameOrAbbr>,
        zip: Option<ZipCode>,
    ) -> Result<Self, BriteVerifyTypeError> {
        VerificationRequestBuilder::from_values(email, phone, address1, address2, city, state, zip)
            .build()
    }
}

impl TryFrom<String> for VerificationRequest {
    type Error = BriteVerifyTypeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&'_ str> for VerificationRequest {
    type Error = BriteVerifyTypeError;

    fn try_from(value: &'_ str) -> Result<Self, Self::Error> {
        if let Ok(request) = serde_json::from_str::<VerificationRequest>(value) {
            return Ok(request);
        }

        if value.contains('@') {
            return Ok(Self {
                email: Some(value.to_string()),
                ..Self::default()
            });
        }

        const PHONE_CHARS: &str = "0123456789 +().- ext";

        if value
            .to_ascii_lowercase()
            .chars()
            .all(|ch| PHONE_CHARS.contains(ch))
        {
            return Ok(Self {
                phone: Some(value.to_string()),
                ..Self::default()
            });
        }

        Err(BriteVerifyTypeError::AmbiguousTryFromValue(
            value.to_string(),
        ))
    }
}

/// Incremental builder for `VerificationRequest`s
#[derive(Debug, Default)]
pub struct VerificationRequestBuilder {
    _email: Option<String>,
    _phone: Option<String>,
    _address: AddressArrayBuilder,
}

impl VerificationRequestBuilder {
    /// Create a new `VerificationRequestBuilder` instance
    pub fn new() -> VerificationRequestBuilder {
        Self::default()
    }

    /// Build a `VerificationRequest` from the current
    /// builder state
    pub fn build(self) -> Result<VerificationRequest, BriteVerifyTypeError> {
        if self._email.is_some() || self._phone.is_some() || self._address.buildable() {
            Ok(VerificationRequest {
                email: self._email,
                phone: self._phone,
                address: self._address.build().ok(),
            })
        } else {
            Err(BriteVerifyTypeError::UnbuildableRequest(Box::new(self)))
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
    pub fn from_values<
        EmailAddress: ToString,
        PhoneNumber: ToString,
        AddressLine1: ToString,
        AddressLine2: ToString,
        CityName: ToString,
        StateNameOrAbbr: ToString,
        ZipCode: ToString,
    >(
        email: Option<EmailAddress>,
        phone: Option<PhoneNumber>,
        address1: Option<AddressLine1>,
        address2: Option<AddressLine2>,
        city: Option<CityName>,
        state: Option<StateNameOrAbbr>,
        zip: Option<ZipCode>,
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
#[cfg_attr(any(test, tarpaulin, feature = "ci"), derive(PartialEq))]
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
    /// [[ref](https://knowledge.validity.com/hc/en-us/articles/360047111771-Understanding-Statuses-in-BriteVerify#h_01F79WHSGY6FJ6YN1083JWR3QJ)]
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
    /// The "formal" code representing any
    /// error(s) encountered by the BriteVerify
    /// API while verifying the email address
    /// [[ref](https://knowledge.validity.com/hc/en-us/articles/360047111771-Understanding-Statuses-in-BriteVerify)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<VerificationError>,
    /// The human-readable form of the response's
    /// associated "formal" error code
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// The `phone` element of a verification response
#[cfg_attr(any(test, tarpaulin, feature = "ci"), derive(PartialEq))]
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
    #[serde(default)]
    pub service_type: Option<String>,
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
#[cfg_attr(any(test, tarpaulin, feature = "ci"), derive(PartialEq))]
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

/// A response returned by one of the BriteVerify
/// API's single-transaction, real-time endpoints
#[cfg_attr(any(test, tarpaulin, feature = "ci"), derive(PartialEq))]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct VerificationResponse {
    /// Verification data for the requested
    /// email address
    #[serde(default)]
    pub email: Option<EmailVerificationArray>,
    /// Verification data for the requested
    /// phone number
    #[serde(default)]
    pub phone: Option<PhoneNumberVerificationArray>,
    /// Verification data for the requested
    /// street address
    #[serde(default)]
    pub address: Option<AddressVerificationArray>,
    #[serde(
        serialize_with = "crate::utils::duration_to_float",
        deserialize_with = "crate::utils::float_to_duration"
    )]
    /// How long (in seconds) the BriteVerify
    /// API took (internally) to fulfill the
    /// originating verification request
    pub duration: Duration,
}

// </editor-fold desc="// Single-Transaction Responses ...">

// <editor-fold desc="// Test Helpers & Factory Implementations ...">

#[cfg(test)]
#[doc(hidden)]
mod foundry {
    // Standard Library Imports
    use std::collections::HashMap;

    // Third Party Imports
    use serde::de::Error;
    use serde_json::{Map as JsonMap, Value};

    type RawAddressMap = HashMap<String, Option<String>>;
    type RawAddressJson = JsonMap<String, Value>;

    impl TryFrom<Value> for super::StreetAddressArray {
        type Error = serde_json::Error;

        fn try_from(value: Value) -> Result<Self, Self::Error> {
            (&value).try_into()
        }
    }

    impl TryFrom<&Value> for super::StreetAddressArray {
        type Error = serde_json::Error;

        fn try_from(value: &Value) -> Result<Self, Self::Error> {
            match value.as_object() {
                None => Err(Self::Error::custom(format!(
                    "Cannot create a `StreetAddressArray` from: {:#?}",
                    value.as_str()
                ))),
                Some(data) => {
                    if let Some(obj) = data.get("address") {
                        return obj.try_into();
                    }

                    data.try_into()
                }
            }
        }
    }

    impl TryFrom<RawAddressMap> for super::StreetAddressArray {
        type Error = serde_json::Error;

        fn try_from(data: RawAddressMap) -> Result<Self, Self::Error> {
            (&data).try_into()
        }
    }

    impl TryFrom<&RawAddressMap> for super::StreetAddressArray {
        type Error = serde_json::Error;

        fn try_from(data: &RawAddressMap) -> Result<Self, Self::Error> {
            let (address1, address2, city, state, zip) = (
                data.get("address1").unwrap().clone(),
                data.get("address2").unwrap().clone(),
                data.get("city").unwrap().clone(),
                data.get("state").unwrap().clone(),
                data.get("zip").unwrap().clone(),
            );

            match super::AddressArrayBuilder::from_values(address1, address2, city, state, zip)
                .build()
            {
                Ok(address) => Ok(address),
                Err(_) => Err(Self::Error::custom(format!(
                    "One or more required fields missing from: {data:#?}"
                ))),
            }
        }
    }

    impl TryFrom<RawAddressJson> for super::StreetAddressArray {
        type Error = serde_json::Error;

        fn try_from(value: RawAddressJson) -> Result<Self, Self::Error> {
            (&value).try_into()
        }
    }

    impl TryFrom<&RawAddressJson> for super::StreetAddressArray {
        type Error = serde_json::Error;

        fn try_from(data: &RawAddressJson) -> Result<Self, Self::Error> {
            ["address1", "city", "state", "zip"]
                .into_iter()
                .map(|key| (key.to_string(), data.get(key)))
                .map(|(key, value)| (key, value.map(|value| value.to_string())))
                .collect::<HashMap<String, Option<String>>>()
                .try_into()
        }
    }

    impl super::AddressArrayBuilder {
        #[cfg_attr(tarpaulin, coverage(off))]
        #[cfg_attr(tarpaulin, tarpaulin::skip)]
        /// The current value of builder's `_address1` field
        pub fn address1_value(&self) -> Option<&String> {
            self._address1.as_ref()
        }

        #[cfg_attr(tarpaulin, coverage(off))]
        #[cfg_attr(tarpaulin, tarpaulin::skip)]
        /// The current value of builder's `_address2` field
        pub fn address2_value(&self) -> Option<&String> {
            self._address2.as_ref()
        }

        #[cfg_attr(tarpaulin, coverage(off))]
        #[cfg_attr(tarpaulin, tarpaulin::skip)]
        /// The current value of builder's `_city1` field
        pub fn city_value(&self) -> Option<&String> {
            self._city.as_ref()
        }

        #[cfg_attr(tarpaulin, coverage(off))]
        #[cfg_attr(tarpaulin, tarpaulin::skip)]
        /// The current value of builder's `_state` field
        pub fn state_value(&self) -> Option<&String> {
            self._state.as_ref()
        }

        #[cfg_attr(tarpaulin, coverage(off))]
        #[cfg_attr(tarpaulin, tarpaulin::skip)]
        /// The current value of builder's `_zip` field
        pub fn zip_value(&self) -> Option<&String> {
            self._zip.as_ref()
        }
    }
}

// </editor-fold desc="// Test Helpers & Factory Implementations ...">

// <editor-fold desc="// I/O-Free Tests ...">

#[cfg(test)]
mod tests {
    // Standard-Library Imports
    use std::clone::Clone;

    // Third-Party Dependencies
    use anyhow::Result;
    use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};

    // <editor-fold desc="// Constants ...">

    const STATE: &str = "CA";
    const ZIP: &str = "90210";
    const CITY: &str = "Any Town";
    const ADDRESS1: &str = "123 Main St.";
    const ADDRESS2: Option<&str> = Some("P.O. Box 456");
    const EMAIL: &str = "test@example.com";
    const PHONE: &str = "+1 (954) 555-1234 ext. 6789";

    // </editor-fold desc="// Constants ...">

    /// Test that the `AddressArrayBuilder` builds the expected
    /// `StreetAddressArray` from discrete values
    #[rstest::rstest]
    fn test_address_from_values() {
        let instance = super::AddressArrayBuilder::from_values(
            Some(ADDRESS1),
            ADDRESS2,
            Some(CITY),
            Some(STATE),
            Some(ZIP),
        )
        .build();

        assert!(instance.is_ok(), "{:#?}", instance.unwrap_err());

        let instance = instance.unwrap();

        assert_str_eq!(ZIP, instance.zip);
        assert_str_eq!(CITY, instance.city);
        assert_str_eq!(STATE, instance.state);
        assert_str_eq!(ADDRESS1, instance.address1);
        assert_str_eq!(format!("{ADDRESS2:?}"), format!("{:?}", instance.address2));
    }

    /// Test that `StreetAddressArray`s can be compared
    /// for equality while the test suite is active
    #[rstest::rstest]
    fn test_address_equality() -> Result<()> {
        let left = super::StreetAddressArray::from_values(ADDRESS1, ADDRESS2, CITY, STATE, ZIP);

        #[allow(clippy::redundant_clone)]
        let mut right = left.clone();

        assert_eq!(left, right);

        right.address2 = None;

        Ok(assert_ne!(left, right))
    }

    /// Test that the `AddressArrayBuilder` refuses
    /// to build "incomplete" `StreetAddressArray`s
    #[rstest::rstest]
    fn test_address_buildability() {
        let builder = super::StreetAddressArray::builder();

        assert_eq!(
            !builder.buildable(),
            builder.address1_value().is_none()
                && builder.address2_value().is_none()
                && builder.city_value().is_none()
                && builder.state_value().is_none()
                && builder.zip_value().is_none()
        );

        // Addresses cannot be built unless all fields (except
        // `address2`) contain non-`None`, non-empty values
        let builder = builder
            .address2(ADDRESS2.unwrap())
            .city(CITY)
            .state(STATE)
            .zip(ZIP)
            .build();

        assert!(builder.is_err());

        let builder = match builder.unwrap_err() {
            super::BriteVerifyTypeError::UnbuildableAddressArray(inner) => inner,
            _ => panic!(),
        };

        let builder = builder.address1(ADDRESS1).build();

        assert!(builder.is_ok());
    }

    /// Test that `VerificationRequest`s can be
    /// (fallibly) created from "bare" strings
    #[rstest::rstest]
    fn test_try_into_verification_request() {
        assert!(super::VerificationRequest::try_from(EMAIL)
            .is_ok_and(|req| req.email.is_some_and(|email| email == EMAIL)));
        assert!(super::VerificationRequest::try_from(PHONE)
            .is_ok_and(|req| req.phone.is_some_and(|phone| phone == PHONE)));

        let address_data = format!(
            r#"{{"address":{{"address1":"{ADDRESS1}","city":"{CITY}","state":"{STATE}","zip":"{ZIP}"}}}}"#
        );
        assert!(
            super::VerificationRequest::try_from(address_data).is_ok_and(|req| req.email.is_none()
                && req.phone.is_none()
                && req.address.is_some())
        );

        assert!(super::VerificationRequest::try_from(format!(
            r#"{ADDRESS1}, {CITY}, {STATE} {ZIP}"#
        ))
        .is_err_and(|error| {
            matches!(error, super::BriteVerifyTypeError::AmbiguousTryFromValue(_))
        }));
    }

    /// Test that `VerificationRequestBuilder`s properly
    /// enforce the non-empty field requirements for each
    /// buildable request type
    #[rstest::rstest]
    fn test_verification_request_buildability() {
        let builder = super::VerificationRequest::builder();

        assert!(!builder.buildable());

        let builder = builder
            .address2(ADDRESS2.unwrap())
            .city(CITY)
            .state(STATE)
            .zip(ZIP);

        assert!(!builder.buildable());

        let builder = builder.email(EMAIL).phone(PHONE).address1(ADDRESS1);

        assert!(builder.buildable());
        assert!(builder.build().is_ok());

        // Unbuildable Request
        let mut build_result = super::VerificationRequest::builder().build();

        assert!(
            build_result.as_ref().is_err_and(|error| matches!(
                error,
                super::BriteVerifyTypeError::UnbuildableRequest(_)
            )),
            "Expected Err(UnbuildableRequest), got: {:#?}",
            build_result.as_ref(),
        );

        // Email Request
        build_result = super::VerificationRequest::builder().email(EMAIL).build();

        assert!(
            build_result.as_ref().is_ok_and(|req| req.email.is_some()
                && req.phone.is_none()
                && req.address.is_none()),
            "Expected Ok(VerificationRequest) w/ Some(email), got: {:#?}",
            build_result.as_ref(),
        );

        // Phone Request
        build_result = super::VerificationRequest::builder().phone(PHONE).build();

        assert!(
            build_result.as_ref().is_ok_and(|req| req.email.is_none()
                && req.phone.is_some()
                && req.address.is_none()),
            "Expected Ok(VerificationRequest) w/ Some(phone), got: {:#?}",
            build_result.as_ref(),
        );

        // Address Request
        build_result = super::VerificationRequest::builder()
            .address1(ADDRESS1)
            .address2(ADDRESS2.unwrap())
            .city(CITY)
            .state(STATE)
            .zip(ZIP)
            .build();

        assert!(
            build_result.as_ref().is_ok_and(|req| req.email.is_none()
                && req.phone.is_none()
                && req.address.is_some()),
            "Expected Ok(VerificationRequest) w/ Some(address), got: {:#?}",
            build_result.as_ref(),
        );

        // Email & Phone Request
        build_result = super::VerificationRequest::builder()
            .email(EMAIL)
            .phone(PHONE)
            .build();

        assert!(
            build_result.as_ref().is_ok_and(|req| req.email.is_some()
                && req.phone.is_some()
                && req.address.is_none()),
            "Expected Ok(VerificationRequest) w/ Some(email) & Some(phone), got: {:#?}",
            build_result.as_ref(),
        );

        // Email & Address Request
        build_result = super::VerificationRequest::builder()
            .email(EMAIL)
            .address1(ADDRESS1)
            .address2(ADDRESS2.unwrap())
            .city(CITY)
            .state(STATE)
            .zip(ZIP)
            .build();

        assert!(
            build_result.as_ref().is_ok_and(|req| req.email.is_some()
                && req.phone.is_none()
                && req.address.is_some()),
            "Expected Ok(VerificationRequest) w/ Some(email) & Some(address), got: {:#?}",
            build_result.as_ref(),
        );

        // Phone & Address Request
        build_result = super::VerificationRequest::builder()
            .phone(PHONE)
            .address1(ADDRESS1)
            .address2(ADDRESS2.unwrap())
            .city(CITY)
            .state(STATE)
            .zip(ZIP)
            .build();

        assert!(
            build_result.as_ref().is_ok_and(|req| req.email.is_none()
                && req.phone.is_some()
                && req.address.is_some()),
            "Expected Ok(VerificationRequest) w/ Some(phone) & Some(address), got: {:#?}",
            build_result.as_ref(),
        );

        // "Full" Request
        build_result = super::VerificationRequest::from_values(
            Some(EMAIL),
            Some(PHONE),
            Some(ADDRESS1),
            ADDRESS2,
            Some(CITY),
            Some(STATE),
            Some(ZIP),
        );

        assert!(
            build_result.as_ref().is_ok_and(|req| req.email.is_some()
                && req.phone.is_some()
                && req.address.is_some()),
            "Expected Ok(VerificationRequest) w/ all fields populated, got: {:#?}",
            build_result.as_ref(),
        );
    }
}

// </editor-fold desc="// I/O-Free Tests ...">

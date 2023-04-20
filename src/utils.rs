//! ## Utility Functions
///
// Standard Library Imports
use std::time::Duration;

// Third Party Imports
use anyhow::Result;
use chrono::{
    prelude::{DateTime, NaiveDateTime, Utc},
    LocalResult as ChronoResult,
};
use http::Uri;
use serde_json::Value;

use crate::types::BulkListDirective;

#[doc(hidden)]
#[cfg(any(test, feature = "examples"))]
pub use self::test_utils::*;

// <editor-fold desc="// Utility Functions ...">

/// Deserializer implementation for enabling `serde`
/// to interpret the floating point `duration` values
/// returned by the BriteVerify API as `std::time::Duration`s.
pub(crate) fn float_to_duration<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Duration, D::Error> {
    let value: f64 = <f64 as serde::Deserialize>::deserialize(deserializer)?;

    match Duration::try_from_secs_f64(value) {
        Ok(duration) => Ok(duration),
        Err(error) => Err(serde::de::Error::custom(format!("{error}"))),
    }
}

/// Serializer implementation for enabling `serde`
/// to interpret `std::time::Duration` values as their
/// corresponding floating point `duration` values as
/// originally returned by the BriteVerify API.
pub(crate) fn duration_to_float<S: serde::Serializer>(
    value: &Duration,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_f64(value.as_secs_f64())
}

/// Deserializer implementation for overriding how `serde`
/// deserializes `Option<String>`-type values, preferring
/// `None` over empty strings.
pub(crate) fn empty_string_is_none<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    let value: Option<String> = <Option<String> as serde::Deserialize>::deserialize(deserializer)?;

    match value {
        None => Ok(None),
        Some(value) => {
            if value.trim().is_empty() {
                Ok(None)
            } else {
                Ok(Some(value))
            }
        }
    }
}

/// Serializer implementation for enabling `serde`
/// to interpret `http::Uri` values as rust `String`s.
pub(crate) fn serialize_uri<S: serde::Serializer>(
    value: &Option<Uri>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    if let Some(uri) = value {
        serializer.serialize_str(uri.to_string().as_str())
    } else {
        serializer.serialize_none()
    }
}

/// Deserializer implementation for enabling `serde`
/// to interpret string values as `http::Uri`s.
pub(crate) fn deserialize_uri<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Uri>, D::Error> {
    let value: Option<String> = <Option<String> as serde::Deserialize>::deserialize(deserializer)?;

    match value {
        None => Ok(None),
        Some(string) => {
            if string.trim().is_empty() {
                Ok(None)
            } else {
                match Uri::try_from(string) {
                    Ok(uri) => Ok(Some(uri)),
                    Err(error) => Err(serde::de::Error::custom(format!("{error}"))),
                }
            }
        }
    }
}

/// Fallibly cast the weirdly formatted timestamps
/// returned by the BriteVerify API to `chrono::DateTime<Utc>`s.
pub(crate) fn bv_timestamp_to_dt<T: AsRef<str>>(value: T) -> ChronoResult<DateTime<Utc>> {
    let value = value.as_ref();
    match NaiveDateTime::parse_from_str(value, "%m-%d-%Y %I:%M %P") {
        Ok(timestamp) => timestamp.and_local_timezone(Utc),
        Err(error) => {
            tracing::error!("Unparsable timestamp value: {value}\n{error:#?}");
            ChronoResult::None
        }
    }
}

#[doc(hidden)]
/// Simple abstraction for logic shared by
/// `deserialize_timestamp` and `deserialize_maybe_timestamp`
fn _deserialize_timestamp<SerdeError: serde::de::Error>(
    timestamp: String,
) -> Result<DateTime<Utc>, SerdeError> {
    match bv_timestamp_to_dt(&timestamp) {
        ChronoResult::None => Err(serde::de::Error::custom(format!(
            "Couldn't parse the supplied value into a valid timestamp: {timestamp:?}"
        ))),
        ChronoResult::Single(parsed) | ChronoResult::Ambiguous(parsed, _) => Ok(parsed),
    }
}

/// Deserializer implementation for enabling `serde`
/// to properly cast the weirdly formatted timestamps
/// returned by the BriteVerify API to `chrono::DateTime<Utc>`s.
pub(crate) fn deserialize_timestamp<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<DateTime<Utc>, D::Error> {
    let timestamp: String = <String as serde::Deserialize>::deserialize(deserializer)?;
    _deserialize_timestamp(timestamp)
}

/// Deserializer implementation for enabling `serde`
/// to properly cast the weirdly formatted timestamps
/// returned by the BriteVerify API to `chrono::DateTime<Utc>`s.
pub(crate) fn deserialize_maybe_timestamp<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<DateTime<Utc>>, D::Error> {
    let value: Option<String> = <Option<String> as serde::Deserialize>::deserialize(deserializer)?;

    match value {
        None => Ok(None),
        Some(timestamp) => {
            if timestamp.is_empty() {
                Ok(None)
            } else {
                match _deserialize_timestamp(timestamp) {
                    Ok(result) => Ok(Some(result)),
                    Err(error) => Err(error),
                }
            }
        }
    }
}

/// Utility function for ensuring `serde` omits unknown
/// `directive` values when sending bulk verification
/// requests to the BriteVerify API.
pub(crate) fn is_unknown_list_directive(directive: &BulkListDirective) -> bool {
    matches!(directive, BulkListDirective::Unknown)
}

/// Deserializer implementation for enabling `serde`
/// to gracefully handle the maybe-stringified boolean
/// values the BriteVerify API returns for addresses.
pub(crate) fn deserialize_boolean<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<bool, D::Error> {
    let value = <Value as serde::Deserialize>::deserialize(deserializer)?;

    if value.is_boolean() {
        return Ok(value.as_bool().unwrap());
    }

    let value = value.to_string();
    let trimmed = value
        .strip_prefix('"')
        .unwrap_or(&value)
        .strip_suffix('"')
        .unwrap_or(&value)
        .to_string();

    match trimmed.parse::<bool>() {
        Ok(flag) => Ok(flag),
        Err(error) => Err(serde::de::Error::custom(format!(
            "Couldn't deserialize '{value}' due to: {error:?}"
        ))),
    }
}

// </editor-fold desc="// Utility Functions ...">

// <editor-fold desc="// Test Factory Utilities ...">

#[doc(hidden)]
#[cfg_attr(tarpaulin, no_coverage)]
#[cfg(any(test, feature = "examples"))]
/// Utility functions for `briteverify-rs`'s test suite and examples
pub mod test_utils {
    use chrono::{Datelike, Timelike};
    use once_cell::sync::Lazy;
    use warlocks_cauldron as wc;

    pub(crate) static FAKE: Lazy<wc::ComplexProvider> =
        Lazy::new(|| wc::ComplexProvider::new(&wc::Locale::EN));

    static ADDR2_POOL: Lazy<wc::RandomPool<String>> = Lazy::new(|| {
        wc::RandomPool::new(
            vec!["Unit #", "P.O. Box", "Suite #", "Bldg", "Apt. #", "#"]
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>(),
        )
    });

    static SEP_POOL: Lazy<wc::RandomPool<String>> = Lazy::new(|| {
        wc::RandomPool::new(
            vec!["", ".", "-", "_"]
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>(),
        )
    });

    static TLD_POOL: Lazy<wc::RandomPool<String>> = Lazy::new(|| {
        wc::RandomPool::new(
            vec!["ca", "ru", "biz", "gov", "org", "com", "net", "co.uk"]
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>(),
        )
    });

    static HOST_POOL: Lazy<wc::RandomPool<String>> = Lazy::new(|| {
        wc::RandomPool::new(
            vec!["example", "test", "bounce-me", "invalid", "not-real"]
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>(),
        )
    });

    /// A struct that can generate instances of
    /// itself populated with realistic dummy data
    pub trait RandomizableStruct: Sized {
        /// Get a randomly generated instance
        fn random() -> Self;
    }

    /// An enum that can provide a random selection
    /// from a pool of its own members on demand
    pub trait RandomizableEnum: Sized + strum::IntoEnumIterator {
        /// Get a randomly chosen enum member
        fn random() -> Self {
            wc::Choice::get(<Self as strum::IntoEnumIterator>::iter())
        }
    }

    /// Randomly generate a fake IETF RFC5322-compliant email address
    pub fn random_email() -> String {
        let (first, last): (String, String) = if wc::Choice::prob(0.50) {
            (
                FAKE.text.word().to_lowercase(),
                FAKE.text.word().to_lowercase(),
            )
        } else {
            (
                FAKE.person.first_name(None).to_lowercase(),
                FAKE.person.last_name(None).to_lowercase(),
            )
        };

        let digits = if wc::Choice::prob(0.25) {
            FAKE.address.street_number().to_string()
        } else {
            "".to_string()
        };

        format!(
            "{}{}{}{}@{}.{}",
            first,
            SEP_POOL.get(),
            last,
            digits,
            HOST_POOL.get(),
            TLD_POOL.get()
        )
    }

    fn _one_week_ago(now: &super::DateTime<super::Utc>) -> super::DateTime<super::Utc> {
        now.with_day(now.day() - 7).unwrap()
    }

    fn _a_few_hours_ago(now: &super::DateTime<super::Utc>) -> super::DateTime<super::Utc> {
        let offset = wc::Numeric::number(1u32, 5u32);
        now.with_hour(now.hour() - offset).unwrap()
    }

    /// Randomly generate a timestamp from within the past week
    pub fn within_the_last_week() -> super::DateTime<super::Utc> {
        let now = super::Utc::now();
        let start = _one_week_ago(&now);

        let pool =
            wc::Datetime::bulk_create_datetimes::<super::Utc>(start, now, wc::Duration::hours(8));

        wc::Choice::get(pool.into_iter())
    }

    /// Randomly generate a timestamp from a few hours in the past
    pub fn within_the_last_few_hours() -> super::DateTime<super::Utc> {
        let now = super::Utc::now();
        let start = _a_few_hours_ago(&now);

        let pool = wc::Datetime::bulk_create_datetimes::<super::Utc>(
            start,
            now,
            wc::Duration::minutes(15),
        );

        wc::Choice::get(pool.into_iter())
    }

    /// Randomly generate an address's (optional) "line 2"
    pub fn address_line2() -> Option<String> {
        if wc::Choice::prob(0.50) {
            None
        } else {
            Some(format!(
                "{} {}",
                ADDR2_POOL.get(),
                FAKE.address.street_number(),
            ))
        }
    }
}

// </editor-fold desc="// Test Factory Utilities ...">

// <editor-fold desc="// I/O-Free Tests ...">

#[cfg(test)]
mod tests {
    // Standard Library Imports
    use std::ops::Deref;

    // Third-Party Dependencies
    use anyhow::Result;
    use chrono::{Datelike, Timelike};
    use once_cell::sync::Lazy;
    use pretty_assertions::assert_eq;
    use rstest::{fixture, rstest};
    use serde_test::{Deserializer, Token};
    use strum::IntoEnumIterator;
    use warlocks_cauldron as wc;

    // Crate-Level Dependencies
    use super::{ChronoResult, DateTime, Duration, Uri, Utc};

    const TIMESTAMP: &str = "01-11-2023 4:45 pm";
    static RECENT_DATETIMES: Lazy<Vec<DateTime<Utc>>> = Lazy::new(|| {
        let start_date = super::within_the_last_week()
            .with_second(0)
            .and_then(|value| value.with_nanosecond(0))
            .unwrap();

        wc::Datetime::bulk_create_datetimes::<Utc>(start_date, Utc::now(), wc::Duration::minutes(1))
    });

    #[fixture]
    fn recent_datetimes() -> &'static Vec<DateTime<Utc>> {
        RECENT_DATETIMES.deref()
    }

    /// Test that the `float_to_duration` utility
    /// returns a valid `Duration` when the supplied
    /// value is a valid `f64`
    #[rstest]
    fn test_valid_float_to_duration() -> Result<()> {
        let tokens: [Token; 1] = [Token::F64(1.0)];
        let expected: Duration = Duration::from_secs(1);

        let mut deserializer: Deserializer = Deserializer::new(&tokens);
        let result: Duration = super::float_to_duration(&mut deserializer)?;

        Ok(assert_eq!(result, expected))
    }

    /// Test that the `float_to_duration` utility
    /// returns an error `Duration` when the supplied
    /// value cannot be deserialized as an `f64` or
    /// when the deserialized value cannot be converted
    /// to a valid `Duration`
    #[rstest]
    fn test_invalid_float_to_duration() -> () {
        let tokens: [Token; 1] = [Token::F64(-1.0)];

        let mut deserializer: Deserializer = Deserializer::new(&tokens);

        let result = super::float_to_duration(&mut deserializer);

        assert!(result.is_err())
    }

    /// Test that the `duration_to_float` utility
    /// behaves as expected when supplied with a
    /// valid `Duration` and usable `Serializer`
    #[rstest]
    fn test_duration_to_float() -> () {
        let value = Duration::from_secs(10);
        let mut serializer = serde_json::Serializer::new(<Vec<u8>>::new());

        let result = super::duration_to_float(&value, &mut serializer);

        assert!(result.is_ok())
    }

    /// Test that the `float_to_duration` utility
    /// returns an error `Duration` when the supplied
    /// value cannot be deserialized as an `f64` or
    /// when the deserialized value cannot be converted
    /// to a valid `Duration`
    #[rstest]
    fn test_string_to_duration_fails() -> Result<()> {
        let tokens: [Token; 1] = [Token::String("got-em")];

        let mut deserializer: Deserializer = Deserializer::new(&tokens);

        let result = super::float_to_duration(&mut deserializer);

        Ok(assert!(result.is_err()))
    }

    /// Test that the `empty_string_is_none` utility
    /// returns `None` when the supplied value is either
    /// an empty string or the `None` token
    #[rstest]
    fn test_empty_string_is_none() -> Result<()> {
        let tokens: [[Token; 2]; 2] =
            [[Token::Some, Token::String("")], [Token::None, Token::None]];

        for token_array in tokens.iter() {
            let mut deserializer: Deserializer = Deserializer::new(token_array);
            let result: Option<String> = super::empty_string_is_none(&mut deserializer)?;

            assert!(result.is_none());
        }

        Ok(())
    }

    /// Test that the `empty_string_is_none` utility
    /// returns a valid `String` when the supplied
    /// value is a non-empty string
    #[rstest]
    fn test_non_empty_string_is_not_none() -> Result<()> {
        let expected: String = "got-em".to_string();
        let tokens: [Token; 2] = [Token::Some, Token::String("got-em")];

        let mut deserializer: Deserializer = Deserializer::new(&tokens);
        let result: Option<String> = super::empty_string_is_none(&mut deserializer)?;

        assert!(result.is_some());

        Ok(assert_eq!(result.unwrap(), expected))
    }

    /// Test that the `serialize_uri` utility
    /// behaves as expected
    #[rstest]
    fn test_serialize_uri() -> () {
        let (some_value, none_value): (Option<Uri>, Option<Uri>) =
            (Some(Uri::from_static("https://example.com")), None);
        let mut serializer = serde_json::Serializer::new(<Vec<u8>>::new());

        let some_result = super::serialize_uri(&some_value, &mut serializer);
        let none_result = super::serialize_uri(&none_value, &mut serializer);

        assert!(some_result.is_ok());
        assert!(none_result.is_ok());
    }

    /// Test that the `deserialize_uri` utility
    /// returns `None` when the supplied value is either
    /// an empty string or the `None` token
    #[rstest]
    fn test_deserialize_empty_uri() -> () {
        let tokens: [[Token; 2]; 2] =
            [[Token::Some, Token::String("")], [Token::None, Token::None]];

        for token_array in tokens.iter() {
            let mut deserializer: Deserializer = Deserializer::new(token_array);
            let result: Result<Option<Uri>, _> = super::deserialize_uri(&mut deserializer);

            assert!(result.is_ok());
            assert!(result.unwrap().is_none());
        }
    }

    /// Test that the `deserialize_uri` utility
    /// returns a valid `Uri` when the supplied
    /// value is a non-empty URI-like string
    #[rstest]
    fn test_deserialize_valid_uri() -> () {
        let src: &str = "https://example.com";
        let expected: Uri = Uri::from_static(src);
        let tokens: [Token; 2] = [Token::Some, Token::String(src)];

        let mut deserializer: Deserializer = Deserializer::new(&tokens);
        let result: Result<Option<Uri>, _> = super::deserialize_uri(&mut deserializer);

        assert!(result.is_ok());

        let result = result.unwrap();

        assert!(result.is_some());

        let deserialized = result.unwrap();

        assert_eq!(expected, deserialized)
    }

    /// Test that the `deserialize_uri` utility
    /// returns an error when the supplied
    /// value is a non-empty, non-URI-like string
    #[rstest]
    fn test_deserialize_invalid_uri() -> () {
        let tokens: [Token; 2] = [
            Token::Some,
            Token::String("the most dangerous type of canoes are volcanoes"),
        ];

        let mut deserializer: Deserializer = Deserializer::new(&tokens);
        let result = super::deserialize_uri(&mut deserializer);

        assert!(result.is_err())
    }

    /// Test that the `bv_timestamp_to_dt` utility
    /// returns a valid `DateTime<Utc>` when the
    /// supplied value is a BriteVerify-formatted
    /// timestamp string (i.e."%m-%d-%Y %I:%M %P")
    #[rstest]
    fn test_valid_bv_timestamp(recent_datetimes: &[DateTime<Utc>]) -> Result<()> {
        for value in recent_datetimes.iter() {
            let parsed =
                match super::bv_timestamp_to_dt(value.format("%m-%d-%Y %I:%M %P").to_string()) {
                    ChronoResult::None => {
                        anyhow::bail!("Couldn't parse: {value:#?}")
                    }
                    ChronoResult::Single(stamp) | ChronoResult::Ambiguous(stamp, _) => stamp,
                };

            assert_eq!(value, &parsed);
        }

        Ok(())
    }

    /// Test that the `bv_timestamp_to_dt` utility
    /// returns `chrono::LocalResult::None` when the
    /// supplied value is not a BriteVerify-formatted
    /// timestamp string
    #[rstest]
    fn test_invalid_bv_timestamp(recent_datetimes: &[DateTime<Utc>]) -> () {
        for value in recent_datetimes.iter() {
            let parsed = super::bv_timestamp_to_dt(value.to_rfc2822());
            assert_eq!(parsed, ChronoResult::None);
        }
    }

    /// Test that the `deserialize_timestamp` utility
    /// returns a valid `DateTime<Utc>` when the value
    /// being deserialized is a BriteVerify-formatted
    /// timestamp string (i.e."%m-%d-%Y %I:%M %P")
    #[rstest]
    fn test_deserialize_timestamp() -> () {
        let tokens: [Token; 1] = [Token::String(TIMESTAMP)];

        let mut deserializer: Deserializer = Deserializer::new(&tokens);
        let result = super::deserialize_timestamp(&mut deserializer);

        assert!(result.is_ok());

        let deserialized = result.unwrap();

        assert_eq!(deserialized.day(), 11u32);
        assert_eq!(deserialized.month(), 1u32);
        assert_eq!(deserialized.year(), 2023i32);

        assert_eq!(deserialized.minute(), 45u32);
        assert_eq!(deserialized.hour12(), (true, 4u32));
    }

    /// Test that the `deserialize_timestamp` utility
    /// returns an error when the value being deserialized
    /// is anything other than a BriteVerify-formatted timestamp
    #[rstest]
    fn test_deserialize_non_timestamp() -> () {
        let tokens: [Token; 1] = [Token::String(
            "I thought I'd do was I'd pretend I was one of those deaf-mutes",
        )];

        let mut deserializer: Deserializer = Deserializer::new(&tokens);
        let result = super::deserialize_timestamp(&mut deserializer);

        assert!(result.is_err());
    }

    /// Test that the `deserialize_maybe_timestamp`
    /// utility behaves as expected when the value
    /// being deserialized is a properly formatted
    /// timestamp string value
    #[rstest]
    fn test_deserialize_some_timestamp() -> () {
        let tokens: [Token; 2] = [Token::Some, Token::String(TIMESTAMP)];

        let mut deserializer: Deserializer = Deserializer::new(&tokens);
        let result = super::deserialize_maybe_timestamp(&mut deserializer);

        assert!(result.is_ok());

        let result = result.unwrap();

        assert!(result.is_some());

        let deserialized = result.unwrap();

        assert_eq!(deserialized.hour(), 16u32)
    }

    /// Test that the `deserialize_maybe_timestamp`
    /// utility behaves as expected when the value
    /// being deserialized is either `null` or an
    /// empty string
    #[rstest]
    fn test_deserialize_empty_timestamp() -> () {
        let tokens: [[Token; 2]; 2] =
            [[Token::Some, Token::String("")], [Token::None, Token::None]];

        for token_array in tokens.iter() {
            let mut deserializer: Deserializer = Deserializer::new(token_array);
            let result = super::deserialize_maybe_timestamp(&mut deserializer);

            assert!(result.is_ok());
            assert!(result.unwrap().is_none());
        }
    }

    /// Test that the `deserialize_maybe_timestamp` utility
    /// returns an error when the value being deserialized
    /// is anything other than a BriteVerify-formatted timestamp
    #[rstest]
    fn test_deserialize_some_non_timestamp() -> () {
        let tokens: [Token; 2] = [
            Token::Some,
            Token::String("Ostensibly, the ol' razzle dazzle"),
        ];

        let mut deserializer: Deserializer = Deserializer::new(&tokens);
        let result = super::deserialize_maybe_timestamp(&mut deserializer);

        assert!(result.is_err());
    }

    /// Test that the `is_unknown_list_directive`
    /// utility correctly identifies the "unknown"
    /// variant of the `ListDirective` enum
    #[rstest]
    fn test_is_unknown_list_directive() -> Result<()> {
        for member in super::BulkListDirective::iter() {
            let result = super::is_unknown_list_directive(&member);
            let is_known_member = member == super::BulkListDirective::Unknown;

            assert!(
                (is_known_member && result) || (!is_known_member && !result),
                "member: {:?}, known: {:?}",
                member,
                result
            );
        }

        Ok(())
    }

    /// Test that the `deserialize_boolean` utility
    /// returns a valid `bool` when the supplied
    /// value represents a valid `bool` (either
    /// directly or as a string)
    #[rstest]
    fn test_deserialize_boolean() -> () {
        let tokens: [[Token; 1]; 2] = [[Token::Bool(true)], [Token::String("true")]];

        for token_array in tokens.iter() {
            let mut deserializer: Deserializer = Deserializer::new(token_array);
            let result = super::deserialize_boolean(&mut deserializer);

            assert!(
                result.is_ok(),
                "Expected a valid boolean value, got: {:?}",
                result
            );
            assert_eq!(result.unwrap(), true);
        }
    }

    /// Test that the `deserialize_boolean` utility
    /// returns an error when the supplied value
    /// represents something other than a valid `bool`
    #[rstest]
    fn test_deserialize_non_boolean() -> () {
        let tokens: [Token; 1] = [Token::String(
            "a literal boolean value, you know, like 'true' or maybe 'false'",
        )];

        let mut deserializer: Deserializer = Deserializer::new(&tokens);
        let result = super::deserialize_boolean(&mut deserializer);

        assert!(result.is_err())
    }
}

// </editor-fold desc="// I/O-Free Tests ...">

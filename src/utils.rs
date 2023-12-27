//! ## Utility Functions

// Standard Library Imports
use std::{
    fmt::{Debug, Display},
    time::Duration,
};

// Third Party Imports
use anyhow::Result;
use chrono::{
    prelude::{DateTime, NaiveDateTime, Utc},
    LocalResult as ChronoResult,
};
use http::Uri;
use serde_json::Value;

// Crate-Level Imports
use crate::types::BulkListDirective;

#[cfg(test)]
#[doc(hidden)]
pub use self::test_utils::*;

// <editor-fold desc="// Utility Functions ...">

#[doc(hidden)]
/// Remove any '"' characters enclosing the supplied string value
fn unquote(value: String) -> Option<String> {
    let is_quote = |val: char| -> bool { val == '"' || val == '\'' };

    let value = value
        .trim_start_matches(is_quote)
        .trim_end_matches(is_quote)
        .trim();

    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

/// Determine if the supplied client/builder has
/// an API key / authorization header set already
pub(crate) fn has_auth_header<T: Debug>(obj: &T) -> bool {
    let obj_repr = std::format!("{:?}", obj);

    obj_repr.contains(r#""authorization": Sensitive"#)
        || obj_repr.contains(r#""authorization": "ApiKey:"#)
}

/// Deserializer implementation for enabling `serde`
/// to interpret the floating point `duration` values
/// returned by the BriteVerify API as `std::time::Duration`s.
pub(crate) fn float_to_duration<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Duration, D::Error> {
    let value: f64 = <f64 as serde::Deserialize>::deserialize(deserializer)?;

    match Duration::try_from_secs_f64(value) {
        Ok(duration) => Ok(duration),
        Err(error) => Err(serde::de::Error::custom(std::format!("{error}"))),
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
                    Err(error) => Err(serde::de::Error::custom(std::format!("{error}"))),
                }
            }
        }
    }
}

/// Deserializer implementation for enabling `serde`
/// to properly deserialize the ambiguously-typed
/// values the BriteVerify API returns for external
/// identifier fields.
pub(crate) fn deserialize_ext_id<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    let value: Option<Value> = <Option<Value> as serde::Deserialize>::deserialize(deserializer)?;

    value.map_or(Ok(None), |value| match value {
        Value::Null => Ok(None),
        Value::String(ext_id) => serde_json::to_string(&ext_id)
            .map(unquote)
            .map_err(serde::de::Error::custom),
        Value::Number(ext_id) => serde_json::to_string(&ext_id)
            .map(unquote)
            .map_err(serde::de::Error::custom),
        value => Err(serde::de::Error::invalid_type(
            serde::de::Unexpected::Other(value.to_string().as_str()),
            &"a scalar-type value (e.g. u64 or str)",
        )),
    })
}

/// Fallibly cast the weirdly formatted timestamps
/// returned by the BriteVerify API to `chrono::DateTime<Utc>`s.
pub(crate) fn bv_timestamp_to_dt<T: AsRef<str>>(value: T) -> ChronoResult<DateTime<Utc>> {
    let value = value.as_ref();
    match NaiveDateTime::parse_from_str(value, "%m-%d-%Y %I:%M %P") {
        Ok(timestamp) => timestamp.and_local_timezone(Utc),
        Err(error) => {
            log::error!("Unparsable timestamp value: {value}\n{error:#?}");
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
        ChronoResult::None => Err(serde::de::Error::custom(std::format!(
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
#[cfg_attr(tarpaulin, coverage(off))]
#[cfg_attr(tarpaulin, tarpaulin::skip)]
pub(crate) fn is_unknown_list_directive(directive: &BulkListDirective) -> bool {
    std::matches!(directive, BulkListDirective::Unknown)
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
        Err(error) => Err(serde::de::Error::custom(std::format!(
            "Couldn't deserialize '{value}' due to: {error:?}"
        ))),
    }
}

#[doc(hidden)]
#[allow(dead_code)]
#[cfg_attr(tarpaulin, coverage(off))]
#[cfg_attr(tarpaulin, tarpaulin::skip)]
/// Compare the supplied string values for equality without
/// regard for character casing
pub(crate) fn caseless_eq<StringLike: AsRef<str>>(left: StringLike, right: StringLike) -> bool {
    left.as_ref().eq_ignore_ascii_case(right.as_ref())
}

#[doc(hidden)]
#[cfg_attr(tarpaulin, coverage(off))]
#[cfg_attr(tarpaulin, tarpaulin::skip)]
#[cfg(any(test, tarpaulin, feature = "ci"))]
/// Serializer implementation for enabling `serde`
/// to properly cast `chrono::DateTime<Utc>`s back
/// to the weirdly formatted timestamps returned by
/// the BriteVerify API.
pub fn serialize_timestamp<S: serde::Serializer>(
    value: &DateTime<Utc>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let timestamp: String = std::format!("{}", value.format("%m-%d-%Y %I:%M %P"));
    serializer.serialize_str(&timestamp)
}

// </editor-fold desc="// Utility Functions ...">

// <editor-fold desc="// Extension Traits ...">

pub(crate) trait ExtensibleUrl: reqwest::IntoUrl {
    /// Append the supplied "segment" to a URL
    fn append_path<PathSegment: Display>(&self, segment: PathSegment) -> Self;

    /// "Extend" a URL by appending each of the supplied segments
    fn extend_path<Segments>(&self, segments: Segments) -> Self
    where
        Segments: IntoIterator,
        Segments::Item: Display;
}

impl ExtensibleUrl for url::Url {
    fn append_path<PathSegment: Display>(&self, segment: PathSegment) -> Self {
        let mut url = self.clone();

        url.path_segments_mut()
            .map(|mut segments| {
                segments.push(segment.to_string().as_str());
            })
            .unwrap_or(());

        url
    }

    fn extend_path<Segments>(&self, segments: Segments) -> Self
    where
        Segments: IntoIterator,
        Segments::Item: Display,
    {
        let mut url = self.clone();

        url.path_segments_mut()
            .map(|mut parts| {
                parts.extend(segments.into_iter().map(|seg| std::format!("{seg}")));
            })
            .unwrap_or(());

        url
    }
}

// </editor-fold desc="// Extension Traits ...">

// <editor-fold desc="// Test Factory Utilities ...">

#[cfg(test)]
#[doc(hidden)]
/// Utility functions for `briteverify-rs`'s test suite and examples
pub mod test_utils {
    // Third-Party Imports
    use chrono::{DateTime, Utc};
    use rand::{seq::IteratorRandom, Rng};

    #[cfg_attr(tarpaulin, coverage(off))]
    fn _one_week_ago(now: &DateTime<Utc>) -> DateTime<Utc> {
        *now - chrono::Duration::days(7i64)
    }

    #[cfg_attr(tarpaulin, coverage(off))]
    fn _a_few_hours_ago(now: &DateTime<Utc>) -> DateTime<Utc> {
        let offset = rand::thread_rng().gen_range(1i64..=5i64);

        *now - chrono::Duration::hours(offset)
    }

    /// Create a range of DateTime values with the specified interval
    #[cfg_attr(tarpaulin, coverage(off))]
    pub fn datetime_range(
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
        step: chrono::Duration,
    ) -> Vec<DateTime<Utc>> {
        let mut values = Vec::<DateTime<Utc>>::from([*start]);

        let mut last = *values.last().unwrap_or(start);

        while &last < end {
            last += step;
            values.push(last)
        }

        values
    }

    /// Create a range of DateTime values with the specified interval
    #[cfg_attr(tarpaulin, coverage(off))]
    pub fn random_datetime_between(
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
        step: chrono::Duration,
    ) -> DateTime<Utc> {
        let pool = datetime_range(start, end, step);

        loop {
            if let Some(value) = pool.iter().choose(&mut rand::thread_rng()) {
                break *value;
            }
        }
    }

    /// Randomly generate a timestamp from within the past week
    #[cfg_attr(tarpaulin, coverage(off))]
    pub fn within_the_last_week() -> DateTime<Utc> {
        let now = Utc::now();
        let start = _one_week_ago(&now);

        random_datetime_between(&start, &now, chrono::Duration::hours(8))
    }

    /// Randomly generate a timestamp from a few hours in the past
    #[cfg_attr(tarpaulin, coverage(off))]
    pub fn within_the_last_few_hours() -> DateTime<Utc> {
        let now = Utc::now();
        let start = _a_few_hours_ago(&now);

        random_datetime_between(&start, &now, chrono::Duration::minutes(15))
    }
}

// </editor-fold desc="// Test Factory Utilities ...">

// <editor-fold desc="// I/O-Free Tests ...">

#[cfg(test)]
mod tests {

    // Third-Party Dependencies
    use chrono::{Datelike, Timelike};
    use once_cell::sync::OnceCell;
    use pretty_assertions::{assert_eq, assert_str_eq};
    use rstest::{fixture, rstest};
    use serde_assert::{Deserializer, Token};

    // Crate-Level Dependencies
    use super::{ChronoResult, DateTime, Duration, Result, Uri, Utc};

    const TIMESTAMP: &str = "01-11-2023 4:45 pm";
    static RECENT_DATETIMES: OnceCell<Vec<DateTime<Utc>>> = OnceCell::new();

    #[fixture]
    fn recent_datetimes() -> &'static Vec<DateTime<Utc>> {
        RECENT_DATETIMES.get_or_init(|| {
            let start_date = super::within_the_last_week()
                .with_second(0)
                .and_then(|value| value.with_nanosecond(0))
                .unwrap();

            super::datetime_range(&start_date, &Utc::now(), chrono::Duration::minutes(1))
        })
    }

    /// Test that the `float_to_duration` utility
    /// returns a valid `Duration` when the supplied
    /// value is a valid `f64`
    #[rstest]
    fn test_valid_float_to_duration() -> Result<()> {
        let tokens: [Token; 1] = [Token::F64(1.0)];
        let expected: Duration = Duration::from_secs(1);

        let mut deserializer = Deserializer::builder(tokens).self_describing(true).build();
        let result: Duration = super::float_to_duration(&mut deserializer)?;

        Ok(assert_eq!(result, expected))
    }

    /// Test that the `float_to_duration` utility
    /// returns an error `Duration` when the supplied
    /// value cannot be deserialized as an `f64` or
    /// when the deserialized value cannot be converted
    /// to a valid `Duration`
    #[rstest]
    fn test_invalid_float_to_duration() {
        let tokens: [Token; 1] = [Token::F64(-1.0)];

        let mut deserializer = Deserializer::builder(tokens).self_describing(true).build();

        let result = super::float_to_duration(&mut deserializer);

        assert!(result.is_err())
    }

    /// Test that the `duration_to_float` utility
    /// behaves as expected when supplied with a
    /// valid `Duration` and usable `Serializer`
    #[rstest]
    fn test_duration_to_float() {
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
        let tokens: [Token; 1] = [Token::Str("got-em".to_string())];

        let mut deserializer = Deserializer::builder(tokens).self_describing(true).build();

        let result = super::float_to_duration(&mut deserializer);

        Ok(assert!(result.is_err()))
    }

    /// Test that the `empty_string_is_none` utility
    /// returns `None` when the supplied value is either
    /// an empty string or the `None` token
    #[rstest]
    fn test_empty_string_is_none() -> Result<()> {
        let tokens: [[Token; 2]; 2] = [
            [Token::Some, Token::Str("".to_string())],
            [Token::None, Token::None],
        ];

        for token_array in tokens.into_iter() {
            let mut deserializer = Deserializer::builder(token_array)
                .self_describing(true)
                .build();
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
        let tokens: [Token; 2] = [Token::Some, Token::Str("got-em".to_string())];

        let mut deserializer = Deserializer::builder(tokens).self_describing(true).build();
        let result: Option<String> = super::empty_string_is_none(&mut deserializer)?;

        assert!(result.is_some());

        Ok(assert_eq!(result.unwrap(), expected))
    }

    /// Test that the `serialize_uri` utility
    /// behaves as expected
    #[rstest]
    fn test_serialize_uri() {
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
    fn test_deserialize_empty_uri() {
        let tokens: [[Token; 2]; 2] = [
            [Token::Some, Token::Str("".to_string())],
            [Token::None, Token::None],
        ];

        for token_array in tokens.into_iter() {
            let mut deserializer = Deserializer::builder(token_array)
                .self_describing(true)
                .build();
            let result: Result<Option<Uri>, _> = super::deserialize_uri(&mut deserializer);

            assert!(result.is_ok());
            assert!(result.unwrap().is_none());
        }
    }

    /// Test that the `deserialize_uri` utility
    /// returns a valid `Uri` when the supplied
    /// value is a non-empty URI-like string
    #[rstest]
    fn test_deserialize_valid_uri() {
        let src: &str = "https://example.com";
        let expected: Uri = Uri::from_static(src);
        let tokens: [Token; 2] = [Token::Some, Token::Str(src.to_string())];

        let mut deserializer = Deserializer::builder(tokens).self_describing(true).build();
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
    fn test_deserialize_invalid_uri() {
        let tokens: [Token; 2] = [
            Token::Some,
            Token::Str("the most dangerous type of canoes are volcanoes".to_string()),
        ];

        let mut deserializer = Deserializer::builder(tokens).self_describing(true).build();
        let result = super::deserialize_uri(&mut deserializer);

        assert!(result.is_err())
    }

    /// Test that the `deserialize_ext_id` utility
    /// returns a non-empty, unquoted string when
    /// the supplied value is a non-empty scalar
    /// value (e.g. a number, or a string)
    #[rstest]
    #[case::no_id(vec![Token::None], Ok(Option::<String>::None))]
    #[case::null_id(vec![Token::Some, Token::Unit], Ok(Option::<String>::None))]
    #[case::empty_id(vec![Token::Some, Token::Str("  ".to_string())], Ok(Option::<String>::None))]
    #[case::numeric_id(vec![Token::Some, Token::I64(12345)], Ok(Some("12345".to_string())))]
    #[case::string_id(vec![Token::Some, Token::Str("12345".to_string())], Ok(Some("12345".to_string())))]
    #[case::array_id(vec![Token::Some, Token::Seq { len: Some(1) }, Token::Str("12345".to_string()), Token::SeqEnd], Err(None))]
    #[case::object_id(vec![Token::Some, Token::Map { len: Some(1) }, Token::Str("12345".to_string()), Token::I32(12345), Token::MapEnd], Err(None))]
    fn test_deserialize_ext_id(
        #[case] tokens: Vec<Token>,
        #[case] expected: Result<Option<String>, Option<String>>,
    ) {
        let mut deserializer = Deserializer::builder(tokens).self_describing(true).build();
        let result = super::deserialize_ext_id(&mut deserializer);

        match expected {
            Ok(None) => {
                assert!(
                    result
                        .as_ref()
                        .is_ok_and(|external_id| external_id.is_none()),
                    "Expected Ok(None), got: {:#?}",
                    result.as_ref(),
                )
            }
            Ok(Some(expected_id)) => {
                assert!(
                    result
                        .as_ref()
                        .is_ok_and(|external_id| external_id.as_ref().is_some_and(|id| {
                            assert_str_eq!(expected_id.as_str(), id.as_str());
                            true
                        })),
                    "Expected Ok(Some(external_id)), got: {:#?}",
                    result.as_ref(),
                )
            }
            Err(_) => {
                assert!(
                    result.as_ref().is_err(),
                    "Expected Err(serde::de::Error), got: {:#?}",
                    result.as_ref(),
                )
            }
        }
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
    fn test_invalid_bv_timestamp(recent_datetimes: &[DateTime<Utc>]) {
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
    fn test_deserialize_timestamp() {
        let tokens: [Token; 1] = [Token::Str(TIMESTAMP.to_string())];

        let mut deserializer = Deserializer::builder(tokens).self_describing(true).build();
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
    fn test_deserialize_non_timestamp() {
        let tokens: [Token; 1] = [Token::Str(
            "I thought I'd do was I'd pretend I was one of those deaf-mutes".to_string(),
        )];

        let mut deserializer = Deserializer::builder(tokens).self_describing(true).build();
        let result = super::deserialize_timestamp(&mut deserializer);

        assert!(result.is_err());
    }

    /// Test that the `deserialize_maybe_timestamp`
    /// utility behaves as expected when the value
    /// being deserialized is a properly formatted
    /// timestamp string value
    #[rstest]
    fn test_deserialize_some_timestamp() {
        let tokens: [Token; 2] = [Token::Some, Token::Str(TIMESTAMP.to_string())];

        let mut deserializer = Deserializer::builder(tokens).self_describing(true).build();
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
    fn test_deserialize_empty_timestamp() {
        let tokens: [[Token; 2]; 2] = [
            [Token::Some, Token::Str("".to_string())],
            [Token::None, Token::None],
        ];

        for token_array in tokens.into_iter() {
            let mut deserializer = Deserializer::builder(token_array)
                .self_describing(true)
                .build();
            let result = super::deserialize_maybe_timestamp(&mut deserializer);

            assert!(result.is_ok());
            assert!(result.unwrap().is_none());
        }
    }

    /// Test that the `deserialize_maybe_timestamp` utility
    /// returns an error when the value being deserialized
    /// is anything other than a BriteVerify-formatted timestamp
    #[rstest]
    fn test_deserialize_some_non_timestamp() {
        let tokens: [Token; 2] = [
            Token::Some,
            Token::Str("Ostensibly, the ol' razzle dazzle".to_string()),
        ];

        let mut deserializer = Deserializer::builder(tokens).self_describing(true).build();
        let result = super::deserialize_maybe_timestamp(&mut deserializer);

        assert!(result.is_err());
    }

    /// Test that the `deserialize_boolean` utility
    /// returns a valid `bool` when the supplied
    /// value represents a valid `bool` (either
    /// directly or as a string)
    #[rstest]
    #[case::actual_bool([Token::Bool(true)])]
    #[case::boolean_string([Token::Str("true".to_string())])]
    fn test_deserialize_boolean(#[case] tokens: [Token; 1]) {
        let mut deserializer = Deserializer::builder(tokens).self_describing(true).build();
        let result = super::deserialize_boolean(&mut deserializer);

        assert!(
            result.is_ok(),
            "Expected a valid boolean value, got: {:?}",
            result
        );

        assert_eq!(result.unwrap(), true);
    }

    /// Test that the `deserialize_boolean` utility
    /// returns an error when the supplied value
    /// represents something other than a valid `bool`
    #[rstest]
    fn test_deserialize_non_boolean() {
        let tokens: [Token; 1] = [Token::Str(
            "a literal boolean value, you know, like 'true' or maybe 'false'".to_string(),
        )];

        let mut deserializer = Deserializer::builder(tokens).self_describing(true).build();
        let result = super::deserialize_boolean(&mut deserializer);

        assert!(result.is_err())
    }
}

// </editor-fold desc="// I/O-Free Tests ...">

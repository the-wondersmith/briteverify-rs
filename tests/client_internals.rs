#![allow(clippy::unit_arg)]
//! ## Integration Tests For [`BriteVerifyClient`](BriteVerifyClient)'s
//! ## Miscellaneous Internal Utility Methods

// Module Declarations
pub mod utils;

// Standard Library Imports
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc, Mutex,
    },
};

// Third Part Imports
use anyhow::Result;
use http_types::{mime::JSON, StatusCode};
use once_cell::sync::Lazy;
use pretty_assertions::assert_str_eq;
use rstest::{fixture, rstest};
use wiremock::{
    http::{Method as HttpMethod, Url},
    Match, Mock, MockServer, Request, Respond, ResponseTemplate,
};

// Crate-Level Imports
use briteverify_rs::{errors::BriteVerifyClientError, BriteVerifyClient};
use utils::BriteVerifyRequest;

// <editor-fold desc="// Constants ...">

const AUTH_KEY_ERROR: &str =
    r#"{"errors":{"user":"not authorized or over daily test limit for untrusted domains"}}"#;
const RATE_LIMIT_BODY: &str = "wouldn't that bring about chaos?";
static REQUEST_COUNTS: Lazy<Arc<Mutex<HashMap<Url, u8>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

// </editor-fold desc="// Constants ...">

// <editor-fold desc="// Fixtures ...">

#[fixture]
/// An unregistered `Mock` that will respond to any request lacking
/// the proper `Authorization` header with the official response body
/// from the BriteVerify API's published Postman collection / documentation
fn mock_auth_error() -> Mock {
    Mock::given(is_unauthorized_request).respond_with(unauthorized_response)
}

// </editor-fold desc="// Fixtures ...">

// <editor-fold desc="// Utility Functions ...">

// <editor-fold desc="// Request Matchers ...">

/// Check if the supplied request is not authorized
/// [[ref](https://docs.briteverify.com/#8fce7493-92a4-43f0-bd0d-bd2dfdb65bf5)]
pub fn is_unauthorized_request(request: &Request) -> bool {
    !request.has_valid_api_key()
        && [HttpMethod::Get, HttpMethod::Post, HttpMethod::Delete].contains(&request.method)
}

// </editor-fold desc="// Request Matchers ...">

// <editor-fold desc="// Response Generators ...">

/// Return an error response indicating that the supplied API key
/// was either invalid, expired, or otherwise not properly authorized
/// [[ref](https://docs.briteverify.com/#8fce7493-92a4-43f0-bd0d-bd2dfdb65bf5)]
pub fn unauthorized_response(_: &Request) -> ResponseTemplate {
    ResponseTemplate::new(StatusCode::Unauthorized).set_body_raw(AUTH_KEY_ERROR, &JSON.to_string())
}

// </editor-fold desc="// Response Generators ...">

// </editor-fold desc="// Utility Functions ...">

// <editor-fold desc="// Auto-Retry Test Helper ...">

#[derive(Debug)]
struct StatefulRateLimit(pub Arc<AtomicU8>);

impl Match for StatefulRateLimit {
    fn matches(&self, request: &Request) -> bool {
        let url = &request.url;
        let mut count_map = REQUEST_COUNTS.lock().unwrap();

        let call_count = count_map
            .entry(url.clone())
            .and_modify(|count| *count += 1)
            .or_insert(1)
            .to_owned();

        self.0.store(call_count, Ordering::SeqCst);

        url.to_string().ends_with("/auto-retry")
    }
}

impl Respond for StatefulRateLimit {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let call_count = self.0.load(Ordering::SeqCst);

        if call_count < 2u8 {
            ResponseTemplate::new(StatusCode::TooManyRequests).insert_header("retry-after", "1")
        } else {
            REQUEST_COUNTS
                .lock()
                .unwrap()
                .insert(request.url.clone(), 0);

            ResponseTemplate::new(StatusCode::Ok).set_body_raw(RATE_LIMIT_BODY, &JSON.to_string())
        }
    }
}

// </editor-fold desc="// Auto-Retry Test Helper ...">

// <editor-fold desc="// Integration Tests ...">

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`BriteVerifyClient`](BriteVerifyClient)
/// behaves as expected when a request is met with an authorization
/// error response (per the official BriteVerify API Postman collection)
async fn errors_with_bad_api_keys(#[from(mock_auth_error)] mock: Mock) -> Result<()> {
    let (client, server) =
        utils::client_and_server(Some(r#"what's dwarven for "friend" again?"#), None).await;

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    let url = format!("{}://{}/auth-check", "http", server.address());
    let response = client.build_and_send(client.get(url)).await;

    Ok(assert!(response
        .expect_err("Client method was expected to return an error but did not")
        .to_string()
        .contains("Invalid or unauthorized BriteVerify API key")))
}

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`BriteVerifyClient`](BriteVerifyClient)
/// behaves as expected when auto-retry is enabled and a
/// request cannot be cloned
/// ___
/// **NOTE:** This can't currently happen "in the real world", as no
/// client method will create a request with an unclonable body. This
/// test currently exists exclusively because there is a branch in the
/// client's `_build_and_send` method that handles the case.
/// ___
async fn errors_with_unclonable_requests() -> Result<()> {
    // Create a `BriteVerifyClient` instance with auto-retry enabled
    let client = BriteVerifyClient::builder()
        .api_key("fear is the true enemy, the only enemy")
        .retry_enabled(true)
        .build()?;

    // Per the reqwest documentation,`None` is only returned
    // by `Request::try_clone` in the case that the request's
    // body is unclonable.
    // [[ref](https://docs.rs/reqwest/latest/reqwest/struct.RequestBuilder.html#method.try_clone)]
    //
    // This example is taken directly from the associated documentation
    // [[ref](https://docs.rs/reqwest/latest/reqwest/struct.Body.html#method.wrap_stream)]
    // as a way to create a request body that is unclonable, and
    // will therefore take the branch in `_build_and_send` that
    // handles that specific case.
    let body_stream = futures_util::stream::iter(Vec::<Result<_, std::io::Error>>::from([
        Ok("flair"),
        Ok("is"),
        Ok("what"),
        Ok("marks"),
        Ok("the"),
        Ok("difference"),
        Ok("between"),
        Ok("artistry"),
        Ok("and"),
        Ok("mere"),
        Ok("competence"),
    ]));

    let request_body = reqwest::Body::wrap_stream(body_stream);

    let response = client
        .build_and_send(client.post("https://example.com").body(request_body))
        .await;

    Ok(assert!(
        response
            .as_ref()
            .is_err_and(|error| matches!(error, BriteVerifyClientError::UnclonableRequest)),
        "Expected Err(BriteVerifyClientError), got: {:#?}",
        response
    ))
}

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`BriteVerifyClient`](BriteVerifyClient)
/// behaves as expected when auto-retry is enabled and the
/// BriteVerify API responds with a request with an error
/// due to rate limit exhaustion
async fn handles_rate_limit_responses() -> Result<()> {
    let server = MockServer::start().await;

    // Create a `BriteVerifyClient` instance with auto-retry enabled
    let client = utils::client_for_server(&server, Some("computer, belay that order"), true);

    let call_count = Arc::new(AtomicU8::from(0u8));

    #[allow(unused_variables)]
    let mock = Mock::given(StatefulRateLimit(Arc::clone(&call_count)))
        .respond_with(StatefulRateLimit(Arc::clone(&call_count)))
        .mount_as_scoped(&server)
        .await;

    let url = format!("{}://{}/auto-retry", "http", server.address()).parse::<Url>()?;

    let response = client.build_and_send(client.get(url)).await;

    assert!(
        response.as_ref().is_ok(),
        "Expected Ok(response), got: {:#?}",
        response
    );

    let response = response.unwrap();

    assert_eq!(
        mock.received_requests().await.len() as u64,
        call_count.load(Ordering::SeqCst) as u64,
    );

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    Ok(assert_str_eq!(
        RATE_LIMIT_BODY,
        response.text().await.unwrap_or("error".to_string())
    ))
}

// </editor-fold desc="// Integration Tests ...">

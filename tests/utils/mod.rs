//! ## Test Suite Utilities

// Module Declarations
pub mod v1_mock_data;
pub mod v3_mock_data;

// Standard Library Imports
use std::ops::Deref;

// Third Part Imports
use anyhow::Result;
use http::uri::Scheme;
use http_types::{
    headers::{AUTHORIZATION, CONTENT_TYPE},
    mime::JSON,
    Method as HttpMethod, StatusCode,
};
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{from_str as json_from_str, Value};
use uuid::Uuid;
use wiremock::{Match, MockServer, Request, Respond, ResponseTemplate};

// Crate-Level Imports
use briteverify_rs::BriteVerifyClient;

// <editor-fold desc="// Constants ...">

pub static TEST_API_KEY: Lazy<String> = Lazy::new(|| Uuid::new_v4().to_string());
pub static API_KEY_HEADER: Lazy<String> = Lazy::new(|| format!("ApiKey: {}", TEST_API_KEY.deref()));

// <editor-fold desc="// Endpoint URL Regexes ...">

// Common "Parts"
const PAGE: &str = "(?<page>page=[0-9]+&?)";
const DATE: &str = "(?<date>(date=[0-9]{4}(-[0-9]{2}){2})&?)";
const STATE: &str = "(?<state>state=[a-z_-]+&?)";
const EXT_ID: &str = r#"(?:accounts/[\S]+/)"#;
const LIST_ID: &str = r#"[0-9a-fA-F\-]{36}"#;

static BASE_LISTS: Lazy<String> = Lazy::new(|| format!(r#"/api/v3/{EXT_ID}?lists"#));

// v1 "Single Transaction" Endpoints
pub static V1_VERIFY: Lazy<Regex> =
    Lazy::new(|| r#"(?i:/api/(?:public/)?v1/fullverify/?$)"#.parse::<Regex>().unwrap());

// v3 "Bulk" Endpoints
pub static V3_LISTS: Lazy<Regex> = Lazy::new(|| {
    let query = format!(r#"(?<query>({PAGE}|{STATE}|{DATE}){{1,3}})"#);

    format!(r#"(?i:{}(\?{query})?/?$)"#, BASE_LISTS.deref())
        .parse::<Regex>()
        .unwrap()
});
pub static V3_LIST_STATE: Lazy<Regex> = Lazy::new(|| {
    format!(r#"(?i:{}/{LIST_ID}/?$)"#, BASE_LISTS.deref())
        .parse::<Regex>()
        .unwrap()
});
pub static V3_LIST_RESULTS: Lazy<Regex> = Lazy::new(|| {
    format!(r#"(?i:{}/{LIST_ID}/export/[0-9]+/?$)"#, BASE_LISTS.deref())
        .parse::<Regex>()
        .unwrap()
});

// </editor-fold desc="// Endpoint URL Regexes ...">

// </editor-fold desc="// Constants ...">

// <editor-fold desc="// Utility Functions ...">

/// Create a `BriteVerifyClient` instance pre-configured for use
/// with the supplied `wiremock::MockServer` instance
pub fn client_for_server(
    server: &MockServer,
    api_key: Option<&str>,
    enable_retry: bool,
) -> BriteVerifyClient {
    let server_addr = *server.address();

    BriteVerifyClient::builder()
        .https_only(false)
        .retry_enabled(enable_retry)
        .set_v1_url_scheme(Scheme::HTTP)
        .set_v3_url_scheme(Scheme::HTTP)
        .set_v1_url_port(server_addr.port())
        .set_v3_url_port(server_addr.port())
        .resolve_v1_url_to(server_addr)
        .resolve_v3_url_to(server_addr)
        .api_key(api_key.unwrap_or(TEST_API_KEY.deref()))
        .build()
        .unwrap()
}

/// Create a `BriteVerifyClient` instance pre-configured for use
/// with the supplied `wiremock::MockServer` instance
pub async fn client_and_server(
    api_key: Option<&str>,
    enable_retry: Option<bool>,
) -> (BriteVerifyClient, MockServer) {
    let server = MockServer::start().await;
    let client = client_for_server(&server, api_key, enable_retry.unwrap_or(false));

    (client, server)
}

/// Create a [`ResponseTemplate`](ResponseTemplate) from the supplied JSON blob
pub fn official_response(from: MockRequestResponse) -> ResponseTemplate {
    ResponseTemplate::new(StatusCode::Ok).set_body_raw(from.response, &JSON.to_string())
}

// </editor-fold desc="// Utility Functions ...">

// <editor-fold desc="// `wiremock` Extension Traits ...">

pub trait BriteVerifyRequest {
    /// Determine if a request's `content-type` header
    /// indicates that its body's MIME-type is JSON-serialized
    /// text
    fn has_json_content(&self) -> bool;

    /// Determine if a request has the proper authorization header set
    fn has_valid_api_key(&self) -> bool;

    /// Determine if a request's body content
    /// matches the supplied `serde_json::Value`
    fn body_json_matches_value(&self, value: &Value) -> bool;

    /// Check if a request is a valid BriteVerify API
    /// "single-transaction / real time" verification request
    fn is_v1_verification_request(&self) -> bool;
}

impl BriteVerifyRequest for Request {
    fn has_json_content(&self) -> bool {
        self.headers
            .get(&CONTENT_TYPE)
            .map_or(false, |value| value.as_str() == JSON.to_string())
    }

    fn has_valid_api_key(&self) -> bool {
        self.headers
            .get(&AUTHORIZATION)
            .map_or(false, |value| value.as_str() == API_KEY_HEADER.deref())
    }

    fn body_json_matches_value(&self, value: &Value) -> bool {
        self.body_json::<Value>().is_ok_and(|body| &body == value)
    }

    fn is_v1_verification_request(&self) -> bool {
        self.method == HttpMethod::Post
            && V1_VERIFY.is_match(self.url.as_str())
            && self.has_json_content()
            && self.has_valid_api_key()
    }
}

// </editor-fold desc="// `wiremock` Extension Traits ...">

// <editor-fold desc="// Mock Request/Response Template ...">

#[derive(Copy, Clone, Hash, Debug)]
pub struct MockRequestResponse {
    pub request: &'static str,
    pub response: &'static str,
}

impl MockRequestResponse {
    /// Extract the specified key from the supplied [`Value`](Value)
    pub fn extract(key: &str, from: Value) -> Option<String> {
        from.as_object()
            .and_then(|obj| obj.get(key))
            .and_then(|value| value.as_str())
            .map(|value| value.to_string())
    }

    /// Extract the specified key from the request body
    pub fn extract_from_request(&self, key: &str) -> Option<String> {
        self.request_body_json::<Value>()
            .map_or(None, |value| MockRequestResponse::extract(key, value))
    }

    /// Extract the specified key from the request body
    pub fn extract_from_response(&self, key: &str) -> Option<String> {
        self.response_body_json::<Value>()
            .map_or(None, |value| MockRequestResponse::extract(key, value))
    }

    pub fn request_body_json<'de, T: serde::Deserialize<'de>>(&self) -> Result<T> {
        Ok(json_from_str(self.request)?)
    }
    pub fn response_body_json<'de, T: serde::Deserialize<'de>>(&self) -> Result<T> {
        Ok(json_from_str(self.response)?)
    }
}

#[allow(unused_variables)]
impl Match for MockRequestResponse {
    fn matches(&self, request: &Request) -> bool {
        todo!()
    }
}

#[allow(unused_variables)]
impl Respond for MockRequestResponse {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        todo!()
    }
}

// </editor-fold desc="// Mock Request/Response Template ...">

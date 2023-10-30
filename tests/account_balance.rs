//! ## Integration Tests For [`BriteVerifyClient`](briteverify_rs::BriteVerifyClient)'s
//! ## Account Balance Methods

// Module Declarations
pub mod utils;

// Third Part Imports
use chrono::{Datelike, Timelike};
use http_types::Method as HttpMethod;
use once_cell::sync::Lazy;
use regex::Regex;
use rstest::{fixture, rstest};
use wiremock::{Mock, Request, ResponseTemplate};

// Crate-Level Imports
use utils::{official_response, BriteVerifyRequest, MockRequestResponse};

// <editor-fold desc="// Constants ...">

const OFFICIAL_ACCOUNT_BALANCE: MockRequestResponse = MockRequestResponse {
    request: "",
    response: r#"{
  "credits": 2165,
  "credits_in_reserve": 500,
  "recorded_on": "2021-07-27T21:10:10.000+0000"
}"#,
};

static V3_CREDITS: Lazy<Regex> = Lazy::new(|| Regex::new("/api/v3/accounts/credits/?$").unwrap());

// </editor-fold desc="// Constants ...">

// <editor-fold desc="// Fixtures ...">

#[fixture]
/// An unregistered `Mock` that will respond to valid account
/// balance requests with the official response body from the
/// BriteVerify API's published Postman collection / documentation
fn mock_account_balance() -> Mock {
    Mock::given(is_valid_account_balance_request).respond_with(account_balance_response)
}

// </editor-fold desc="// Fixtures ...">

// <editor-fold desc="// Utility Functions ...">

/// Check if the supplied request is a valid account credit balance request
/// [[ref](https://docs.briteverify.com/#07beceb3-2961-4d5b-93a4-9cfeb30f42fa)]
fn is_valid_account_balance_request(request: &Request) -> bool {
    request.method == HttpMethod::Get
        && V3_CREDITS.is_match(request.url.as_str())
        && request.has_valid_api_key()
}

/// Return an account credit balance response
/// [[ref](https://docs.briteverify.com/#07beceb3-2961-4d5b-93a4-9cfeb30f42fa)]
fn account_balance_response(_: &Request) -> ResponseTemplate {
    official_response(OFFICIAL_ACCOUNT_BALANCE)
}

// </editor-fold desc="// Utility Functions ...">

// <editor-fold desc="// Integration Tests ...">

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`current_credits`](briteverify_rs::BriteVerifyClient::current_credits)
/// method extracts and returns the expected field value from a returned
/// [`AccountCreditBalance`](briteverify_rs::types::AccountCreditBalance) response
async fn gets_current_credits(#[from(mock_account_balance)] mock: Mock) {
    let (client, server) = utils::client_and_server(None, None).await;

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    let response = client.current_credits().await;

    assert!(
        response.as_ref().is_ok_and(|credits| credits == &2165u32),
        "Expected Ok(2165), got: {:#?}",
        response
    );
}

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`current_credits_in_reserve`](briteverify_rs::BriteVerifyClient::current_credits_in_reserve)
/// method extracts and returns the expected field value from a returned
/// [`AccountCreditBalance`](briteverify_rs::types::AccountCreditBalance) response
async fn gets_credits_in_reserve(#[from(mock_account_balance)] mock: Mock) {
    let (client, server) = utils::client_and_server(None, None).await;

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    let response = client.current_credits_in_reserve().await;

    assert!(
        response.as_ref().is_ok_and(|reserve| reserve == &500u32),
        "Expected Ok(500), got: {:#?}",
        response
    );
}

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`get_account_balance`](briteverify_rs::BriteVerifyClient::get_account_balance)
/// method sends the expected request and properly handles the returned
/// response (per the official BriteVerify API Postman collection)
async fn gets_full_account_balances(#[from(mock_account_balance)] mock: Mock) {
    let (client, server) = utils::client_and_server(None, None).await;

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    let response = client.get_account_balance().await;

    assert!(
        response.as_ref().is_ok_and(|balance| {
            let (date, time) = (balance.recorded_on.date_naive(), balance.recorded_on.time());

            date.year() == 2021
                && date.month() == 7
                && date.day() == 27
                && time.hour() == 21
                && time.minute() == 10
                && time.second() == 10
        }),
        "Expected Ok(balance) w/ date of '2021-07-27T21:10:10', got: {:#?}",
        response,
    );
}

// </editor-fold desc="// Integration Tests ...">

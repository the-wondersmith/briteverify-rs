#![allow(clippy::unit_arg)]
//! ## Integration Tests For [`BriteVerifyClient`](briteverify_rs::BriteVerifyClient)'s
//! ## Single-Transaction / "Real Time" Verification Methods

// Module Declarations
pub mod utils;

// Third Part Imports
use anyhow::Result;

use pretty_assertions::{assert_eq, assert_str_eq};
use rstest::{fixture, rstest};
use serde_json::Value;
use wiremock::{Mock, Request, ResponseTemplate};

// Crate-Level Imports
use briteverify_rs::{errors::BriteVerifyClientError, types};
use utils::{official_response, v1_mock_data as mock_data, BriteVerifyRequest};

// <editor-fold desc="// Utility Functions ...">

// <editor-fold desc="// Request Matchers ...">

/// Check if the body of the supplied request matches the official
/// request body for a "full" validation request whose supplied data
/// is in fact "valid" from the BriteVerify API's published Postman
/// collection / documentation
/// [[ref](https://docs.briteverify.com/#a7246384-e91e-48a9-8aed-7b71cb74dd42)]
pub fn is_valid_full_request(request: &Request) -> bool {
    request.is_v1_verification_request()
        && match (
            request.body_json::<types::VerificationRequest>(),
            mock_data::OFFICIAL_VALID_FULL_VERIFY.request_body_json::<types::VerificationRequest>(),
        ) {
            (Ok(body), Ok(official)) => body == official,
            _ => false,
        }
}

/// Check if the body of the supplied request matches the official
/// request body for a "full" validation request whose supplied data
/// is "invalid" from the BriteVerify API's published Postman
/// collection / documentation
/// [[ref](https://docs.briteverify.com/#a7246384-e91e-48a9-8aed-7b71cb74dd42)]
pub fn is_invalid_full_request(request: &Request) -> bool {
    request.is_v1_verification_request()
        && match (
            request.body_json::<types::VerificationRequest>(),
            mock_data::OFFICIAL_INVALID_FULL_VERIFY
                .request_body_json::<types::VerificationRequest>(),
        ) {
            (Ok(body), Ok(official)) => body == official,
            _ => false,
        }
}

/// Check if the body of the supplied request matches the official
/// request body for an "email-only" validation request whose supplied
/// email address is in fact "valid" from the BriteVerify API's published
/// Postman collection / documentation
/// [[ref](https://docs.briteverify.com/#e5dd413c-6411-4078-8b4c-0e787f6a9325)]
pub fn is_valid_email_request(request: &Request) -> bool {
    request.is_v1_verification_request()
        && match (
            request.body_json::<Value>(),
            mock_data::OFFICIAL_EMAIL_VALID.request_body_json::<Value>(),
        ) {
            (Ok(body), Ok(official)) => body == official,
            _ => false,
        }
}

/// Check if the body of the supplied request matches the official
/// request body for an "email-only" validation request whose supplied
/// email address is "invalid" from the BriteVerify API's published
/// Postman collection / documentation
/// [[ref](https://docs.briteverify.com/#e5dd413c-6411-4078-8b4c-0e787f6a9325)]
pub fn is_invalid_email_request(request: &Request) -> bool {
    request.is_v1_verification_request()
        && match (
            request.body_json::<Value>(),
            mock_data::OFFICIAL_EMAIL_INVALID.request_body_json::<Value>(),
        ) {
            (Ok(body), Ok(official)) => body == official,
            _ => false,
        }
}

/// Check if the body of the supplied request matches the official
/// request body for an "email-only" validation request whose supplied
/// email address is "disposable" from the BriteVerify API's published
/// Postman collection / documentation
/// [[ref](https://docs.briteverify.com/#e5dd413c-6411-4078-8b4c-0e787f6a9325)]
pub fn is_disposable_email_request(request: &Request) -> bool {
    request.is_v1_verification_request()
        && match (
            request.body_json::<Value>(),
            mock_data::OFFICIAL_EMAIL_DISPOSABLE.request_body_json::<Value>(),
        ) {
            (Ok(body), Ok(official)) => body == official,
            _ => false,
        }
}

/// Check if the body of the supplied request matches the official
/// request body for an "phone-only" validation request whose supplied
/// phone number is in fact "valid" from the BriteVerify API's published
/// Postman collection / documentation
/// [[ref](https://docs.briteverify.com/#86e335f4-d1b2-4902-9051-4506a48a6b94)]
pub fn is_valid_phone_request(request: &Request) -> bool {
    request.is_v1_verification_request()
        && match (
            request.body_json::<Value>(),
            mock_data::OFFICIAL_PHONE_VALID.request_body_json::<Value>(),
        ) {
            (Ok(body), Ok(official)) => body == official,
            _ => false,
        }
}

/// Check if the body of the supplied request matches the official
/// request body for a "phone-only" validation request whose supplied
/// phone number is "invalid" from the BriteVerify API's published
/// Postman collection / documentation
/// [[ref](https://docs.briteverify.com/#86e335f4-d1b2-4902-9051-4506a48a6b94)]
pub fn is_invalid_phone_request(request: &Request) -> bool {
    request.is_v1_verification_request()
        && match (
            request.body_json::<Value>(),
            mock_data::OFFICIAL_PHONE_INVALID.request_body_json::<Value>(),
        ) {
            (Ok(body), Ok(official)) => body == official,
            _ => false,
        }
}

/// Check if the body of the supplied request matches the official
/// request body for an "address-only" validation request whose supplied
/// street address is in fact "valid" from the BriteVerify API's published
/// Postman collection / documentation
/// [[ref](https://docs.briteverify.com/#f588d8d3-8250-4a8a-9e58-f89c81af6bed)]
pub fn is_valid_address_request(request: &Request) -> bool {
    request.is_v1_verification_request()
        && match (
            request.body_json::<types::VerificationRequest>(),
            mock_data::OFFICIAL_ADDRESS_VALID.request_body_json::<types::VerificationRequest>(),
        ) {
            (Ok(body), Ok(official)) => body == official,
            _ => false,
        }
}

/// Check if the body of the supplied request matches the official
/// request body for an "address-only" validation request whose supplied
/// street address requires "correction" from the BriteVerify API's published
/// Postman collection / documentation
/// [[ref](https://docs.briteverify.com/#f588d8d3-8250-4a8a-9e58-f89c81af6bed)]
pub fn is_corrected_address_request(request: &Request) -> bool {
    request.is_v1_verification_request()
        && match (
            request.body_json::<types::VerificationRequest>(),
            mock_data::OFFICIAL_ADDRESS_CORRECTED.request_body_json::<types::VerificationRequest>(),
        ) {
            (Ok(body), Ok(official)) => body == official,
            _ => false,
        }
}

/// Check if the body of the supplied request matches the official
/// request body for an "address-only" validation request whose supplied
/// street address is missing a suite number from the BriteVerify API's
/// published Postman collection / documentation
/// [[ref](https://docs.briteverify.com/#f588d8d3-8250-4a8a-9e58-f89c81af6bed)]
pub fn is_missing_suite_address_request(request: &Request) -> bool {
    request.is_v1_verification_request()
        && match (
            request.body_json::<types::VerificationRequest>(),
            mock_data::OFFICIAL_ADDRESS_MISSING_SUITE
                .request_body_json::<types::VerificationRequest>(),
        ) {
            (Ok(body), Ok(official)) => body == official,
            _ => false,
        }
}

/// Check if the body of the supplied request matches the official
/// request body for an "address-only" validation request whose supplied
/// street address contains an unknown / non-existent street name from the
/// BriteVerify API's published Postman collection / documentation
/// [[ref](https://docs.briteverify.com/#f588d8d3-8250-4a8a-9e58-f89c81af6bed)]
pub fn is_unknown_street_address_request(request: &Request) -> bool {
    request.is_v1_verification_request()
        && match (
            request.body_json::<types::VerificationRequest>(),
            mock_data::OFFICIAL_ADDRESS_UNKNOWN_STREET
                .request_body_json::<types::VerificationRequest>(),
        ) {
            (Ok(body), Ok(official)) => body == official,
            _ => false,
        }
}

// </editor-fold desc="// Request Matchers ...">

// <editor-fold desc="// Response Generators ...">

/// Return a "valid" verification result for a "complete" set of verifiable values
/// [[ref](https://docs.briteverify.com/#a7246384-e91e-48a9-8aed-7b71cb74dd42)]
pub fn valid_full_response(_: &Request) -> ResponseTemplate {
    official_response(mock_data::OFFICIAL_VALID_FULL_VERIFY)
}

/// Return an "invalid" verification result for a "complete" set of verifiable values
/// [[ref](https://docs.briteverify.com/#a7246384-e91e-48a9-8aed-7b71cb74dd42)]
pub fn invalid_full_response(_: &Request) -> ResponseTemplate {
    official_response(mock_data::OFFICIAL_INVALID_FULL_VERIFY)
}

/// Return an email-only verification result for a "valid" email address
/// [[ref](https://docs.briteverify.com/#e5dd413c-6411-4078-8b4c-0e787f6a9325)]
pub fn valid_email_response(_: &Request) -> ResponseTemplate {
    official_response(mock_data::OFFICIAL_EMAIL_VALID)
}

/// Return an email-only verification result for an "invalid" email address
/// [[ref](https://docs.briteverify.com/#e5dd413c-6411-4078-8b4c-0e787f6a9325)]
pub fn invalid_email_response(_: &Request) -> ResponseTemplate {
    official_response(mock_data::OFFICIAL_EMAIL_INVALID)
}

/// Return an email-only verification result for a "disposable" email address
/// [[ref](https://docs.briteverify.com/#e5dd413c-6411-4078-8b4c-0e787f6a9325)]
pub fn disposable_email_response(_: &Request) -> ResponseTemplate {
    official_response(mock_data::OFFICIAL_EMAIL_DISPOSABLE)
}

/// Return a phone-only verification result for a "valid" phone number
/// [[ref](https://docs.briteverify.com/#86e335f4-d1b2-4902-9051-4506a48a6b94)]
pub fn valid_phone_response(_: &Request) -> ResponseTemplate {
    official_response(mock_data::OFFICIAL_PHONE_VALID)
}

/// Return a phone-only verification result for an "invalid" phone number
/// [[ref](https://docs.briteverify.com/#86e335f4-d1b2-4902-9051-4506a48a6b94)]
pub fn invalid_phone_response(_: &Request) -> ResponseTemplate {
    official_response(mock_data::OFFICIAL_PHONE_INVALID)
}

/// Return an address-only verification result for a "valid" street address
/// [[ref](https://docs.briteverify.com/#f588d8d3-8250-4a8a-9e58-f89c81af6bed)]
pub fn valid_address_response(_: &Request) -> ResponseTemplate {
    official_response(mock_data::OFFICIAL_ADDRESS_VALID)
}

/// Return an address-only verification result for a "corrected" street address
/// [[ref](https://docs.briteverify.com/#f588d8d3-8250-4a8a-9e58-f89c81af6bed)]
pub fn corrected_address_response(_: &Request) -> ResponseTemplate {
    official_response(mock_data::OFFICIAL_ADDRESS_CORRECTED)
}

/// Return an address-only verification result for a street address that
/// was supplied with a missing suite number in its original request
/// [[ref](https://docs.briteverify.com/#f588d8d3-8250-4a8a-9e58-f89c81af6bed)]
pub fn missing_suite_address_response(_: &Request) -> ResponseTemplate {
    official_response(mock_data::OFFICIAL_ADDRESS_MISSING_SUITE)
}

/// Return an address-only verification result for a street address that
/// was supplied with an unknown/non-existent street name in its original request
/// [[ref](https://docs.briteverify.com/#f588d8d3-8250-4a8a-9e58-f89c81af6bed)]
pub fn unknown_street_address_response(_: &Request) -> ResponseTemplate {
    official_response(mock_data::OFFICIAL_ADDRESS_UNKNOWN_STREET)
}

// </editor-fold desc="// Response Generators ...">

// </editor-fold desc="// Utility Functions ...">

// <editor-fold desc="// Fixtures ...">

#[fixture]
/// An unregistered `Mock` that will respond to single-transaction
/// "full" verification requests with the official response body
/// from the BriteVerify API's published Postman collection / documentation
fn mock_valid_full() -> Mock {
    Mock::given(is_valid_full_request).respond_with(valid_full_response)
}

#[fixture]
/// An unregistered `Mock` that will respond to single-transaction
/// "full" verification requests with the official response body
/// from the BriteVerify API's published Postman collection / documentation
fn mock_invalid_full() -> Mock {
    Mock::given(is_invalid_full_request).respond_with(invalid_full_response)
}

#[fixture]
/// An unregistered `Mock` that will respond to single-transaction
/// "email-only" verification requests with the official response body
/// for a "valid" email address from the BriteVerify API's published
/// Postman collection / documentation
fn mock_valid_email() -> Mock {
    Mock::given(is_valid_email_request).respond_with(valid_email_response)
}

#[fixture]
/// An unregistered `Mock` that will respond to single-transaction
/// "email-only" verification requests with the official response body
/// for an "invalid" email address from the BriteVerify API's published
/// Postman collection / documentation
fn mock_invalid_email() -> Mock {
    Mock::given(is_invalid_email_request).respond_with(invalid_email_response)
}

#[fixture]
/// An unregistered `Mock` that will respond to single-transaction
/// "email-only" verification requests with the official response body
/// for a "disposable" email address from the BriteVerify API's published
/// Postman collection / documentation
fn mock_disposable_email() -> Mock {
    Mock::given(is_disposable_email_request).respond_with(disposable_email_response)
}

#[fixture]
/// An unregistered `Mock` that will respond to single-transaction
/// "phone-only" verification requests with the official response body
/// for a "valid" phone number from the BriteVerify API's published
/// Postman collection / documentation
fn mock_valid_phone() -> Mock {
    Mock::given(is_valid_phone_request).respond_with(valid_phone_response)
}

#[fixture]
/// An unregistered `Mock` that will respond to single-transaction
/// "phone-only" verification requests with the official response body
/// for an "invalid" phone number from the BriteVerify API's published
/// Postman collection / documentation
fn mock_invalid_phone() -> Mock {
    Mock::given(is_invalid_phone_request).respond_with(invalid_phone_response)
}

#[fixture]
/// An unregistered `Mock` that will respond to single-transaction
/// "address-only" verification requests with the official response body
/// for a "valid" street address from the BriteVerify API's published
/// Postman collection / documentation
fn mock_valid_address() -> Mock {
    Mock::given(is_valid_address_request).respond_with(valid_address_response)
}

#[fixture]
/// An unregistered `Mock` that will respond to single-transaction
/// "address-only" verification requests with the official response body
/// for a "corrected" street address from the BriteVerify API's published
/// Postman collection / documentation
fn mock_corrected_address() -> Mock {
    Mock::given(is_corrected_address_request).respond_with(corrected_address_response)
}

#[fixture]
/// An unregistered `Mock` that will respond to single-transaction
/// "address-only" verification requests with the official response body
/// for a street address that is missing a suite number from the BriteVerify
/// API's published Postman collection / documentation
fn mock_missing_suite_address() -> Mock {
    Mock::given(is_missing_suite_address_request).respond_with(missing_suite_address_response)
}

#[fixture]
/// An unregistered `Mock` that will respond to single-transaction
/// "address-only" verification requests with the official response body
/// for a street address containing an unknown/non-existent street name
/// from the BriteVerify API's published Postman collection / documentation
fn mock_unknown_street_address() -> Mock {
    Mock::given(is_unknown_street_address_request).respond_with(unknown_street_address_response)
}

// </editor-fold desc="// Fixtures ...">

// <editor-fold desc="// Integration Tests ...">

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`verify_contact`](briteverify_rs::BriteVerifyClient::verify_contact)
/// method propagates non-200 responses that aren't otherwise handled as errors.
async fn errors_on_non_200_verifications() {
    let (client, _) = utils::client_and_server(None, None).await;

    let response = client.verify_email("test@example.com").await;

    assert!(
        response
            .as_ref()
            .is_err_and(|error| matches!(error, BriteVerifyClientError::UnusableResponse(_))),
        "Expected Err(BriteVerifyClientError::UnusableResponse(_)), got: {:#?}",
        response
    )
}

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`verify_contact`](briteverify_rs::BriteVerifyClient::verify_contact)
/// method sends the expected request and properly handles the returned
/// response (per the official BriteVerify API Postman collection)
async fn handles_valid_full_verifications(#[from(mock_valid_full)] mock: Mock) -> Result<()> {
    let (client, server) = utils::client_and_server(None, None).await;

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    let data =
        mock_data::OFFICIAL_VALID_FULL_VERIFY.request_body_json::<types::VerificationRequest>()?;

    let (email_data, phone_data, address_data) = (
        data.email.as_ref().unwrap().as_str(),
        data.phone.as_ref().unwrap().as_str(),
        data.address.as_ref().unwrap(),
    );

    let response = client
        .verify_contact(
            email_data,
            phone_data,
            address_data.address1.as_str(),
            address_data.address2.as_ref(),
            address_data.city.as_str(),
            address_data.state.as_str(),
            address_data.zip.as_str(),
        )
        .await?;

    let (resp_email, resp_phone, resp_address) = (
        response.email.as_ref().unwrap(),
        response.phone.as_ref().unwrap(),
        response.address.as_ref().unwrap(),
    );

    // Email address assertions
    assert!(!resp_email.disposable);
    assert!(resp_email.role_address);
    assert_str_eq!(email_data, resp_email.address.as_str());
    assert_eq!(resp_email.status, types::VerificationStatus::Valid);

    // Phone number assertions
    assert_str_eq!(phone_data, resp_phone.number);
    assert_eq!(resp_phone.status, types::VerificationStatus::Valid);
    assert_str_eq!(
        resp_phone.service_type.as_ref().map_or("", String::as_str),
        "land"
    );

    // Street address assertions
    assert!(address_data.address2.is_none());
    assert_str_eq!(address_data.city.as_str(), resp_address.city);
    assert_str_eq!(address_data.state.as_str(), resp_address.state);
    assert!(resp_address.zip.starts_with(address_data.zip.as_str()));
    assert_eq!(resp_address.status, types::VerificationStatus::Valid);
    Ok(assert!(
        !resp_address.corrected || address_data.address1.as_str() != resp_address.address1
    ))
}

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`verify_contact`](briteverify_rs::BriteVerifyClient::verify_contact)
/// method sends the expected request and properly handles the returned
/// response (per the official BriteVerify API Postman collection)
async fn handles_invalid_full_verifications(#[from(mock_invalid_full)] mock: Mock) -> Result<()> {
    let (client, server) = utils::client_and_server(None, None).await;

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    let data = mock_data::OFFICIAL_INVALID_FULL_VERIFY
        .request_body_json::<types::VerificationRequest>()?;

    let (email_data, phone_data, address_data) = (
        data.email.as_ref().unwrap().as_str(),
        data.phone.as_ref().unwrap().as_str(),
        data.address.as_ref().unwrap(),
    );

    let response = client
        .verify_contact(
            email_data,
            phone_data,
            address_data.address1.as_str(),
            address_data.address2.as_ref(),
            address_data.city.as_str(),
            address_data.state.as_str(),
            address_data.zip.as_str(),
        )
        .await?;

    let (resp_email, resp_phone, resp_address) = (
        response.email.as_ref().unwrap(),
        response.phone.as_ref().unwrap(),
        response.address.as_ref().unwrap(),
    );

    // Email address assertions
    assert!(!resp_email.disposable);
    assert!(!resp_email.role_address);
    assert!(resp_email.error.is_some());
    assert_str_eq!(email_data, resp_email.address);
    assert_eq!(resp_email.status, types::VerificationStatus::Invalid);
    assert_eq!(
        types::VerificationError::EmailDomainInvalid,
        resp_email
            .error_code
            .unwrap_or(types::VerificationError::Unknown),
    );

    // Phone number assertions
    assert_ne!(phone_data, resp_phone.number);
    assert_eq!(resp_phone.status, types::VerificationStatus::Valid);
    assert_str_eq!(
        resp_phone.service_type.as_ref().map_or("", String::as_str),
        "land"
    );

    assert!(resp_address.errors.is_empty());
    assert!(resp_address.address2.is_none());
    assert_str_eq!(address_data.city.as_str(), resp_address.city);
    assert_str_eq!(address_data.state.as_str(), resp_address.state);
    assert!(resp_address.zip.starts_with(address_data.zip.as_str()));
    assert_eq!(resp_address.status, types::VerificationStatus::Valid);
    Ok(assert!(
        !resp_address.corrected || address_data.address1.as_str() != resp_address.address1
    ))
}

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`verify_email`](briteverify_rs::BriteVerifyClient::verify_email)
/// method sends the expected request and properly handles the returned
/// response (per the official BriteVerify API Postman collection)
async fn handles_valid_email_verifications(#[from(mock_valid_email)] mock: Mock) -> Result<()> {
    let (client, server) = utils::client_and_server(None, None).await;

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    let email = mock_data::OFFICIAL_EMAIL_VALID
        .extract_from_request("email")
        .unwrap();
    let response = client.verify_email(&email).await?;

    assert!(!response.disposable);
    assert!(response.role_address);
    assert_str_eq!(email, response.address);
    Ok(assert_eq!(
        response.status,
        types::VerificationStatus::Valid
    ))
}

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`verify_email`](briteverify_rs::BriteVerifyClient::verify_email)
/// method sends the expected request and properly handles the returned
/// response (per the official BriteVerify API Postman collection)
async fn handles_invalid_email_verifications(#[from(mock_invalid_email)] mock: Mock) -> Result<()> {
    let (client, server) = utils::client_and_server(None, None).await;

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    let email = mock_data::OFFICIAL_EMAIL_INVALID
        .extract_from_request("email")
        .unwrap();
    let response = client.verify_email(&email).await?;

    assert!(!response.disposable);
    assert!(!response.role_address);
    assert!(response.error.is_some());
    assert_str_eq!(email, response.address);
    assert_eq!(response.status, types::VerificationStatus::Invalid);
    Ok(assert_eq!(
        types::VerificationError::EmailAccountInvalid,
        response
            .error_code
            .unwrap_or(types::VerificationError::Unknown),
    ))
}

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`verify_email`](briteverify_rs::BriteVerifyClient::verify_email)
/// method sends the expected request and properly handles the returned
/// response (per the official BriteVerify API Postman collection)
async fn handles_disposable_email_verifications(
    #[from(mock_disposable_email)] mock: Mock,
) -> Result<()> {
    let (client, server) = utils::client_and_server(None, None).await;

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    let email = mock_data::OFFICIAL_EMAIL_DISPOSABLE
        .extract_from_request("email")
        .unwrap();
    let response = client.verify_email(&email).await?;

    assert!(response.disposable);
    assert!(!response.role_address);
    assert!(response.error.is_none());
    assert!(response.error_code.is_none());
    assert_str_eq!(email, response.address);
    Ok(assert_eq!(
        response.status,
        types::VerificationStatus::AcceptAll,
    ))
}

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`verify_phone_number`](briteverify_rs::BriteVerifyClient::verify_phone_number)
/// method sends the expected request and properly handles the returned
/// response (per the official BriteVerify API Postman collection)
async fn handles_valid_phone_verifications(#[from(mock_valid_phone)] mock: Mock) -> Result<()> {
    let (client, server) = utils::client_and_server(None, None).await;

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    let number = mock_data::OFFICIAL_PHONE_VALID
        .extract_from_request("phone")
        .unwrap();
    let response = client.verify_phone_number(&number).await?;

    assert!(response.errors.is_empty());
    assert_str_eq!(number, response.number);
    assert!(response.phone_location.is_none());
    assert_str_eq!(response.service_type.unwrap_or(String::new()), "land");
    Ok(assert_eq!(
        response.status,
        types::VerificationStatus::Valid
    ))
}

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`verify_phone_number`](briteverify_rs::BriteVerifyClient::verify_phone_number)
/// method sends the expected request and properly handles the returned
/// response (per the official BriteVerify API Postman collection)
async fn handles_invalid_phone_verifications(#[from(mock_invalid_phone)] mock: Mock) -> Result<()> {
    let (client, server) = utils::client_and_server(None, None).await;

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    let number = mock_data::OFFICIAL_PHONE_INVALID
        .extract_from_request("phone")
        .unwrap();
    let response = client.verify_phone_number(&number).await?;

    assert_eq!(response.errors.len(), 1);
    assert_str_eq!(number, response.number);
    assert!(response.service_type.is_none());
    assert!(response.phone_location.is_none());
    Ok(assert_eq!(
        response.status,
        types::VerificationStatus::Invalid
    ))
}

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`verify_phone_number`](briteverify_rs::BriteVerifyClient::verify_street_address)
/// method sends the expected request and properly handles the returned
/// response (per the official BriteVerify API Postman collection)
async fn handles_valid_address_verifications(#[from(mock_valid_address)] mock: Mock) -> Result<()> {
    let (client, server) = utils::client_and_server(None, None).await;

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    let data = mock_data::OFFICIAL_ADDRESS_VALID
        .request_body_json::<types::VerificationRequest>()?
        .address
        .unwrap();

    let response = client
        .verify_street_address(
            data.address1.as_str(),
            data.address2.as_ref(),
            data.city.as_str(),
            data.state.as_str(),
            data.zip.as_str(),
        )
        .await?;

    assert!(!response.corrected);
    assert!(response.address2.is_none());
    assert_str_eq!(data.zip.as_str(), response.zip);
    assert_str_eq!(data.city.as_str(), response.city);
    assert_str_eq!(data.state.as_str(), response.state);
    assert_str_eq!(data.address1.as_str(), response.address1);
    Ok(assert_eq!(
        response.status,
        types::VerificationStatus::Valid
    ))
}

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`verify_street_address`](briteverify_rs::BriteVerifyClient::verify_street_address)
/// method sends the expected request and properly handles the returned
/// response (per the official BriteVerify API Postman collection)
async fn handles_corrected_address_verifications(
    #[from(mock_corrected_address)] mock: Mock,
) -> Result<()> {
    let (client, server) = utils::client_and_server(None, None).await;

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    let data = mock_data::OFFICIAL_ADDRESS_CORRECTED
        .request_body_json::<types::VerificationRequest>()?
        .address
        .unwrap();

    let response = client
        .verify_street_address(
            data.address1.as_str(),
            data.address2.as_ref(),
            data.city.as_str(),
            data.state.as_str(),
            data.zip.as_str(),
        )
        .await?;

    assert!(response.corrected);
    assert!(response.address2.is_none());
    assert_ne!(data.zip.as_str(), response.zip);
    assert_str_eq!(data.city.as_str(), response.city);
    assert_str_eq!(data.state.as_str(), response.state);
    assert_ne!(data.address1.as_str(), response.address1);
    Ok(assert_eq!(
        response.status,
        types::VerificationStatus::Valid
    ))
}

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`verify_street_address`](briteverify_rs::BriteVerifyClient::verify_street_address)
/// method sends the expected request and properly handles the returned
/// response (per the official BriteVerify API Postman collection)
async fn handles_missing_suite_address_verifications(
    #[from(mock_missing_suite_address)] mock: Mock,
) -> Result<()> {
    let (client, server) = utils::client_and_server(None, None).await;

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    let data = mock_data::OFFICIAL_ADDRESS_MISSING_SUITE
        .request_body_json::<types::VerificationRequest>()?
        .address
        .unwrap();

    let response = client
        .verify_street_address(
            data.address1.as_str(),
            data.address2.as_ref(),
            data.city.as_str(),
            data.state.as_str(),
            data.zip.as_str(),
        )
        .await?;

    assert!(response.corrected);
    assert!(response.address2.is_none());
    assert_eq!(1, response.errors.len());
    assert_str_eq!(data.city.as_str(), response.city);
    assert_str_eq!(data.state.as_str(), response.state);
    assert_ne!(data.address1.as_str(), response.address1);
    assert!(response.zip.starts_with(data.zip.as_str()));
    Ok(assert_eq!(
        response.status,
        types::VerificationStatus::Invalid
    ))
}

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`verify_street_address`](briteverify_rs::BriteVerifyClient::verify_street_address)
/// method sends the expected request and properly handles the returned
/// response (per the official BriteVerify API Postman collection)
async fn handles_unknown_street_address_verifications(
    #[from(mock_unknown_street_address)] mock: Mock,
) -> Result<()> {
    let (client, server) = utils::client_and_server(None, None).await;

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    let data = mock_data::OFFICIAL_ADDRESS_UNKNOWN_STREET
        .request_body_json::<types::VerificationRequest>()?
        .address
        .unwrap();

    let response = client
        .verify_street_address(
            data.address1.as_str(),
            data.address2.as_ref(),
            data.city.as_str(),
            data.state.as_str(),
            data.zip.as_str(),
        )
        .await?;

    assert!(!response.corrected);
    assert!(response.address2.is_none());
    assert_eq!(1, response.errors.len());
    assert_str_eq!(data.zip.as_str(), response.zip);
    assert_str_eq!(data.city.as_str(), response.city);
    assert_str_eq!(data.state.as_str(), response.state);
    assert_str_eq!(data.address1.as_str(), response.address1);
    Ok(assert_eq!(
        response.status,
        types::VerificationStatus::Invalid
    ))
}

// </editor-fold desc="// Integration Tests ...">

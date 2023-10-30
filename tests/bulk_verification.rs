#![allow(clippy::unit_arg, dead_code, unused_imports, unused_variables)]
//! ## Integration Tests For [`BriteVerifyClient`](briteverify_rs::BriteVerifyClient)'s
//! ## Bulk Verification Methods

// Module Declarations
pub mod utils;

// Standard Library Imports
use std::str::Split;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc, Mutex,
    },
};

// Third Party Imports
use anyhow::Result;
use http_types::{mime::JSON, Method as HttpMethod, StatusCode};
use once_cell::sync::Lazy;
use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
use rstest::{fixture, rstest};
use serde_json::Value;
use test_log::test as test_log;
use wiremock::{http::Url, matchers, Match, Mock, Request, Respond, ResponseTemplate};

// Crate-Level Imports
use briteverify_rs::{errors::BriteVerifyClientError, types};
use utils::{
    official_response, v3_mock_data as mock_data, BriteVerifyRequest, MockRequestResponse,
    V3_LISTS, V3_LIST_RESULTS, V3_LIST_STATE,
};

// <editor-fold desc="// Constants ...">

// </editor-fold desc="// Constants ...">

// <editor-fold desc="// Utility Functions ...">

pub fn is_date_string(value: &str) -> bool {
    value.chars().all(|c| c == '-' || c.is_ascii_digit())
}

// <editor-fold desc="// Request Matchers ...">

/// Check if the supplied request matches the official
/// request specification for a "get bulk list states"
/// from the BriteVerify API's published Postman collection
/// [[ref](https://docs.briteverify.com/#0b5a2a7a-4062-4327-ab0a-4675592e3cd6)]
pub fn is_get_lists_request(request: &Request) -> bool {
    request.has_valid_api_key()
        && request.method == HttpMethod::Get
        && V3_LISTS.is_match(request.url.as_str())
}

/// Check if the supplied request matches the official
/// request specification for a bulk list CRUD-type
/// request from the BriteVerify API's Postman collection
/// [[ref](https://docs.briteverify.com/#38b4c9eb-31b1-4b8e-9295-a783d8043bc1)]
pub fn is_list_crud_request(request: &Request) -> bool {
    request.has_valid_api_key()
        && V3_LIST_STATE.is_match(request.url.as_str())
        && [HttpMethod::Get, HttpMethod::Post, HttpMethod::Delete].contains(&request.method)
}

// </editor-fold desc="// Request Matchers ...">

// <editor-fold desc="// Response Generators ...">

/// Return an empty collection of bulk verification lists
/// [[ref](https://docs.briteverify.com/#0b5a2a7a-4062-4327-ab0a-4675592e3cd6)]
pub fn no_lists_found_response(_: &Request) -> ResponseTemplate {
    official_response(mock_data::OFFICIAL_NO_LISTS_FOUND)
}

/// Return a list of bulk verification lists
/// [[ref](https://docs.briteverify.com/#0b5a2a7a-4062-4327-ab0a-4675592e3cd6)]
pub fn unfiltered_lists_response(_: &Request) -> ResponseTemplate {
    official_response(mock_data::OFFICIAL_GET_ALL_LISTS)
}

/// Return a list of bulk verification lists filtered by date
/// [[ref](https://docs.briteverify.com/#0b5a2a7a-4062-4327-ab0a-4675592e3cd6)]
pub fn lists_by_date_response(_: &Request) -> ResponseTemplate {
    official_response(mock_data::OFFICIAL_LISTS_BY_DATE)
}

/// Return a list of bulk verification lists filtered by page
/// [[ref](https://docs.briteverify.com/#0b5a2a7a-4062-4327-ab0a-4675592e3cd6)]
pub fn lists_by_page_response(_: &Request) -> ResponseTemplate {
    official_response(mock_data::OFFICIAL_LISTS_BY_PAGE)
}

/// Return a list of bulk verification lists filtered by state
/// [[ref](https://docs.briteverify.com/#0b5a2a7a-4062-4327-ab0a-4675592e3cd6)]
pub fn lists_by_state_response(_: &Request) -> ResponseTemplate {
    official_response(mock_data::OFFICIAL_LISTS_BY_STATE)
}

/// Return a list of bulk verification lists filtered by external identifier
/// [[ref](https://docs.briteverify.com/#0b5a2a7a-4062-4327-ab0a-4675592e3cd6)]
pub fn lists_by_ext_id_response(_: &Request) -> ResponseTemplate {
    official_response(mock_data::OFFICIAL_LISTS_BY_EXTERNAL_ID)
}

/// Return the state of the bulk verification specified by its
/// BriteVerify API-issued unique identifier. If the requested
/// list doesn't exist (read: has no corresponding data in the
/// examples from the official API docs) return the appropriate
/// "list not found" response.
/// [[ref](https://docs.briteverify.com/#b09c09dc-e11e-44a8-b53d-9f1fd9c6792d)]
pub fn list_state_by_id_response(request: &Request) -> ResponseTemplate {
    request
        .url
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|list_id| {
            [
                &mock_data::OFFICIAL_LIST_STATE_EXPIRED,
                &mock_data::OFFICIAL_LIST_STATE_COMPLETE,
                &mock_data::OFFICIAL_LIST_STATE_VERIFYING,
                &mock_data::OFFICIAL_LIST_STATE_TERMINATED,
                &mock_data::OFFICIAL_LIST_STATE_AUTO_TERMINATED,
                &mock_data::OFFICIAL_LIST_STATE_WITH_EXTERNAL_ID,
            ]
            .into_iter()
            .find(|example| example.response.contains(list_id))
        })
        .map(|data| official_response(*data))
        .unwrap_or(ResponseTemplate::new(StatusCode::NotFound).set_body_raw(
            mock_data::ERROR_LIST_STATE_NOT_FOUND.response,
            &JSON.to_string(),
        ))
}

/// Return the "result" of deleting the specified bulk verification
/// list. If the requested list doesn't exist (read: has no
/// corresponding data in the examples from the official API docs)
/// return the appropriate "list not found" response.
/// [[ref](https://docs.briteverify.com/#6c9b9c05-a4a0-435e-a064-af7d9476719d)]
pub fn delete_list_response(request: &Request) -> ResponseTemplate {
    request
        .url
        .path_segments()
        .and_then(|segments| segments.last())
        .and_then(|list_id| {
            [
                &mock_data::OFFICIAL_DELETE_PREPPED_LIST,
                &mock_data::OFFICIAL_DELETE_COMPLETED_LIST,
                &mock_data::OFFICIAL_DELETE_DELIVERED_LIST,
                &mock_data::OFFICIAL_DELETE_IMPORT_ERRORED_LIST,
            ]
            .into_iter()
            .find(|example| example.response.contains(list_id))
        })
        .map(|data| official_response(*data))
        .unwrap_or(ResponseTemplate::new(StatusCode::NotFound).set_body_raw(
            mock_data::ERROR_INVALID_LIST_STATE.response,
            &JSON.to_string(),
        ))
}

/// Return the "result" of updating a given bulk verification list
/// [[ref](https://docs.briteverify.com/#38b4c9eb-31b1-4b8e-9295-a783d8043bc1)]
pub fn update_list_response(request: &Request) -> ResponseTemplate {
    request
        .url
        .path_segments()
        .and_then(|segments| -> Option<ResponseTemplate> {
            match (
                segments
                    .last()
                    .and_then(|list_id| list_id.parse::<uuid::Uuid>().ok()),
                serde_json::from_slice::<types::BulkVerificationRequest>(&request.body),
            ) {
                (Some(list_id), Ok(body)) => {
                    let mock_data = match (&body.directive, body.contacts.is_empty()) {
                        (&types::BulkListDirective::Start, true) => mock_data::OFFICIAL_VERIFY_LIST,
                        (&types::BulkListDirective::Terminate, true) => {
                            mock_data::OFFICIAL_TERMINATE_LIST
                        }
                        _ => {
                            return None;
                        }
                    };

                    Some(official_response(mock_data))
                }
                _ => None,
            }
        })
        .unwrap_or(ResponseTemplate::new(StatusCode::BadRequest))
}

// </editor-fold desc="// Response Generators ...">

// </editor-fold desc="// Utility Functions ...">

// <editor-fold desc="// Auto-Retry Test Helper ...">

#[derive(Clone, Debug)]
struct StatefulPageResponder {
    pub current_page: Arc<AtomicU8>,
    pub page_map: HashMap<u8, MockRequestResponse>,
}

impl Match for StatefulPageResponder {
    fn matches(&self, request: &Request) -> bool {
        is_get_lists_request(request) && {
            let page = 1 + self.current_page.load(Ordering::SeqCst);
            self.current_page.store(page, Ordering::SeqCst);

            // request
            //   .url
            //   .query_pairs()
            //   .any(|(key, value)| key == "page" && value.parse::<u8>().unwrap_or(0) == page)

            self.page_map.contains_key(&page)
        }
    }
}

impl Respond for StatefulPageResponder {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let page = self.current_page.load(Ordering::SeqCst);

        match self.page_map.get(&page) {
            Some(data) => {
                ResponseTemplate::new(StatusCode::Ok).set_body_raw(data.response, &JSON.to_string())
            }
            None => {
                self.current_page.store(0, Ordering::SeqCst);
                ResponseTemplate::new(StatusCode::NotFound)
            }
        }
    }
}

// </editor-fold desc="// Auto-Retry Test Helper ...">

// <editor-fold desc="// Fixtures ...">

#[fixture]
/// An unregistered `Mock` that will respond to "get all bulk verification
/// lists" requests with the official "empty" response body from the
/// BriteVerify API's published Postman collection / documentation
fn mock_no_lists() -> Mock {
    Mock::given(is_get_lists_request).respond_with(no_lists_found_response)
}

#[fixture]
/// An unregistered `Mock` that will respond to "get all bulk verification
/// lists" requests with the official response body from the BriteVerify API's
/// published Postman collection / documentation
fn mock_all_lists() -> Mock {
    Mock::given(is_get_lists_request)
        .and(move |request: &Request| -> bool {
            request
                .url
                .query()
                .map_or(true, |val| val.trim().is_empty())
        })
        .respond_with(unfiltered_lists_response)
}

#[fixture]
/// An unregistered `Mock` that will respond to "get all verification lists
/// for {DATE}" requests with the official response body from the BriteVerify
/// API's published Postman collection / documentation
fn mock_lists_by_date() -> Mock {
    Mock::given(is_get_lists_request)
        .and(move |request: &Request| -> bool {
            request
                .url
                .query_pairs()
                .any(|(key, value)| key == "date" && is_date_string(value.as_ref()))
        })
        .respond_with(lists_by_date_response)
}

#[fixture]
/// An unregistered `Mock` that will respond to "get all verification lists
/// for {PAGE}" requests with the official response body from the BriteVerify
/// API's published Postman collection / documentation
fn mock_lists_by_page() -> Mock {
    let page = Arc::new(AtomicU8::new(0));

    let matcher = StatefulPageResponder {
        current_page: Arc::clone(&page),
        page_map: HashMap::from([
            (1, mock_data::OFFICIAL_MULTIPLE_LIST_PAGES),
            (2, mock_data::OFFICIAL_LISTS_BY_PAGE),
        ]),
    };

    let responder = matcher.clone();

    Mock::given(matcher)
        .and(move |request: &Request| -> bool {
            request
                .url
                .query_pairs()
                .any(|(key, value)| key == "page" && value.parse::<u64>().is_ok())
        })
        .respond_with(responder)
}

#[fixture]
/// An unregistered `Mock` that will respond to "get all verification lists
/// for {STATE}" requests with the official response body from the BriteVerify
/// API's published Postman collection / documentation
fn mock_lists_by_state() -> Mock {
    Mock::given(is_get_lists_request)
        .and(move |request: &Request| -> bool {
            request.url.query_pairs().any(|(key, value)| {
                key == "state" && !types::BatchState::from(value.as_ref()).is_unknown()
            })
        })
        .respond_with(lists_by_state_response)
}

#[fixture]
/// An unregistered `Mock` that will respond to "get all verification lists
/// for {EXT_ID}" requests with the official response body from the BriteVerify
/// API's published Postman collection / documentation
fn mock_lists_by_ext_id() -> Mock {
    Mock::given(is_get_lists_request)
        .and(move |request: &Request| -> bool {
            request
                .url
                .path_segments()
                .is_some_and(|mut segments| segments.any(|part| part == "accounts"))
        })
        .respond_with(lists_by_ext_id_response)
}

#[fixture]
/// An unregistered `Mock` that will respond to "get verification list
/// state for {LIST_ID}" requests with the official response body from
/// the BriteVerify API's published Postman collection / documentation
fn mock_list_state_by_id() -> Mock {
    Mock::given(is_list_crud_request).respond_with(list_state_by_id_response)
}

#[fixture]
/// An unregistered `Mock` that will respond to "create/update verification
/// list {LIST_ID}" requests with the official response body from the BriteVerify
/// API's published Postman collection / documentation
fn mock_update_list() -> Mock {
    Mock::given(is_list_crud_request).respond_with(update_list_response)
}

#[fixture]
/// An unregistered `Mock` that will respond to "delete verification list
/// {LIST_ID}" requests with the official response body from the BriteVerify
/// API's published Postman collection / documentation
fn mock_delete_list() -> Mock {
    Mock::given(is_list_crud_request).respond_with(delete_list_response)
}

// </editor-fold desc="// Fixtures ...">

// <editor-fold desc="// Integration Tests ...">

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`get_lists`](briteverify_rs::BriteVerifyClient::get_lists)
/// method sends the expected request and properly handles the returned
/// response (per the official BriteVerify API Postman collection)
async fn gets_bulk_lists_without_filters(#[from(mock_all_lists)] mock: Mock) -> Result<()> {
    let (client, server) = utils::client_and_server(None, None).await;

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    let response = client.get_lists().await?;

    assert_eq!(3, response.len());

    let (actual_ids, expected_ids) = (
        response.ids(),
        vec![
            "a2595a63-ae71-4dda-91d4-57bdb331aa3a",
            "1433fe1c-cc4b-48fb-8989-d3ec83502c54",
            "288be984-2094-4925-8790-4ebfeab7d757",
        ],
    );

    assert_eq!(expected_ids, actual_ids);

    let completed = response.get_list_by_id("1433fe1c-cc4b-48fb-8989-d3ec83502c54");

    Ok(assert!(
        completed.is_some_and(|list| matches!(list.state, types::BatchState::Complete)),
        "Expected Some(&VerificationListState) w/ list.state == 'complete', got: {:#?}",
        completed
    ))
}

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`get_lists_by_date`](briteverify_rs::BriteVerifyClient::get_lists_by_date)
/// method sends the expected request and properly handles the returned
/// response (per the official BriteVerify API Postman collection)
async fn gets_bulk_lists_by_date(#[from(mock_lists_by_date)] mock: Mock) -> Result<()> {
    let (client, server) = utils::client_and_server(None, None).await;

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    let filter_date = chrono::NaiveDate::from_ymd_opt(2021, 8, 10).unwrap();

    let response: types::GetListStatesResponse = client.get_lists_by_date(filter_date).await?;

    assert_eq!(1, response.len());

    let list = response.iter().next().unwrap();

    assert_str_eq!("9fda9ed3-2819-4ce0-9811-e2207d1b3da0", list.id.as_str());

    Ok(assert_eq!(filter_date, list.created_at.date_naive()))
}

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`get_lists_by_page`](briteverify_rs::BriteVerifyClient::get_lists_by_page)
/// method sends the expected request and properly handles the returned
/// response (per the official BriteVerify API Postman collection)
async fn gets_bulk_lists_by_page(#[from(mock_lists_by_page)] mock: Mock) -> Result<()> {
    let (client, server) = utils::client_and_server(None, None).await;

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    let (page_1, page_2) = (
        client.get_lists_by_page(1u32).await?,
        client.get_lists_by_page(2u32).await?,
    );

    assert_eq!(3, page_1.len());
    assert_eq!(3, page_2.len());
    assert_eq!((1u64, 2u64), (page_1.current_page(), page_1.total_pages()));

    Ok(assert_eq!(
        (2u64, 2u64),
        (page_2.current_page(), page_2.total_pages())
    ))
}

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`get_lists_by_state`](briteverify_rs::BriteVerifyClient::get_lists_by_state)
/// method sends the expected request and properly handles the returned
/// response (per the official BriteVerify API Postman collection)
async fn gets_bulk_lists_by_state(#[from(mock_lists_by_state)] mock: Mock) -> Result<()> {
    let (client, server) = utils::client_and_server(None, None).await;

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    let filter_state = types::BatchState::Terminated;

    let response = client.get_lists_by_state(filter_state).await?;

    assert_eq!(1, response.len());

    let list = response.iter().next().unwrap();

    assert_str_eq!("fb6c70e4-d6e9-43c1-84a4-790b4e090b00", list.id.as_str());

    Ok(assert_eq!(filter_state, list.state))
}

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`get_lists_by_state`](briteverify_rs::BriteVerifyClient::get_lists_by_state)
/// method sends the expected request and properly handles the returned
/// response (per the official BriteVerify API Postman collection)
async fn gets_bulk_lists_by_ext_id(#[from(mock_lists_by_ext_id)] mock: Mock) -> Result<()> {
    let (client, server) = utils::client_and_server(None, None).await;

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    let response = client
        .get_filtered_lists(
            <Option<u32>>::None,
            <Option<chrono::NaiveDate>>::None,
            <Option<types::BatchState>>::None,
            Some(1234),
        )
        .await?;

    let completed = response
        .get_list_by_id("aad5327e-a4fa-4dac-8f24-4f622320a58a")
        .unwrap();

    Ok(assert!(
        completed
            .results_path
            .as_ref()
            .is_some_and(|url| url.to_string().contains("/accounts/")),
        "Expected Some(url) with an 'accounts' segment, got: {:#?}",
        completed.results_path.as_ref()
    ))
}

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`get_lists`](briteverify_rs::BriteVerifyClient::get_lists)
/// method behaves as expected when no errors occur, but no lists
/// actually exist upstream.
async fn handles_no_lists_found_gracefully(#[from(mock_no_lists)] mock: Mock) -> Result<()> {
    let (client, server) = utils::client_and_server(None, None).await;

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    let response = client.get_lists().await?;

    assert!(
        response.lists.is_empty(),
        "Expected an empty <Vec<_>>, got: {:#?}",
        &response.lists,
    );

    Ok(assert!(
        response.message.is_none(),
        "Expected <Option<String>>::None, got: {:#?}",
        &response.message,
    ))
}

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`get_lists`](briteverify_rs::BriteVerifyClient::get_lists_by_state)
/// method refuses to fetch bulk verification lists with an unknown
/// `BatchState` filter value.
async fn declines_bulk_lists_by_unknown_state() -> Result<()> {
    let (client, _) = utils::client_and_server(None, None).await;

    let response = client.get_lists_by_state("ensign babyface".into()).await?;

    assert!(response.is_empty());

    Ok(assert_str_eq!(
        "Declined to request lists using 'unknown' as list state filter",
        response.message.as_ref().map_or("", |msg| msg.as_str())
    ))
}

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`get_lists`](briteverify_rs::BriteVerifyClient::get_filtered_lists)
/// method refuses use [`Unknown`](BatchState::Unknown)
/// as a bulk verification list state filter value.
async fn omits_unknown_state_from_request_filters() -> Result<()> {
    let (client, server) = utils::client_and_server(None, None).await;

    let mock = Mock::given(is_get_lists_request).respond_with(
        move |request: &Request| -> ResponseTemplate {
            if request.url.query_pairs().any(|(key, _)| key == "state") {
                lists_by_state_response(request)
            } else {
                ResponseTemplate::new(StatusCode::UnprocessableEntity)
            }
        },
    );

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    let response = client
        .get_filtered_lists(
            <Option<u32>>::None,
            <Option<chrono::NaiveDate>>::None,
            Some("Commander William Thomas Riker"),
            <Option<&str>>::None,
        )
        .await;

    Ok(assert!(
        response.as_ref().is_err_and(|error| match error {
            BriteVerifyClientError::UnusableResponse(resp) => {
                resp.status() == http::StatusCode::UNPROCESSABLE_ENTITY
            }
            _ => false,
        }),
        "Expected Err(UnusableResponse) w/ status code 422, got: {:#?}",
        response
    ))
}

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`get_list_by_id`](briteverify_rs::BriteVerifyClient::get_list_by_id)
/// and [`get_list_by_external_id`](briteverify_rs::BriteVerifyClient::get_list_by_external_id)
/// methods send the expected requests and properly handle the returned
/// responses (per the official BriteVerify API Postman collection)
async fn gets_list_state_by_id(#[from(mock_list_state_by_id)] mock: Mock) -> Result<()> {
    let (client, server) = utils::client_and_server(None, None).await;

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    // <editor-fold desc="...">

    let expired = client
        .get_list_by_id("eda3acb3-099e-4a39-8563-3dcdde5a4411")
        .await?;
    let completed = client
        .get_list_by_id("52233c90-3dbe-47d4-910b-1fa9d1e8829c")
        .await?;
    let not_found = client
        .get_list_by_id("00000000-1111-2222-3333-444444444444")
        .await;
    let verifying = client
        .get_list_by_id("d3b7e1c9-0bb3-4d93-9809-560921dc91b6")
        .await?;
    let terminated = client
        .get_list_by_id("2880123d-172d-477b-aea0-11ba417eb07f")
        .await?;
    let external_id = client
        .get_list_by_external_id("c7995898-1368-4aa4-9427-236f25192b30", 12345)
        .await?;
    let auto_terminated = client
        .get_list_by_id("5cb2df8b-619d-4843-bf37-3d8b9565815f")
        .await?;

    // </editor-fold desc="...">

    // expired
    assert!(
        expired.external_id.as_ref().is_none(),
        "Expected <Option<String>>::None, got: {:#?}",
        expired.external_id.as_ref(),
    );
    assert!(
        expired.expiration_date.as_ref().is_some(),
        "Expected Some(DateTime<Utc>), got: {:#?}",
        expired.expiration_date.as_ref(),
    );
    assert_eq!(expired.state, types::BatchState::Complete);
    assert!(
        expired
            .errors
            .first()
            .is_some_and(|error| error.status == types::BatchState::Expired),
        r#"Expected Some(VerificationListErrorMessage) w/ `code` field "expired", got: {:#?}"#,
        &expired.errors,
    );

    // completed
    assert!(
        completed.external_id.as_ref().is_none(),
        "Expected <Option<String>>::None, got: {:#?}",
        completed.external_id.as_ref(),
    );
    assert!(
        completed.results_path.as_ref().is_some(),
        "Expected Some(url), got: {:?}",
        completed.results_path.as_ref(),
    );
    assert!(
        completed.expiration_date.as_ref().is_some(),
        "Expected Some(DateTime<Utc>), got: {:#?}",
        completed.expiration_date.as_ref(),
    );
    assert_eq!(completed.state, types::BatchState::Complete);

    // not_found
    assert!(
        not_found.as_ref().is_err_and(|error| match error {
            BriteVerifyClientError::BulkListNotFound(response) =>
                response.list_id.as_ref().is_some_and(|list_id| {
                    assert_str_eq!("00000000-1111-2222-3333-444444444444", list_id.as_str());
                    true
                }),
            _ => false,
        }),
        "Expected Err(BulkListNotFound) w/ `list_id` Some(list_id), got: {:#?}",
        not_found.as_ref(),
    );

    // verifying
    assert!(
        verifying.external_id.as_ref().is_none(),
        "Expected <Option<String>>::None, got: {:#?}",
        verifying.external_id.as_ref(),
    );
    assert!(
        verifying.expiration_date.as_ref().is_none(),
        "Expected <Option<DateTime<Utc>>>::None, got: {:#?}",
        verifying.expiration_date.as_ref(),
    );
    assert_eq!(verifying.state, types::BatchState::Verifying);

    // terminated
    assert!(
        terminated.external_id.as_ref().is_none(),
        "Expected <Option<String>>::None, got: {:#?}",
        terminated.external_id.as_ref(),
    );
    assert!(
        terminated.expiration_date.as_ref().is_none(),
        "Expected <Option<DateTime<Utc>>>::None, got: {:#?}",
        terminated.expiration_date.as_ref(),
    );
    assert!(
        terminated.errors.first().is_some_and(|error| {
            error.status == types::BatchState::ImportError // prevent-rustfmt
                && error.message.as_ref().map_or("", String::as_str).contains("user terminated")
        }),
        r#"Expected Some(VerificationListErrorMessage) \
        w/ `code` field "import_error" & `message` field \
        containing "user terminated", got: {:#?}"#,
        &terminated.errors,
    );
    assert_eq!(terminated.state, types::BatchState::Terminated);

    // external_id
    assert!(
        external_id.external_id.as_ref().is_some(),
        "Expected Some(external_id), got: {:#?}",
        external_id.external_id.as_ref(),
    );
    assert!(
        external_id.results_path.as_ref().is_some(),
        "Expected Some(url), got: {:?}",
        external_id.results_path.as_ref(),
    );
    assert!(
        external_id.expiration_date.as_ref().is_some(),
        "Expected Some(DateTime<Utc>), got: {:#?}",
        external_id.expiration_date.as_ref(),
    );
    assert_eq!(external_id.state, types::BatchState::Complete);

    // auto_terminated
    assert!(
        auto_terminated.external_id.as_ref().is_none(),
        "Expected <Option<String>>::None, got: {:#?}",
        auto_terminated.external_id.as_ref(),
    );
    assert!(
        auto_terminated.expiration_date.as_ref().is_none(),
        "Expected <Option<DateTime<Utc>>>::None, got: {:#?}",
        auto_terminated.expiration_date.as_ref(),
    );
    assert!(
        auto_terminated.errors.first().is_some_and(|error| {
            error.status == types::BatchState::ImportError // prevent-rustfmt
                && error.message.as_ref().map_or("", String::as_str).contains("auto-terminated")
        }),
        r#"Expected Some(VerificationListErrorMessage) \
        w/ `code` field "import_error" & `message` field \
        containing "user terminated", got: {:#?}"#,
        &auto_terminated.errors,
    );
    assert_eq!(auto_terminated.state, types::BatchState::Terminated);

    Ok(())
}

#[rstest]
#[ignore]
#[test_log::test(tokio::test)]
/// Test that the [`update_list`](briteverify_rs::BriteVerifyClient::update_list)
/// method sends the expected request and properly handles the returned
/// response (per the official BriteVerify API Postman collection)
async fn updates_lists(#[from(mock_update_list)] mock: Mock) -> Result<()> {
    let (client, server) = utils::client_and_server(None, None).await;

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    // mock_data::OFFICIAL_VERIFY_LIST -> queue_list_for_processing
    // mock_data::OFFICIAL_APPEND_TO_LIST -> update_list
    // mock_data::OFFICIAL_TERMINATE_LIST -> terminate_list_by_id

    let response = client
        .update_list(
            uuid::Uuid::new_v4(),
            Vec::<types::VerificationRequest>::new(),
            true,
        )
        .await?;

    assert_eq!(response.status, types::BatchState::Success);
    assert_eq!(response.list.state, types::BatchState::Verifying);

    let response = client
        .update_list(
            uuid::Uuid::new_v4(),
            Vec::<types::VerificationRequest>::new(),
            false,
        )
        .await?;

    assert_eq!(response.list.state, types::BatchState::Unknown);
    assert_eq!(response.status, types::BatchState::InvalidState);

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`delete_list_by_id`](briteverify_rs::BriteVerifyClient::delete_list_by_id)
/// method sends the expected request and properly handles the returned
/// response (per the official BriteVerify API Postman collection)
async fn deletes_lists_by_id(#[from(mock_delete_list)] mock: Mock) -> Result<()> {
    let (client, server) = utils::client_and_server(None, None).await;

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    // <editor-fold desc="...">

    let prepped = client
        .delete_list_by_id("ec137d51-cbad-4924-9fcb-57d7566b031d")
        .await?;
    let completed = client
        .delete_list_by_id("13ae1f20-9483-4e0e-857d-58d83f371859")
        .await?;
    let delivered = client
        .delete_list_by_id("6fcd86e6-e197-4b3f-a6d6-f531f1990206")
        .await?;
    let not_found = client
        .delete_list_by_id("00000000-1111-2222-3333-444444444444")
        .await;
    let import_errored = client
        .delete_list_by_id("9984e0f5-420c-4d5f-b8ff-867d96192d8e")
        .await?;

    // </editor-fold desc="...">

    // prepped
    assert!(
        prepped.message.is_empty(),
        "Expected an empty string, got: {:#?}",
        prepped.message.as_str(),
    );
    assert!(
        prepped.list.expiration_date.as_ref().is_none(),
        "Expected Some(DateTime<Utc>), got: {:#?}",
        prepped.list.expiration_date.as_ref(),
    );
    assert!(
        prepped.list.results_path.as_ref().is_none(),
        "Expected ≤Option<Url>≥::None, got: {:#?}",
        prepped.list.results_path.as_ref(),
    );
    assert_eq!(prepped.status, types::BatchState::Success);
    assert_eq!(prepped.list.state, types::BatchState::Deleted);

    // completed
    assert!(
        completed.message.is_empty(),
        "Expected an empty string, got: {:#?}",
        completed.message.as_str(),
    );
    assert!(
        completed.list.expiration_date.as_ref().is_none(),
        "Expected Some(DateTime<Utc>), got: {:#?}",
        completed.list.expiration_date.as_ref(),
    );
    assert!(
        completed.list.results_path.as_ref().is_none(),
        "Expected ≤Option<Url>≥::None, got: {:#?}",
        completed.list.results_path.as_ref(),
    );
    assert_eq!(completed.status, types::BatchState::Success);
    assert_eq!(completed.list.state, types::BatchState::Deleted);

    // delivered
    assert!(
        delivered.message.is_empty(),
        "Expected an empty string, got: {:#?}",
        delivered.message.as_str(),
    );
    assert!(
        delivered.list.expiration_date.as_ref().is_none(),
        "Expected Some(DateTime<Utc>), got: {:#?}",
        delivered.list.expiration_date.as_ref(),
    );
    assert!(
        delivered.list.results_path.as_ref().is_none(),
        "Expected ≤Option<Url>≥::None, got: {:#?}",
        delivered.list.results_path.as_ref(),
    );
    assert_eq!(delivered.status, types::BatchState::Success);
    assert_eq!(delivered.list.state, types::BatchState::Deleted);

    // not_found
    assert!(
        not_found.as_ref().is_err_and(|error| match error {
            BriteVerifyClientError::BulkListNotFound(response) =>
                matches!(response.status, types::BatchState::InvalidState) && response.list_id.as_ref().is_some_and(|list_id| {
                    assert_str_eq!("00000000-1111-2222-3333-444444444444", list_id.as_str());
                    true
                }),
            _ => false,
        }),
        "Expected Err(BulkListNotFound) w/ `list_id` Some(list_id) & status 'invalid_state', got: {:#?}",
        not_found.as_ref(),
    );

    // import_errored
    assert!(
        import_errored.message.is_empty(),
        "Expected an empty string, got: {:#?}",
        import_errored.message.as_str(),
    );
    assert!(
        import_errored.list.expiration_date.as_ref().is_none(),
        "Expected Some(DateTime<Utc>), got: {:#?}",
        import_errored.list.expiration_date.as_ref(),
    );
    assert!(
        import_errored.list.results_path.as_ref().is_none(),
        "Expected ≤Option<Url>≥::None, got: {:#?}",
        import_errored.list.results_path.as_ref(),
    );
    assert_eq!(import_errored.status, types::BatchState::Success);
    assert_eq!(import_errored.list.state, types::BatchState::Deleted);

    Ok(())
}

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`terminate_list_by_id`](briteverify_rs::BriteVerifyClient::terminate_list_by_id)
/// method sends the expected request and properly handles the returned
/// response (per the official BriteVerify API Postman collection)
async fn terminates_lists_by_id() -> Result<()> {
    let (client, server) = utils::client_and_server(None, None).await;

    let list_id = uuid::Uuid::new_v4();

    let mock = Mock::given(is_list_crud_request)
        .and(move |request: &Request| -> bool {
            request.url.as_str().contains(list_id.to_string().as_str())
                && serde_json::from_slice::<types::BulkVerificationRequest>(&request.body)
                    .is_ok_and(|body| {
                        body.contacts.is_empty()
                            && matches!(body.directive, types::BulkListDirective::Terminate)
                    })
        })
        .respond_with(ResponseTemplate::new(StatusCode::ImATeapot));

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    let response = client.terminate_list_by_id(list_id).await;

    Ok(assert!(
        response.as_ref().is_err_and(|error| match error {
            BriteVerifyClientError::UnusableResponse(reply) => {
                matches!(reply.status(), http::StatusCode::IM_A_TEAPOT)
            }
            _ => false,
        }),
        "Expected Err(UnusableResponse) w/ HTTP status code 419 (i am a teapot), got: {:#?}",
        response.as_ref(),
    ))
}

#[rstest]
#[test_log::test(tokio::test)]
/// Test that the [`queue_list_for_processing`](briteverify_rs::BriteVerifyClient::queue_list_for_processing)
/// method sends the expected request and properly handles the returned
/// response (per the official BriteVerify API Postman collection)
async fn queues_lists_for_processing() -> Result<()> {
    let (client, server) = utils::client_and_server(None, None).await;

    let list_id = uuid::Uuid::new_v4();

    let mock = Mock::given(is_list_crud_request)
        .and(move |request: &Request| -> bool {
            request.url.as_str().contains(list_id.to_string().as_str())
                && serde_json::from_slice::<types::BulkVerificationRequest>(&request.body)
                    .is_ok_and(|body| matches!(body.directive, types::BulkListDirective::Start))
        })
        .respond_with(ResponseTemplate::new(StatusCode::ImATeapot));

    #[allow(unused_variables)]
    let guard = mock.mount_as_scoped(&server).await;

    let response = client.queue_list_for_processing(list_id).await;

    Ok(assert!(
        response.as_ref().is_err_and(|error| match error {
            BriteVerifyClientError::UnusableResponse(reply) => {
                matches!(reply.status(), http::StatusCode::IM_A_TEAPOT)
            }
            _ => false,
        }),
        "Expected Err(UnusableResponse) w/ HTTP status code 419 (i am a teapot), got: {:#?}",
        response.as_ref(),
    ))
}

// </editor-fold desc="// Integration Tests ...">

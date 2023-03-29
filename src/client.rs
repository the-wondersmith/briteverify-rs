//! ## BriteVerify API Client
///
// Standard Library Imports
use std::{env, ops::Deref, time::Duration};

// Third-Party Imports
use anyhow::{Context, Result};
use reqwest;
use reqwest::StatusCode;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Crate Imports
use crate::types;

// <editor-fold desc="// Constants ...">

static V1_API_BASE_URL: &'static str = "https://bpi.briteverify.com/api/v1";
static V3_API_BASE_URL: &'static str = "https://bulk-api.briteverify.com/api/v3";

// </editor-fold desc="// Constants ...">

// <editor-fold desc="// Client ...">

/// `briteverify-rs`'s [`reqwest`](https://docs.rs/reqwest/latest/reqwest/)-based client
#[derive(Debug)]
pub struct BriteVerifyClient(reqwest::Client);

impl Deref for BriteVerifyClient {
    type Target = reqwest::Client;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl BriteVerifyClient {
    /// Create a new [`BriteVerifyClient`][BriteVerifyClient] instance
    pub fn new<ApiKey: ToString>(api_key: ApiKey) -> Result<Self> {
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new(
                env::var("LOG_LEVELS")
                    .unwrap_or_else(|_| "briteverify_rs=debug,reqwest=info".to_string()),
            ))
            .with(tracing_subscriber::fmt::layer())
            .init();

        let api_key: String = format!("ApiKey: {}", api_key.to_string());

        let mut auth_header = reqwest::header::HeaderValue::from_str(&api_key)
            .context("Could not create a valid header value from the supplied API key")?;
        auth_header.set_sensitive(true);

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Authorization", auth_header);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .cookie_store(true)
            .timeout(Duration::from_secs(360))
            .connect_timeout(Duration::from_secs(360))
            .build()
            .context("Could not create a usable `reqwest` client")?;

        Ok(Self(client))
    }

    // <editor-fold desc="// Real-Time Single Transaction Endpoints ... ">

    /// Get your current account credit balance
    /// [[ref](https://docs.briteverify.com/#07beceb3-2961-4d5b-93a4-9cfeb30f42fa)]
    pub async fn current_credits(&self) -> Result<u32> {
        Ok(self.get_account_balance().await?.credits)
    }

    /// Get the total number of credits your account currently has in reserve
    /// [[ref](https://docs.briteverify.com/#07beceb3-2961-4d5b-93a4-9cfeb30f42fa)]
    pub async fn current_credits_in_reserve(&self) -> Result<u32> {
        Ok(self.get_account_balance().await?.credits_in_reserve)
    }

    /// Get your account credit balance, total number of credits
    /// in reserve, and the timestamp of when your balance was
    /// most recently recorded
    /// [[ref](https://docs.briteverify.com/#07beceb3-2961-4d5b-93a4-9cfeb30f42fa)]
    pub async fn get_account_balance(&self) -> Result<types::AccountCreditBalance> {
        let response = self
            .get(format!("{V3_API_BASE_URL}/accounts/credits"))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(response.json::<types::AccountCreditBalance>().await?),
            _ => {
                todo!("Add proper handling for non-200 responses")
            }
        }
    }

    /// [internal-implementation]
    /// Actually perform a single-transaction verification
    async fn _full_verify<Displayable: ToString>(
        &self,
        email: Option<Displayable>,
        phone: Option<Displayable>,
        address1: Option<Displayable>,
        address2: Option<Displayable>,
        city: Option<Displayable>,
        state: Option<Displayable>,
        zip: Option<Displayable>,
    ) -> Result<types::VerificationResponse> {
        let request = types::VerificationRequest::from_values(
            email, phone, address1, address2, city, state, zip,
        )?;

        let response = self
            .0
            .post(format!("{V1_API_BASE_URL}/fullverify"))
            .json(&request)
            .send()
            .await?;

        match &response.status() {
            &StatusCode::OK => Ok(response.json::<types::VerificationResponse>().await?),
            _ => {
                println!("{:#?}", &response);
                let data = response.text().await?;
                println!("Content: {data:#?}");
                todo!("Add proper handling for non-200 responses")
            }
        }
    }

    /// Verify a "complete" contact record
    /// [[ref](https://docs.briteverify.com/#a7246384-e91e-48a9-8aed-7b71cb74dd42)]
    pub async fn verify_contact<Displayable: ToString>(
        &self,
        email: Displayable,
        phone: Displayable,
        address1: Displayable,
        address2: Option<Displayable>,
        city: Displayable,
        state: Displayable,
        zip: Displayable,
    ) -> Result<types::FullVerificationResponse> {
        let response = self
            ._full_verify(
                Some(email),
                Some(phone),
                Some(address1),
                address2,
                Some(city),
                Some(state),
                Some(zip),
            )
            .await?;

        match response {
            types::VerificationResponse::Full(data) => Ok(data),
            _ => {
                anyhow::bail!("How did this even happen?")
            }
        }
    }

    /// Verify a single email address
    /// [[ref](https://docs.briteverify.com/#e5dd413c-6411-4078-8b4c-0e787f6a9325)]
    pub async fn verify_email<EmailAddress: ToString>(
        &self,
        email: EmailAddress,
    ) -> Result<types::EmailVerificationResponse> {
        let response = self
            ._full_verify(Some(email), None, None, None, None, None, None)
            .await?;

        match response {
            types::VerificationResponse::Email(data) => Ok(data),
            _ => {
                anyhow::bail!("How did this even happen?")
            }
        }
    }

    /// Verify a single phone number
    /// [[ref](https://docs.briteverify.com/#86e335f4-d1b2-4902-9051-4506a48a6b94)]
    pub async fn verify_phone_number<PhoneNumber: ToString>(
        &self,
        phone: PhoneNumber,
    ) -> Result<types::PhoneNumberVerificationResponse> {
        let response = self
            ._full_verify(None, Some(phone), None, None, None, None, None)
            .await?;

        match response {
            types::VerificationResponse::Phone(data) => Ok(data),
            _ => {
                anyhow::bail!("How did this even happen?")
            }
        }
    }

    /// Verify a single street address
    /// [[ref](https://docs.briteverify.com/#f588d8d3-8250-4a8a-9e58-f89c81af6bed)]
    pub async fn verify_street_address<Displayable: ToString>(
        &self,
        address1: Displayable,
        address2: Option<Displayable>,
        city: Displayable,
        state: Displayable,
        zip: Displayable,
    ) -> Result<types::AddressVerificationResponse> {
        let response = self
            ._full_verify(
                None,
                None,
                Some(address1),
                address2,
                Some(city),
                Some(state),
                Some(zip),
            )
            .await?;

        match response {
            types::VerificationResponse::Address(data) => Ok(data),
            _ => {
                anyhow::bail!("How did this even happen?")
            }
        }
    }

    // </editor-fold desc="// Real-Time Single Transaction Endpoints ... ">

    // <editor-fold desc="// Bulk Verification (v3) Endpoints ... ">

    /// Retrieve the complete, unfiltered list of all bulk verification
    /// lists created within the last 7 calendar days
    /// [[ref](https://docs.briteverify.com/#0b5a2a7a-4062-4327-ab0a-4675592e3cd6)]
    pub async fn get_lists(&self) -> Result<types::GetListStatesResponse> {
        self.get_filtered_lists(
            <Option<u32>>::None,
            <Option<chrono::NaiveDate>>::None,
            <Option<String>>::None,
        )
        .await
    }

    /// Retrieve the complete list of all bulk verification lists created
    /// within the last 7 calendar days filtered by the specified criteria
    /// [[ref](https://docs.briteverify.com/#0b5a2a7a-4062-4327-ab0a-4675592e3cd6)]
    pub async fn get_filtered_lists<'header, Date: chrono::Datelike, State: ToString>(
        &self,
        page: Option<u32>,
        date: Option<Date>,
        state: Option<State>,
    ) -> Result<types::GetListStatesResponse> {
        let mut params: Vec<(&'header str, String)> = Vec::new();

        if let Some(page) = page {
            params.push(("page", page.to_string()));
        }

        if let Some(date) = date {
            params.push((
                "date",
                format!("{}-{:0>2}-{:0>2}", date.year(), date.month(), date.day()),
            ));
        }

        if let Some(state) = state {
            let value = state.to_string();
            let filter = types::BatchState::from(value.as_str());

            if filter == types::BatchState::Unknown {
                tracing::warn!(
                    "Declining to include unknown list state as request filter: {value:#?}"
                );
            } else {
                params.push(("state", filter.to_string()));
            }
        }

        let mut request = self.get(format!("{V3_API_BASE_URL}/lists"));

        if !params.is_empty() {
            request = request.query(&params);
        }

        let response = request.send().await?;

        match response.status() {
            StatusCode::OK => Ok(response.json::<types::GetListStatesResponse>().await?),
            _ => {
                todo!("Add proper handling for non-200 responses")
            }
        }
    }

    /// Retrieve the complete list of all bulk verification lists filtered
    /// by the specified date [[ref](https://docs.briteverify.com/#0b5a2a7a-4062-4327-ab0a-4675592e3cd6)]
    ///
    /// > **NOTE:** Regardless of specified date, the BriteVerify API
    /// > does not appear to persist bulk verification lists older than
    /// > 7 calendar days
    pub async fn get_lists_by_date<Date: chrono::Datelike>(
        &self,
        date: Date,
    ) -> Result<types::GetListStatesResponse> {
        self.get_filtered_lists(
            <Option<u32>>::None,
            Some(date),
            <Option<types::BatchState>>::None,
        )
        .await
    }

    /// Retrieve the specified "page" of bulk verification lists
    /// [[ref](https://docs.briteverify.com/#0b5a2a7a-4062-4327-ab0a-4675592e3cd6)]
    pub async fn get_lists_by_page(&self, page: u32) -> Result<types::GetListStatesResponse> {
        self.get_filtered_lists(
            Some(page),
            <Option<chrono::NaiveDate>>::None,
            <Option<types::BatchState>>::None,
        )
        .await
    }

    /// Retrieve the complete list of all bulk verification lists whose status
    /// matches the specified one created within the last 7 calendar days
    /// [[ref](https://docs.briteverify.com/#0b5a2a7a-4062-4327-ab0a-4675592e3cd6)]
    pub async fn get_lists_by_state(
        &self,
        state: types::BatchState,
    ) -> Result<types::GetListStatesResponse> {
        match state {
            types::BatchState::Unknown => {
                tracing::warn!("Declining to request lists using 'unknown' as list state filter");
                Ok(types::GetListStatesResponse::default())
            }
            _ => {
                self.get_filtered_lists(
                    <Option<u32>>::None,
                    <Option<chrono::NaiveDate>>::None,
                    Some(state),
                )
                .await
            }
        }
    }

    /// Create a new bulk verification list with the supplied records
    /// and (optionally) queue it for immediate processing
    /// [[ref](https://docs.briteverify.com/#38b4c9eb-31b1-4b8e-9295-a783d8043bc1)]
    pub async fn create_list<
        Contact: Into<types::VerificationRequest>,
        ContactCollection: IntoIterator<Item = Contact>,
    >(
        &self,
        contacts: ContactCollection,
        auto_start: bool,
    ) -> Result<types::CreateListResponse> {
        self._create_or_update_list(<Option<String>>::None, contacts, auto_start)
            .await
    }

    /// Append records to the specified bulk verification list and (optionally)
    /// queue it for immediate processing
    /// [[ref](https://docs.briteverify.com/#38b4c9eb-31b1-4b8e-9295-a783d8043bc1:~:text=customer%2DID/lists-,list_id,-(optional))]
    pub async fn update_list<
        ListId: ToString,
        Contact: Into<types::VerificationRequest>,
        ContactCollection: IntoIterator<Item = Contact>,
    >(
        &self,
        list_id: ListId,
        contacts: ContactCollection,
        auto_start: bool,
    ) -> Result<types::UpdateListResponse> {
        self._create_or_update_list(Some(list_id), contacts, auto_start)
            .await
    }

    /// [internal-implementation]
    /// Create a new or mutate an extant bulk verification list
    async fn _create_or_update_list<
        ListId: ToString,
        Contact: Into<types::VerificationRequest>,
        Directive: Into<types::BulkListDirective>,
        ContactCollection: IntoIterator<Item = Contact>,
    >(
        &self,
        list_id: Option<ListId>,
        contacts: ContactCollection,
        directive: Directive,
    ) -> Result<types::CreateListResponse> {
        let directive = directive.into();

        let request = types::BulkVerificationRequest::new(contacts, directive);

        let url: String = {
            if let Some(id) = list_id {
                let list_id = id.to_string();
                format!("{V3_API_BASE_URL}/lists/{list_id}")
            } else {
                format!("{V3_API_BASE_URL}/lists")
            }
        };

        let response = self.post(url).json(&request).send().await?;

        match response.status() {
            StatusCode::OK | StatusCode::CREATED | StatusCode::BAD_REQUEST => {
                Ok(response.json::<types::CreateListResponse>().await?)
            }
            _ => {
                todo!("Add proper handling for non-200 responses")
            }
        }
    }

    /// Retrieve current "state" of the specified bulk verification list
    /// [[ref](https://docs.briteverify.com/#b09c09dc-e11e-44a8-b53d-9f1fd9c6792d)]
    pub async fn get_list_by_id<ListId: ToString>(
        &self,
        list_id: ListId,
    ) -> Result<types::VerificationListState> {
        let list_id: String = list_id.to_string();

        let response = self
            .get(format!("{V3_API_BASE_URL}/lists/{list_id}"))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(response.json::<types::VerificationListState>().await?),
            _ => {
                todo!("Add proper handling for non-200 responses")
            }
        }
    }

    /// Delete the specified batch verification list
    /// [[ref](https://docs.briteverify.com/#6c9b9c05-a4a0-435e-a064-af7d9476719d)]
    ///
    /// ___
    /// **NOTE:** This action *cannot* be reversed and
    /// once completed, the list will *no longer exist*.
    /// The list must be in one of the following states
    /// to be deleted:
    /// - [Prepped](types::enums::BatchState::Prepped)
    /// - [Complete](types::enums::BatchState::Complete)
    /// - [Delivered](types::enums::BatchState::Delivered)
    /// - [ImportError](types::enums::BatchState::ImportError)
    /// ___
    pub async fn delete_list_by_id<ListId: ToString>(
        &self,
        list_id: ListId,
    ) -> Result<types::DeleteListResponse> {
        let list_id: String = list_id.to_string();

        let response = self
            .delete(format!("{V3_API_BASE_URL}/lists/{list_id}"))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(response.json::<types::DeleteListResponse>().await?),
            _ => {
                todo!("Add proper handling for non-200 responses")
            }
        }
    }

    /// Abandon the specified unprocessed bulk verification list
    /// [[ref](https://docs.briteverify.com/#6c9b9c05-a4a0-435e-a064-af7d9476719d:~:text=To-,abandon,-an%20open%20list)]
    pub async fn terminate_list_by_id<ListId: ToString>(
        &self,
        list_id: ListId,
    ) -> Result<types::UpdateListResponse> {
        self._create_or_update_list(
            Some(list_id),
            <Vec<types::VerificationRequest>>::new(),
            types::BulkListDirective::Terminate,
        )
        .await
    }

    /// [internal-implementation]
    /// Retrieve the specified page of results from the specified
    /// bulk verification list
    async fn _get_result_page(
        &self,
        list_id: String,
        page_number: u64,
    ) -> Result<types::BulkVerificationResponse> {
        let response = self
            .get(format!(
                "{V3_API_BASE_URL}/lists/{list_id}/export/{page_number}"
            ))
            .send()
            .await?;
        match response.status() {
            StatusCode::OK => Ok(response.json::<types::BulkVerificationResponse>().await?),
            _ => {
                todo!("Add proper handling for non-200 responses")
            }
        }
    }

    /// Get the verification results for the specified bulk verification list
    /// [[ref](https://docs.briteverify.com/#0a0cc29d-6d9f-4b0d-9aa5-4166775a8831)]
    ///
    /// ___
    /// **NOTE:** Verification results are only available once
    /// a list has finished verifying in its entirety. It is not
    /// possible to retrieve verification results piecemeal.
    /// ___
    pub async fn get_results_by_list_id<ListId: ToString>(
        &self,
        list_id: ListId,
    ) -> Result<Vec<types::BulkVerificationResult>> {
        let list_id = list_id.to_string();
        let list_status = self.get_list_by_id(&list_id).await?;

        if list_status.page_count.is_none() {
            anyhow::bail!("Missing page count!");
        }

        let page_count = std::cmp::max(1u64, list_status.page_count.unwrap());

        let pages: Vec<_> = futures_util::future::join_all(
            (1..=page_count).map(|page_number| self._get_result_page(list_id.clone(), page_number)),
        )
        .await
        .into_iter()
        .filter(Result::is_ok) // TODO: Change this filter to log failed page fetches
        .map(|task_result| task_result.unwrap().results)
        .collect();

        let results: Vec<types::BulkVerificationResult> = itertools::concat(pages);

        Ok(results)
    }

    /// Queue the specified (open) bulk verification list for immediate processing
    /// [[ref](https://docs.briteverify.com/#0a0cc29d-6d9f-4b0d-9aa5-4166775a8831:~:text=immediately%20start%20a%20list)]
    pub async fn queue_list_for_processing<ListId: ToString>(
        &self,
        list_id: ListId,
    ) -> Result<types::UpdateListResponse> {
        self._create_or_update_list(
            Some(list_id),
            <Vec<types::VerificationRequest>>::new(),
            types::BulkListDirective::Start,
        )
        .await
    }

    // </editor-fold desc="// Bulk Verification (v3) Endpoints ... ">
}

// </editor-fold desc="// Client ...">

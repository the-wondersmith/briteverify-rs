//! # BriteVerify API Bulk Verification Example
// Third-Party Imports
use anyhow::Context;

// Crate-Level Imports
use briteverify_rs::BriteVerifyClient;

/// Example of creating a new BriteVerify API bulk
/// verification list and retrieving the processed
/// results once they're ready
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key: String = std::env::var("BV_API_KEY")
        .context("The 'BV_API_KEY' environment variable must be set for the example to work!")?;

    let client = BriteVerifyClient::new(api_key)?;

    let list_id = uuid::Uuid::new_v4().to_string();

    let response = client.get_results_by_list_id(list_id).await?;

    Ok(println!("{response:#?}"))
}

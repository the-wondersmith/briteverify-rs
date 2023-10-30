//! # BriteVerify API Account Balance Check Example
// Third-Party Imports
use anyhow::Context;

// Crate-Level Imports
use briteverify_rs::BriteVerifyClient;

/// Example of querying the current credit
/// balance of a BriteVerify API account
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key: String = std::env::var("BV_API_KEY")
        .context("The 'BV_API_KEY' environment variable must be set for this example to work!")?;

    let client = BriteVerifyClient::new(api_key)?;

    let response = client.get_account_balance().await?;

    Ok(println!("{response:#?}"))
}

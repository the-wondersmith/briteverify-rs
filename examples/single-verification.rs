//! # BriteVerify API Real Time Verification Example
// Third-Party Imports
use anyhow::Context;

// Crate-Level Imports
use briteverify_rs::BriteVerifyClient;

/// Example of using the BriteVerify API's single-transaction,
/// real-time endpoints to verify a single "contact" record
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key: String = std::env::var("BV_API_KEY")
        .context("The 'BV_API_KEY' environment variable must be set for the example to work!")?;

    let client = BriteVerifyClient::new(api_key)?;

    let response = client
        .verify_contact(
            "test@example.com",
            "+1(954) 494-1234",
            "123 Main St",
            Some("P.O. Box 100"),
            "Any Town",
            "CA",
            "90210",
        )
        .await?;

    Ok(println!("{response:#?}"))
}

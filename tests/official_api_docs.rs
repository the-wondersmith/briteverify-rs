#![allow(clippy::unit_arg)]
//! ## Upstream API Docs
//!
//! Utility test for ensuring the JSON request/response blobs
//! hard-coded into the client's integration test suite are current
//! with the latest published version of the BriteVerify API's
//! publicly available [Postman Collection](https://docs.briteverify.com/api/collections/11411276/SzmjyuQH?segregateAuth=true&versionTag=latest)

// Standard Library Imports
use std::ops::Deref;

// Third-Part Imports
use anyhow::Result;
use once_cell::sync::Lazy;
use pretty_assertions::assert_eq;
use reqwest::{header::ACCEPT, Client};
use rstest::rstest;
use serde_json::Value;

// <editor-fold desc="// Struct Definitions ...">

#[derive(Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Info {
    #[serde(default)]
    pub team: u64,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub owner: String,
    #[serde(default, rename = "publishedId")]
    pub published_id: String,
    #[serde(default, rename = "activeVersionTag")]
    pub active_version_tag: String,
    #[serde(default, rename = "publishDate")]
    pub publish_date: String,
    #[serde(default, rename = "publicUrl")]
    pub public_url: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, rename = "_postman_id")]
    pub postman_id: String,
    #[serde(default, rename = "collectionId")]
    pub collection_id: String,
    #[serde(default)]
    pub schema: String,
    #[serde(default, rename = "privateUrl")]
    pub private_url: String,
}

#[derive(Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Collection {
    #[serde(default)]
    pub info: Info,
}

#[derive(Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PostmanCollectionMeta {
    #[serde(default, rename = "versionOptions")]
    pub version_options: Vec<Value>,
    #[serde(default, rename = "activeVersionTag")]
    pub active_version_tag: String,
    #[serde(default, rename = "latestAvailableVersionTag")]
    pub latest_available_version_tag: String,
    #[serde(default)]
    pub collection: Collection,
}

// </editor-fold desc="// Struct Definitions ...">

// <editor-fold desc="// Constants ...">

static STAMPED: Lazy<PostmanCollectionMeta> = Lazy::new(|| PostmanCollectionMeta {
    version_options: Vec::new(),
    active_version_tag: "latest".to_string(),
    latest_available_version_tag: "latest".to_string(),
    collection: Collection {
        info: Info {
            team: 86454,
            version: "8.3.4".to_string(),
            owner: "11411276".to_string(),
            published_id: "SzmjyuQH".to_string(),
            active_version_tag: "latest".to_string(),
            publish_date: "2020-08-20T16:16:56.000Z".to_string(),
            public_url: "https://docs.briteverify.com".to_string(),
            name: "BriteVerify API Suite Documentation".to_string(),
            postman_id: "84e52e3e-e6e9-4a84-9fa6-84b8ae4e9692".to_string(),
            collection_id: "84e52e3e-e6e9-4a84-9fa6-84b8ae4e9692".to_string(),
            schema: "https://schema.getpostman.com/json/collection/v2.0.0/collection.json"
                .to_string(),
            private_url:
                "https://go.postman.co/documentation/11411276-84e52e3e-e6e9-4a84-9fa6-84b8ae4e9692"
                    .to_string(),
        },
    },
});

// </editor-fold desc="// Constants ...">

// <editor-fold desc="// Test Functions ...">

#[rstest]
#[test_log::test(tokio::test)]
/// Pull the meta-data about the most current version of the
/// Postman Collection that serves as the BriteVerify API's
/// publicly available documentation and compare it to the
/// version test suite's current "stamp" data. If there is
/// a more recent version of the API docs or request/response
/// body examples, this test should fail.
async fn fixtures_are_current() -> Result<()> {
    let client = Client::new();
    let response = client
        .get("https://docs.briteverify.com/view/SzmjyuQH")
        .header(ACCEPT, "application/json")
        .send()
        .await?;

    let data = response.json::<PostmanCollectionMeta>().await?;

    Ok(assert_eq!(STAMPED.deref(), &data))
}

// <editor-fold desc="// Test Functions ...">

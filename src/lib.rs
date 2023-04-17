#![forbid(unsafe_code)]
#![deny(missing_docs, missing_debug_implementations)]

//! # briteverify-rs
//!
//! `briteverify-rs` provides a type-safe, ergonomic client for
//! the [BriteVerify API](https://docs.briteverify.com/) based on
//! the popular [reqwest](https://docs.rs/reqwest/latest/reqwest/)
//! HTTP client library.
//!
//! It aims to provide a simple and convenient way to interact with
//! the BriteVerify API from Rust. It tries to be easy to use, with
//! a focus on allowing devs to quickly get up and running with the API.
//!
//! The [`BriteVerifyClient`][BriteVerifyClient] is asynchronous. It
//! does not currently support synchronous usage at all.
//!
//! ## Features
//! `briteverify-rs` provides:
//!
//! - Fully documented reqwest-based client for the BriteVerify API
//! - Type-safe requests and responses w/ [serde](https://docs.rs/serde/latest/serde/) support
//! - Support for all[¹](#first-note) [single-transaction](https://docs.briteverify.com/#79e00732-b734-4308-ac7f-820d62dde01f)
//!   and [bulk](https://docs.briteverify.com/#382f454d-dad2-49c3-b320-c7d117fcc20a)[²](#second-note) BriteVerify API endpoints
//! - Easy-to-use API that follows Rust conventions
//!
//! ---
//! - <span id="first-note">**1:**</span> `briteverify-rs` makes a best-effort attempt to stay current with
//!    the BriteVerify API, but is ultimately maintained independently.
//!    Best-effort means that no guarantees are made, but PRs are always
//!    accepted.
//! - <span id="second-note">**2:**</span> `briteverify-rs` currently implements support for the `v3` bulk
//!    endpoints, with no plans to add support for legacy or deprecated
//!    endpoints (namely the `v2` endpoints in this case).
//!---
//!
//! ## Basic Usage
//!
//! ```no_run
//! # use anyhow::Context;
//! # use briteverify_rs::{BriteVerifyClient, types::AccountCreditBalance};
//! #
//! # #[tokio::main]
//! # async fn doc() -> anyhow::Result<()> {
//! let response: AccountCreditBalance = BriteVerifyClient::new("YOUR API KEY HERE")?
//!     .get_account_balance()
//!    .await?;
//!
//! println!("{response:#?}");
//! # Ok(())
//! # }
//! ```
//!
pub mod client;
pub mod errors;
pub mod types;
#[cfg(feature = "examples")]
pub mod utils;
#[cfg(not(feature = "examples"))]
pub(crate) mod utils;

pub use client::{BriteVerifyClient, BriteVerifyClientBuilder};

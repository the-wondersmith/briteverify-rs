# briteverify-rs

[![crate](https://img.shields.io/crates/v/briteverify-rs.svg)](https://crates.io/crates/briteverify-rs)
[![documentation](https://docs.rs/briteverify-rs/badge.svg)](https://docs.rs/briteverify-rs)
[![tests](https://github.com/the-wondersmith/briteverify-rs/actions/workflows/tests.yml/badge.svg)](https://github.com/the-wondersmith/briteverify-rs/actions)
[![coverage](https://coveralls.io/repos/github/the-wondersmith/briteverify-rs/badge.svg?branch=main)](https://coveralls.io/github/the-wondersmith/briteverify-rs?branch=main)

`briteverify-rs` provides a type-safe, ergonomic client for
the [BriteVerify API](https://docs.briteverify.com/) based on
the popular [reqwest](https://docs.rs/reqwest/latest/reqwest/)
HTTP client library.

It aims to provide a simple and convenient way to interact with
the BriteVerify API from Rust. It tries to be easy to use, with
a focus on allowing devs to quickly get up and running with the API.

### License

[`AGPL-3.0-or-later`](https://spdx.org/licenses/AGPL-3.0-or-later.html)

### Basic Usage

```rust
let response: AccountCreditBalance = BriteVerifyClient::new("YOUR API KEY HERE")?
    .get_account_balance()
    .await?;

println!("{response:#?}");
```

### Features
`briteverify-rs` provides:

- Fully documented reqwest-based client for the BriteVerify API
- Type-safe requests and responses w/ [serde](https://docs.rs/serde/latest/serde/) support
- Support for all[¹](#first-note) [single-transaction](https://docs.briteverify.com/#79e00732-b734-4308-ac7f-820d62dde01f)
  and [bulk](https://docs.briteverify.com/#382f454d-dad2-49c3-b320-c7d117fcc20a)[²](#second-note) BriteVerify API endpoints
- Easy-to-use API that follows Rust conventions

---
- <span id="first-note" style="font-weight: bold">1:</span> `briteverify-rs` makes a best-effort attempt to stay current with
   the BriteVerify API, but is ultimately maintained independently.
   Best-effort means that no guarantees are made, but PRs are always
   accepted.
- <span id="second-note" style="font-weight: bold">2:</span> `briteverify-rs` currently implements support for the `v3` bulk
   endpoints, with no plans to add support for legacy or deprecated
   endpoints (namely the `v2` endpoints in this case).
---

### TODO:
- Tests 😅
//! ## BriteVerify API Account Balance Types ([ref](https://docs.briteverify.com/#f3a4f0cd-7d6d-4487-94dc-7bd9d70deb93))
// Standard Library Imports
use std::fmt;

// Third Party Imports
use chrono::prelude::{DateTime, Utc};

// Conditional Imports
#[cfg(all(not(doc), any(test, feature = "examples")))]
pub use self::foundry::*;

// <editor-fold desc="// AccountCreditBalance ...">

/// Account credit balance and credits
/// [in reserve](https://docs.briteverify.com/#07beceb3-2961-4d5b-93a4-9cfeb30f42fa),
/// current as of the `recorded_on` timestamp.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AccountCreditBalance {
    /// The total number of available
    /// non-reserve verification credits
    pub credits: u32,
    /// The total number of credits being
    /// held for currently processing bulk
    /// verification lists
    pub credits_in_reserve: u32,
    /// The timestamp the current balance
    /// data should be considered "current"
    /// as of
    pub recorded_on: DateTime<Utc>,
}

impl Default for AccountCreditBalance {
    fn default() -> Self {
        Self {
            credits: 0,
            credits_in_reserve: 0,
            recorded_on: Utc::now(),
        }
    }
}

impl fmt::Display for AccountCreditBalance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}

// </editor-fold desc="// AccountCreditBalance ...">

// <editor-fold desc="// Test Helpers & Factory Implementations ...">

#[cfg(all(not(doc), any(test, feature = "examples")))]
mod foundry {
    // Third Party Imports
    use warlocks_cauldron as wc;

    // Crate-Level Imports
    use crate::utils::RandomizableStruct;

    impl RandomizableStruct for super::AccountCreditBalance {
        /// Randomly generate an `AccountCreditBalance` instance
        fn random() -> Self {
            Self {
                credits: wc::Numeric::number(0u32, 20_000u32),
                credits_in_reserve: wc::Numeric::number(0u32, 200u32),
                recorded_on: crate::utils::within_the_last_few_hours(),
            }
        }
    }
}

// </editor-fold desc="// Test Helpers & Factory Implementations ...">

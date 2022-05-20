#![deny(clippy::all)]
#![deny(unused_must_use)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::bool_assert_comparison)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::extra_unused_lifetimes)]

pub mod deposit_tracking;
pub mod interest_rates;
pub mod staking_metrics;
pub mod token_balances;
pub mod token_prices;
pub mod v1;
pub mod vaults;

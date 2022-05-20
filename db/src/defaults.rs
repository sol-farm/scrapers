use crate::models::*;
use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use solana_program::pubkey::Pubkey;
use std::str::FromStr;
use thiserror::Error;

/// this is really the time at which this value is first accessed
/// it's used as an optimization for the ::Default handlers defined below
pub static CURRENT_TIME: Lazy<DateTime<Utc>> = Lazy::new(Utc::now);

#[derive(Error, Debug)]
pub enum DBError {
    /// this error is returned when attempting to inserting a v1 liquidated positon
    /// record that already exists in the database, AND the caller has specified a None
    /// value for `ended_at` indicating that the user is attempting to insert a record
    /// which already exists, as opposed to updating an already existing record
    #[error("position with temp_liquidation_account {} already exists", 0)]
    V1PositionAlreadyExists(String),
}

impl Default for VaultTvl {
    fn default() -> Self {
        Self {
            id: 0,
            farm_name: String::default(),
            total_shares: 0_f64,
            total_underlying: 0_f64,
            value_locked: 0_f64,
            scraped_at: *CURRENT_TIME,
        }
    }
}

impl Default for Vault {
    fn default() -> Self {
        Self {
            id: 0,
            account_address: String::default(),
            account_data: vec![],
            farm_name: String::default(),
            scraped_at: *CURRENT_TIME,
            last_compound_ts: None,
            last_compound_ts_unix: 0,
        }
    }
}

impl Default for TokenPrice {
    fn default() -> Self {
        Self {
            id: 0,
            asset: String::default(),
            price: 0_f64,
            platform: String::default(),
            pc_in_lp: 0_f64,
            coin_in_lp: 0_f64,
            asset_identifier: String::default(),
            period_start: *CURRENT_TIME,
            period_end: *CURRENT_TIME,
            period_observed_prices: vec![],
            period_running_average: 0_f64,
            last_period_average: 0_f64,
            feed_stopped: false,
            token_mint: String::default(),
        }
    }
}

impl Default for DepositTracking {
    fn default() -> Self {
        Self {
            id: 0,
            owner_address: String::default(),
            account_address: String::default(),
            account_data: vec![],
            vault_account_address: String::default(),
            scraped_at: *CURRENT_TIME,
            current_balance: 0_f64,
            current_shares: 0_f64,
            balance_usd_value: 0_f64,
        }
    }
}

impl Default for InterestRate {
    fn default() -> Self {
        Self {
            id: 0,
            platform: String::default(),
            asset: String::default(),
            lending_rate: 0_f64,
            borrow_rate: 0_f64,
            utilization_rate: 0_f64,
            available_amount: 0_f64,
            borrowed_amount: 0_f64,
            scraped_at: *CURRENT_TIME,
        }
    }
}

impl Default for TokenBalance {
    fn default() -> Self {
        Self {
            id: 0,
            token_account: String::default(),
            token_mint: String::default(),
            identifier: String::default(),
            balance: 0_f64,
            scraped_at: *CURRENT_TIME,
        }
    }
}

impl Default for StakingAnalytic {
    fn default() -> Self {
        Self {
            id: 0,
            tokens_locked: 0_f64,
            tokens_staked: 0_f64,
            stulip_total_supply: 0_f64,
            apy: 0_f64,
            price_float: 0_f64,
            price_uint: 0,
            active_unstakes: 0,
            scraped_at: *CURRENT_TIME,
        }
    }
}

impl Default for RealizeYield {
    fn default() -> Self {
        Self {
            id: 0,
            total_deposited_balance: 0_f64,
            vault_address: "".to_string(),
            farm_name: "".to_string(),
            apr: 0_f64,
            gain_per_second: 0_f64,
            scraped_at: *CURRENT_TIME,
        }
    }
}

impl Default for InterestRateCurve {
    fn default() -> Self {
        Self {
            id: 0,
            platform: "".to_string(),
            asset: "".to_string(),
            rate_name: "".to_string(),
            min_borrow_rate: 0_f64,
            max_borrow_rate: 0_f64,
            optimal_borrow_rate: 0_f64,
            optimal_utilization_rate: 0_f64,
            degen_borrow_rate: 0_f64,
            degen_utilization_rate: 0_f64,
        }
    }
}

impl Default for LendingOptimizerDistribution {
    fn default() -> Self {
        Self {
            id: 0,
            vault_name: "".to_string(),
            standalone_vault_deposited_balances: vec![],
            standalone_vault_platforms: vec![],
        }
    }
}

impl Default for InterestRateMovingAverage {
    fn default() -> Self {
        Self {
            id: 0,
            platform: "".to_string(),
            asset: "".to_string(),
            rate_name: "".to_string(),
            period_start: *CURRENT_TIME,
            period_end: *CURRENT_TIME,
            period_running_average: 0_f64,
            period_observed_rates: vec![],
            last_period_running_average: 0_f64,
        }
    }
}

impl Default for AdvertisedYield {
    fn default() -> Self {
        Self {
            id: 0,
            vault_address: "".to_string(),
            farm_name: "".to_string(),
            apr: 0_f64,
            scraped_at: *CURRENT_TIME,
        }
    }
}

impl Default for V1ObligationLtv {
    fn default() -> Self {
        Self {
            id: 0,
            account_address: "".to_string(),
            authority: "".to_string(),
            user_farm: "".to_string(),
            ltv: 0_f64,
            scraped_at: *CURRENT_TIME,
            leveraged_farm: "".to_string(),
        }
    }
}

impl Default for V1UserFarm {
    fn default() -> Self {
        Self {
            id: 0,
            account_address: "".to_string(),
            authority: "".to_string(),
            obligations: vec![],
            obligation_indexes: vec![],
            leveraged_farm: "".to_string(),
        }
    }
}

impl Default for V1LiquidatedPosition {
    fn default() -> Self {
        Self {
            id: 0,
            authority: "".to_string(),
            temp_liquidation_account: "".to_string(),
            user_farm: "".to_string(),
            obligation: "".to_string(),
            started_at: *CURRENT_TIME,
            ended_at: None,
            liquidation_event_id: "".to_string(),
            leveraged_farm: "".to_string(),
        }
    }
}

impl Default for HistoricTsharePrice {
    fn default() -> Self {
        Self {
            id: 0,
            farm_name: "".to_string(),
            price: 0_f64,
            total_supply: 0_f64,
            holder_count: 0_f64,
            scraped_at: *CURRENT_TIME,
        }
    }
}

impl Default for V1ObligationAccount {
    fn default() -> Self {
        Self {
            id: 0,
            account: "".to_string(),
            authority: "".to_string(),
        }
    }
}

impl V1ObligationLtv {
    pub fn account(&self) -> Pubkey {
        Pubkey::from_str(&self.account_address).unwrap()
    }
    // this is the user farm owner
    pub fn authority(&self) -> Pubkey {
        Pubkey::from_str(&self.authority).unwrap()
    }
    pub fn user_farm(&self) -> Pubkey {
        Pubkey::from_str(&self.user_farm).unwrap()
    }
    pub fn leveraged_farm(&self) -> Pubkey {
        Pubkey::from_str(&self.leveraged_farm).unwrap()
    }
}

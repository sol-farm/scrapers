use crate::defaults::DBError;
use crate::filters::{
    cmp_ltvs, AdvertisedYieldMatcher, DepositTrackingMatcher, HistoricTSharePriceMatcher,
    InterestRateCurveMatcher, InterestRateMatcher, InterestRateMovingAverageMatcher,
    LendingOptimizerDistributionMatcher, RealizeYieldMatcher, StakingAnalyticMatcher,
    TokenBalanceMatcher, TokenPriceMatcher, V1LiquidatedPositionMatcher,
    V1ObligationAccountMatcher, V1ObligationLtvMatcher, V1UserFarmMatcher, VaultMatcher,
    VaultTvlMatcher,
};
use crate::models::{
    AdvertisedYield, DepositTracking, HistoricTsharePrice, InterestRate, InterestRateCurve,
    InterestRateMovingAverage, LendingOptimizerDistribution, RealizeYield, StakingAnalytic,
    TokenBalance, TokenPrice, V1LiquidatedPosition, V1ObligationAccount, V1ObligationLtv,
    V1UserFarm, Vault, VaultTvl,
};
use crate::schema::*;
use ::r2d2::Pool;
use anyhow::{anyhow, Result};
use arrform::{arrform, ArrForm};
use chrono::{prelude::*, Duration};
use diesel::r2d2;
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use diesel::*;
use diesel_derives_traits::{Model, NewModel};
use diesel_filter::Paginate;
use into_query::IntoQuery;
use log::warn;
use oracle::moving_average::{MovingAverage, MovingAverageCalculator};
use std::collections::HashMap;
use std::convert::TryInto;
use std::sync::Arc;

const _OOB_LIMIT: f64 = 25_f64;
/// a hardcoded moving average window of 10 minutes, which is approximately
/// 2.25x the current rebalance duration. in the future we should make this a dynamic
/// window that takes into account the current rebalance time
#[cfg(not(test))]
pub const MOVING_AVERAGE_WINDOW_IN_SECONDS: i64 = 600;

#[cfg(test)]
pub const MOVING_AVERAGE_WINDOW_IN_SECONDS: i64 = 15;

#[derive(Debug, Insertable, NewModel)]
#[table_name = "vault"]
#[model(Vault)]
pub struct NewVault {
    pub farm_name: String,
    pub account_address: String,
    pub account_data: Vec<u8>,
    pub scraped_at: DateTime<Utc>,
    pub last_compound_ts: Option<DateTime<Utc>>,
    pub last_compound_ts_unix: i64,
}

#[derive(Debug, Insertable, NewModel)]
#[table_name = "vault_tvl"]
#[model(VaultTvl)]
pub struct NewVaultTvl {
    pub farm_name: String,
    pub total_shares: f64,
    pub total_underlying: f64,
    pub value_locked: f64,
    pub scraped_at: DateTime<Utc>,
}

#[derive(Debug, Insertable, NewModel)]
#[table_name = "deposit_tracking"]
#[model(DepositTracking)]
pub struct NewDepositTracking {
    pub owner_address: String,
    pub account_address: String,
    pub account_data: Vec<u8>,
    pub vault_account_address: String,
    pub scraped_at: DateTime<Utc>,
    pub current_balance: f64,
    pub current_shares: f64,
    pub balance_usd_value: f64,
}

#[derive(Debug, Insertable, NewModel)]
#[table_name = "token_price"]
#[model(TokenPrice)]
pub struct NewTokenPrice {
    pub asset: String,
    pub platform: String,
    pub price: f64,
    pub coin_in_lp: f64,
    pub pc_in_lp: f64,
    pub asset_identifier: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub period_observed_prices: Vec<f64>,
    pub period_running_average: f64,
    pub last_period_average: f64,
    pub feed_stopped: bool,
    pub token_mint: String,
}

#[derive(Debug, Insertable, NewModel)]
#[table_name = "interest_rate"]
#[model(InterestRate)]
pub struct NewInterestRate {
    pub platform: String,
    pub asset: String,
    pub lending_rate: f64,
    pub borrow_rate: f64,
    pub utilization_rate: f64,
    pub available_amount: f64,
    pub borrowed_amount: f64,
    pub scraped_at: DateTime<Utc>,
}

#[derive(Debug, Insertable, NewModel)]
#[table_name = "token_balance"]
#[model(TokenBalance)]
pub struct NewTokenBalance {
    pub token_account: String,
    pub token_mint: String,
    pub identifier: String,
    pub balance: f64,
    pub scraped_at: DateTime<Utc>,
}

#[derive(Debug, Insertable, NewModel)]
#[table_name = "staking_analytic"]
#[model(StakingAnalytic)]
pub struct NewStakingAnalytic {
    pub tokens_staked: f64,
    pub tokens_locked: f64,
    pub stulip_total_supply: f64,
    pub apy: f64,
    pub price_float: f64,
    pub price_uint: i64,
    pub active_unstakes: i64,
    pub scraped_at: DateTime<Utc>,
}

#[derive(Debug, Insertable, NewModel)]
#[table_name = "realize_yield"]
#[model(RealizeYield)]
pub struct NewRealizeYield {
    pub vault_address: String,
    pub farm_name: String,
    pub total_deposited_balance: f64,
    pub gain_per_second: f64,
    pub apr: f64,
    pub scraped_at: DateTime<Utc>,
}

#[derive(Debug, Insertable, NewModel)]
#[table_name = "interest_rate_curve"]
#[model(InterestRateCurve)]
pub struct NewInterestRateCurve {
    pub platform: String,
    pub asset: String,
    pub rate_name: String,
    pub min_borrow_rate: f64,
    pub max_borrow_rate: f64,
    pub optimal_borrow_rate: f64,
    pub optimal_utilization_rate: f64,
    pub degen_borrow_rate: f64,
    pub degen_utilization_rate: f64,
}

#[derive(Debug, Insertable, NewModel)]
#[table_name = "lending_optimizer_distribution"]
#[model(LendingOptimizerDistribution)]
pub struct NewLendingOptimizerDistribution {
    pub vault_name: String,
    pub standalone_vault_platforms: Vec<String>,
    pub standalone_vault_deposited_balances: Vec<f64>,
}

#[derive(Debug, Insertable, NewModel)]
#[table_name = "interest_rate_moving_average"]
#[model(InterestRateMovingAverage)]
pub struct NewInterestRateMovingAverage {
    pub platform: String,
    pub asset: String,
    pub rate_name: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub period_running_average: f64,
    pub period_observed_rates: Vec<f64>,
    pub last_period_running_average: f64,
}

#[derive(Debug, Insertable, NewModel)]
#[table_name = "advertised_yield"]
#[model(AdvertisedYield)]
pub struct NewAdvertisedYield {
    pub vault_address: String,
    pub farm_name: String,
    pub apr: f64,
    pub scraped_at: DateTime<Utc>,
}

#[derive(Debug, Insertable, NewModel)]
#[table_name = "v1_obligation_ltv"]
#[model(V1ObligationLtv)]
pub struct NewV1ObligationLtv {
    pub authority: String,
    pub user_farm: String,
    pub account_address: String,
    pub ltv: f64,
    pub scraped_at: DateTime<Utc>,
    pub leveraged_farm: String,
}

#[derive(Debug, Insertable, NewModel)]
#[table_name = "v1_user_farm"]
#[model(V1UserFarm)]
pub struct NewV1UserFarm {
    pub account_address: String,
    pub authority: String,
    pub obligations: Vec<String>,
    pub obligation_indexes: Vec<i32>,
    pub leveraged_farm: String,
}

#[derive(Debug, Insertable, NewModel)]
#[table_name = "v1_liquidated_position"]
#[model(V1LiquidatedPosition)]
pub struct NewV1LiquidatedPosition {
    pub liquidation_event_id: String,
    pub temp_liquidation_account: String,
    pub authority: String,
    pub user_farm: String,
    pub obligation: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub leveraged_farm: String,
}

#[derive(Debug, Insertable, NewModel)]
#[table_name = "historic_tshare_price"]
#[model(HistoricTsharePrice)]
pub struct NewHistoricTSharePrice {
    pub farm_name: String,
    pub price: f64,
    pub total_supply: f64,
    pub holder_count: f64,
    pub scraped_at: DateTime<Utc>,
}

#[derive(Debug, Insertable, NewModel)]
#[table_name = "v1_obligation_account"]
#[model(V1ObligationAccount)]
pub struct NewV1ObligationAccount {
    pub account: String,
    pub authority: String,
}

/// returns a PgConnection pool
pub fn new_connection_pool(
    database_url: String,
    max_pool_size: u32,
) -> Result<Pool<ConnectionManager<PgConnection>>> {
    if max_pool_size < 1 {
        return Err(anyhow!("max_pool_size less than 1"));
    }
    let manager: ConnectionManager<PgConnection> = ConnectionManager::new(database_url);
    let pool = r2d2::Pool::builder()
        .min_idle(Some(1)) // always keep min idle to 1
        .max_size(max_pool_size)
        .build(manager)?;
    Ok(pool)
}

/// provides a high-level wrapper around PgConnection
/// allowing for database management, querying, etc..
pub struct DBClient<'a> {
    pub conn: &'a PgConnection,
    pub oob_limit: f64,
}

/// one may notice that some getter functions are "duplicated" in that
/// they have a publicly visible function, and a private function
/// prefixed with __
///
/// the reason we have this format for some functions and not others
/// is that certain metrics we collect are only ever inserted into the
/// database, for example if we want to have historical interest rates.
/// in that case we dont care about atomicity with regards to updating the
/// database as everything is indexed with the time the data was scraped.
///
/// however in certain situations we want to check the database first before
/// updating some record, in which case we want to ensure consistency with
/// the data reflected after an update.
///
/// to be able to have this consistency, and have code deduplication we have
/// two layers for getter functions, the public layer, and the inner layer
/// which can be used by transactions within the `DBClient` helper.
impl<'a> DBClient<'a> {
    /// establishes a connection to the database server
    pub fn establish_connection(database_url: String) -> PgConnection {
        PgConnection::establish(database_url.as_str())
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
    }
    /// searches for a vault account account using the given matches as a filter method
    pub fn get_vault_account(self: &Arc<Self>, matcher: &VaultMatcher) -> QueryResult<Vec<Vault>> {
        DBClient::__get_vault_account(self.conn, matcher)
    }
    fn __get_vault_account(conn: &PgConnection, matcher: &VaultMatcher) -> QueryResult<Vec<Vault>> {
        get_vault_account(conn, matcher)
    }
    /// deletes the first vault account account returned by the matcher
    pub fn delete_vault_account(self: &Arc<Self>, matcher: &VaultMatcher) -> Result<()> {
        let mut account = self.get_vault_account(matcher)?;
        if account.is_empty() {
            return Err(anyhow!("found no vault account matching {}", matcher));
        }
        let dt = std::mem::take(&mut account[0]);
        dt.destroy(self.conn)?;
        Ok(())
    }
    /// creates (or updates) a new vault account with the given information
    pub fn put_vault_account(
        self: &Arc<Self>,
        farm_name: String,
        account_address: String,
        account_data: Vec<u8>,
        scraped_at: DateTime<Utc>,
        last_compound_ts: Option<DateTime<Utc>>,
    ) -> Result<()> {
        let last_compound_ts_unix = if let Some(ts) = last_compound_ts {
            ts.timestamp()
        } else {
            0
        };
        self.conn.transaction::<_, anyhow::Error, _>(|| {
            let mut accts = DBClient::__get_vault_account(
                self.conn,
                &VaultMatcher::Account(vec![account_address.clone()]),
            )?;
            if accts.is_empty() {
                // create it
                let vault_acct = NewVault {
                    farm_name,
                    account_address,
                    account_data,
                    scraped_at,
                    last_compound_ts,
                    last_compound_ts_unix,
                };
                vault_acct.save(self.conn)?;
            } else {
                let mut acct = std::mem::take(&mut accts[0]);
                acct.account_data = account_data;
                acct.scraped_at = scraped_at;
                acct.last_compound_ts = last_compound_ts;
                acct.last_compound_ts_unix = last_compound_ts_unix;
                acct.farm_name = farm_name;
                acct.save(self.conn)?;
            }
            Ok(())
        })?;
        Ok(())
    }
    /// searches for a deposit tracking account using the given matches as a filter method
    pub fn get_deposit_tracking_account(
        self: &Arc<Self>,
        matcher: &DepositTrackingMatcher,
    ) -> QueryResult<Vec<DepositTracking>> {
        DBClient::__get_deposit_tracking_account(self.conn, matcher)
    }
    fn __get_deposit_tracking_account(
        conn: &PgConnection,
        matcher: &DepositTrackingMatcher,
    ) -> QueryResult<Vec<DepositTracking>> {
        get_deposit_tracking_account(conn, matcher)
    }
    /// deletes the first deposit tracking account returned by the matcher
    pub fn delete_deposit_tracking_account(
        self: &Arc<Self>,
        matcher: &DepositTrackingMatcher,
    ) -> Result<()> {
        let mut account = self.get_deposit_tracking_account(matcher)?;
        if account.is_empty() {
            return Err(anyhow!(
                "found no deposit tracking account matching {}",
                matcher
            ));
        }
        let dt = std::mem::take(&mut account[0]);
        dt.destroy(self.conn)?;
        Ok(())
    }
    /// creates (or updates) a new deposit tracking account with the given information
    pub fn put_deposit_tracking_account(
        self: &Arc<Self>,
        owner_address: String,
        account_address: String,
        account_data: Vec<u8>,
        vault_account_address: String,
        scraped_at: DateTime<Utc>,
        current_balance: f64,
        current_shares: f64,
        balance_usd_value: f64,
    ) -> Result<()> {
        self.conn.transaction::<_, anyhow::Error, _>(|| {
            let mut accts = DBClient::__get_deposit_tracking_account(
                self.conn,
                &DepositTrackingMatcher::Account(vec![account_address.clone()]),
            )?;
            if accts.is_empty() {
                // create the account
                let dt_acct = NewDepositTracking {
                    owner_address,
                    account_address,
                    vault_account_address,
                    account_data,
                    scraped_at,
                    current_balance,
                    current_shares,
                    balance_usd_value,
                };
                dt_acct.save(self.conn)?;
            } else {
                let mut acct = std::mem::take(&mut accts[0]);
                acct.account_data = account_data;
                acct.scraped_at = scraped_at;
                acct.current_balance = current_balance;
                acct.current_shares = current_shares;
                acct.balance_usd_value = balance_usd_value;
                acct.save(self.conn)?;
            }
            Ok(())
        })?;
        Ok(())
    }
    /// creates (or updates) a new token price account with the given information
    /// if updating a price record, we update the first matching record
    pub fn put_token_price(
        self: &Arc<Self>,
        asset: &str,
        platform: &str,
        price: f64,
        coin_in_lp: f64,
        pc_in_lp: f64,
        token_mint: &str,
    ) -> Result<()> {
        // we need to do some parsing of the asset
        // to accomodate for pre-v2 naming styles
        let asset_chunks: Vec<_> = asset.split('-').collect();
        let asset_identifier = if asset_chunks.len() >= 3 {
            format!("{}-{}-{}", platform, asset_chunks[1], asset_chunks[2])
        } else {
            format!("{}-{}", platform, asset)
        };

        let oob_limit = if self.oob_limit == 0_f64 {
            // sane default
            _OOB_LIMIT
        } else {
            self.oob_limit
        };
        // NA is an invalid platform only used by token prices
        // so return an AssetAndPlantform matcher as an optimization
        let matcher = if platform.eq("NA") {
            // NA is only a valid polat
            TokenPriceMatcher::AssetAndPlatform(vec![(asset.to_string(), platform.to_string())])
        } else {
            TokenPriceMatcher::AssetIdentifier(vec![asset_identifier.to_string()])
        };
        self.conn.transaction::<_, anyhow::Error, _>(|| {
            let now = Utc::now();
            let mut prices = DBClient::__get_token_price(self.conn, &matcher, Some(1))?;
            if prices.is_empty() {
                // create the token price record
                let tp_acct = NewTokenPrice {
                    asset: asset.to_string(),
                    asset_identifier,
                    price,
                    coin_in_lp,
                    pc_in_lp,
                    platform: platform.to_string(),
                    period_start: now,
                    period_observed_prices: vec![price],
                    period_running_average: price,
                    last_period_average: 0_f64,
                    period_end: match now.checked_add_signed(chrono::Duration::hours(1)) {
                        Some(end) => end,
                        None => return Err(anyhow!("failed to get period_end")),
                    },
                    token_mint: token_mint.to_string(),
                    feed_stopped: false,
                };
                tp_acct.save(self.conn)?;
                Ok(())
            } else {
                let mut price_record = std::mem::take(&mut prices[0]);
                // handle observing the price, and updating the twap price records
                price_record.observe_price(now, price, oob_limit)?;
                // make sure to record the lp composition state
                price_record.coin_in_lp = coin_in_lp;
                price_record.pc_in_lp = pc_in_lp;
                price_record.save(self.conn)?;
                Ok(())
            }
        })?;
        Ok(())
    }
    /// searches for a token price  using the given matches as a filter method
    pub fn get_token_price(
        self: &Arc<Self>,
        matcher: &TokenPriceMatcher,
        limit: Option<i64>,
    ) -> QueryResult<Vec<TokenPrice>> {
        DBClient::__get_token_price(self.conn, matcher, limit)
    }
    fn __get_token_price(
        conn: &PgConnection,
        matcher: &TokenPriceMatcher,
        limit: Option<i64>,
    ) -> QueryResult<Vec<TokenPrice>> {
        get_token_price(conn, matcher, limit)
    }
    /// deletes the first token price returned by the matcher
    pub fn delete_token_price(self: &Arc<Self>, matcher: &TokenPriceMatcher) -> Result<()> {
        let mut accounts = self.get_token_price(matcher, None)?;
        if accounts.is_empty() {
            return Err(anyhow!("found no token price matching {}", matcher));
        }
        for account in accounts.iter_mut() {
            std::mem::take(account).destroy(self.conn)?;
        }
        Ok(())
    }
    pub fn get_interest_rate(
        self: &Arc<Self>,
        matcher: &InterestRateMatcher,
    ) -> QueryResult<Vec<InterestRate>> {
        get_interest_rate(self.conn, matcher, None)
    }
    /// creates (or updates) a new token price account with the given inforamtion
    /// if updating an interest rate record we update the first matching record
    pub fn put_interest_rate(
        self: &Arc<Self>,
        lending_platform: String,
        lending_asset: String,
        borrow_rate: f64,
        utilization_rate: f64,
        lending_rate: f64,
        available_amount: f64,
        borrowed_amount: f64,
        scraped_at: DateTime<Utc>,
    ) -> Result<()> {
        let lending_platform_upper = lending_platform.to_ascii_uppercase();
        let lending_asset_upper = lending_asset.to_ascii_uppercase();

        let rate_name_upper = arrform!(512, "{}-{}", lending_platform_upper, lending_asset_upper)
            .as_str()
            .to_owned();
        // now handle moving average update within a transaction
        // since we have multiple rate scrapers, and need this to be atomic
        self.conn.transaction(|| {
            // first store the most recent rate information
            let new_rate = NewInterestRate {
                platform: lending_platform_upper.clone(),
                asset: lending_asset_upper.clone(),
                lending_rate,
                borrow_rate,
                utilization_rate,
                available_amount,
                borrowed_amount,
                scraped_at,
            };
            new_rate.save(self.conn)?;
            // we dont need to limit the query as there should only ever be 1 valid matching result
            let mut results =
                InterestRateMovingAverageMatcher::RateName(vec![rate_name_upper.clone()])
                    .to_filter()
                    .into_query()
                    .get_results::<InterestRateMovingAverage>(self.conn)?;
            if results.is_empty() {
                // create the rate for the first time
                // we need to make sure to record in native utc as we may have
                // multiple backend services running in different time zones
                let new_period_start = Utc::now();
                let new_period_end =
                    new_period_start + Duration::seconds(MOVING_AVERAGE_WINDOW_IN_SECONDS);
                NewInterestRateMovingAverage {
                    platform: lending_platform_upper,
                    asset: lending_asset_upper,
                    rate_name: rate_name_upper,
                    period_start: new_period_start,
                    period_end: new_period_end,
                    period_running_average: lending_rate,
                    period_observed_rates: vec![lending_rate],
                    last_period_running_average: 0_f64,
                }
                .save(self.conn)?;
            } else {
                let mut ma = std::mem::take(&mut results[0]);
                let mut calculator = MovingAverageCalculator::new(
                    ma.period_start,
                    ma.period_end,
                    ma.period_observed_rates.clone(),
                );
                match calculator.observe_value(lending_rate) {
                    Ok(new_average) => {
                        ma.period_observed_rates = calculator.observed_values();
                        ma.period_running_average = new_average;
                        ma.save(self.conn)?;
                    }
                    Err(err) => {
                        if err.to_string() == *"ErrPeriodFinished" {
                            // start a new period
                            let new_period_start = Utc::now();
                            let new_period_end = new_period_start
                                + Duration::seconds(MOVING_AVERAGE_WINDOW_IN_SECONDS);
                            let last_average = ma.period_running_average;

                            ma.period_start = new_period_start;
                            ma.period_end = new_period_end;
                            ma.period_running_average = lending_rate;
                            ma.period_observed_rates = vec![lending_rate];
                            ma.last_period_running_average = last_average;
                            ma.save(self.conn)?;
                        } else {
                            return Err(anyhow!(
                                "unexpected error encountered when observing value {:#?}",
                                err
                            ));
                        }
                    }
                }
            }
            Ok(())
        })?;
        Ok(())
    }
    /// deletes all matching interest rates
    pub fn delete_interest_rate(self: &Arc<Self>, matcher: &InterestRateMatcher) -> Result<()> {
        let mut account = self.get_interest_rate(matcher)?;
        if account.is_empty() {
            return Err(anyhow!("found no interest rate matching {}", matcher));
        }
        for dt in account.iter_mut() {
            let dt = std::mem::take(dt);
            dt.destroy(self.conn)?;
        }
        Ok(())
    }
    /// deletes all interest rates
    pub fn delete_interest_rates(self: &Arc<Self>) -> Result<()> {
        let mut account = self.get_interest_rate(&InterestRateMatcher::All)?;
        if account.is_empty() {
            return Err(anyhow!("found no matching interest rates"));
        }
        for dt in account.iter_mut() {
            std::mem::take(dt).destroy(self.conn)?;
        }
        Ok(())
    }
    pub fn put_vault_tvl(
        self: &Arc<Self>,
        farm_name: String,
        total_shares: f64,
        total_underlying: f64,
        value_locked: f64,
        scraped_at: DateTime<Utc>,
    ) -> Result<()> {
        NewVaultTvl {
            farm_name,
            total_shares,
            total_underlying,
            value_locked,
            scraped_at,
        }
        .save(self.conn)?;
        Ok(())
    }
    pub fn get_vault_tvl(
        self: &Arc<Self>,
        matcher: &VaultTvlMatcher,
    ) -> QueryResult<Vec<VaultTvl>> {
        get_vault_tvl(self.conn, matcher, None)
    }
    pub fn delete_vault_tvl(self: &Arc<Self>, matcher: &VaultTvlMatcher) -> Result<()> {
        let mut vaults = self.get_vault_tvl(matcher)?;
        for vault in vaults.iter_mut() {
            let vault = std::mem::take(vault);
            vault.destroy(self.conn)?;
        }
        Ok(())
    }
    pub fn put_token_balance(
        self: &Arc<Self>,
        token_account: String,
        token_mint: String,
        identifier: String,
        balance: f64,
        scraped_at: DateTime<Utc>,
    ) -> Result<()> {
        NewTokenBalance {
            token_account,
            token_mint,
            identifier,
            balance,
            scraped_at,
        }
        .save(self.conn)?;
        Ok(())
    }
    pub fn get_token_balance(
        self: &Arc<Self>,
        matcher: &TokenBalanceMatcher,
    ) -> QueryResult<Vec<TokenBalance>> {
        let filter = matcher.to_filter();
        filter.into_query().get_results::<TokenBalance>(self.conn)
    }
    pub fn delete_token_balance(self: &Arc<Self>, matcher: &TokenBalanceMatcher) -> Result<()> {
        let mut balances = self.get_token_balance(matcher)?;
        for balance in balances.iter_mut() {
            let balance = std::mem::take(balance);
            balance.destroy(self.conn)?;
        }
        Ok(())
    }
    pub fn put_staking_analytic(
        self: &Arc<Self>,
        tokens_staked: f64,
        tokens_locked: f64,
        stulip_total_supply: f64,
        apy: f64,
        price_float: f64,
        price_uint: u64,
        active_unstakes: i64,
        scraped_at: DateTime<Utc>,
    ) -> Result<()> {
        NewStakingAnalytic {
            tokens_staked,
            tokens_locked,
            stulip_total_supply,
            apy,
            price_float,
            scraped_at,
            active_unstakes,
            price_uint: price_uint.try_into()?,
        }
        .save(self.conn)?;
        Ok(())
    }
    pub fn get_staking_analytic(
        self: &Arc<Self>,
        matcher: &StakingAnalyticMatcher,
    ) -> QueryResult<Vec<StakingAnalytic>> {
        let filter = matcher.to_filter();
        filter
            .into_query()
            .get_results::<StakingAnalytic>(self.conn)
    }
    pub fn delete_staking_analytic(
        self: &Arc<Self>,
        matcher: &StakingAnalyticMatcher,
    ) -> Result<()> {
        let mut results = self.get_staking_analytic(matcher)?;
        for result in results.iter_mut() {
            std::mem::take(result).destroy(self.conn)?;
        }
        Ok(())
    }
    pub fn put_realize_yield(
        self: &Arc<Self>,
        vault_address: String,
        farm_name: String,
        total_deposited_balance: f64,
        apr: f64,
        gain_per_second: f64,
        scraped_at: DateTime<Utc>,
    ) -> Result<()> {
        NewRealizeYield {
            vault_address,
            farm_name,
            total_deposited_balance,
            gain_per_second,
            apr,
            scraped_at,
        }
        .save(self.conn)?;
        Ok(())
    }
    pub fn get_realize_yield(
        self: &Arc<Self>,
        matcher: &RealizeYieldMatcher,
        limit: Option<i64>,
    ) -> QueryResult<Vec<RealizeYield>> {
        get_realized_yield(self.conn, matcher, limit)
    }
    pub fn delete_realize_yield(self: &Arc<Self>, matcher: &RealizeYieldMatcher) -> Result<()> {
        let mut results = self.get_realize_yield(matcher, None)?;
        for result in results.iter_mut() {
            std::mem::take(result).destroy(self.conn)?;
        }
        Ok(())
    }
    pub fn put_interest_rate_curve(
        self: &Arc<Self>,
        platform: String,
        asset: String,
        min_borrow_rate: f64,
        max_borrow_rate: f64,
        optimal_borrow_rate: f64,
        optimal_utilization_rate: f64,
        degen_borrow_rate: f64,
        degen_utilization_rate: f64,
    ) -> Result<()> {
        let rate_name = format!(
            "{}-{}",
            platform.to_ascii_uppercase(),
            asset.to_ascii_uppercase()
        );
        NewInterestRateCurve {
            platform: platform.to_ascii_uppercase(),
            asset: asset.to_ascii_uppercase(),
            rate_name,
            min_borrow_rate,
            max_borrow_rate,
            optimal_borrow_rate,
            optimal_utilization_rate,
            degen_borrow_rate,
            degen_utilization_rate,
        }
        .save(self.conn)?;
        Ok(())
    }
    pub fn get_interest_rate_curve(
        self: &Arc<Self>,
        matcher: &InterestRateCurveMatcher,
    ) -> QueryResult<Vec<InterestRateCurve>> {
        matcher
            .to_filter()
            .into_query()
            .get_results::<InterestRateCurve>(self.conn)
    }
    pub fn delete_interest_rate_curve(
        self: &Arc<Self>,
        matcher: &InterestRateCurveMatcher,
    ) -> Result<()> {
        let mut curves = self.get_interest_rate_curve(matcher)?;
        for curve in curves.iter_mut() {
            std::mem::take(curve).destroy(self.conn)?;
        }
        Ok(())
    }
    pub fn put_lending_optimizer_distribution(
        self: &Arc<Self>,
        vault_name: String,
        standalone_vault_platforms: Vec<String>,
        standalone_vault_deposited_balances: Vec<f64>,
    ) -> Result<()> {
        self.conn.transaction::<_, anyhow::Error, _>(|| {
            let mut results = DBClient::__get_lending_optimizer_distribution(
                self.conn,
                &LendingOptimizerDistributionMatcher::VaultName(vec![vault_name.clone()]),
            )?;
            if results.is_empty() {
                // create the vault
                NewLendingOptimizerDistribution {
                    vault_name,
                    standalone_vault_platforms,
                    standalone_vault_deposited_balances,
                }
                .save(self.conn)?;
            } else {
                // update the vault
                let mut distribution_stats = std::mem::take(&mut results[0]);
                distribution_stats.standalone_vault_deposited_balances =
                    standalone_vault_deposited_balances;
                distribution_stats.standalone_vault_platforms = standalone_vault_platforms;
                distribution_stats.save(self.conn)?;
            }
            Ok(())
        })?;
        Ok(())
    }
    pub fn get_lending_optimizer_distribution(
        self: &Arc<Self>,
        matcher: &LendingOptimizerDistributionMatcher,
    ) -> QueryResult<Vec<LendingOptimizerDistribution>> {
        DBClient::__get_lending_optimizer_distribution(self.conn, matcher)
    }
    fn __get_lending_optimizer_distribution(
        conn: &PgConnection,
        matcher: &LendingOptimizerDistributionMatcher,
    ) -> QueryResult<Vec<LendingOptimizerDistribution>> {
        get_lending_optimizer_distribution(conn, matcher)
    }
    pub fn delete_lending_optimizer_distribution(
        self: &Arc<Self>,
        matcher: &LendingOptimizerDistributionMatcher,
    ) -> Result<()> {
        let mut results = self.get_lending_optimizer_distribution(matcher)?;
        for result in results.iter_mut() {
            std::mem::take(result).destroy(self.conn)?;
        }
        Ok(())
    }
    /// returns all matching interest rate moving averages
    pub fn get_interest_rate_moving_average(
        self: &Arc<Self>,
        matcher: &InterestRateMovingAverageMatcher,
    ) -> QueryResult<Vec<InterestRateMovingAverage>> {
        matcher
            .to_filter()
            .into_query()
            .get_results::<InterestRateMovingAverage>(self.conn)
    }
    /// operates under the assumption that each matcher returns the same number
    /// of elements, and they are ordered correctly
    ///
    /// todo(bonedaddy): sort
    pub fn get_interest_rate_with_moving_average(
        self: &Arc<Self>,
        ma_matcher: &InterestRateMovingAverageMatcher,
        rate_matcher: &InterestRateMatcher,
    ) -> Result<Vec<(InterestRateMovingAverage, InterestRate)>> {
        let mut response = None;
        //  run everything in a transaction so results are loaded from the same scraped time
        self.conn.transaction::<_, anyhow::Error, _>(|| {
            let moving_averages = ma_matcher
                .to_filter()
                .into_query()
                .get_results::<InterestRateMovingAverage>(self.conn)?;
            let interest_rates = get_interest_rate(self.conn, rate_matcher, Some(1))?;
            if moving_averages.is_empty() {
                return Err(anyhow!("failed to find moving averages"));
            }
            if interest_rates.is_empty() {
                return Err(anyhow!("failed to find interest rates"));
            }
            let mut combos = Vec::with_capacity(interest_rates.len());
            for pair in moving_averages.into_iter().zip(interest_rates) {
                combos.push(pair)
            }
            response = Some(combos.to_owned());
            Ok(())
        })?;
        if let Some(response) = response {
            Ok(response)
        } else {
            Err(anyhow!("failed to find interest rates"))
        }
    }
    pub fn delete_interest_rate_moving_average(
        self: &Arc<Self>,
        matcher: &InterestRateMovingAverageMatcher,
    ) -> Result<()> {
        let mut results = self.get_interest_rate_moving_average(matcher)?;
        for result in results.iter_mut() {
            std::mem::take(result).destroy(self.conn)?;
        }
        Ok(())
    }
    pub fn put_advertised_yield(
        self: &Arc<Self>,
        vault_address: &str,
        farm_name: &str,
        apr: f64,
        scraped_at: DateTime<Utc>,
    ) -> Result<()> {
        self.conn.transaction::<_, anyhow::Error, _>(|| {
            let mut results = DBClient::__get_advertised_yield(
                self.conn,
                &AdvertisedYieldMatcher::FarmName(vec![farm_name.to_string()]),
            )?;

            if results.is_empty() {
                NewAdvertisedYield {
                    vault_address: vault_address.to_owned(),
                    farm_name: farm_name.to_owned(),
                    apr,
                    scraped_at,
                }
                .save(self.conn)?;
                Ok(())
            } else {
                let mut result = std::mem::take(&mut results[0]);
                result.apr = apr;
                result.scraped_at = scraped_at;
                result.save(self.conn)?;
                Ok(())
            }
        })?;
        Ok(())
    }
    pub fn get_advertised_yield(
        self: &Arc<Self>,
        matcher: &AdvertisedYieldMatcher,
    ) -> QueryResult<Vec<AdvertisedYield>> {
        DBClient::__get_advertised_yield(self.conn, matcher)
    }
    fn __get_advertised_yield(
        conn: &PgConnection,
        matcher: &AdvertisedYieldMatcher,
    ) -> QueryResult<Vec<AdvertisedYield>> {
        get_advertised_yield(conn, matcher)
    }
    pub fn delete_advertised_yield(
        self: &Arc<Self>,
        matcher: &AdvertisedYieldMatcher,
    ) -> Result<()> {
        let mut results = self.get_advertised_yield(matcher)?;
        for result in results.iter_mut() {
            std::mem::take(result).destroy(self.conn)?;
        }
        Ok(())
    }
    pub fn put_v1_obligation_account(
        self: &Arc<Self>,
        account: &str,
        authority: &str,
    ) -> Result<()> {
        self.conn.transaction::<_, anyhow::Error, _>(|| {
            let result = DBClient::__get_v1_obligation_account(
                self.conn,
                &V1ObligationAccountMatcher::AccountAddress(vec![account.to_string()]),
            )?;
            if !result.is_empty() {
                return Err(anyhow!("record for account {} already exists", account));
            } else {
                NewV1ObligationAccount {
                    account: account.to_string(),
                    authority: authority.to_string(),
                }
                .save(self.conn)?;
            }
            Ok(())
        })?;
        Ok(())
    }
    pub fn get_v1_obligation_account(
        self: &Arc<Self>,
        matcher: &V1ObligationAccountMatcher,
    ) -> QueryResult<Vec<V1ObligationAccount>> {
        DBClient::__get_v1_obligation_account(self.conn, matcher)
    }
    /// returns all obligations sorted by ltv
    pub fn get_v1_obligation_ltv_sorted(
        self: &Arc<Self>,
        matcher: &V1ObligationLtvMatcher,
    ) -> QueryResult<Vec<V1ObligationLtv>> {
        // for some reason we cant seem to use sql filter methods, i think diesel doesnt support it on f64 fields
        let mut results = DBClient::__get_v1_obligation_ltv(self.conn, matcher)?;
        results.sort_unstable_by(cmp_ltvs);
        Ok(results)
    }
    pub fn delete_v1_obligation_account(
        self: &Arc<Self>,
        matcher: &V1ObligationAccountMatcher,
    ) -> Result<()> {
        let mut results = self.get_v1_obligation_account(matcher)?;
        for result in results.iter_mut() {
            std::mem::take(result).destroy(self.conn)?;
        }
        Ok(())
    }
    /// unlike the obligation_account table which simply serves as a database
    /// of all obligation accounts, this stores detailed information about the obligation
    /// including ltv, the associated user farm, and leveraged farm, etc..
    pub fn put_v1_obligation_ltv(
        self: &Arc<Self>,
        authority: &str,
        user_farm: &str,
        account_address: &str,
        leveraged_farm: &str,
        ltv: f64,
        scraped_at: DateTime<Utc>,
    ) -> Result<()> {
        self.conn.transaction::<_, anyhow::Error, _>(|| {
            let mut result = DBClient::__get_v1_obligation_ltv(
                self.conn,
                &V1ObligationLtvMatcher::AccountAddress(vec![account_address.to_string()]),
            )?;
            if result.is_empty() {
                NewV1ObligationLtv {
                    authority: authority.to_string(),
                    user_farm: user_farm.to_string(),
                    account_address: account_address.to_string(),
                    leveraged_farm: leveraged_farm.to_string(),
                    ltv,
                    scraped_at,
                }
                .save(self.conn)?;
            } else {
                // update ltv and scraped_at values
                // wed apr 20 2022:
                //      * temporarily updating all values to fix a bug involving authority being set to user farm
                result[0].authority = authority.to_string();
                result[0].user_farm = user_farm.to_string();
                result[0].account_address = account_address.to_string();
                result[0].leveraged_farm = leveraged_farm.to_string();
                result[0].ltv = ltv;
                result[0].scraped_at = scraped_at;
                std::mem::take(&mut result[0]).save(self.conn)?;
            }
            Ok(())
        })?;
        Ok(())
    }
    pub fn get_v1_obligation_ltv(
        self: &Arc<Self>,
        matcher: &V1ObligationLtvMatcher,
    ) -> QueryResult<Vec<V1ObligationLtv>> {
        DBClient::__get_v1_obligation_ltv(self.conn, matcher)
    }
    pub fn delete_v1_obligation_ltv(
        self: &Arc<Self>,
        matcher: &V1ObligationLtvMatcher,
    ) -> Result<()> {
        let mut results = self.get_v1_obligation_ltv(matcher)?;
        for result in results.iter_mut() {
            std::mem::take(result).destroy(self.conn)?;
        }
        Ok(())
    }
    pub fn put_v1_user_farm(
        self: &Arc<Self>,
        authority: &str,
        account_address: &str,
        leveraged_farm: &str,
        obligations: &[String],
        obligation_indexes: &[i32],
    ) -> Result<()> {
        self.conn.transaction::<_, anyhow::Error, _>(|| {
            let mut result = DBClient::__get_v1_user_farm(
                self.conn,
                &V1UserFarmMatcher::AccountAddress(vec![account_address.to_string()]),
            )?;
            if result.is_empty() {
                NewV1UserFarm {
                    authority: authority.to_string(),
                    account_address: account_address.to_string(),
                    obligations: obligations.to_vec(),
                    obligation_indexes: obligation_indexes.to_vec(),
                    leveraged_farm: leveraged_farm.to_string(),
                }
                .save(self.conn)?;
            } else {
                result[0].obligations = obligations.to_vec();
                result[0].obligation_indexes = obligation_indexes.to_vec();
                // may 4th: to avoid compelx migrations and to have the data self correct
                //          simply set levergaed_farm
                result[0].leveraged_farm = leveraged_farm.to_string();
                std::mem::take(&mut result[0]).save(self.conn)?;
            }
            Ok(())
        })?;
        Ok(())
    }
    pub fn get_v1_user_farm(
        self: &Arc<Self>,
        matcher: &V1UserFarmMatcher,
    ) -> QueryResult<Vec<V1UserFarm>> {
        DBClient::__get_v1_user_farm(self.conn, matcher)
    }
    pub fn delete_v1_user_farm(self: &Arc<Self>, matcher: &V1UserFarmMatcher) -> Result<()> {
        let mut results = self.get_v1_user_farm(matcher)?;
        for result in results.iter_mut() {
            std::mem::take(result).destroy(self.conn)?;
        }
        Ok(())
    }
    pub fn put_v1_liquidated_position(
        self: &Arc<Self>,
        temp_liquidation_account: &str,
        authority: &str,
        user_farm: &str,
        liquidation_event_id: &str,
        obligation: &str,
        leveraged_farm: &str,
        started_at: DateTime<Utc>,
        ended_at: Option<DateTime<Utc>>,
    ) -> Result<()> {
        self.conn.transaction::<_, anyhow::Error, _>(|| {
            let mut results = DBClient::__get_v1_liquidate_position(
                self.conn,
                &V1LiquidatedPositionMatcher::LiquidationEventId(vec![
                    liquidation_event_id.to_string()
                ]),
            )?;
            if results.is_empty() {
                // create  the position information
                NewV1LiquidatedPosition {
                    authority: authority.to_string(),
                    liquidation_event_id: liquidation_event_id.to_string(),
                    temp_liquidation_account: temp_liquidation_account.to_string(),
                    user_farm: user_farm.to_string(),
                    obligation: obligation.to_string(),
                    leveraged_farm: leveraged_farm.to_string(),
                    started_at,
                    ended_at,
                }
                .save(self.conn)?;
            } else if let Some(ended_at) = ended_at {
                results[0].ended_at = Some(ended_at);
                // may 4th: to fix any data we previously collected
                //          which may have not had its leveraged farm information set
                //          do so here
                results[0].leveraged_farm = leveraged_farm.to_string();
                std::mem::take(&mut results[0]).save(self.conn)?;
            } else {
                return Err(
                    DBError::V1PositionAlreadyExists(temp_liquidation_account.to_string()).into(),
                );
            }
            Ok(())
        })?;
        Ok(())
    }
    pub fn get_v1_liquidated_position(
        self: &Arc<Self>,
        matcher: &V1LiquidatedPositionMatcher,
    ) -> QueryResult<Vec<V1LiquidatedPosition>> {
        DBClient::__get_v1_liquidate_position(self.conn, matcher)
    }
    pub fn delete_v1_liquidated_position(
        self: &Arc<Self>,
        matcher: &V1LiquidatedPositionMatcher,
    ) -> Result<()> {
        let mut results = self.get_v1_liquidated_position(matcher)?;
        for result in results.iter_mut() {
            std::mem::take(result).destroy(self.conn)?;
        }
        Ok(())
    }
    pub fn put_historic_tshare_price(
        self: &Arc<Self>,
        farm_name: &str,
        price: f64,
        total_supply: f64,
        holder_count: f64,
        scraped_at: DateTime<Utc>,
    ) -> Result<()> {
        NewHistoricTSharePrice {
            farm_name: farm_name.to_string(),
            price,
            total_supply,
            holder_count,
            scraped_at,
        }
        .save(self.conn)?;
        Ok(())
    }
    pub fn get_historic_tshare_price(
        self: &Arc<Self>,
        matcher: &HistoricTSharePriceMatcher,
    ) -> QueryResult<Vec<HistoricTsharePrice>> {
        get_historic_tshare_price(self.conn, matcher)
    }
    pub fn delete_historic_tshare_price(
        self: &Arc<Self>,
        matcher: &HistoricTSharePriceMatcher,
    ) -> Result<()> {
        let mut results = self.get_historic_tshare_price(matcher)?;
        for result in results.iter_mut() {
            std::mem::take(result).destroy(self.conn)?;
        }
        Ok(())
    }
    fn __get_v1_user_farm(
        conn: &PgConnection,
        matcher: &V1UserFarmMatcher,
    ) -> QueryResult<Vec<V1UserFarm>> {
        Ok(query_paginated_v1_user_farms(conn, matcher, None, None)?.0)
    }
    fn __get_v1_obligation_ltv(
        conn: &PgConnection,
        matcher: &V1ObligationLtvMatcher,
    ) -> QueryResult<Vec<V1ObligationLtv>> {
        Ok(query_paginated_v1_obligations(conn, matcher, None, None)?.0)
    }
    fn __get_v1_liquidate_position(
        conn: &PgConnection,
        matcher: &V1LiquidatedPositionMatcher,
    ) -> QueryResult<Vec<V1LiquidatedPosition>> {
        Ok(query_paginated_v1_liquidated_positions(conn, matcher, None, None)?.0)
    }
    fn __get_v1_obligation_account(
        conn: &PgConnection,
        matcher: &V1ObligationAccountMatcher,
    ) -> QueryResult<Vec<V1ObligationAccount>> {
        Ok(matcher
            .to_filter()
            .into_query()
            .get_results::<V1ObligationAccount>(conn)?)
    }
}

pub fn get_token_price(
    conn: &PgConnection,
    matcher: &TokenPriceMatcher,
    limit: Option<i64>,
) -> QueryResult<Vec<TokenPrice>> {
    let query = matcher.to_filter().into_query();
    let query = if let Some(limit) = limit {
        query.limit(limit)
    } else {
        query
    };
    query.get_results::<TokenPrice>(conn)
}

pub fn get_realized_yield(
    conn: &PgConnection,
    matcher: &RealizeYieldMatcher,
    limit: Option<i64>,
) -> QueryResult<Vec<RealizeYield>> {
    use crate::schema::realize_yield::dsl::*;
    let query = matcher.to_filter().into_query();
    let query = if let Some(limit) = limit {
        // if af a limit of 1 is given, return the most recent result
        query.order(scraped_at.desc()).limit(limit)
    } else {
        query
    };
    query.get_results::<RealizeYield>(conn)
}

pub fn get_interest_rate(
    conn: &PgConnection,
    matcher: &InterestRateMatcher,
    limit: Option<i64>,
) -> QueryResult<Vec<InterestRate>> {
    use crate::schema::interest_rate::dsl::*;
    let query = matcher.to_filter().into_query();
    let query = if let Some(limit) = limit {
        // ensure we select the most recently scraped interest rate sample
        query.order(scraped_at.desc()).limit(limit)
    } else {
        query
    };
    query.get_results::<InterestRate>(conn)
}

pub fn get_advertised_yield(
    conn: &PgConnection,
    matcher: &AdvertisedYieldMatcher,
) -> QueryResult<Vec<AdvertisedYield>> {
    matcher
        .to_filter()
        .into_query()
        .get_results::<AdvertisedYield>(conn)
}

pub fn get_vault_tvl(
    conn: &PgConnection,
    matcher: &VaultTvlMatcher,
    limit: Option<i64>,
) -> QueryResult<Vec<VaultTvl>> {
    use crate::schema::vault_tvl::dsl::*;
    let query = matcher.to_filter().into_query();
    let query = if let Some(limit) = limit {
        // ensure we select the most recently scraped interest rate sample
        query.order(scraped_at.desc()).limit(limit)
    } else {
        query
    };
    query.get_results::<VaultTvl>(conn)
}

/// optionally enables querying for v1 obligations
/// using pagination to control which results are returned.
///
/// when pagination is enabled, the 2nd element returns the total number of results
///
/// when pagination is not enabled, the 2nd element is 0
pub fn query_paginated_v1_obligations(
    conn: &PgConnection,
    matcher: &V1ObligationLtvMatcher,
    page: Option<i64>,
    per_page: Option<i64>,
) -> QueryResult<(Vec<V1ObligationLtv>, i64)> {
    let query = matcher.to_filter().into_query();
    if page.is_some() && per_page.is_some() {
        query.paginate(page).per_page(per_page).load_and_count(conn)
    } else {
        let results = query.get_results::<V1ObligationLtv>(conn)?;
        Ok((results, 0))
    }
}

/// optionally enables querying for v1 user farms
/// using pagination to control which results are returned.
///
/// when pagination is enabled, the 2nd element returns the total number of results
///
/// when pagination is not enabled, the 2nd element is 0
pub fn query_paginated_v1_user_farms(
    conn: &PgConnection,
    matcher: &V1UserFarmMatcher,
    page: Option<i64>,
    per_page: Option<i64>,
) -> QueryResult<(Vec<V1UserFarm>, i64)> {
    let query = matcher.to_filter().into_query();
    if page.is_some() && per_page.is_some() {
        query.paginate(page).per_page(per_page).load_and_count(conn)
    } else {
        let results = query.get_results::<V1UserFarm>(conn)?;
        Ok((results, 0))
    }
}

pub fn query_paginated_v1_liquidated_positions(
    conn: &PgConnection,
    matcher: &V1LiquidatedPositionMatcher,
    page: Option<i64>,
    per_page: Option<i64>,
) -> QueryResult<(Vec<V1LiquidatedPosition>, i64)> {
    let query = matcher.to_filter().into_query();
    if page.is_some() && per_page.is_some() {
        query.paginate(page).per_page(per_page).load_and_count(conn)
    } else {
        let results = query.get_results::<V1LiquidatedPosition>(conn)?;
        Ok((results, 0))
    }
}

pub fn get_lending_optimizer_distribution(
    conn: &PgConnection,
    matcher: &LendingOptimizerDistributionMatcher,
) -> QueryResult<Vec<LendingOptimizerDistribution>> {
    matcher
        .to_filter()
        .into_query()
        .get_results::<LendingOptimizerDistribution>(conn)
}

pub fn get_deposit_tracking_account(
    conn: &PgConnection,
    matcher: &DepositTrackingMatcher,
) -> QueryResult<Vec<DepositTracking>> {
    matcher
        .to_filter()
        .into_query()
        .get_results::<DepositTracking>(conn)
}

pub fn get_vault_account(conn: &PgConnection, matcher: &VaultMatcher) -> QueryResult<Vec<Vault>> {
    matcher.to_filter().into_query().get_results::<Vault>(conn)
}

pub fn get_historic_tshare_price(
    conn: &PgConnection,
    matcher: &HistoricTSharePriceMatcher,
) -> QueryResult<Vec<HistoricTsharePrice>> {
    matcher
        .to_filter()
        .into_query()
        .get_results::<HistoricTsharePrice>(conn)
}

pub fn query_paginated_historic_prices(
    conn: &PgConnection,
    matcher: &HistoricTSharePriceMatcher,
    started_at: DateTime<Utc>,
    ended_at: DateTime<Utc>,
    page: Option<i64>,
    per_page: Option<i64>,
    sort_by_time: bool,
) -> QueryResult<(Vec<HistoricTsharePrice>, i64)> {
    use crate::schema::historic_tshare_price::dsl;
    let query = matcher
        .to_filter()
        .into_query()
        .filter(dsl::scraped_at.between(started_at, ended_at));
    let query = if sort_by_time {
        query.order_by(dsl::scraped_at.asc())
    } else {
        query
    };
    if page.is_some() && per_page.is_some() {
        query.paginate(page).per_page(per_page).load_and_count(conn)
    } else {
        let results = query.get_results::<HistoricTsharePrice>(conn)?;
        Ok((results, 0))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::env;
    use std::str::FromStr;
    #[test]
    #[allow(unused_must_use)]
    fn test_vault_tvl() {
        env::set_var(
            "DATABASE_URL",
            "postgres://postgres:password123@localhost/tulip",
        );
        // we need at least one test which uses the establish_connection function
        let conn = DBClient::establish_connection(
            "postgres://postgres:password123@localhost/tulip".to_string(),
        );
        let client = Arc::new(DBClient {
            conn: &conn,
            oob_limit: 25.0,
        });

        crate::run_migrations(client.conn);
        std::thread::sleep(std::time::Duration::from_secs(2));
        let cleanup = || {
            client.delete_vault_tvl(&VaultTvlMatcher::All);
        };
        cleanup();
        std::thread::sleep(std::time::Duration::from_secs(2));
        let farm_name = "farm-one";
        let total_shares = 100_f64;
        let total_underlying = 101_f64;
        let value_locked = 102_f64;
        client
            .put_vault_tvl(
                farm_name.to_string(),
                total_shares,
                total_underlying,
                value_locked,
                Utc::now(),
            )
            .unwrap();
        let tvls = client
            .get_vault_tvl(&VaultTvlMatcher::FarmName(vec![farm_name.to_string()]))
            .unwrap();
        assert!(tvls.len() == 1);
        assert_eq!(tvls[0].total_shares, total_shares);
        assert_eq!(tvls[0].total_underlying, total_underlying);
        assert_eq!(tvls[0].value_locked, value_locked);
        let total_shares2 = 1002_f64;
        let total_underlying2 = 1012_f64;
        let value_locked2 = 1022_f64;
        client
            .put_vault_tvl(
                farm_name.to_string(),
                total_shares2,
                total_underlying2,
                value_locked2,
                Utc::now(),
            )
            .unwrap();
        let tvls = client
            .get_vault_tvl(&VaultTvlMatcher::FarmName(vec![farm_name.to_string()]))
            .unwrap();
        assert!(tvls.len() == 2);
        assert_eq!(tvls[0].total_shares, total_shares);
        assert_eq!(tvls[0].total_underlying, total_underlying);
        assert_eq!(tvls[0].value_locked, value_locked);
        assert_eq!(tvls[1].total_shares, total_shares2);
        assert_eq!(tvls[1].total_underlying, total_underlying2);
        assert_eq!(tvls[1].value_locked, value_locked2);
        cleanup();
    }
    #[test]
    #[allow(unused_must_use)]
    fn test_new_token_balance() {
        use crate::test_utils::TestDb;
        env::set_var(
            "DATABASE_URL",
            "postgres://postgres:password123@localhost/tulip",
        );
        let test_db = TestDb::new();
        let conn = test_db.conn();
        let client = Arc::new(DBClient {
            conn: &conn,
            oob_limit: 25.0,
        });

        crate::run_migrations(client.conn);
        std::thread::sleep(std::time::Duration::from_secs(2));
        let cleanup = || {
            client.delete_token_balance(&TokenBalanceMatcher::All);
        };
        cleanup();
        std::thread::sleep(std::time::Duration::from_secs(2));
        let account_one = String::from("account-1");
        let mint_one = String::from("mint-1");
        let identifier_one = String::from("identifier-1");
        let account_two = String::from("account-2");
        let _identifier_two = String::from("identifier-2");
        let balance_one = 1_f64;
        let balance_two = 2_f64;
        let scraped_at = Utc::now();
        client.put_token_balance(
            account_one.clone(),
            mint_one.clone(),
            identifier_one.clone(),
            balance_one,
            scraped_at,
        );

        let balances = client
            .get_token_balance(&TokenBalanceMatcher::Account(vec![account_one.clone()]))
            .unwrap();
        assert!(balances.len() == 1);
        let balances = client.get_token_balance(&TokenBalanceMatcher::All).unwrap();
        assert!(balances.len() == 1);
        assert_eq!(balances[0].balance, 1_f64);
        assert_eq!(balances[0].token_account, account_one);
        assert_eq!(balances[0].token_mint, mint_one);

        client.put_token_balance(
            account_one.clone(),
            mint_one.clone(),
            identifier_one.clone(),
            balance_two,
            scraped_at,
        );
        let balances = client
            .get_token_balance(&TokenBalanceMatcher::Account(vec![account_one.clone()]))
            .unwrap();
        assert!(balances.len() == 2);
        let balances = client.get_token_balance(&TokenBalanceMatcher::All).unwrap();
        assert!(balances.len() == 2);
        assert_eq!(balances[1].balance, balance_two);
        assert_eq!(balances[1].token_account, account_one);
        assert_eq!(balances[1].token_mint, mint_one);

        client.put_token_balance(
            account_two.clone(),
            mint_one.clone(),
            identifier_one,
            balance_two,
            scraped_at,
        );
        let balances = client
            .get_token_balance(&TokenBalanceMatcher::Account(vec![account_one]))
            .unwrap();
        assert!(balances.len() == 2);
        let balances = client.get_token_balance(&TokenBalanceMatcher::All).unwrap();
        assert!(balances.len() == 3);
        assert_eq!(balances[2].balance, balance_two);
        assert_eq!(balances[2].token_account, account_two);
        assert_eq!(balances[2].token_mint, mint_one);

        cleanup();
        let balances = client.get_token_balance(&TokenBalanceMatcher::All).unwrap();
        assert!(balances.is_empty());
    }
    #[test]
    #[allow(unused_must_use)]
    fn test_new_token_price() {
        use crate::test_utils::TestDb;
        env::set_var(
            "DATABASE_URL",
            "postgres://postgres:password123@localhost/tulip",
        );
        let test_db = TestDb::new();
        let conn = test_db.conn();
        let client = Arc::new(DBClient {
            conn: &conn,
            oob_limit: 25.0,
        });

        crate::run_migrations(client.conn);
        std::thread::sleep(std::time::Duration::from_secs(2));
        let cleanup = || {
            client.delete_token_price(&TokenPriceMatcher::All);
        };
        cleanup();
        std::thread::sleep(std::time::Duration::from_secs(2));
        // test the create route
        client
            .put_token_price("token1", "platform1", 420.69, 69.420, 69.69, "mint1")
            .unwrap();
        let price = client
            .get_token_price(&TokenPriceMatcher::Asset(vec!["token1".to_string()]), None)
            .unwrap();
        assert!(price[0].asset.eq(&"token1"));
        assert!(price[0].platform.eq(&"platform1"));
        assert!(price[0].price.eq(&420.69));
        assert!(price[0].coin_in_lp.eq(&69.420));
        assert!(price[0].pc_in_lp.eq(&69.69));
        assert!(price[0].token_mint.eq(&"mint1"));
        assert_eq!(price[0].period_observed_prices.len(), 1);
        assert_eq!(price[0].period_running_average, 420.69);
        assert!(!price[0].feed_stopped);
        // test the update route
        client
            .put_token_price("token1", "platform1", 420.42, 42.42, 42.43, "mint2")
            .unwrap();
        let price = client
            .get_token_price(&TokenPriceMatcher::Asset(vec!["token1".to_string()]), None)
            .unwrap();
        assert!(price[0].asset.eq(&"token1"));
        assert!(price[0].platform.eq(&"platform1"));
        assert!(price[0].price.eq(&420.42));
        assert!(price[0].coin_in_lp.eq(&42.42));
        assert!(price[0].pc_in_lp.eq(&42.43));
        // we check here to ensure that the price update when given a different token mint doesn ot accidentally override the token mint
        assert!(price[0].token_mint.eq(&"mint1"));
        assert_eq!(price[0].period_observed_prices.len(), 2);
        assert_eq!(price[0].period_running_average, 420.555);
        // test the create route
        client
            .put_token_price("token2", "platform2", 69.69, 69.420, 69.69, "mint2")
            .unwrap();
        let price = client
            .get_token_price(&TokenPriceMatcher::Asset(vec!["token2".to_string()]), None)
            .unwrap();
        assert!(price[0].asset.eq(&"token2"));
        assert!(price[0].platform.eq(&"platform2"));
        assert!(price[0].price.eq(&69.69));
        assert!(price[0].coin_in_lp.eq(&69.420));
        assert!(price[0].pc_in_lp.eq(&69.69));
        assert!(price[0].token_mint.eq(&"mint2"));
        // test the update route
        client
            .put_token_price("token2", "platform2", 69.1337, 69.420, 69.69, "mint3")
            .unwrap();
        let price = client
            .get_token_price(&TokenPriceMatcher::Asset(vec!["token2".to_string()]), None)
            .unwrap();
        assert!(price[0].asset.eq(&"token2"));
        assert!(price[0].platform.eq(&"platform2"));
        assert!(price[0].price.eq(&69.1337));
        assert!(price[0].coin_in_lp.eq(&69.420));
        assert!(price[0].pc_in_lp.eq(&69.69));
        // another validation that update doesnt update token mint
        assert!(price[0].token_mint.eq(&"mint2"));

        let price = client
            .get_token_price(&TokenPriceMatcher::Asset(vec!["token2".to_string()]), None)
            .unwrap();
        assert!(price[0].asset.eq(&"token2"));
        assert!(price[0].platform.eq(&"platform2"));
        assert!(price[0].price.eq(&69.1337));
        assert!(price[0].coin_in_lp.eq(&69.420));
        assert!(price[0].pc_in_lp.eq(&69.69));
        assert!(price[0].token_mint.eq(&"mint2"));
        assert_eq!(price[0].asset_identifier, "platform2-token2".to_string());
        client
            .put_token_price(
                // lp = ORCA-USDC
                // platform = ORCA
                "ORCA-ORCA-USDC",
                "ORCA",
                420.69,
                69.420,
                46920_f64,
                "mint3",
            )
            .unwrap();
        let price = client
            .get_token_price(
                &TokenPriceMatcher::AssetIdentifier(vec!["ORCA-ORCA-USDC".to_string()]),
                None,
            )
            .unwrap();
        println!("prices {:#?}", price);
        assert_eq!(price[0].asset, "ORCA-ORCA-USDC".to_string());
        assert!(price[0].platform.eq(&"ORCA"));
        assert!(price[0].price.eq(&420.69));
        assert!(price[0].coin_in_lp.eq(&69.42));
        assert!(price[0].pc_in_lp.eq(&46920.0));
        assert_eq!(price[0].asset_identifier, "ORCA-ORCA-USDC".to_string());
        assert_eq!(price[0].token_mint, "mint3");
        cleanup();
    }
    #[test]
    #[allow(unused_must_use)]
    fn test_new_new_vault() {
        use crate::test_utils::TestDb;
        env::set_var(
            "DATABASE_URL",
            "postgres://postgres:password123@localhost/tulip",
        );
        let test_db = TestDb::new();
        let conn = test_db.conn();
        let client = Arc::new(DBClient {
            conn: &conn,
            oob_limit: 25.0,
        });
        #[allow(unused_must_use)]
        crate::run_migrations(client.conn);
        std::thread::sleep(std::time::Duration::from_secs(2));

        let vault_acct = client
            .get_vault_account(&VaultMatcher::FarmName(vec!["test_farm".to_string()]))
            .unwrap();
        assert_eq!(vault_acct.len(), 0);
        let time_1 = Utc::now();
        client
            .put_vault_account(
                "test_farm".to_string(),
                "test_address".to_string(),
                "test_data".as_bytes().to_vec(),
                time_1,
                None,
            )
            .unwrap();

        let vault_acct = client
            .get_vault_account(&VaultMatcher::FarmName(vec!["test_farm".to_string()]))
            .unwrap();
        assert_eq!(vault_acct.len(), 1);
        // for some reason directly comparing the time objects occasionally results in nanoseconds being different
        // so compare everything but that
        assert_eq!(&vault_acct[0].scraped_at.day(), &time_1.day());
        assert_eq!(&vault_acct[0].scraped_at.hour(), &time_1.hour());
        assert_eq!(&vault_acct[0].scraped_at.minute(), &time_1.minute());
        assert_eq!(&vault_acct[0].scraped_at.second(), &time_1.second());

        let time_2 = Utc::now();
        client
            .put_vault_account(
                "test_farm".to_string(),
                "test_address".to_string(),
                "test_data".as_bytes().to_vec(),
                time_2,
                None,
            )
            .unwrap();
        let vault_acct = client
            .get_vault_account(&VaultMatcher::FarmName(vec!["test_farm".to_string()]))
            .unwrap();
        assert_eq!(vault_acct.len(), 1);
        assert_eq!(&vault_acct[0].scraped_at.day(), &time_2.day());
        assert_eq!(&vault_acct[0].scraped_at.hour(), &time_2.hour());
        assert_eq!(&vault_acct[0].scraped_at.minute(), &time_2.minute());
        assert_eq!(&vault_acct[0].scraped_at.second(), &time_2.second());

        let vault_acct = client
            .get_vault_account(&VaultMatcher::Account(vec!["test_address".to_string()]))
            .unwrap();
        assert_eq!(vault_acct.len(), 1);

        client
            .delete_vault_account(&VaultMatcher::FarmName(vec!["test_farm".to_string()]))
            .unwrap();

        let vault_acct = client
            .get_vault_account(&VaultMatcher::FarmName(vec!["test_farm".to_string()]))
            .unwrap();
        assert_eq!(vault_acct.len(), 0);

        let vault_acct = client
            .get_vault_account(&VaultMatcher::FarmName(vec!["test_address".to_string()]))
            .unwrap();
        assert_eq!(vault_acct.len(), 0);
    }
    #[test]
    #[allow(unused_must_use)]
    fn test_new_deposit_tracking_account() {
        use crate::test_utils::TestDb;
        env::set_var(
            "DATABASE_URL",
            "postgres://postgres:password123@localhost/tulip",
        );
        let test_db = TestDb::new();
        let conn = test_db.conn();
        let client = Arc::new(DBClient {
            conn: &conn,
            oob_limit: 25.0,
        });
        crate::run_migrations(client.conn);
        std::thread::sleep(std::time::Duration::from_secs(2));

        let owner_address = String::from("owner1");
        let account_address_1 = String::from("acct1");
        let account_data_1 = String::from("acct1-data");
        let vault_account_1 = String::from("vault-acct1");
        let current_balance_1 = 10_f64;
        let current_shares_1 = 10_f64;
        let balance_usd_value_1 = 100_f64;

        let account_address_2 = String::from("acct2");
        let account_data_2 = String::from("acct2-data");
        let vault_account_2 = String::from("vault-acct2");
        let current_balance_2 = 20_f64;
        let current_shares_2 = 20_f64;
        let balance_usd_value_2 = 200_f64;

        let owner_address2 = String::from("owner2");
        let account_address_3 = String::from("acct3");
        let account_data_3 = String::from("acct3-data");
        let vault_account_3 = String::from("vault-acct3");
        let current_balance_3 = 30_f64;
        let current_shares_3 = 30_f64;
        let balance_usd_value_3 = 300_f64;

        let cleanup = || {
            client
                .delete_deposit_tracking_account(&DepositTrackingMatcher::Account(vec![
                    "acct1".to_string()
                ]))
                .unwrap();
            client
                .delete_deposit_tracking_account(&DepositTrackingMatcher::Account(vec![
                    "acct2".to_string()
                ]))
                .unwrap();
            client
                .delete_deposit_tracking_account(&DepositTrackingMatcher::Account(vec![
                    "acct3".to_string()
                ]))
                .unwrap();
        };
        let time_1 = Utc::now();
        // test creating accounts
        {
            client
                .put_deposit_tracking_account(
                    owner_address.clone(),
                    account_address_1.clone(),
                    account_data_1.as_bytes().to_vec(),
                    vault_account_1.clone(),
                    time_1,
                    current_balance_1,
                    current_shares_1,
                    balance_usd_value_1,
                )
                .unwrap();
            client
                .put_deposit_tracking_account(
                    owner_address.clone(),
                    account_address_2.clone(),
                    account_data_2.as_bytes().to_vec(),
                    vault_account_2.clone(),
                    time_1,
                    current_balance_2,
                    current_shares_2,
                    balance_usd_value_2,
                )
                .unwrap();
            client
                .put_deposit_tracking_account(
                    owner_address2.clone(),
                    account_address_3.clone(),
                    account_data_3.as_bytes().to_vec(),
                    vault_account_3.clone(),
                    time_1,
                    current_balance_3,
                    current_shares_3,
                    balance_usd_value_3,
                )
                .unwrap();
        }
        // test finding accounts by address
        {
            let results = client
                .get_deposit_tracking_account(&DepositTrackingMatcher::Account(vec![
                    account_address_1.clone(),
                ]))
                .unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(&results[0].account_address, &account_address_1);
            assert_eq!(&results[0].owner_address, &owner_address);
            assert_eq!(&results[0].vault_account_address, &vault_account_1);
            assert_eq!(&results[0].current_balance, &current_balance_1);
            assert_eq!(&results[0].current_shares, &current_shares_1);
            assert_eq!(&results[0].balance_usd_value, &balance_usd_value_1);
            assert_eq!(
                &results[0].account_data,
                &account_data_1.as_bytes().to_vec()
            );
            assert_eq!(&results[0].scraped_at.day(), &time_1.day());
            assert_eq!(&results[0].scraped_at.hour(), &time_1.hour());
            assert_eq!(&results[0].scraped_at.minute(), &time_1.minute());
            assert_eq!(&results[0].scraped_at.second(), &time_1.second());

            let results = client
                .get_deposit_tracking_account(&DepositTrackingMatcher::Account(vec![
                    account_address_2.clone(),
                ]))
                .unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(&results[0].account_address, &account_address_2);
            assert_eq!(&results[0].owner_address, &owner_address);
            assert_eq!(&results[0].vault_account_address, &vault_account_2);
            assert_eq!(&results[0].current_balance, &current_balance_2);
            assert_eq!(&results[0].current_shares, &current_shares_2);
            assert_eq!(&results[0].balance_usd_value, &balance_usd_value_2);
            assert_eq!(
                &results[0].account_data,
                &account_data_2.as_bytes().to_vec()
            );
            assert_eq!(&results[0].scraped_at.day(), &time_1.day());
            assert_eq!(&results[0].scraped_at.hour(), &time_1.hour());
            assert_eq!(&results[0].scraped_at.minute(), &time_1.minute());
            assert_eq!(&results[0].scraped_at.second(), &time_1.second());

            let results = client
                .get_deposit_tracking_account(&DepositTrackingMatcher::Account(vec![
                    account_address_3.clone(),
                ]))
                .unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(&results[0].account_address, &account_address_3);
            assert_eq!(&results[0].owner_address, &owner_address2);
            assert_eq!(&results[0].vault_account_address, &vault_account_3);
            assert_eq!(&results[0].current_balance, &current_balance_3);
            assert_eq!(&results[0].current_shares, &current_shares_3);
            assert_eq!(&results[0].balance_usd_value, &balance_usd_value_3);
            assert_eq!(
                &results[0].account_data,
                &account_data_3.as_bytes().to_vec()
            );
            assert_eq!(&results[0].scraped_at.day(), &time_1.day());
            assert_eq!(&results[0].scraped_at.hour(), &time_1.hour());
            assert_eq!(&results[0].scraped_at.minute(), &time_1.minute());
            assert_eq!(&results[0].scraped_at.second(), &time_1.second());
        }
        // test finding accounts by vault
        {
            let results = client
                .get_deposit_tracking_account(&DepositTrackingMatcher::Vault(vec![
                    vault_account_1.clone()
                ]))
                .unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(&results[0].account_address, &account_address_1);
            assert_eq!(&results[0].owner_address, &owner_address);
            assert_eq!(&results[0].vault_account_address, &vault_account_1);
            assert_eq!(
                &results[0].account_data,
                &account_data_1.as_bytes().to_vec()
            );
            assert_eq!(&results[0].scraped_at.day(), &time_1.day());
            assert_eq!(&results[0].scraped_at.hour(), &time_1.hour());
            assert_eq!(&results[0].scraped_at.minute(), &time_1.minute());
            assert_eq!(&results[0].scraped_at.second(), &time_1.second());

            let results = client
                .get_deposit_tracking_account(&DepositTrackingMatcher::Vault(vec![
                    vault_account_2.clone()
                ]))
                .unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(&results[0].account_address, &account_address_2);
            assert_eq!(&results[0].owner_address, &owner_address);
            assert_eq!(&results[0].vault_account_address, &vault_account_2);
            assert_eq!(
                &results[0].account_data,
                &account_data_2.as_bytes().to_vec()
            );
            assert_eq!(&results[0].scraped_at.day(), &time_1.day());
            assert_eq!(&results[0].scraped_at.hour(), &time_1.hour());
            assert_eq!(&results[0].scraped_at.minute(), &time_1.minute());
            assert_eq!(&results[0].scraped_at.second(), &time_1.second());

            let results = client
                .get_deposit_tracking_account(&DepositTrackingMatcher::Vault(vec![
                    vault_account_3.clone()
                ]))
                .unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(&results[0].account_address, &account_address_3);
            assert_eq!(&results[0].owner_address, &owner_address2);
            assert_eq!(&results[0].vault_account_address, &vault_account_3);
            assert_eq!(
                &results[0].account_data,
                &account_data_3.as_bytes().to_vec()
            );
            assert_eq!(&results[0].scraped_at.day(), &time_1.day());
            assert_eq!(&results[0].scraped_at.hour(), &time_1.hour());
            assert_eq!(&results[0].scraped_at.minute(), &time_1.minute());
            assert_eq!(&results[0].scraped_at.second(), &time_1.second());
        }
        // test finding accounts by owner
        {
            let results = client
                .get_deposit_tracking_account(&DepositTrackingMatcher::Owner(vec![
                    owner_address.clone()
                ]))
                .unwrap();
            assert_eq!(results.len(), 2);
            assert_eq!(&results[0].account_address, &account_address_1);
            assert_eq!(&results[0].owner_address, &owner_address);
            assert_eq!(&results[0].vault_account_address, &vault_account_1);
            assert_eq!(
                &results[0].account_data,
                &account_data_1.as_bytes().to_vec()
            );
            assert_eq!(&results[1].account_address, &account_address_2);
            assert_eq!(&results[1].owner_address, &owner_address);
            assert_eq!(&results[1].vault_account_address, &vault_account_2);
            assert_eq!(
                &results[1].account_data,
                &account_data_2.as_bytes().to_vec()
            );
            assert_eq!(&results[0].scraped_at.day(), &time_1.day());
            assert_eq!(&results[0].scraped_at.hour(), &time_1.hour());
            assert_eq!(&results[0].scraped_at.minute(), &time_1.minute());
            assert_eq!(&results[0].scraped_at.second(), &time_1.second());

            let results = client
                .get_deposit_tracking_account(&DepositTrackingMatcher::Owner(vec![
                    owner_address2.clone()
                ]))
                .unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(&results[0].account_address, &account_address_3);
            assert_eq!(&results[0].owner_address, &owner_address2);
            assert_eq!(&results[0].vault_account_address, &vault_account_3);
            assert_eq!(
                &results[0].account_data,
                &account_data_3.as_bytes().to_vec()
            );
            assert_eq!(&results[0].scraped_at.day(), &time_1.day());
            assert_eq!(&results[0].scraped_at.hour(), &time_1.hour());
            assert_eq!(&results[0].scraped_at.minute(), &time_1.minute());
            assert_eq!(&results[0].scraped_at.second(), &time_1.second());
        }

        // cleanup accounts
        cleanup();

        // ensure we can create the accounts again since we deleted em
        client
            .put_deposit_tracking_account(
                owner_address.clone(),
                account_address_1,
                account_data_1.as_bytes().to_vec(),
                vault_account_1,
                time_1,
                current_balance_1,
                current_shares_1,
                balance_usd_value_1,
            )
            .unwrap();
        client
            .put_deposit_tracking_account(
                owner_address,
                account_address_2,
                account_data_2.as_bytes().to_vec(),
                vault_account_2,
                time_1,
                current_balance_1,
                current_shares_1,
                balance_usd_value_1,
            )
            .unwrap();
        client
            .put_deposit_tracking_account(
                owner_address2,
                account_address_3,
                account_data_3.as_bytes().to_vec(),
                vault_account_3,
                time_1,
                current_balance_1,
                current_shares_1,
                balance_usd_value_1,
            )
            .unwrap();

        // cleanup

        cleanup();
    }
    #[test]
    #[allow(unused_must_use)]
    fn test_new_interest_rate() {
        use crate::test_utils::TestDb;
        env::set_var(
            "DATABASE_URL",
            "postgres://postgres:password123@localhost/tulip",
        );
        let test_db = TestDb::new();
        let conn = test_db.conn();
        let client = Arc::new(DBClient {
            conn: &conn,
            oob_limit: 25.0,
        });
        crate::run_migrations(client.conn);
        std::thread::sleep(std::time::Duration::from_secs(2));
        let scraped_at = Utc::now();

        let platform_1 = String::from("platform1");
        let asset_1 = String::from("asset1");
        let asset_1_2 = String::from("asset1-2");

        let rate = 101_f64;
        let available_amount = 102_f64;
        let borrowed_amount = 103_f64;
        let utilization_rate = 104_f64;
        let lending_rate = 10_f64;

        let platform_2 = String::from("platform2");
        let asset_2 = String::from("asset2");
        let asset_2_2 = String::from("asset3-2");

        let platform_3 = String::from("platform3");
        let asset_3 = String::from("asset3");
        let asset_3_2 = String::from("asset3-2");

        let cleanup = || {
            client.delete_interest_rate(&InterestRateMatcher::All);
        };
        cleanup();
        let _time_1 = Utc::now();
        // test creating accounts
        {
            client
                .put_interest_rate(
                    platform_1.clone(),
                    asset_1.clone(),
                    rate,
                    utilization_rate,
                    lending_rate,
                    available_amount,
                    borrowed_amount,
                    scraped_at,
                )
                .unwrap();
            client
                .put_interest_rate(
                    platform_1.clone(),
                    asset_1_2.clone(),
                    rate,
                    utilization_rate,
                    lending_rate,
                    available_amount,
                    borrowed_amount,
                    scraped_at,
                )
                .unwrap();
            client
                .put_interest_rate(
                    platform_2.clone(),
                    asset_2.clone(),
                    rate,
                    utilization_rate,
                    lending_rate,
                    available_amount,
                    borrowed_amount,
                    scraped_at,
                )
                .unwrap();
            client
                .put_interest_rate(
                    platform_2.clone(),
                    asset_2_2.clone(),
                    rate,
                    utilization_rate,
                    lending_rate,
                    available_amount,
                    borrowed_amount,
                    scraped_at,
                )
                .unwrap();
            client
                .put_interest_rate(
                    platform_3.clone(),
                    asset_3.clone(),
                    rate,
                    utilization_rate,
                    lending_rate,
                    available_amount,
                    borrowed_amount,
                    scraped_at,
                )
                .unwrap();
            client
                .put_interest_rate(
                    platform_3.clone(),
                    asset_3_2.clone(),
                    rate,
                    utilization_rate,
                    lending_rate,
                    available_amount,
                    borrowed_amount,
                    scraped_at,
                )
                .unwrap();
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
        // test finding accounts by platform 1
        {
            let results = client
                .get_interest_rate(&InterestRateMatcher::Platform(vec![platform_1.clone()]))
                .unwrap();
            assert_eq!(results.len(), 2);
            let mut got_rate_1 = false;
            let mut got_rate_2 = false;
            println!("results {:#?}", results);
            for interest_rate in results.iter() {
                if interest_rate.asset.eq(&asset_1.to_ascii_uppercase()) {
                    got_rate_1 = true;
                    assert_eq!(interest_rate.lending_rate, lending_rate);
                    assert_eq!(interest_rate.available_amount, available_amount);
                    assert_eq!(interest_rate.borrowed_amount, borrowed_amount);
                } else if interest_rate.asset.eq(&asset_1_2.to_ascii_uppercase()) {
                    got_rate_2 = true;
                    assert_eq!(interest_rate.lending_rate, lending_rate);
                    assert_eq!(interest_rate.available_amount, available_amount);
                    assert_eq!(interest_rate.borrowed_amount, borrowed_amount);
                }
            }
            if !got_rate_1 || !got_rate_2 {
                panic!("failed to find rate 1 or 2");
            }
        }
        // test finding accounts by platform 2
        {
            let results = client
                .get_interest_rate(&InterestRateMatcher::Platform(vec![platform_2.clone()]))
                .unwrap();
            assert_eq!(results.len(), 2);
            let mut got_rate_1 = false;
            let mut got_rate_2 = false;

            for interest_rate in results.iter() {
                if interest_rate.asset.eq(&asset_2.to_ascii_uppercase()) {
                    got_rate_1 = true;
                    assert_eq!(interest_rate.available_amount, available_amount);
                    assert_eq!(interest_rate.borrowed_amount, borrowed_amount);
                    assert_eq!(interest_rate.lending_rate, lending_rate);
                } else if interest_rate.asset.eq(&asset_2_2.to_ascii_uppercase()) {
                    got_rate_2 = true;
                    assert_eq!(interest_rate.available_amount, available_amount);
                    assert_eq!(interest_rate.borrowed_amount, borrowed_amount);
                    assert_eq!(interest_rate.lending_rate, lending_rate);
                }
            }
            if !got_rate_1 || !got_rate_2 {
                panic!("failed to find rate 1 or 2");
            }
        }
        // test finding accounts by platform 3
        {
            let results = client
                .get_interest_rate(&InterestRateMatcher::Platform(vec![platform_3.clone()]))
                .unwrap();
            assert_eq!(results.len(), 2);
            let mut got_rate_1 = false;
            let mut got_rate_2 = false;

            for interest_rate in results.iter() {
                if interest_rate.asset.eq(&asset_3.to_ascii_uppercase()) {
                    got_rate_1 = true;
                    assert_eq!(interest_rate.lending_rate, lending_rate);
                    assert_eq!(interest_rate.available_amount, available_amount);
                    assert_eq!(interest_rate.borrowed_amount, borrowed_amount);
                } else if interest_rate.asset.eq(&asset_3_2.to_ascii_uppercase()) {
                    got_rate_2 = true;
                    assert_eq!(interest_rate.lending_rate, lending_rate);
                    assert_eq!(interest_rate.available_amount, available_amount);
                    assert_eq!(interest_rate.borrowed_amount, borrowed_amount);
                }
            }
            if !got_rate_1 || !got_rate_2 {
                panic!("failed to find rate 1 or 2");
            }
        }
        // test finding accounts by asset nad platform
        {
            let results = client
                .get_interest_rate(&InterestRateMatcher::AssetAndPlatform(vec![(
                    asset_1.clone(),
                    platform_1.clone(),
                )]))
                .unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].lending_rate, lending_rate);
            assert_eq!(results[0].available_amount, available_amount);
            assert_eq!(results[0].borrowed_amount, borrowed_amount);
            let results = client
                .get_interest_rate(&InterestRateMatcher::AssetAndPlatform(vec![(
                    asset_1_2.clone(),
                    platform_1.clone(),
                )]))
                .unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].lending_rate, lending_rate);
            assert_eq!(results[0].available_amount, available_amount);
            assert_eq!(results[0].borrowed_amount, borrowed_amount);
        }
        {
            let results = client
                .get_interest_rate(&InterestRateMatcher::AssetAndPlatform(vec![(
                    asset_2.clone(),
                    platform_2.clone(),
                )]))
                .unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].lending_rate, lending_rate);
            assert_eq!(results[0].available_amount, available_amount);
            assert_eq!(results[0].borrowed_amount, borrowed_amount);
            let results = client
                .get_interest_rate(&InterestRateMatcher::AssetAndPlatform(vec![(
                    asset_2_2.clone(),
                    platform_2.clone(),
                )]))
                .unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].lending_rate, lending_rate);
            assert_eq!(results[0].available_amount, available_amount);
            assert_eq!(results[0].borrowed_amount, borrowed_amount);
        }
        {
            let results = client
                .get_interest_rate(&InterestRateMatcher::AssetAndPlatform(vec![(
                    asset_3.clone(),
                    platform_3.clone(),
                )]))
                .unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].lending_rate, lending_rate);
            assert_eq!(results[0].available_amount, available_amount);
            assert_eq!(results[0].borrowed_amount, borrowed_amount);
            let results = client
                .get_interest_rate(&InterestRateMatcher::AssetAndPlatform(vec![(
                    asset_3_2,
                    platform_3.clone(),
                )]))
                .unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].lending_rate, lending_rate);
            assert_eq!(results[0].available_amount, available_amount);
            assert_eq!(results[0].borrowed_amount, borrowed_amount);
        }
        // cleanup accounts
        cleanup();

        // ensure we can create the accounts again since we deleted em
        client
            .put_interest_rate(
                platform_1.clone(),
                asset_1,
                rate,
                utilization_rate,
                lending_rate,
                available_amount,
                borrowed_amount,
                scraped_at,
            )
            .unwrap();
        client
            .put_interest_rate(
                platform_1,
                asset_1_2,
                rate,
                utilization_rate,
                lending_rate,
                available_amount,
                borrowed_amount,
                scraped_at,
            )
            .unwrap();
        client
            .put_interest_rate(
                platform_2.clone(),
                asset_2,
                rate,
                utilization_rate,
                lending_rate,
                available_amount,
                borrowed_amount,
                scraped_at,
            )
            .unwrap();
        client
            .put_interest_rate(
                platform_2,
                asset_2_2,
                rate,
                utilization_rate,
                lending_rate,
                available_amount,
                borrowed_amount,
                scraped_at,
            )
            .unwrap();
        client
            .put_interest_rate(
                platform_3,
                asset_3,
                rate,
                utilization_rate,
                lending_rate,
                available_amount,
                borrowed_amount,
                scraped_at,
            )
            .unwrap();

        // cleanup

        cleanup();
    }
    #[test]
    #[allow(unused_must_use)]
    fn test_update_interest_rate() {
        use crate::test_utils::TestDb;
        env::set_var(
            "DATABASE_URL",
            "postgres://postgres:password123@localhost/tulip",
        );
        let test_db = TestDb::new();
        let conn = test_db.conn();
        let client = Arc::new(DBClient {
            conn: &conn,
            oob_limit: 25.0,
        });
        crate::run_migrations(client.conn);
        std::thread::sleep(std::time::Duration::from_secs(2));
        let scraped_at = Utc::now();

        let platform_1 = String::from("platform1");
        let asset_1 = String::from("asset1");

        let rate = 101_f64;
        let available_amount = 102_f64;
        let borrowed_amount = 103_f64;
        let utilization_rate = 104_f64;
        let lending_rate = 104_f64;

        let cleanup = || {
            client.delete_interest_rates();
        };

        cleanup();

        std::thread::sleep(std::time::Duration::from_secs(2));

        client
            .put_interest_rate(
                platform_1.clone(),
                asset_1.clone(),
                rate,
                utilization_rate,
                lending_rate,
                available_amount,
                borrowed_amount,
                scraped_at,
            )
            .unwrap();
        let results = client
            .get_interest_rate(&InterestRateMatcher::AssetAndPlatform(vec![(
                asset_1.clone(),
                platform_1.clone(),
            )]))
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].utilization_rate, utilization_rate);
        assert_eq!(results[0].lending_rate, lending_rate);
        assert_eq!(results[0].borrow_rate, rate);
        assert_eq!(results[0].available_amount, available_amount);
        assert_eq!(results[0].borrowed_amount, borrowed_amount);

        let new_rate = 201_f64;
        let new_available_amount = 202_f64;
        let new_borrowed_amount = 203_f64;
        let new_utilization_rate = 204_f64;

        client
            .put_interest_rate(
                platform_1.clone(),
                asset_1.clone(),
                new_rate,
                new_utilization_rate,
                lending_rate,
                new_available_amount,
                new_borrowed_amount,
                scraped_at,
            )
            .unwrap();
        let results = client
            .get_interest_rate(&InterestRateMatcher::AssetAndPlatform(vec![(
                asset_1, platform_1,
            )]))
            .unwrap();

        assert!(results.len() == 2);
        assert!(results[1].utilization_rate == new_utilization_rate);
        assert!(results[1].lending_rate == lending_rate);
        assert!(results[1].available_amount == new_available_amount);
        assert!(results[1].borrowed_amount == new_borrowed_amount);
        assert!(results[1].borrow_rate == new_rate);

        cleanup();
    }
    #[test]
    #[allow(unused_must_use)]
    fn test_staking_analytics() {
        use crate::test_utils::TestDb;
        env::set_var(
            "DATABASE_URL",
            "postgres://postgres:password123@localhost/tulip",
        );
        let test_db = TestDb::new();
        let conn = test_db.conn();
        let client = Arc::new(DBClient {
            conn: &conn,
            oob_limit: 25.0,
        });

        crate::run_migrations(client.conn);
        std::thread::sleep(std::time::Duration::from_secs(2));
        let cleanup = || {
            client.delete_staking_analytic(&StakingAnalyticMatcher::All);
        };
        cleanup();
        std::thread::sleep(std::time::Duration::from_secs(2));
        client
            .put_staking_analytic(
                100_f64,
                101_f64,
                103_f64,
                104_f64,
                105_f64,
                106_u64,
                107_i64,
                Utc::now(),
            )
            .unwrap();
        let results = client
            .get_staking_analytic(&StakingAnalyticMatcher::All)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].tokens_staked, 100_f64);
        assert_eq!(results[0].tokens_locked, 101_f64);
        assert_eq!(results[0].stulip_total_supply, 103_f64);
        assert_eq!(results[0].apy, 104_f64);
        assert_eq!(results[0].price_float, 105_f64);
        assert_eq!(results[0].price_uint, 106_i64);
        assert_eq!(results[0].active_unstakes, 107_i64);
        client
            .put_staking_analytic(
                200_f64,
                201_f64,
                203_f64,
                204_f64,
                205_f64,
                206_u64,
                207_i64,
                Utc::now(),
            )
            .unwrap();
        let results = client
            .get_staking_analytic(&StakingAnalyticMatcher::All)
            .unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[1].tokens_staked, 200_f64);
        assert_eq!(results[1].tokens_locked, 201_f64);
        assert_eq!(results[1].stulip_total_supply, 203_f64);
        assert_eq!(results[1].apy, 204_f64);
        assert_eq!(results[1].price_float, 205_f64);
        assert_eq!(results[1].price_uint, 206_i64);
        assert_eq!(results[1].active_unstakes, 207_i64);

        cleanup();
    }
    #[test]
    #[allow(unused_must_use)]
    fn test_realize_yield() {
        use crate::test_utils::TestDb;
        env::set_var(
            "DATABASE_URL",
            "postgres://postgres:password123@localhost/tulip",
        );
        let test_db = TestDb::new();
        let conn = test_db.conn();
        let client = Arc::new(DBClient {
            conn: &conn,
            oob_limit: 25.0,
        });

        crate::run_migrations(client.conn);
        std::thread::sleep(std::time::Duration::from_secs(2));
        let cleanup = || {
            client.delete_realize_yield(&RealizeYieldMatcher::All);
        };
        cleanup();
        std::thread::sleep(std::time::Duration::from_secs(2));

        let vault_one = String::from("vault-one");
        let farm_one = String::from("farm-one");
        let total_deposited_one = 1000_f64;
        let gps_one = 1001_f64;
        let apr_one = 1002_f64;
        let total_deposited_two = 1420_f64;
        let gps_two = 1421_f64;
        let apr_two = 1422_f64;
        let total_deposited_three = 142069_f64;
        let gps_three = 1421_f64;
        let apr_three = 1422_f64;
        let scraped_at_one = Utc::now();

        let results = client
            .get_realize_yield(&RealizeYieldMatcher::All, None)
            .unwrap();
        assert_eq!(results.len(), 0);

        client
            .put_realize_yield(
                vault_one.clone(),
                farm_one.clone(),
                total_deposited_one,
                apr_one,
                gps_one,
                scraped_at_one,
            )
            .unwrap();

        let results = client
            .get_realize_yield(&RealizeYieldMatcher::All, None)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(&results[0].vault_address, &vault_one);
        assert_eq!(&results[0].farm_name, &farm_one);
        assert_eq!(results[0].total_deposited_balance, total_deposited_one);
        assert_eq!(results[0].gain_per_second, gps_one);
        assert_eq!(results[0].apr, apr_one);
        std::thread::sleep(std::time::Duration::from_millis(1500));
        client
            .put_realize_yield(
                vault_one.clone(),
                farm_one.clone(),
                total_deposited_two,
                apr_two,
                gps_two,
                scraped_at_one,
            )
            .unwrap();
        // test searching all returns both
        let results = client
            .get_realize_yield(&RealizeYieldMatcher::All, None)
            .unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(&results[1].vault_address, &vault_one);
        assert_eq!(&results[1].farm_name, &farm_one);
        assert_eq!(results[1].total_deposited_balance, total_deposited_two);
        assert_eq!(results[1].gain_per_second, gps_two);
        assert_eq!(results[1].apr, apr_two);
        // ensure limit selection works
        let results = client
            .get_realize_yield(&RealizeYieldMatcher::All, Some(1))
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(&results[0].vault_address, &vault_one);
        assert_eq!(&results[0].farm_name, &farm_one);
        assert_eq!(results[0].total_deposited_balance, total_deposited_one);
        assert_eq!(results[0].gain_per_second, gps_one);
        assert_eq!(results[0].apr, apr_one);
        std::thread::sleep(std::time::Duration::from_millis(1500));
        client
            .put_realize_yield(
                vault_one.clone(),
                farm_one.clone(),
                total_deposited_three,
                apr_three,
                gps_three,
                scraped_at_one,
            )
            .unwrap();
        let results = client
            .get_realize_yield(&RealizeYieldMatcher::All, None)
            .unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(&results[2].vault_address, &vault_one);
        assert_eq!(&results[2].farm_name, &farm_one);
        assert_eq!(results[2].total_deposited_balance, total_deposited_three);
        assert_eq!(results[2].gain_per_second, gps_three);
        assert_eq!(results[2].apr, apr_three);
        println!(
            "gain_per_second {} apr {}",
            results[2].gain_per_second, results[2].apr
        );
    }
    #[test]
    #[allow(unused_must_use)]
    fn test_interst_rate_curve() {
        use crate::test_utils::TestDb;
        env::set_var(
            "DATABASE_URL",
            "postgres://postgres:password123@localhost/tulip",
        );
        let test_db = TestDb::new();
        let conn = test_db.conn();
        let client = Arc::new(DBClient {
            conn: &conn,
            oob_limit: 25.0,
        });

        crate::run_migrations(client.conn);
        std::thread::sleep(std::time::Duration::from_secs(2));
        let cleanup = || {
            client.delete_interest_rate_curve(&InterestRateCurveMatcher::All);
        };
        cleanup();
        std::thread::sleep(std::time::Duration::from_secs(2));
        // note: dont use these as actual curve values
        // tulip usdc
        client
            .put_interest_rate_curve(
                "tulip".to_string(),
                "usdc".to_string(),
                420_f64,
                42069_f64,
                1_f64,
                2_f64,
                3_f64,
                4_f64,
            )
            .unwrap();
        // solend usdc
        client
            .put_interest_rate_curve(
                "solend".to_string(),
                "usdc".to_string(),
                1420_f64,
                142069_f64,
                11_f64,
                12_f64,
                0_f64, // note used by solend
                0_f64, // not usd by solend
            )
            .unwrap();
        // solend usdt
        client
            .put_interest_rate_curve(
                "solend".to_string(),
                "usdt".to_string(),
                2420_f64,
                242069_f64,
                21_f64,
                22_f64,
                0_f64, // note used by solend
                0_f64, // not usd by solend
            )
            .unwrap();
        // tulip usdt
        client
            .put_interest_rate_curve(
                "tulip".to_string(),
                "usdt".to_string(),
                3420_f64,
                342069_f64,
                31_f64,
                32_f64,
                33_f64,
                34_f64,
            )
            .unwrap();
        // mango usdc
        client
            .put_interest_rate_curve(
                "mango".to_string(),
                "usdc".to_string(),
                4420_f64,
                442069_f64,
                41_f64,
                42_f64,
                0_f64,
                0_f64,
            )
            .unwrap();
        // should error
        {
            // tulip usdc
            assert!(client
                .put_interest_rate_curve(
                    "tulip".to_string(),
                    "usdc".to_string(),
                    420_f64,
                    42069_f64,
                    1_f64,
                    2_f64,
                    3_f64,
                    4_f64,
                )
                .is_err());
            // solend usdc
            assert!(client
                .put_interest_rate_curve(
                    "solend".to_string(),
                    "usdc".to_string(),
                    1420_f64,
                    142069_f64,
                    11_f64,
                    12_f64,
                    0_f64, // note used by solend
                    0_f64, // not usd by solend
                )
                .is_err());
            // solend usdt
            assert!(client
                .put_interest_rate_curve(
                    "solend".to_string(),
                    "usdt".to_string(),
                    2420_f64,
                    242069_f64,
                    21_f64,
                    22_f64,
                    0_f64, // note used by solend
                    0_f64, // not usd by solend
                )
                .is_err());
            // tulip usdt
            assert!(client
                .put_interest_rate_curve(
                    "tulip".to_string(),
                    "usdt".to_string(),
                    3420_f64,
                    342069_f64,
                    31_f64,
                    32_f64,
                    33_f64,
                    34_f64,
                )
                .is_err());
            // mango usdc
            assert!(client
                .put_interest_rate_curve(
                    "mango".to_string(),
                    "usdc".to_string(),
                    4420_f64,
                    442069_f64,
                    41_f64,
                    42_f64,
                    0_f64,
                    0_f64,
                )
                .is_err());
        }

        // test loading rates

        {
            let rates = client
                .get_interest_rate_curve(&InterestRateCurveMatcher::Platform(vec![
                    "tulip".to_string()
                ]))
                .unwrap();
            assert_eq!(rates.len(), 2);
            assert_eq!(rates[0].platform, "tulip".to_ascii_uppercase());
            if rates[0].asset == "usdc".to_ascii_uppercase() {
                let got_rate = &client
                    .get_interest_rate_curve(&InterestRateCurveMatcher::RateName(vec![
                        "tulip-usdc".to_string(),
                    ]))
                    .unwrap()[0];
                assert_eq!(rates[0].rate_name, "TULIP-USDC");
                assert_eq!(rates[0].min_borrow_rate, 420_f64);
                assert_eq!(rates[0].max_borrow_rate, 42069_f64);
                assert_eq!(rates[0].optimal_borrow_rate, 1_f64);
                assert_eq!(rates[0].optimal_utilization_rate, 2_f64);
                assert_eq!(rates[0].degen_borrow_rate, 3_f64);
                assert_eq!(rates[0].degen_utilization_rate, 4_f64);
                assert_eq!(got_rate.rate_name, "TULIP-USDC");
                assert_eq!(got_rate.min_borrow_rate, 420_f64);
                assert_eq!(got_rate.max_borrow_rate, 42069_f64);
                assert_eq!(got_rate.optimal_borrow_rate, 1_f64);
                assert_eq!(got_rate.optimal_utilization_rate, 2_f64);
                assert_eq!(got_rate.degen_borrow_rate, 3_f64);
                assert_eq!(got_rate.degen_utilization_rate, 4_f64);
            } else if rates[0].asset == "usdt".to_ascii_uppercase() {
                let got_rate = &client
                    .get_interest_rate_curve(&InterestRateCurveMatcher::RateName(vec![
                        "tulip-usdt".to_string(),
                    ]))
                    .unwrap()[0];
                assert_eq!(rates[0].rate_name, "TULIP-USDT");
                assert_eq!(rates[0].min_borrow_rate, 3420_f64);
                assert_eq!(rates[0].max_borrow_rate, 342069_f64);
                assert_eq!(rates[0].optimal_borrow_rate, 31_f64);
                assert_eq!(rates[0].optimal_utilization_rate, 32_f64);
                assert_eq!(rates[0].degen_borrow_rate, 33_f64);
                assert_eq!(rates[0].degen_utilization_rate, 34_f64);
                assert_eq!(got_rate.rate_name, "TULIP-USDT");
                assert_eq!(got_rate.min_borrow_rate, 3420_f64);
                assert_eq!(got_rate.max_borrow_rate, 342069_f64);
                assert_eq!(got_rate.optimal_borrow_rate, 31_f64);
                assert_eq!(got_rate.optimal_utilization_rate, 32_f64);
                assert_eq!(got_rate.degen_borrow_rate, 33_f64);
                assert_eq!(got_rate.degen_utilization_rate, 34_f64);
            } else {
                panic!("invalid asset found");
            }
            if rates[1].asset == "usdc".to_ascii_uppercase() {
                let got_rate = &client
                    .get_interest_rate_curve(&InterestRateCurveMatcher::RateName(vec![
                        "tulip-usdc".to_string(),
                    ]))
                    .unwrap()[0];
                assert_eq!(rates[1].rate_name, "TULIP-USDC");
                assert_eq!(rates[1].min_borrow_rate, 420_f64);
                assert_eq!(rates[1].max_borrow_rate, 42069_f64);
                assert_eq!(rates[1].optimal_borrow_rate, 1_f64);
                assert_eq!(rates[1].optimal_utilization_rate, 2_f64);
                assert_eq!(rates[1].degen_borrow_rate, 3_f64);
                assert_eq!(rates[1].degen_utilization_rate, 4_f64);
                assert_eq!(got_rate.rate_name, "TULIP-USDC");
                assert_eq!(got_rate.min_borrow_rate, 420_f64);
                assert_eq!(got_rate.max_borrow_rate, 42069_f64);
                assert_eq!(got_rate.optimal_borrow_rate, 1_f64);
                assert_eq!(got_rate.optimal_utilization_rate, 2_f64);
                assert_eq!(got_rate.degen_borrow_rate, 3_f64);
                assert_eq!(got_rate.degen_utilization_rate, 4_f64);
            } else if rates[1].asset == "usdt".to_ascii_uppercase() {
                let got_rate = &client
                    .get_interest_rate_curve(&InterestRateCurveMatcher::RateName(vec![
                        "tulip-usdt".to_string(),
                    ]))
                    .unwrap()[0];
                assert_eq!(rates[1].rate_name, "TULIP-USDT");
                assert_eq!(rates[1].min_borrow_rate, 3420_f64);
                assert_eq!(rates[1].max_borrow_rate, 342069_f64);
                assert_eq!(rates[1].optimal_borrow_rate, 31_f64);
                assert_eq!(rates[1].optimal_utilization_rate, 32_f64);
                assert_eq!(rates[1].degen_borrow_rate, 33_f64);
                assert_eq!(rates[1].degen_utilization_rate, 34_f64);
                assert_eq!(got_rate.rate_name, "TULIP-USDT");
                assert_eq!(got_rate.min_borrow_rate, 3420_f64);
                assert_eq!(got_rate.max_borrow_rate, 342069_f64);
                assert_eq!(got_rate.optimal_borrow_rate, 31_f64);
                assert_eq!(got_rate.optimal_utilization_rate, 32_f64);
                assert_eq!(got_rate.degen_borrow_rate, 33_f64);
                assert_eq!(got_rate.degen_utilization_rate, 34_f64);
            }
        }
        {
            let rates = client
                .get_interest_rate_curve(&InterestRateCurveMatcher::Platform(vec![
                    "solend".to_string()
                ]))
                .unwrap();
            assert_eq!(rates.len(), 2);
            assert_eq!(rates[0].platform, "solend".to_ascii_uppercase());
            if rates[0].asset == "usdc".to_ascii_uppercase() {
                let got_rate = &client
                    .get_interest_rate_curve(&InterestRateCurveMatcher::RateName(vec![
                        "solend-usdc".to_string(),
                    ]))
                    .unwrap()[0];
                assert_eq!(rates[0].rate_name, "SOLEND-USDC");
                assert_eq!(rates[0].min_borrow_rate, 1420_f64);
                assert_eq!(rates[0].max_borrow_rate, 142069_f64);
                assert_eq!(rates[0].optimal_borrow_rate, 11_f64);
                assert_eq!(rates[0].optimal_utilization_rate, 12_f64);
                assert_eq!(rates[0].degen_borrow_rate, 0_f64);
                assert_eq!(rates[0].degen_utilization_rate, 0_f64);
                assert_eq!(got_rate.rate_name, "SOLEND-USDC");
                assert_eq!(got_rate.min_borrow_rate, 1420_f64);
                assert_eq!(got_rate.max_borrow_rate, 142069_f64);
                assert_eq!(got_rate.optimal_borrow_rate, 11_f64);
                assert_eq!(got_rate.optimal_utilization_rate, 12_f64);
                assert_eq!(got_rate.degen_borrow_rate, 0_f64);
                assert_eq!(got_rate.degen_utilization_rate, 0_f64);
            } else if rates[0].asset == "usdt".to_ascii_uppercase() {
                let got_rate = &client
                    .get_interest_rate_curve(&InterestRateCurveMatcher::RateName(vec![
                        "solend-usdt".to_string(),
                    ]))
                    .unwrap()[0];
                assert_eq!(rates[0].rate_name, "SOLEND-USDT");
                assert_eq!(rates[0].min_borrow_rate, 2420_f64);
                assert_eq!(rates[0].max_borrow_rate, 242069_f64);
                assert_eq!(rates[0].optimal_borrow_rate, 21_f64);
                assert_eq!(rates[0].optimal_utilization_rate, 22_f64);
                assert_eq!(rates[0].degen_borrow_rate, 0_f64);
                assert_eq!(rates[0].degen_utilization_rate, 0_f64);
                assert_eq!(got_rate.rate_name, "SOLEND-USDT");
                assert_eq!(got_rate.min_borrow_rate, 2420_f64);
                assert_eq!(got_rate.max_borrow_rate, 242069_f64);
                assert_eq!(got_rate.optimal_borrow_rate, 21_f64);
                assert_eq!(got_rate.optimal_utilization_rate, 22_f64);
                assert_eq!(got_rate.degen_borrow_rate, 0_f64);
                assert_eq!(got_rate.degen_utilization_rate, 0_f64);
            } else {
                panic!("invalid asset found");
            }
            if rates[1].asset == "usdc".to_ascii_uppercase() {
                let got_rate = &client
                    .get_interest_rate_curve(&InterestRateCurveMatcher::RateName(vec![
                        "solend-usdc".to_string(),
                    ]))
                    .unwrap()[0];
                assert_eq!(rates[0].rate_name, "SOLEND-USDC");
                assert_eq!(rates[0].min_borrow_rate, 1420_f64);
                assert_eq!(rates[0].max_borrow_rate, 142069_f64);
                assert_eq!(rates[0].optimal_borrow_rate, 11_f64);
                assert_eq!(rates[0].optimal_utilization_rate, 12_f64);
                assert_eq!(rates[0].degen_borrow_rate, 0_f64);
                assert_eq!(rates[0].degen_utilization_rate, 0_f64);
                assert_eq!(got_rate.rate_name, "SOLEND-USDC");
                assert_eq!(got_rate.min_borrow_rate, 1420_f64);
                assert_eq!(got_rate.max_borrow_rate, 142069_f64);
                assert_eq!(got_rate.optimal_borrow_rate, 11_f64);
                assert_eq!(got_rate.optimal_utilization_rate, 12_f64);
                assert_eq!(got_rate.degen_borrow_rate, 0_f64);
                assert_eq!(got_rate.degen_utilization_rate, 0_f64);
            } else if rates[1].asset == "usdt".to_ascii_uppercase() {
                let got_rate = &client
                    .get_interest_rate_curve(&InterestRateCurveMatcher::RateName(vec![
                        "solend-usdt".to_string(),
                    ]))
                    .unwrap()[0];
                assert_eq!(rates[1].rate_name, "SOLEND-USDT");
                assert_eq!(rates[1].min_borrow_rate, 2420_f64);
                assert_eq!(rates[1].max_borrow_rate, 242069_f64);
                assert_eq!(rates[1].optimal_borrow_rate, 21_f64);
                assert_eq!(rates[1].optimal_utilization_rate, 22_f64);
                assert_eq!(rates[1].degen_borrow_rate, 0_f64);
                assert_eq!(rates[1].degen_utilization_rate, 0_f64);
                assert_eq!(got_rate.rate_name, "SOLEND-USDT");
                assert_eq!(got_rate.min_borrow_rate, 2420_f64);
                assert_eq!(got_rate.max_borrow_rate, 242069_f64);
                assert_eq!(got_rate.optimal_borrow_rate, 21_f64);
                assert_eq!(got_rate.optimal_utilization_rate, 22_f64);
                assert_eq!(got_rate.degen_borrow_rate, 0_f64);
                assert_eq!(got_rate.degen_utilization_rate, 0_f64);
            }
        }
        {
            let rates = client
                .get_interest_rate_curve(&InterestRateCurveMatcher::Platform(vec![
                    "mango".to_string()
                ]))
                .unwrap();
            let got_rate = &client
                .get_interest_rate_curve(&InterestRateCurveMatcher::RateName(vec![
                    "mango-usdc".to_string()
                ]))
                .unwrap()[0];
            assert_eq!(rates.len(), 1);
            assert_eq!(rates[0].rate_name, "MANGO-USDC");
            assert_eq!(rates[0].min_borrow_rate, 4420_f64);
            assert_eq!(rates[0].max_borrow_rate, 442069_f64);
            assert_eq!(rates[0].optimal_borrow_rate, 41_f64);
            assert_eq!(rates[0].optimal_utilization_rate, 42_f64);
            assert_eq!(rates[0].degen_borrow_rate, 0_f64);
            assert_eq!(rates[0].degen_utilization_rate, 0_f64);
            assert_eq!(got_rate.rate_name, "MANGO-USDC");
            assert_eq!(got_rate.min_borrow_rate, 4420_f64);
            assert_eq!(got_rate.max_borrow_rate, 442069_f64);
            assert_eq!(got_rate.optimal_borrow_rate, 41_f64);
            assert_eq!(got_rate.optimal_utilization_rate, 42_f64);
            assert_eq!(got_rate.degen_borrow_rate, 0_f64);
            assert_eq!(got_rate.degen_utilization_rate, 0_f64);
        }
        cleanup();
    }
    #[test]
    #[allow(unused_must_use)]
    fn test_lending_optimizer_distribution() {
        use crate::test_utils::TestDb;
        env::set_var(
            "DATABASE_URL",
            "postgres://postgres:password123@localhost/tulip",
        );
        let test_db = TestDb::new();
        let conn = test_db.conn();
        let client = Arc::new(DBClient {
            conn: &conn,
            oob_limit: 25.0,
        });

        crate::run_migrations(client.conn);
        std::thread::sleep(std::time::Duration::from_secs(2));
        let cleanup = || {
            client.delete_lending_optimizer_distribution(&LendingOptimizerDistributionMatcher::All);
        };
        cleanup();
        std::thread::sleep(std::time::Duration::from_secs(2));
        let vault_name_one = String::from("vault-name-one");
        let vault_name_two = String::from("vault-name-two");
        let vault_standalones_one = vec![
            String::from("standalone-one-one"),
            String::from("standalone-one-two"),
        ];
        let vault_standalones_two = vec![
            String::from("standalone-two-one"),
            String::from("standalone-two-two"),
        ];
        let vault_deposits_one = vec![100_f64, 150_f64];
        let vault_deposit_two = vec![420_f64, 69_f64];
        client
            .put_lending_optimizer_distribution(
                vault_name_one.clone(),
                vault_standalones_one.clone(),
                vault_deposits_one.clone(),
            )
            .unwrap();
        // try putting it again and it shouldnt error
        // put there should still only be one
        client
            .put_lending_optimizer_distribution(
                vault_name_one.clone(),
                vault_standalones_one.clone(),
                vault_deposits_one.clone(),
            )
            .unwrap();
        let results = client
            .get_lending_optimizer_distribution(&LendingOptimizerDistributionMatcher::All)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(&results[0].standalone_vault_deposited_balances[0], &100_f64,);
        assert_eq!(&results[0].standalone_vault_deposited_balances[1], &150_f64,);
        assert_eq!(
            &results[0].standalone_vault_platforms[0],
            &String::from("standalone-one-one")
        );
        assert_eq!(
            &results[0].standalone_vault_platforms[1],
            &String::from("standalone-one-two"),
        );
        // put the updated values
        client
            .put_lending_optimizer_distribution(
                vault_name_one,
                vault_standalones_one,
                vault_deposit_two,
            )
            .unwrap();
        let results = client
            .get_lending_optimizer_distribution(&LendingOptimizerDistributionMatcher::All)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(&results[0].standalone_vault_deposited_balances[0], &420_f64,);
        assert_eq!(&results[0].standalone_vault_deposited_balances[1], &69_f64,);
        assert_eq!(
            &results[0].standalone_vault_platforms[0],
            &String::from("standalone-one-one")
        );
        assert_eq!(
            &results[0].standalone_vault_platforms[1],
            &String::from("standalone-one-two"),
        );
        client
            .put_lending_optimizer_distribution(
                vault_name_two,
                vault_standalones_two,
                vault_deposits_one,
            )
            .unwrap();
        assert_eq!(
            client
                .get_lending_optimizer_distribution(&LendingOptimizerDistributionMatcher::All)
                .unwrap()
                .len(),
            2
        );
        let results = client
            .get_lending_optimizer_distribution(&LendingOptimizerDistributionMatcher::VaultName(
                vec![String::from("vault-name-two")],
            ))
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(&results[0].standalone_vault_deposited_balances[0], &100_f64,);
        assert_eq!(&results[0].standalone_vault_deposited_balances[1], &150_f64,);
        assert_eq!(
            &results[0].standalone_vault_platforms[0],
            &String::from("standalone-two-one")
        );
        assert_eq!(
            &results[0].standalone_vault_platforms[1],
            &String::from("standalone-two-two"),
        );
    }
    #[test]
    #[allow(unused_must_use)]
    fn test_interest_rate_moving_average() {
        use crate::test_utils::TestDb;
        env::set_var(
            "DATABASE_URL",
            "postgres://postgres:password123@localhost/tulip",
        );
        let test_db = TestDb::new();
        let conn = test_db.conn();
        let client = Arc::new(DBClient {
            conn: &conn,
            oob_limit: 25.0,
        });

        crate::run_migrations(client.conn);
        std::thread::sleep(std::time::Duration::from_secs(2));
        let cleanup = || {
            client.delete_interest_rate_moving_average(&InterestRateMovingAverageMatcher::All);
            client.delete_interest_rate(&InterestRateMatcher::All);
        };
        cleanup();
        std::thread::sleep(std::time::Duration::from_secs(2));

        let platform_1 = "platform_1";
        let asset_1 = "asset_1";
        let asset_2 = "asset_2";
        let borrow_rate = 100_f64;
        let utilization_rate = 100_f64;
        let lending_rate = 420_f64;
        let available_amount = 1000_f64;
        let borrowed_amount = 1000_f64;
        {
            // tests the interest rate moving average creation route
            client
                .put_interest_rate(
                    platform_1.to_string(),
                    asset_1.to_string(),
                    borrow_rate,
                    utilization_rate,
                    lending_rate,
                    available_amount,
                    borrowed_amount,
                    Utc::now(),
                )
                .unwrap();
            let rates = client
                .get_interest_rate_moving_average(&InterestRateMovingAverageMatcher::RateName(
                    vec![format!("{}-{}", platform_1, asset_1)],
                ))
                .unwrap();
            assert_eq!(rates.len(), 1);
            assert_eq!(rates[0].period_observed_rates.len(), 1);
            assert_eq!(rates[0].period_observed_rates[0], 420_f64);
            assert_eq!(rates[0].period_running_average, 420_f64);
            assert_eq!(rates[0].asset, asset_1.to_ascii_uppercase());
            assert_eq!(rates[0].platform, platform_1.to_ascii_uppercase());

            // now test updating the moving average
            client
                .put_interest_rate(
                    platform_1.to_string(),
                    asset_1.to_string(),
                    borrow_rate,
                    utilization_rate,
                    69_f64,
                    available_amount,
                    borrowed_amount,
                    Utc::now(),
                )
                .unwrap();
            let rates = client
                .get_interest_rate_moving_average(&InterestRateMovingAverageMatcher::RateName(
                    vec![format!("{}-{}", platform_1, asset_1)],
                ))
                .unwrap();
            assert_eq!(rates.len(), 1);
            assert_eq!(rates[0].period_observed_rates.len(), 2);
            assert_eq!(rates[0].period_observed_rates[0], 420_f64);
            assert_eq!(rates[0].period_observed_rates[1], 69_f64);
            assert_eq!(rates[0].period_running_average, 244.5);
            assert_eq!(rates[0].asset, asset_1.to_ascii_uppercase());
            assert_eq!(rates[0].platform, platform_1.to_ascii_uppercase());

            // now test updating the moving average
            client
                .put_interest_rate(
                    platform_1.to_string(),
                    asset_1.to_string(),
                    borrow_rate,
                    utilization_rate,
                    420_f64,
                    available_amount,
                    borrowed_amount,
                    Utc::now(),
                )
                .unwrap();
            let rates = client
                .get_interest_rate_moving_average(&InterestRateMovingAverageMatcher::RateName(
                    vec![format!("{}-{}", platform_1, asset_1)],
                ))
                .unwrap();
            assert_eq!(rates.len(), 1);
            assert_eq!(rates[0].period_observed_rates.len(), 3);
            assert_eq!(rates[0].period_observed_rates[0], 420_f64);
            assert_eq!(rates[0].period_observed_rates[1], 69_f64);
            assert_eq!(rates[0].period_observed_rates[2], 420_f64);
            assert_eq!(rates[0].period_running_average, 303_f64);
            assert_eq!(rates[0].asset, asset_1.to_ascii_uppercase());
            assert_eq!(rates[0].platform, platform_1.to_ascii_uppercase());

            std::thread::sleep(std::time::Duration::from_secs(18));

            // now this update should cause the period to rollover

            client
                .put_interest_rate(
                    platform_1.to_string(),
                    asset_1.to_string(),
                    borrow_rate,
                    utilization_rate,
                    420_f64,
                    available_amount,
                    borrowed_amount,
                    Utc::now(),
                )
                .unwrap();
            let rates = client
                .get_interest_rate_moving_average(&InterestRateMovingAverageMatcher::RateName(
                    vec![format!("{}-{}", platform_1, asset_1)],
                ))
                .unwrap();
            assert_eq!(rates.len(), 1);
            assert_eq!(rates[0].period_observed_rates.len(), 1);
            assert_eq!(rates[0].period_observed_rates[0], 420_f64);
            assert_eq!(rates[0].period_running_average, 420_f64);
            assert_eq!(rates[0].last_period_running_average, 303_f64);
            assert_eq!(rates[0].asset, asset_1.to_ascii_uppercase());
            assert_eq!(rates[0].platform, platform_1.to_ascii_uppercase());

            client
                .put_interest_rate(
                    platform_1.to_string(),
                    asset_1.to_string(),
                    borrow_rate,
                    utilization_rate,
                    1337_f64,
                    available_amount,
                    borrowed_amount,
                    Utc::now(),
                )
                .unwrap();
            let rates = client
                .get_interest_rate_moving_average(&InterestRateMovingAverageMatcher::RateName(
                    vec![format!("{}-{}", platform_1, asset_1)],
                ))
                .unwrap();
            assert_eq!(rates.len(), 1);
            assert_eq!(rates[0].period_observed_rates.len(), 2);
            assert_eq!(rates[0].period_observed_rates[0], 420_f64);
            assert_eq!(rates[0].period_observed_rates[1], 1337_f64);
            assert_eq!(rates[0].period_running_average, 878.5);
            assert_eq!(rates[0].last_period_running_average, 303_f64);
            assert_eq!(rates[0].asset, asset_1.to_ascii_uppercase());
            assert_eq!(rates[0].platform, platform_1.to_ascii_uppercase());
        }
        {
            // tests the interest rate moving average creation route
            client
                .put_interest_rate(
                    platform_1.to_string(),
                    asset_2.to_string(),
                    borrow_rate,
                    utilization_rate,
                    lending_rate,
                    available_amount,
                    borrowed_amount,
                    Utc::now(),
                )
                .unwrap();
            let rates = client
                .get_interest_rate_moving_average(&InterestRateMovingAverageMatcher::RateName(
                    vec![format!("{}-{}", platform_1, asset_2)],
                ))
                .unwrap();
            assert_eq!(rates.len(), 1);
            assert_eq!(rates[0].period_observed_rates.len(), 1);
            assert_eq!(rates[0].period_observed_rates[0], 420_f64);
            assert_eq!(rates[0].period_running_average, 420_f64);
            assert_eq!(rates[0].asset, asset_2.to_ascii_uppercase());
            assert_eq!(rates[0].platform, platform_1.to_ascii_uppercase());

            // now test updating the moving average
            client
                .put_interest_rate(
                    platform_1.to_string(),
                    asset_2.to_string(),
                    borrow_rate,
                    utilization_rate,
                    69_f64,
                    available_amount,
                    borrowed_amount,
                    Utc::now(),
                )
                .unwrap();
            let rates = client
                .get_interest_rate_moving_average(&InterestRateMovingAverageMatcher::RateName(
                    vec![format!("{}-{}", platform_1, asset_2)],
                ))
                .unwrap();
            assert_eq!(rates.len(), 1);
            assert_eq!(rates[0].period_observed_rates.len(), 2);
            assert_eq!(rates[0].period_observed_rates[0], 420_f64);
            assert_eq!(rates[0].period_observed_rates[1], 69_f64);
            assert_eq!(rates[0].period_running_average, 244.5);
            assert_eq!(rates[0].asset, asset_2.to_ascii_uppercase());
            assert_eq!(rates[0].platform, platform_1.to_ascii_uppercase());

            // now test updating the moving average
            client
                .put_interest_rate(
                    platform_1.to_string(),
                    asset_2.to_string(),
                    borrow_rate,
                    utilization_rate,
                    420_f64,
                    available_amount,
                    borrowed_amount,
                    Utc::now(),
                )
                .unwrap();
            let rates = client
                .get_interest_rate_moving_average(&InterestRateMovingAverageMatcher::RateName(
                    vec![format!("{}-{}", platform_1, asset_2)],
                ))
                .unwrap();
            assert_eq!(rates.len(), 1);
            assert_eq!(rates[0].period_observed_rates.len(), 3);
            assert_eq!(rates[0].period_observed_rates[0], 420_f64);
            assert_eq!(rates[0].period_observed_rates[1], 69_f64);
            assert_eq!(rates[0].period_observed_rates[2], 420_f64);
            assert_eq!(rates[0].period_running_average, 303_f64);
            assert_eq!(rates[0].asset, asset_2.to_ascii_uppercase());
            assert_eq!(rates[0].platform, platform_1.to_ascii_uppercase());

            std::thread::sleep(std::time::Duration::from_secs(18));

            // now this update should cause the period to rollover

            client
                .put_interest_rate(
                    platform_1.to_string(),
                    asset_2.to_string(),
                    borrow_rate,
                    utilization_rate,
                    420_f64,
                    available_amount,
                    borrowed_amount,
                    Utc::now(),
                )
                .unwrap();
            let rates = client
                .get_interest_rate_moving_average(&InterestRateMovingAverageMatcher::RateName(
                    vec![format!("{}-{}", platform_1, asset_2)],
                ))
                .unwrap();
            assert_eq!(rates.len(), 1);
            assert_eq!(rates[0].period_observed_rates.len(), 1);
            assert_eq!(rates[0].period_observed_rates[0], 420_f64);
            assert_eq!(rates[0].period_running_average, 420_f64);
            assert_eq!(rates[0].last_period_running_average, 303_f64);
            assert_eq!(rates[0].asset, asset_2.to_ascii_uppercase());
            assert_eq!(rates[0].platform, platform_1.to_ascii_uppercase());

            client
                .put_interest_rate(
                    platform_1.to_string(),
                    asset_2.to_string(),
                    borrow_rate,
                    utilization_rate,
                    1337_f64,
                    available_amount,
                    borrowed_amount,
                    Utc::now(),
                )
                .unwrap();
            let rates = client
                .get_interest_rate_moving_average(&InterestRateMovingAverageMatcher::RateName(
                    vec![format!("{}-{}", platform_1, asset_2)],
                ))
                .unwrap();
            assert_eq!(rates.len(), 1);
            assert_eq!(rates[0].period_observed_rates.len(), 2);
            assert_eq!(rates[0].period_observed_rates[0], 420_f64);
            assert_eq!(rates[0].period_observed_rates[1], 1337_f64);
            assert_eq!(rates[0].period_running_average, 878.5);
            assert_eq!(rates[0].last_period_running_average, 303_f64);
            assert_eq!(rates[0].asset, asset_2.to_ascii_uppercase());
            assert_eq!(rates[0].platform, platform_1.to_ascii_uppercase());
        }
        // test loading interest rate with moving average
        {
            let results = client
                .get_interest_rate_with_moving_average(
                    &InterestRateMovingAverageMatcher::RateName(vec![format!(
                        "{}-{}",
                        platform_1, asset_2
                    )]),
                    &InterestRateMatcher::AssetAndPlatform(vec![(
                        asset_2.to_string(),
                        platform_1.to_string(),
                    )]),
                )
                .unwrap();

            assert_eq!(results.len(), 1);
            assert_eq!(results[0].0.asset, results[0].1.asset);
            assert_eq!(results[0].0.period_observed_rates.len(), 2);
            assert_eq!(results[0].0.period_observed_rates[0], 420_f64);
            assert_eq!(results[0].0.period_observed_rates[1], 1337_f64);
            assert_eq!(results[0].0.period_running_average, 878.5);
            assert_eq!(results[0].0.last_period_running_average, 303_f64);
            assert_eq!(results[0].0.asset, asset_2.to_ascii_uppercase());
            assert_eq!(results[0].0.platform, platform_1.to_ascii_uppercase());

            // ensure the most recent observed rate is equal to the current interest rate
            assert_eq!(
                results[0].0.period_observed_rates[1],
                results[0].1.lending_rate
            );
        }

        cleanup();
    }

    #[test]
    #[allow(unused_must_use)]
    fn test_advertised_yield() {
        use crate::test_utils::TestDb;
        env::set_var(
            "DATABASE_URL",
            "postgres://postgres:password123@localhost/tulip",
        );
        let test_db = TestDb::new();
        let conn = test_db.conn();
        let client = Arc::new(DBClient {
            conn: &conn,
            oob_limit: 25.0,
        });

        crate::run_migrations(client.conn);
        std::thread::sleep(std::time::Duration::from_secs(2));
        let cleanup = || {
            client.delete_advertised_yield(&AdvertisedYieldMatcher::All);
        };
        cleanup();
        std::thread::sleep(std::time::Duration::from_secs(2));
        let farm_one = "farm_one";
        let vault_one = "vault_one";
        let farm_two = "farm_two";
        let vault_two = "vault_two";
        let farm_three = "farm_three";
        let apr_smol = 100_f64;
        let apr_beeg = 420_f64;
        let scraped = Utc::now();

        client
            .put_advertised_yield(vault_one, farm_one, apr_smol, scraped)
            .unwrap();

        let result = client
            .get_advertised_yield(&AdvertisedYieldMatcher::All)
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].farm_name, farm_one);
        assert_eq!(result[0].vault_address, vault_one);
        assert_eq!(result[0].apr, apr_smol);

        client
            .put_advertised_yield(vault_one, farm_one, apr_beeg, scraped)
            .unwrap();

        let result = client
            .get_advertised_yield(&AdvertisedYieldMatcher::All)
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].farm_name, farm_one);
        assert_eq!(result[0].vault_address, vault_one);
        assert_eq!(result[0].apr, apr_beeg);

        client
            .put_advertised_yield(vault_one, farm_two, apr_smol, scraped)
            .unwrap();

        let result = client
            .get_advertised_yield(&AdvertisedYieldMatcher::All)
            .unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[1].farm_name, farm_two);
        assert_eq!(result[1].vault_address, vault_one);
        assert_eq!(result[1].apr, apr_smol);

        client
            .put_advertised_yield(vault_two, farm_three, apr_beeg, scraped)
            .unwrap();

        let result = client
            .get_advertised_yield(&AdvertisedYieldMatcher::All)
            .unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[2].farm_name, farm_three);
        assert_eq!(result[2].vault_address, vault_two);
        assert_eq!(result[2].apr, apr_beeg);
    }
    #[test]
    #[allow(unused_must_use)]
    fn test_paginated_v1_obligations() {
        use crate::test_utils::TestDb;
        env::set_var(
            "DATABASE_URL",
            "postgres://postgres:password123@localhost/tulip",
        );
        let test_db = TestDb::new();
        let conn = test_db.conn();
        let client = Arc::new(DBClient {
            conn: &conn,
            oob_limit: 25.0,
        });
        crate::run_migrations(client.conn);
        std::thread::sleep(std::time::Duration::from_secs(2));

        let conn = test_db.conn();
        for i in 0..30 {
            client
                .put_v1_obligation_ltv(
                    format!("authority-{}", i).as_str(),
                    format!("userfarm-{}", i).as_str(),
                    format!("account-{}", i).as_str(),
                    format!("leveragedfarm-{}", i).as_str(),
                    i as f64,
                    Utc::now(),
                )
                .unwrap();
        }
        std::thread::sleep(std::time::Duration::from_secs(2));

        {
            let results = client
                .get_v1_obligation_ltv_sorted(&V1ObligationLtvMatcher::All)
                .unwrap();
            assert!(results[0].ltv.lt(&results[results.len() - 1].ltv));
        }

        // test non paginated query
        let results = client
            .get_v1_obligation_ltv(&V1ObligationLtvMatcher::All)
            .unwrap();
        assert_eq!(results.len(), 30);
        for (idx, obligation) in results.iter().enumerate() {
            let auth_parts = obligation.authority.split('-').collect::<Vec<&str>>();
            assert_eq!(usize::from_str(auth_parts[1]).unwrap(), idx,);
            let farm_parts = obligation.user_farm.split('-').collect::<Vec<&str>>();
            assert_eq!(usize::from_str(farm_parts[1]).unwrap(), idx);
            let account_parts = obligation.account_address.split('-').collect::<Vec<&str>>();
            assert_eq!(usize::from_str(account_parts[1]).unwrap(), idx);
            let lev_farm_parts = obligation.leveraged_farm.split('-').collect::<Vec<&str>>();
            assert_eq!(usize::from_str(lev_farm_parts[1]).unwrap(), idx);
        }

        let first_ten_no_page = results[0..10].to_vec();
        let second_ten_no_page = results[10..20].to_vec();
        let third_ten_no_page = results[20..30].to_vec();

        // test paginated query
        let results = query_paginated_v1_obligations(
            &conn,
            &V1ObligationLtvMatcher::All,
            Some(1),  // start on page 1
            Some(10), // 10 per page
        )
        .unwrap();
        assert_eq!((&results.0).len(), 10);
        assert_eq!(results.1, 30);
        let first_ten_yes_page = results.0;
        assert_eq!(first_ten_no_page.len(), first_ten_yes_page.len());

        let results = query_paginated_v1_obligations(
            &conn,
            &V1ObligationLtvMatcher::All,
            Some(2),  // start on page 1
            Some(10), // 10 per page
        )
        .unwrap();
        assert_eq!((&results.0).len(), 10);
        assert_eq!(results.1, 30);
        let second_ten_yes_page = results.0;
        assert_eq!(second_ten_no_page.len(), second_ten_yes_page.len());

        let results = query_paginated_v1_obligations(
            &conn,
            &V1ObligationLtvMatcher::All,
            Some(3),  // start on page 1
            Some(10), // 10 per page
        )
        .unwrap();
        assert_eq!((&results.0).len(), 10);
        assert_eq!(results.1, 30);
        let third_ten_yes_page = results.0;
        assert_eq!(third_ten_no_page.len(), third_ten_yes_page.len());

        first_ten_no_page
            .iter()
            .zip(first_ten_yes_page.iter())
            .filter(|(a, b)| {
                assert_eq!(a.id, b.id);
                assert_eq!(a.authority, b.authority);
                assert_eq!(a.account_address, b.account_address);
                assert_eq!(a.ltv, b.ltv);
                true
            });
        second_ten_no_page
            .iter()
            .zip(second_ten_yes_page.iter())
            .filter(|(a, b)| {
                assert_eq!(a.id, b.id);
                assert_eq!(a.authority, b.authority);
                assert_eq!(a.account_address, b.account_address);
                assert_eq!(a.ltv, b.ltv);
                true
            });
        third_ten_no_page
            .iter()
            .zip(third_ten_yes_page.iter())
            .filter(|(a, b)| {
                assert_eq!(a.id, b.id);
                assert_eq!(a.authority, b.authority);
                assert_eq!(a.account_address, b.account_address);
                assert_eq!(a.ltv, b.ltv);
                true
            });
    }
    #[test]
    #[allow(unused_must_use)]
    fn test_paginated_v1_user_farm() {
        use crate::test_utils::TestDb;
        env::set_var(
            "DATABASE_URL",
            "postgres://postgres:password123@localhost/tulip",
        );
        let test_db = TestDb::new();
        let conn = test_db.conn();
        let client = Arc::new(DBClient {
            conn: &conn,
            oob_limit: 25.0,
        });
        crate::run_migrations(client.conn);
        std::thread::sleep(std::time::Duration::from_secs(2));

        let conn = test_db.conn();
        for i in 0..30 {
            client
                .put_v1_user_farm(
                    format!("authority-{}", i).as_str(),
                    format!("account-{}", i).as_str(),
                    format!("leveragedfarm-{}", i).as_str(),
                    &vec![
                        format!("obligation_0-{}", i),
                        format!("obligation_1-{}", i),
                        format!("obligation_2-{}", i),
                    ][..],
                    &vec![0, 1, 2][..],
                )
                .unwrap();
        }
        std::thread::sleep(std::time::Duration::from_secs(2));

        // test non paginated query
        let results = client.get_v1_user_farm(&V1UserFarmMatcher::All).unwrap();
        assert_eq!(results.len(), 30);
        for (idx, user_farm) in results.iter().enumerate() {
            let auth_parts = user_farm.authority.split('-').collect::<Vec<&str>>();
            assert_eq!(usize::from_str(auth_parts[1]).unwrap(), idx,);
            for (oidx, (obligation, obligation_index)) in user_farm
                .clone()
                .obligations
                .iter()
                .zip(user_farm.clone().obligation_indexes)
                .enumerate()
            {
                let obligation_parts = obligation.split('-').collect::<Vec<&str>>();
                assert_eq!(usize::from_str(obligation_parts[1]).unwrap(), idx,);
                let identifiers = obligation_parts[0].split('_').collect::<Vec<&str>>();
                assert_eq!(usize::from_str(identifiers[1]).unwrap(), oidx);
                assert_eq!(oidx, obligation_index as usize);
            }
            let account_parts = user_farm.account_address.split('-').collect::<Vec<&str>>();
            assert_eq!(usize::from_str(account_parts[1]).unwrap(), idx);
            let levfarm_parts = user_farm.leveraged_farm.split('-').collect::<Vec<&str>>();
            assert_eq!(usize::from_str(levfarm_parts[1]).unwrap(), idx);
        }

        let first_ten_no_page = results[0..10].to_vec();
        let second_ten_no_page = results[10..20].to_vec();
        let third_ten_no_page = results[20..30].to_vec();

        // test paginated query
        let results = query_paginated_v1_user_farms(
            &conn,
            &V1UserFarmMatcher::All,
            Some(1),  // start on page 1
            Some(10), // 10 per page
        )
        .unwrap();
        assert_eq!((&results.0).len(), 10);
        assert_eq!(results.1, 30);
        let first_ten_yes_page = results.0;
        assert_eq!(first_ten_no_page.len(), first_ten_yes_page.len());

        let results = query_paginated_v1_user_farms(
            &conn,
            &V1UserFarmMatcher::All,
            Some(2),  // start on page 2
            Some(10), // 10 per page
        )
        .unwrap();
        assert_eq!((&results.0).len(), 10);
        assert_eq!(results.1, 30);
        let second_ten_yes_page = results.0;
        assert_eq!(second_ten_no_page.len(), second_ten_yes_page.len());

        let results = query_paginated_v1_user_farms(
            &conn,
            &V1UserFarmMatcher::All,
            Some(3),  // start on page 3
            Some(10), // 10 per page
        )
        .unwrap();
        assert_eq!((&results.0).len(), 10);
        assert_eq!(results.1, 30);
        let third_ten_yes_page = results.0;
        assert_eq!(third_ten_no_page.len(), third_ten_yes_page.len());

        first_ten_no_page
            .iter()
            .zip(first_ten_yes_page.iter())
            .filter(|(a, b)| {
                assert_eq!(a.id, b.id);
                assert_eq!(a.authority, b.authority);
                assert_eq!(a.account_address, b.account_address);
                assert_eq!(a.obligations, b.obligations);
                assert_eq!(a.obligation_indexes, b.obligation_indexes);
                true
            });
        second_ten_no_page
            .iter()
            .zip(second_ten_yes_page.iter())
            .filter(|(a, b)| {
                assert_eq!(a.id, b.id);
                assert_eq!(a.authority, b.authority);
                assert_eq!(a.account_address, b.account_address);
                assert_eq!(a.obligations, b.obligations);
                assert_eq!(a.obligation_indexes, b.obligation_indexes);
                true
            });
        third_ten_no_page
            .iter()
            .zip(third_ten_yes_page.iter())
            .filter(|(a, b)| {
                assert_eq!(a.id, b.id);
                assert_eq!(a.authority, b.authority);
                assert_eq!(a.account_address, b.account_address);
                assert_eq!(a.obligations, b.obligations);
                assert_eq!(a.obligation_indexes, b.obligation_indexes);
                true
            });
    }
    #[test]
    #[allow(unused_must_use)]
    fn test_paginated_v1_liquidated_positions() {
        use crate::test_utils::TestDb;
        env::set_var(
            "DATABASE_URL",
            "postgres://postgres:password123@localhost/tulip",
        );
        let test_db = TestDb::new();
        let conn = test_db.conn();
        let client = Arc::new(DBClient {
            conn: &conn,
            oob_limit: 25.0,
        });
        crate::run_migrations(client.conn);
        std::thread::sleep(std::time::Duration::from_secs(2));

        let conn = test_db.conn();
        // ensure inserting a record with ended_at being None causes an error
        for i in 0..30 {
            client
                .put_v1_liquidated_position(
                    format!("temp_liquidation_account-{}", i).as_str(),
                    format!("authority-{}", i).as_str(),
                    format!("user_farm-{}", i).as_str(),
                    format!("liquidation-event-{}", i).as_str(),
                    format!("obligation-{}", i).as_str(),
                    format!("leveragedfarm-{}", i).as_str(),
                    Utc::now(),
                    None,
                )
                .unwrap();
        }
        std::thread::sleep(std::time::Duration::from_secs(2));

        // now try re-inserting every single position with the ended_at field
        // set to None, which should trigger an error
        for i in 0..30 {
            assert!(client
                .put_v1_liquidated_position(
                    format!("temp_liquidation_account-{}", i).as_str(),
                    format!("authority-{}", i).as_str(),
                    format!("user_farm-{}", i).as_str(),
                    format!("liquidation-event-{}", i).as_str(),
                    format!("obligation-{}", i).as_str(),
                    format!("leveragfarm-{}", i).as_str(),
                    Utc::now(),
                    None,
                )
                .is_err());
        }

        // now update the 10 most recent records to set ended_at
        for i in 20..30 {
            client
                .put_v1_liquidated_position(
                    format!("temp_liquidation_account-{}", i).as_str(),
                    format!("authority-{}", i).as_str(),
                    format!("user_farm-{}", i).as_str(),
                    format!("liquidation-event-{}", i).as_str(),
                    format!("obligation-{}", i).as_str(),
                    format!("leveragfarm-{}", i).as_str(),
                    Utc::now(),
                    Some(Utc::now()),
                )
                .unwrap();
        }

        // test non paginated query
        let results = client
            .get_v1_liquidated_position(&V1LiquidatedPositionMatcher::All)
            .unwrap();
        assert_eq!(results.len(), 30);
        for (idx, obligation) in results.iter().enumerate() {
            let auth_parts = obligation.authority.split('-').collect::<Vec<&str>>();
            assert_eq!(usize::from_str(auth_parts[1]).unwrap(), idx,);
            let farm_parts = obligation.user_farm.split('-').collect::<Vec<&str>>();
            assert_eq!(usize::from_str(farm_parts[1]).unwrap(), idx);
            let levfarm_parts = obligation.leveraged_farm.split('-').collect::<Vec<&str>>();
            assert_eq!(usize::from_str(levfarm_parts[1]).unwrap(), idx);
            let temp_liquidation_account_parts = obligation
                .temp_liquidation_account
                .split('-')
                .collect::<Vec<&str>>();
            assert_eq!(
                usize::from_str(temp_liquidation_account_parts[1]).unwrap(),
                idx
            );
        }

        let first_ten_no_page = results[0..10].to_vec();
        let second_ten_no_page = results[10..20].to_vec();
        let third_ten_no_page = results[20..30].to_vec();

        // test paginated query
        let results = query_paginated_v1_liquidated_positions(
            &conn,
            &V1LiquidatedPositionMatcher::All,
            Some(1),  // start on page 1
            Some(10), // 10 per page
        )
        .unwrap();
        assert_eq!((&results.0).len(), 10);
        assert_eq!(results.1, 30);
        let first_ten_yes_page = results.0;
        assert_eq!(first_ten_no_page.len(), first_ten_yes_page.len());

        let results = query_paginated_v1_liquidated_positions(
            &conn,
            &V1LiquidatedPositionMatcher::All,
            Some(2),  // start on page 1
            Some(10), // 10 per page
        )
        .unwrap();
        assert_eq!((&results.0).len(), 10);
        assert_eq!(results.1, 30);
        let second_ten_yes_page = results.0;
        assert_eq!(second_ten_no_page.len(), second_ten_yes_page.len());

        let results = query_paginated_v1_liquidated_positions(
            &conn,
            &V1LiquidatedPositionMatcher::All,
            Some(3),  // start on page 1
            Some(10), // 10 per page
        )
        .unwrap();
        assert_eq!((&results.0).len(), 10);
        assert_eq!(results.1, 30);
        let third_ten_yes_page = results.0;
        assert_eq!(third_ten_no_page.len(), third_ten_yes_page.len());

        first_ten_no_page
            .iter()
            .zip(first_ten_yes_page.iter())
            .filter(|(a, b)| {
                assert_eq!(a.id, b.id);
                assert_eq!(a.authority, b.authority);
                assert_eq!(a.temp_liquidation_account, b.temp_liquidation_account);
                assert_eq!(a.user_farm, b.user_farm);
                assert_eq!(a.obligation, b.obligation);
                assert_eq!(a.started_at, b.started_at);
                assert_eq!(a.ended_at, b.ended_at);
                true
            });
        second_ten_no_page
            .iter()
            .zip(second_ten_yes_page.iter())
            .filter(|(a, b)| {
                assert_eq!(a.id, b.id);
                assert_eq!(a.authority, b.authority);
                assert_eq!(a.temp_liquidation_account, b.temp_liquidation_account);
                assert_eq!(a.user_farm, b.user_farm);
                assert_eq!(a.obligation, b.obligation);
                assert_eq!(a.started_at, b.started_at);
                assert_eq!(a.ended_at, b.ended_at);
                true
            });
        third_ten_no_page
            .iter()
            .zip(third_ten_yes_page.iter())
            .filter(|(a, b)| {
                assert_eq!(a.id, b.id);
                assert_eq!(a.authority, b.authority);
                assert_eq!(a.temp_liquidation_account, b.temp_liquidation_account);
                assert_eq!(a.user_farm, b.user_farm);
                assert_eq!(a.obligation, b.obligation);
                assert_eq!(a.started_at, b.started_at);
                assert_eq!(a.ended_at, b.ended_at);
                true
            });
    }
    #[test]
    #[allow(unused_must_use)]
    fn test_historic_tshare_price() {
        use crate::test_utils::TestDb;
        env::set_var(
            "DATABASE_URL",
            "postgres://postgres:password123@localhost/tulip",
        );
        let test_db = TestDb::new();
        let conn = test_db.conn();
        let client = Arc::new(DBClient {
            conn: &conn,
            oob_limit: 25.0,
        });
        crate::run_migrations(client.conn);
        std::thread::sleep(std::time::Duration::from_secs(2));
        let cleanup = || {
            client.delete_historic_tshare_price(&HistoricTSharePriceMatcher::All);
        };
        cleanup();

        client
            .put_historic_tshare_price("token1", 420.69, 6.9, 9.6, Utc::now())
            .unwrap();
        let results = client
            .get_historic_tshare_price(&HistoricTSharePriceMatcher::All)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].price, 420.69);
        assert_eq!(results[0].total_supply, 6.9);
        assert_eq!(results[0].holder_count, 9.6);
        assert_eq!(&results[0].farm_name, "token1");

        client
            .put_historic_tshare_price("token2", 69.420, 4.2, 2.4, Utc::now())
            .unwrap();
        let results = client
            .get_historic_tshare_price(&HistoricTSharePriceMatcher::All)
            .unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[1].price, 69.420);
        assert_eq!(results[1].total_supply, 4.2);
        assert_eq!(results[1].holder_count, 2.4);
        assert_eq!(&results[1].farm_name, "token2");

        client
            .put_historic_tshare_price("token1", 69.69, 8.4, 4.8, Utc::now())
            .unwrap();
        let results = client
            .get_historic_tshare_price(&HistoricTSharePriceMatcher::All)
            .unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[2].price, 69.69);
        assert_eq!(results[2].total_supply, 8.4);
        assert_eq!(results[2].holder_count, 4.8);
        assert_eq!(&results[2].farm_name, "token1");

        let results = client
            .get_historic_tshare_price(&HistoricTSharePriceMatcher::FarmName(vec![
                "token1".to_string()
            ]))
            .unwrap();
        assert_eq!(results.len(), 2);

        cleanup();

        let results = client
            .get_historic_tshare_price(&HistoricTSharePriceMatcher::All)
            .unwrap();
        assert_eq!(results.len(), 0);
    }
    #[test]
    #[allow(unused_must_use)]
    fn test_historic_tshare_price_ranged() {
        use crate::test_utils::TestDb;
        env::set_var(
            "DATABASE_URL",
            "postgres://postgres:password123@localhost/tulip",
        );
        let test_db = TestDb::new();
        let conn = test_db.conn();
        let client = Arc::new(DBClient {
            conn: &conn,
            oob_limit: 25.0,
        });
        crate::run_migrations(client.conn);
        std::thread::sleep(std::time::Duration::from_secs(2));
        let cleanup = || {
            client.delete_historic_tshare_price(&HistoricTSharePriceMatcher::All);
        };
        cleanup();
        let now = Utc::now();
        // store a price recorded 24 hours ago
        client
            .put_historic_tshare_price(
                "token1",
                420.69,
                420.69,
                420.69,
                now.checked_sub_signed(
                    chrono::Duration::from_std(std::time::Duration::from_secs(86400)).unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
        // store a price recorded 12 hours ago
        client
            .put_historic_tshare_price(
                "token1",
                69.69,
                69.69,
                69.69,
                now.checked_sub_signed(
                    chrono::Duration::from_std(std::time::Duration::from_secs(46400)).unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
        // store a price recorded now
        client
            .put_historic_tshare_price("token1", 6969.69, 6969.69, 6969.69, now)
            .unwrap();
        // store a price recorded in the future
        client
            .put_historic_tshare_price(
                "token1",
                69.69,
                69.69,
                69.69,
                now.checked_add_signed(
                    chrono::Duration::from_std(std::time::Duration::from_secs(46400)).unwrap(),
                )
                .unwrap(),
            )
            .unwrap();

        let results = query_paginated_historic_prices(
            &conn,
            &HistoricTSharePriceMatcher::All,
            now.checked_sub_signed(
                chrono::Duration::from_std(std::time::Duration::from_secs(86400)).unwrap(),
            )
            .unwrap(),
            now,
            None,
            None,
            false,
        )
        .unwrap()
        .0;
        assert_eq!(results.len(), 3);

        let results = query_paginated_historic_prices(
            &conn,
            &HistoricTSharePriceMatcher::All,
            now.checked_sub_signed(
                chrono::Duration::from_std(std::time::Duration::from_secs(86400)).unwrap(),
            )
            .unwrap(),
            now.checked_add_signed(
                chrono::Duration::from_std(std::time::Duration::from_secs(46400)).unwrap(),
            )
            .unwrap(),
            None,
            None,
            false,
        )
        .unwrap()
        .0;
        assert_eq!(results.len(), 4)
    }
    #[test]
    #[allow(unused_must_use)]
    fn test_v1_obligation_account() {
        use crate::test_utils::TestDb;
        env::set_var(
            "DATABASE_URL",
            "postgres://postgres:password123@localhost/tulip",
        );
        let test_db = TestDb::new();
        let conn = test_db.conn();
        let client = Arc::new(DBClient {
            conn: &conn,
            oob_limit: 25.0,
        });
        crate::run_migrations(client.conn);
        std::thread::sleep(std::time::Duration::from_secs(2));
        let cleanup = || {
            client.delete_v1_obligation_account(&V1ObligationAccountMatcher::All);
        };
        cleanup();
        client
            .put_v1_obligation_account("account1", "authority1")
            .unwrap();
        let results = client
            .get_v1_obligation_account(&V1ObligationAccountMatcher::AccountAddress(vec![
                "account1".to_string(),
            ]))
            .unwrap();
        assert_eq!(results.len(), 1);
        client
            .put_v1_obligation_account("account2", "authority1")
            .unwrap();
        let results = client
            .get_v1_obligation_account(&V1ObligationAccountMatcher::AccountAddress(vec![
                "account2".to_string(),
            ]))
            .unwrap();
        assert_eq!(results.len(), 1);
        let results = client
            .get_v1_obligation_account(&V1ObligationAccountMatcher::Authority(vec![
                "authority1".to_string()
            ]))
            .unwrap();
        assert_eq!(results.len(), 2);
    }
}

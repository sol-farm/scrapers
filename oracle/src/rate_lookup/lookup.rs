//! provides functions for interest rate lookups

#![deny(unused_must_use)]

use anchor_client::{solana_client::rpc_client::RpcClient, solana_sdk::program_pack::Pack};
use anyhow::{anyhow, Result};
use az::CheckedCast;

use common::mango::mango_common::Loadable;
use common::mango::mango_lib::state::ZERO_I80F48;
use common::mango::mango_lib::state::{MangoCache, MangoGroup, NodeBank, RootBank};
use common::mango::mango_lib::utils::compute_deposit_rate;
use common::math::decimal::Decimal;
use common::solend::solend_token_lending::state as solend_state;
use common::tulip as tulip_state;
use common::{
    math::rate::Rate, port::port_variable_rate_lending_instructions::state as port_state,
};

use config::analytics::Platform;
use config::Configuration;

use std::convert::TryInto;
use std::str::FromStr;
use std::sync::Arc;
#[derive(Default, Debug, Clone)]
pub struct InterestRateSample {
    pub asset: String,
    pub platform: String,
    pub rate: f64, // borrow rate
    pub available_amount: f64,
    pub borrowed_amount: f64,
    pub utilization_rate: f64, // utilization rate
    pub interest_rate: f64,    // lending rate
}

/// lookup interest rates for the given asset and platform
/// values returns are in APR
pub fn interest_rate(
    config: &Arc<Configuration>,
    rpc: &Arc<RpcClient>,
    asset: &str,
    platform: &str,
) -> Result<InterestRateSample> {
    // retrieve the config that describes all the routes needed to compute price
    let rate_config = match config.analytics.interest_rates.rate(platform, asset) {
        Ok(config) => config,
        Err(err) => {
            return Err(anyhow!(
                "failed to find interest_rate for platform({})-asset({}) {:?}",
                platform,
                asset,
                err
            ))
        }
    };

    // fetch a deduped list of accounts needed
    let account_keys = rate_config.account_keys()?;

    // create a hashmap which maps account key -> account data
    let account_map = crate::account_keys_to_account_map(&account_keys, rpc)?;

    let (borrow_rate, interest_rate, utilization_rate, available_amount, borrowed_amount) =
        if let Some(spl_config) = rate_config.spl_lending_config {
            if let Some(reserve) = account_map.get(&spl_config.reserve()) {
                match rate_config.platform {
                    Platform::MangoV3 => return Err(anyhow!("mangov3 unsupported")),
                    Platform::Port => {
                        let reserve_account =
                            port_state::Reserve::unpack_unchecked(&reserve.data[..])?;

                        let available_amount = reserve_account.liquidity.available_amount;
                        let borrowed_amount = reserve_account
                            .liquidity
                            .borrowed_amount_wads
                            .try_floor_u64()?;
                        // convert to a common format to ensure consistency across all metrics
                        // we're using the tulip lending program as the standard :based_cigar:
                        let port_rate = reserve_account.current_borrow_rate()?;
                        let port_util_rate = reserve_account.liquidity.utilization_rate()?;

                        let port_rate_u128 = port_rate.to_scaled_val();
                        let port_util_rate_u128 = port_util_rate.to_scaled_val();

                        let tulip_standard_borrow_rate =
                            Decimal::from(Rate::from_scaled_val_big(port_rate_u128)).to_string();
                        let tulip_standard_util_rate =
                            Decimal::from(Rate::from_scaled_val_big(port_util_rate_u128))
                                .to_string();

                        ensure_i64_to_u64_safety(available_amount, "available_amount (port)")?;
                        ensure_i64_to_u64_safety(borrowed_amount, "borrowed_amount (port)")?;

                        let borrow_rate =
                            f64::from_str(tulip_standard_borrow_rate.as_str()).unwrap();
                        let utilization_rate =
                            f64::from_str(tulip_standard_util_rate.as_str()).unwrap();
                        let interest_rate = borrow_rate * utilization_rate;

                        let available_amount = spl_token::amount_to_ui_amount(
                            available_amount,
                            reserve_account.liquidity.mint_decimals,
                        );
                        let borrowed_amount = spl_token::amount_to_ui_amount(
                            borrowed_amount,
                            reserve_account.liquidity.mint_decimals,
                        );
                        (
                            borrow_rate * 100_f64,
                            interest_rate * 100_f64,
                            utilization_rate * 100_f64,
                            available_amount,
                            borrowed_amount,
                        )
                    }
                    Platform::Tulip => {
                        let reserve_account =
                            tulip_state::lending_reserve::Reserve::unpack_unchecked(
                                &reserve.data[..],
                            )?;
                        let available_amount = reserve_account.liquidity.available_amount;
                        let borrowed_amount = reserve_account
                            .liquidity
                            .borrowed_amount_wads
                            .try_floor_u64()?;

                        // convert to a common format to ensure consistency across all metrics
                        // we're using the tulip lending program as the standard :based_cigar:
                        let tulip_rate = reserve_account.current_borrow_rate()?;
                        let tulip_util_rate = reserve_account.liquidity.utilization_rate()?;
                        let tulip_rate_u128 = tulip_rate.to_scaled_val();
                        let tulip_util_rate_u128 = tulip_util_rate.to_scaled_val();

                        let tulip_standard_borrow_rate =
                            Decimal::from(Rate::from_scaled_val_big(tulip_rate_u128)).to_string();
                        let tulip_standard_util_rate =
                            Decimal::from(Rate::from_scaled_val_big(tulip_util_rate_u128))
                                .to_string();

                        ensure_i64_to_u64_safety(available_amount, "available_amount (port)")?;
                        ensure_i64_to_u64_safety(borrowed_amount, "borrowed_amount (port)")?;

                        let borrow_rate =
                            f64::from_str(tulip_standard_borrow_rate.as_str()).unwrap();
                        let utilization_rate =
                            f64::from_str(tulip_standard_util_rate.as_str()).unwrap();
                        let interest_rate = borrow_rate * utilization_rate;
                        let available_amount = spl_token::amount_to_ui_amount(
                            available_amount,
                            reserve_account.liquidity.mint_decimals,
                        );
                        let borrowed_amount = spl_token::amount_to_ui_amount(
                            borrowed_amount,
                            reserve_account.liquidity.mint_decimals,
                        );
                        (
                            borrow_rate * 100_f64,
                            interest_rate * 100_f64,
                            utilization_rate * 100_f64,
                            available_amount,
                            borrowed_amount,
                        )
                    }
                    Platform::Solend => {
                        let reserve_account =
                            solend_state::Reserve::unpack_unchecked(&reserve.data[..])?;
                        let available_amount = reserve_account.liquidity.available_amount;
                        let borrowed_amount = reserve_account
                            .liquidity
                            .borrowed_amount_wads
                            .try_floor_u64()?;
                        // convert to a common format to ensure consistency across all metrics
                        // we're using the tulip lending program as the standard :based_cigar:
                        let solend_rate = reserve_account.current_borrow_rate()?;
                        let solend_util_rate = reserve_account.liquidity.utilization_rate()?;

                        let solend_rate_u128 = solend_rate.to_scaled_val();
                        let solend_util_rate_u128 = solend_util_rate.to_scaled_val();

                        let tulip_standard_borrow_rate =
                            Decimal::from(Rate::from_scaled_val_big(solend_rate_u128)).to_string();
                        let tulip_standard_util_rate =
                            Decimal::from(Rate::from_scaled_val_big(solend_util_rate_u128))
                                .to_string();

                        ensure_i64_to_u64_safety(available_amount, "available_amount (port)")?;
                        ensure_i64_to_u64_safety(borrowed_amount, "borrowed_amount (port)")?;

                        let borrow_rate =
                            f64::from_str(tulip_standard_borrow_rate.as_str()).unwrap();
                        let utilization_rate =
                            f64::from_str(tulip_standard_util_rate.as_str()).unwrap();
                        let interest_rate = borrow_rate * utilization_rate;
                        let available_amount = spl_token::amount_to_ui_amount(
                            available_amount,
                            reserve_account.liquidity.mint_decimals,
                        );
                        let borrowed_amount = spl_token::amount_to_ui_amount(
                            borrowed_amount,
                            reserve_account.liquidity.mint_decimals,
                        );
                        (
                            borrow_rate * 100_f64,
                            interest_rate * 100_f64,
                            utilization_rate * 100_f64,
                            available_amount,
                            borrowed_amount,
                        )
                    }
                    _ => return Err(anyhow!("invalid platform for interest rate sampling")),
                }
            } else {
                return Err(anyhow!(
                    "failed to fine reserve account for {}",
                    spl_config.reserve()
                ));
            }
        } else if let Some(mango_config) = rate_config.mango_config {
            let group_account = match account_map.get(&mango_config.group()) {
                Some(account) => account,
                None => return Err(anyhow!("mango group account not found")),
            };
            let cache_account = match account_map.get(&mango_config.cache()) {
                Some(account) => account,
                None => return Err(anyhow!("mango cache account not found")),
            };
            let root_bank_account = match account_map.get(&mango_config.root_bank()) {
                Some(account) => account,
                None => return Err(anyhow!("mango root_bank account not found")),
            };
            let node_bank_account = match account_map.get(&mango_config.node_bank()) {
                Some(account) => account,
                None => return Err(anyhow!("mango group account not found")),
            };
            let _group_token_account_account =
                match account_map.get(&mango_config.group_token_account()) {
                    Some(account) => account,
                    None => return Err(anyhow!("mango group account not found")),
                };

            let mango_group = MangoGroup::load_from_bytes(&group_account.data[..])?;
            let mango_root_bank = RootBank::load_from_bytes(&root_bank_account.data[..])?;
            let mango_node_bank = NodeBank::load_from_bytes(&node_bank_account.data[..])?;
            let mango_cache = MangoCache::load_from_bytes(&cache_account.data[..])?;

            let token_index = match mango_group.find_root_bank_index(&mango_config.root_bank()) {
                Some(idx) => idx,
                None => {
                    return Err(anyhow!(
                        "failed to find root bank {} in group {}",
                        mango_config.root_bank(),
                        mango_config.group()
                    ))
                }
            };

            let root_bank_cache = &mango_cache.root_bank_cache[token_index];
            let total_native_deposit = mango_node_bank.deposits * root_bank_cache.deposit_index;
            let total_native_borrow = mango_node_bank.borrows * root_bank_cache.borrow_index;
            // taken from RootBank::update_index
            let utilization = total_native_borrow
                .checked_div(total_native_deposit)
                .unwrap_or(ZERO_I80F48);
            // calculate interest rate

            let (deposit_interest, interest_rate) =
                match compute_deposit_rate(mango_root_bank, utilization) {
                    Some((deposit_interest, interest_rate)) => (deposit_interest, interest_rate),
                    None => return Err(anyhow!("failed to compute deposit and interest rates")),
                };

            let borrow_interest: f64 = match interest_rate.checked_cast() {
                Some(val) => val,
                None => return Err(anyhow!("failed to cast borrow rate to float")),
            };
            let deposit_interest: f64 = match deposit_interest.checked_cast() {
                Some(val) => val,
                None => return Err(anyhow!("failed to cast interest rate to float64")),
            };
            let utilization: f64 = match utilization.checked_cast() {
                Some(val) => val,
                None => return Err(anyhow!("failed to cast utilization to float64")),
            };
            let borrow_interest = borrow_interest * 100_f64;
            let deposit_interest = deposit_interest * 100_f64;
            let utilization = utilization * 100_f64;

            let available_amount = total_native_deposit
                .checked_sub(total_native_borrow)
                .unwrap();

            let available_amount: u64 = match available_amount.checked_cast() {
                Some(val) => val,
                None => return Err(anyhow!("failed to cast avaialble amount to f64")),
            };

            let borrowed_amount: u64 = match total_native_borrow.checked_cast() {
                Some(val) => val,
                None => return Err(anyhow!("failed to cast borrowed amount to f64")),
            };

            let decimals = rate_config.decimals;

            let available_amount = spl_token::amount_to_ui_amount(available_amount, decimals);
            let borrowed_amount = spl_token::amount_to_ui_amount(borrowed_amount, decimals);

            (
                borrow_interest,
                deposit_interest,
                utilization,
                available_amount,
                borrowed_amount,
            )
        } else {
            return Err(anyhow!("no valid config to derive rates from"));
        };

    Ok(InterestRateSample {
        asset: asset.to_string(),
        platform: platform.to_string(),
        rate: borrow_rate,
        utilization_rate,
        available_amount,
        borrowed_amount,
        interest_rate,
    })
}

/// because postgres doens't support u64 data type
/// and only suppotrs i64, we need to store u64 types
/// as i64. however we dont want to lose precision so this
/// validates that we can convert from i64 <--> u64 without losing precision
pub fn ensure_i64_to_u64_safety(value: u64, name: &str) -> Result<()> {
    let value_i: i64 = if let Ok(val) = value.try_into() {
        val
    } else {
        return Err(anyhow!("failed to convert {} to i64", name));
    };
    let value_u: u64 = if let Ok(val) = value_i.try_into() {
        val
    } else {
        return Err(anyhow!("failed to convert {} back to u64", name));
    };
    if value_u != value {
        return Err(anyhow!("{} lost precision during type cast", name));
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use anchor_client::solana_client::rpc_client::RpcClient;

    use config::analytics::interest_rates::AssetRate;
    use config::analytics::interest_rates::{MangoConfiguration, SplLendingConfiguration};

    use config::analytics::Platform;

    use config::Configuration;

    use std::sync::Arc;
    #[test]
    #[allow(unused_must_use)]
    fn tset_tulip_interest_rates() {
        // tests deriving usd price of SOL by the following route
        // SOL -> RAY, RAY -> USDC
        let mut config = Configuration::default();
        config.init_log(false);
        let rpc = Arc::new(RpcClient::new("https://ssc-dao.genesysgo.net".to_string()));
        config.analytics.interest_rates.assets.push(AssetRate {
            asset: "USDC".to_string(),
            platform: Platform::Tulip,
            program_id: common::tulip::LENDING_PROGRAM_ID.to_string(),
            spl_lending_config: Some(SplLendingConfiguration {
                reserve: common::tulip::USDC_RESERVE_ACCOUNT.to_string(),
                pyth_oracle: common::tulip::USDC_PYTH_PRICE_ACCOUNT.to_string(),
                switchboard_oracle: None,
                solend_rate_args: None,
                tulip_rate_args: None,
            }),
            mango_config: None,
            decimals: 6,
        });
        let interest_sample = interest_rate(
            &Arc::new(config),
            &rpc,
            "USDC",
            &Platform::Tulip.to_string(),
        )
        .unwrap();
        println!("rate sample {:#?}", interest_sample);
    }
    #[test]
    #[allow(unused_must_use)]
    fn tset_mango_interest_rates() {
        // tests deriving usd price of SOL by the following route
        // SOL -> RAY, RAY -> USDC
        let mut config = Configuration::default();
        config.init_log(false);
        let rpc = Arc::new(RpcClient::new("https://ssc-dao.genesysgo.net".to_string()));
        config.analytics.interest_rates.assets.push(AssetRate {
            asset: "USDC".to_string(),
            platform: Platform::MangoV3,
            program_id: common::mango::MANGO_V3_PROGRAM_ID.to_string(),
            spl_lending_config: None,
            mango_config: Some(MangoConfiguration {
                group: common::mango::MANGO_V3_GROUP_KEY.to_string(),
                cache: common::mango::MANGO_V3_CACHE.to_string(),
                root_bank: common::mango::MANGO_V3_MAINNET_ONE_USDC_ROOT_KEY.to_string(),
                node_bank: common::mango::MANGO_V3_MAINNET_ONE_USDC_NODE_KEYS[0].to_string(),
                group_token_account: common::mango::MANGO_V3_USDC_TOKEN_VAULT.to_string(),
                rate_args: Default::default(),
            }),
            decimals: 6,
        });
        let interest_sample = interest_rate(
            &Arc::new(config),
            &rpc,
            "USDC",
            &Platform::MangoV3.to_string(),
        )
        .unwrap();
        println!("rate sample {:#?}", interest_sample);
    }
}

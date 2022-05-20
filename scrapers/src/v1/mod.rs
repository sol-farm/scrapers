//! v1 tulip position scrapers
use anchor_client::anchor_lang::AccountDeserialize;
use anchor_client::anchor_lang::AnchorDeserialize;
use anchor_client::solana_sdk::pubkey::Pubkey;
use anyhow::{anyhow, Result};
use chrono::prelude::*;
use common::tulip::lending_obligation::Obligation;
use common::v1::accounts::margin::ObligationLiquidationAccount;
use common::{self, v1::accounts::margin::UserFarm};
use config::Configuration;
use crossbeam_channel::{select, tick};
use db::{client, filters::V1UserFarmMatcher};
use diesel::PgConnection;
use log::{error, info, warn};
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, MemcmpEncodedBytes, MemcmpEncoding, RpcFilterType},
};
use solana_sdk::account_info::IntoAccountInfo;
use solana_sdk::program_pack::Pack;
use std::ops::Deref;
use std::{sync::Arc, time::Duration};

/// does not scrape and calculate obligation ltvs, and instead simply stores
/// previously unseed obiligation accounts into the database
pub fn scrape_obligation_accounts(
    rpc: &Arc<RpcClient>,
    config: &Arc<Configuration>,
    conn: &PgConnection,
    compression: bool,
) {
    let db_client = Arc::new(db::client::DBClient {
        conn: &conn,
        oob_limit: 10_f64,
    });
    let encoding = if compression {
        Some(UiAccountEncoding::Base64Zstd)
    } else {
        Some(UiAccountEncoding::Base64)
    };
    let lending_id = config.programs.v1_lending();
    let work_loop = |db_client: &Arc<db::client::DBClient>, lending_id: &Pubkey| match rpc
        .get_program_accounts_with_config(
            lending_id,
            RpcProgramAccountsConfig {
                filters: Some(vec![RpcFilterType::DataSize(
                    common::v1::accounts::OBLIGATION_ACCOUNT_SIZE as u64,
                )]),
                with_context: None,
                account_config: RpcAccountInfoConfig {
                    encoding,
                    data_slice: None,
                    commitment: None,
                },
            },
        ) {
        Ok(mut accounts) => {
            for (key, account) in accounts.iter_mut() {
                let key = std::mem::take(key);
                let account = std::mem::take(account);
                match Obligation::unpack_unchecked(&account.data[..]) {
                    Ok(obligation) => {
                        match db_client.put_v1_obligation_account(
                            &key.to_string(),
                            &obligation.owner.to_string(),
                        ) {
                            Ok(_) => info!("found new obligation {}", key.to_string()),
                            Err(_) => (),
                        }
                    }
                    Err(err) => error!("failed to unpack obligation {}: {:#?}", key, err),
                }
            }
        }
        Err(err) => error!("failed scrape obligations {:#?}", err),
    };
    let start = Utc::now();
    work_loop(&db_client, &lending_id);
    let time_took = Utc::now().signed_duration_since(start);
    info!(
        "obligation scraping took {} seconds",
        time_took.num_seconds()
    );
}

pub fn scrape_user_farm(
    rpc: &Arc<RpcClient>,
    config: &Arc<Configuration>,
    conn: &PgConnection,
    compression: bool,
) {
    let db_client = Arc::new(db::client::DBClient {
        conn: &conn,
        oob_limit: 10_f64,
    });
    let farm_id = config.programs.v1_farm();
    let encoding = if compression {
        Some(UiAccountEncoding::Base64Zstd)
    } else {
        Some(UiAccountEncoding::Base64)
    };
    let work_loop = |db_client: &Arc<db::client::DBClient>, farm_id: &Pubkey| match rpc
        .get_program_accounts_with_config(
            farm_id,
            RpcProgramAccountsConfig {
                filters: Some(vec![RpcFilterType::DataSize(
                    common::v1::accounts::USER_FARM_ACCOUNT_SIZE as u64,
                )]),
                with_context: None,
                account_config: RpcAccountInfoConfig {
                    encoding,
                    data_slice: None,
                    commitment: None,
                },
            },
        ) {
        Ok(mut accounts) => {
            for (key, account) in accounts.iter_mut() {
                let key = std::mem::take(key);
                let account = std::mem::take(account);
                match UserFarm::deserialize(&mut &account.data[..]) {
                    Ok(user_farm) => {
                        let mut obligations = Vec::with_capacity(3);
                        let mut obligation_indexes = Vec::with_capacity(3);
                        for i in 0..user_farm.obligations.len() {
                            if user_farm.obligations[i].obligation_account != common::DEFAULT_KEY {
                                obligations
                                    .push(user_farm.obligations[i].obligation_account.to_string());
                                obligation_indexes.push(i as i32);
                            }
                        }
                        match db_client.put_v1_user_farm(
                            &user_farm.authority.to_string(),
                            &key.to_string(),
                            &user_farm.leveraged_farm.to_string(),
                            &obligations,
                            &obligation_indexes,
                        ) {
                            Ok(_) => (),
                            Err(err) => error!("failed to put user farm {}: {:#?}", key, err),
                        }
                    }
                    Err(err) => error!("failed to deserialize userfarm {}: {:#?}", key, err),
                }
            }
        }
        Err(err) => error!("failed to scrape user farms {:#?}", err),
    };
    let start = Utc::now();
    work_loop(&db_client, &farm_id);
    let time_took = Utc::now().signed_duration_since(start);
    info!(
        "user farm scraping took {} seconds",
        time_took.num_seconds()
    );
}

pub fn find_temp_liquidation_accounts(
    rpc: &RpcClient,
    config: &Configuration,
    compression: bool,
) -> Result<Vec<(Pubkey, ObligationLiquidationAccount)>> {
    let farm_id = config.programs.v1_farm();
    let encoding = if compression {
        Some(UiAccountEncoding::Base64Zstd)
    } else {
        Some(UiAccountEncoding::Base64)
    };
    match rpc.get_program_accounts_with_config(
        &farm_id,
        RpcProgramAccountsConfig {
            filters: Some(vec![RpcFilterType::DataSize(345 as u64)]),
            with_context: None,
            account_config: RpcAccountInfoConfig {
                encoding,
                data_slice: None,
                commitment: None,
            },
        },
    ) {
        Ok(mut accounts) => {
            let mut response = Vec::with_capacity(accounts.len());
            for (key, account) in accounts.iter_mut() {
                let key = std::mem::take(key);
                let account = std::mem::take(account);
                let mut account_tuple = (key, account);
                let obligation_account_info = account_tuple.into_account_info();
                match ObligationLiquidationAccount::load(&obligation_account_info) {
                    Ok(acct) => {
                        response.push((key, *acct));
                    }
                    Err(err) => error!("failed to deserialize temp liquidation account {:#?}", err),
                };
            }
            Ok(response)
        }
        Err(err) => Err(anyhow!(
            "failed to find temp liquidation accounts {:#?}",
            err
        )),
    }
}

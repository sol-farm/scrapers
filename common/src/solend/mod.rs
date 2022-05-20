#![allow(clippy::too_many_arguments)]

use anchor_lang::prelude::*;
use solana_program::{instruction::Instruction, program_pack::Pack, sysvar};
pub use solend_token_lending;
use solend_token_lending::state::Obligation;
use solend_token_lending::state::Reserve;
use spl_token_lending::instruction::LendingInstruction;
use static_pubkey::static_pubkey;

/// full information available https://github.com/solendprotocol/common/blob/master/src/production.json

/// address of the solend lending program
pub const LENDING_PROGRAM_ID: Pubkey =
    static_pubkey!("So1endDq2YkqhipRh3WViPa8hdiSpxWy6z3Z6tMCpAo");

/// address of the solend lending market
pub const LENDING_MARKET: Pubkey = static_pubkey!("4UpD2fh7xH3VP9QQaXtsS1YY3bxzWhtfpks7FatyKvdY");
/// address of the lendign market owner
pub const DERIVED_LENDING_MARKET_AUTHORITY: Pubkey =
    static_pubkey!("5pHk2TmnqQzRF9L6egy5FfiyBgS7G9cMZ5RFaJAvghzw");
/// address of the solend usdc cToken mint
pub const USDC_COLLATERAL_MINT: Pubkey =
    static_pubkey!("993dVFL2uXWYeoXuEBFXR4BijeXdTv4s6BzsCjJZuwqk");
/// address of the usdc liquidity supply token account
pub const USDC_LIQUIDITY_SUPPLY: Pubkey =
    static_pubkey!("8SheGtsopRUDzdiD6v6BR9a6bqZ9QwywYQY99Fp5meNf");
/// address of the solend usdc reserve
pub const USDC_RESERVE: Pubkey = static_pubkey!("BgxfHJDzm44T7XG68MYKx7YisTjZu73tVovyZSjJMpmw");

/// address of the usdc pyth account
pub const USDC_ORACLE_ADDRESS: Pubkey =
    static_pubkey!("8GWTTbNiXdmyZREXbjsZBmCRuzdPrW55dnZGDkTRjWvb");
/// address of the usdc pyth price account
pub const USDC_ORACLE_PRICE_ADDRESS: Pubkey =
    static_pubkey!("Gnt27xtC473ZT2Mw5u8wZ68Z3gULkSTb5DuxJy7eJotD");
/// address of the usdc swithcboard feed
pub const USDC_SWITCHBOARD_FEED_ADDRESS: Pubkey =
    static_pubkey!("CZx29wKMUxaJDq6aLVQTdViPL754tTR64NAgQBUGxxHb");

pub fn refresh_reserve<'info>(
    lending_program_id: &AccountInfo<'info>,
    clock: &AccountInfo<'info>,
    reserve: &AccountInfo<'info>,
    pyth_price_account: &AccountInfo<'info>,
    switchboard_price_account: &AccountInfo<'info>,
) -> Result<()> {
    let ix = new_refresh_reserve_ix(
        lending_program_id,
        reserve,
        pyth_price_account,
        switchboard_price_account,
    );
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            reserve.clone(),
            pyth_price_account.clone(),
            switchboard_price_account.clone(),
            clock.clone(),
        ],
    )?;
    Ok(())
}

pub fn refresh_obligation<'info>(
    lending_program_id: &AccountInfo<'info>,
    obligation: &AccountInfo<'info>,
    clock: &AccountInfo<'info>,
    reserves: Vec<&AccountInfo<'info>>,
) -> Result<()> {
    let ix = solend_token_lending::instruction::refresh_obligation(
        lending_program_id.key(),
        obligation.key(),
        reserves.iter().map(|reserve| reserve.key()).collect(),
    );
    let mut accounts = Vec::with_capacity(reserves.len() + 3);
    accounts.push(lending_program_id.clone());
    accounts.push(obligation.clone());
    accounts.push(clock.clone());
    for reserve in reserves {
        accounts.push(reserve.clone());
    }
    anchor_lang::solana_program::program::invoke_signed(&ix, &accounts[..], &[])?;
    Ok(())
}

pub fn borrow(
    lending_program_id: Pubkey,
    source_liquidity: Pubkey,
    destinatination_liquidity: Pubkey,
    borrow_reserve: Pubkey,
    borrow_reserve_liquidity_fee_receiver: Pubkey,
    obligation: Pubkey,
    lending_market: Pubkey,
    obligation_owner: Pubkey,
    amount: u64,
) -> Instruction {
    solend_token_lending::instruction::borrow_obligation_liquidity(
        lending_program_id,
        amount,
        source_liquidity,
        destinatination_liquidity,
        borrow_reserve,
        borrow_reserve_liquidity_fee_receiver,
        obligation,
        lending_market,
        obligation_owner,
        None,
    )
}

pub fn repay(
    lending_program_id: Pubkey,
    source_liquidity: Pubkey,
    destination_liquidity: Pubkey,
    repay_reserve: Pubkey,
    obligation: Pubkey,
    lending_market: Pubkey,
    signer: Pubkey,
    amount: u64,
) -> Instruction {
    solend_token_lending::instruction::repay_obligation_liquidity(
        lending_program_id,
        amount,
        source_liquidity,
        destination_liquidity,
        repay_reserve,
        obligation,
        lending_market,
        signer,
    )
}

pub fn new_refresh_reserve_ix<'info>(
    lending_program_id: &AccountInfo<'info>,
    reserve: &AccountInfo<'info>,
    pyth_price_account: &AccountInfo<'info>,
    switchboard_price_account: &AccountInfo<'info>,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(reserve.key(), false),
        AccountMeta::new_readonly(pyth_price_account.key(), false),
        AccountMeta::new_readonly(switchboard_price_account.key(), false),
        AccountMeta::new_readonly(sysvar::clock::id(), false),
    ];
    Instruction {
        program_id: lending_program_id.key(),
        accounts,
        data: LendingInstruction::RefreshReserve.pack(),
    }
}

pub fn load_reserve<'info>(reserve: &AccountInfo<'info>) -> Reserve {
    Reserve::unpack_unchecked(&reserve.data.borrow()[..]).unwrap()
}

pub fn load_obligation<'info>(obligation: &AccountInfo<'info>) -> Obligation {
    Obligation::unpack_unchecked(&obligation.data.borrow()[..]).unwrap()
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    pub fn test_eth_msol_amm() {
        let usdc_reserve =
            Pubkey::from_str("BgxfHJDzm44T7XG68MYKx7YisTjZu73tVovyZSjJMpmw").unwrap();
        let rpc =
            solana_client::rpc_client::RpcClient::new("https://ssc-dao.genesysgo.net".to_string());
        let account = rpc.get_account(&usdc_reserve).unwrap();

        let reserve =
            solend_token_lending::state::Reserve::unpack_unchecked(&account.data[..]).unwrap();
        println!("{:#?}", reserve);
    }
}

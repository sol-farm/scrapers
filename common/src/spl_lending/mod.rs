#![allow(clippy::too_many_arguments)]
//! spl token lending instructions, and utility functions shared by all implementations
use anchor_lang::prelude::*;

use solana_program::instruction::Instruction;
pub use solend_token_lending;
use spl_token_lending::instruction::init_obligation;
pub use spl_token_lending::state as spl_lending_state;

pub fn create_obligation_account<'info>(
    remaining_accounts: &[AccountInfo<'info>],
    obligation_owner: &AccountInfo<'info>,
    clock: &AccountInfo<'info>,
    rent: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    let lending_program_id = remaining_accounts.get(0).unwrap();
    let obligation_account = remaining_accounts.get(1).unwrap();
    let lending_market_account = remaining_accounts.get(2).unwrap();
    let ix = init_obligation(
        *lending_program_id.key,
        *obligation_account.key,
        *lending_market_account.key,
        *obligation_owner.key,
    );
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &[
            obligation_account.clone(),
            lending_market_account.clone(),
            obligation_owner.clone(),
            clock.clone(),
            rent.clone(),
            token_program.clone(),
        ],
        signer_seeds,
    )?;
    Ok(())
}

/// helper function which invokes a refresh reserve
/// instruction, suitable for use by unmodified spl
/// lending program types like tulip or prot
pub fn refresh_reserve<'info>(
    lending_program_id: &AccountInfo<'info>,
    clock: &AccountInfo<'info>,
    reserve: &AccountInfo<'info>,
    oracle: &AccountInfo<'info>,
) -> Result<()> {
    let ix = new_refresh_reserve_ix(lending_program_id, reserve, oracle);
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[reserve.clone(), oracle.clone(), clock.clone()],
    )?;
    Ok(())
}

pub fn new_refresh_reserve_ix<'info>(
    lending_program_id: &AccountInfo<'info>,
    reserve: &AccountInfo<'info>,
    oracle: &AccountInfo<'info>,
) -> Instruction {
    spl_token_lending::instruction::refresh_reserve(
        lending_program_id.key(),
        reserve.key(),
        oracle.key(),
    )
}

pub fn deposit_reserve_liquidity<'info>(
    lending_program_id: &AccountInfo<'info>,
    source_liquidity: &AccountInfo<'info>,
    destination_collateral: &AccountInfo<'info>,
    reserve: &AccountInfo<'info>,
    reserve_liquidity: &AccountInfo<'info>,
    reserve_collateral_mint: &AccountInfo<'info>,
    lending_market: &AccountInfo<'info>,
    lending_market_authority: &AccountInfo<'info>,
    user_transfer_authority: &AccountInfo<'info>,
    clock: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    amount: u64,
) -> Result<()> {
    let ix = spl_token_lending::instruction::deposit_reserve_liquidity(
        lending_program_id.key(),
        amount,
        source_liquidity.key(),
        destination_collateral.key(),
        reserve.key(),
        reserve_liquidity.key(),
        reserve_collateral_mint.key(),
        lending_market.key(),
        user_transfer_authority.key(),
    );
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &[
            source_liquidity.clone(),
            destination_collateral.clone(),
            reserve.clone(),
            reserve_liquidity.clone(),
            reserve_collateral_mint.clone(),
            lending_market.clone(),
            lending_market_authority.clone(),
            user_transfer_authority.clone(),
            clock.clone(),
            token_program.clone(),
        ],
        signer_seeds,
    )?;
    Ok(())
}

pub fn redeem_reserve_collateral<'info>(
    lending_program_id: &AccountInfo<'info>,
    source_collateral: &AccountInfo<'info>,
    destination_liquidity: &AccountInfo<'info>,
    reserve: &AccountInfo<'info>,
    reserve_collateral_mint: &AccountInfo<'info>,
    reserve_liquidity: &AccountInfo<'info>,
    lending_market: &AccountInfo<'info>,
    lending_market_authority: &AccountInfo<'info>,
    user_transfer_authority: &AccountInfo<'info>,
    clock: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    amount: u64,
) -> Result<()> {
    let ix = spl_token_lending::instruction::redeem_reserve_collateral(
        lending_program_id.key(),
        amount,
        source_collateral.key(),
        destination_liquidity.key(),
        reserve.key(),
        reserve_collateral_mint.key(),
        reserve_liquidity.key(),
        lending_market.key(),
        user_transfer_authority.key(),
    );
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &[
            source_collateral.clone(),
            destination_liquidity.clone(),
            reserve.clone(),
            reserve_liquidity.clone(),
            reserve_collateral_mint.clone(),
            lending_market.clone(),
            lending_market_authority.clone(),
            user_transfer_authority.clone(),
            clock.clone(),
            token_program.clone(),
        ],
        signer_seeds,
    )?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use solana_client::rpc_client::RpcClient;
    use solana_program::program_pack::Pack;
    use std::str::FromStr;
    #[test]
    fn test_inspect_port_lending_market() {
        let rpc = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
        let lending_market_data = rpc
            .get_account_data(
                &Pubkey::from_str("6T4XxKerq744sSuj3jaoV6QiZ8acirf4TrPwQzHAoSy5").unwrap(),
            )
            .unwrap();
        let lending_market =
            spl_token_lending::state::LendingMarket::unpack(&lending_market_data[..]).unwrap();
        println!("authority {}", lending_market.owner);
    }
}

#![allow(clippy::needless_lifetimes)]
#![allow(clippy::bool_assert_comparison)]
#![allow(clippy::too_many_arguments)]

//! common math functions, algorithms, numeric types, etc...
//! includes all math related types from the spl token lending program

pub mod common;
pub mod decimal;
pub mod error;
pub mod precise;
pub mod rate;
pub mod uint;
use anchor_lang::solana_program::msg;
use arrform::{arrform, ArrForm};

use std::convert::TryInto;

/// calculate the amount of lp tokens to withdraw based on the amount
/// of of shares given
pub fn calculate_underlying_to_withdraw(
    amount: u64,
    total_vlp_shares: u64,
    total_vault_balance: u64,
) -> u64 {
    ((amount as u128)
        .checked_mul(total_vault_balance as u128)
        .unwrap())
    .checked_div(total_vlp_shares as u128)
    .unwrap()
    .try_into()
    .unwrap()
}

/// calculates the amount of shares to give based on the amount
/// of lp tokens being deposited
pub fn calculate_shares_to_give(
    amount: u64,
    total_vlp_shares: u64,
    total_vault_balance: u64,
) -> u64 {
    ((amount as u128)
        .checked_mul(total_vlp_shares as u128)
        .unwrap())
    .checked_div(total_vault_balance as u128)
    .unwrap()
    .try_into()
    .unwrap()
}

/// calculates the amount of fees to charge factoring in controller and platform fees.
/// fees are expected to be in hundreds, for example 2% would be 200, 4% would be 400
pub fn calculate_fees(reward_account_amount: u64, controller_fee: u64, platform_fee: u64) -> u64 {
    // ( reward amount * (controller fee + platform fee) ) / 100
    (reward_account_amount
        .checked_mul(controller_fee.checked_add(platform_fee).unwrap())
        .unwrap())
    .checked_div(10000)
    .unwrap()
}

/// similar to calculate_fees except it doesn't panic, and returns None if the calculation encounters an error
pub fn calculate_fees_safe(
    reward_account_amount: u64,
    controller_fee: u64,
    platform_fee: u64,
) -> Option<u64> {
    let combined_fees = controller_fee.checked_add(platform_fee)?;
    let a = reward_account_amount.checked_mul(combined_fees)?;
    a.checked_div(10000)
}

pub fn calculate_maximum_coin_pc_amount(
    vault_coin_amount: u64,
    pool_pc_amount: u64,
    pool_coin_amount: u64,
) -> u64 {
    ((vault_coin_amount as u128)
        .checked_mul(pool_pc_amount as u128)
        .unwrap()
        .checked_div(pool_coin_amount as u128))
    .unwrap()
    .try_into()
    .unwrap()
}

pub fn calculate_maximum_pc_coin_amount(
    vault_pc_amount: u64,
    pool_coin_amount: u64,
    pool_pc_amount: u64,
) -> u64 {
    ((vault_pc_amount as u128)
        .checked_mul(pool_coin_amount as u128)
        .unwrap())
    .checked_div(pool_pc_amount as u128)
    .unwrap()
    .try_into()
    .unwrap()
}

pub fn calculate_add_liq_amounts(
    vault_coin_amount: u64,
    vault_pc_amount: u64,
    pool_coin_amount: u64,
    pool_pc_amount: u64,
) -> (u64, u64, u64) {
    let max_coin_amount;
    let max_pc_amount;
    let mut fixed_from_coin = 0;

    // amount of pc tokens required when coin toke  ns are maximized
    let vault_coin_amount_with_slip = vault_coin_amount
        .checked_sub(vault_coin_amount.checked_div(100).unwrap())
        .unwrap();
    let maximum_coin_pc_amount =
        calculate_maximum_coin_pc_amount(vault_coin_amount, pool_pc_amount, pool_coin_amount);

    if maximum_coin_pc_amount <= vault_pc_amount {
        max_coin_amount = vault_coin_amount_with_slip;
        max_pc_amount = maximum_coin_pc_amount;
    } else {
        let vault_pc_amount_with_slip = vault_pc_amount
            .checked_sub(vault_pc_amount.checked_div(100).unwrap())
            .unwrap();
        let maximum_pc_coin_amount =
            calculate_maximum_pc_coin_amount(vault_pc_amount, pool_coin_amount, pool_pc_amount);
        max_coin_amount = maximum_pc_coin_amount;
        max_pc_amount = vault_pc_amount_with_slip;
        fixed_from_coin = 1;
    }

    (max_coin_amount, max_pc_amount, fixed_from_coin)
}

pub fn calculate_amount_out(coin: u64, pc: u64, amount_in: u64) -> u64 {
    ((coin as u128).checked_mul(amount_in as u128).unwrap())
        .checked_div(pc as u128)
        .unwrap()
        .try_into()
        .unwrap()
}

pub fn calculate_min_amount_out(amount_out: u64, slippage: u64) -> u64 {
    (amount_out as u128)
        .checked_sub(
            (amount_out as u128)
                .checked_mul(slippage as u128)
                .unwrap()
                .checked_div(1000_u128)
                .unwrap(),
        )
        .unwrap()
        .try_into()
        .unwrap()
}

pub fn calculate_amount_a_to_swap_to_b(
    amount_a: u64,
    amount_b: u64,
    pool_amount_a: u64,
    pool_amount_b: u64,
) -> u64 {
    msg!(
        "{}",
        arrform!(
            256,
            "amount_a {}, amount_b {}, pool_a {}, pool_b {}",
            amount_a,
            amount_b,
            pool_amount_a,
            pool_amount_b
        )
        .as_str()
    );
    ((amount_a as u128)
        .checked_sub(
            ((amount_b as u128)
                .checked_mul(pool_amount_a as u128)
                .unwrap() as u128)
                .checked_div(pool_amount_b as u128)
                .unwrap() as u128,
        )
        .unwrap() as u128)
        .checked_div(2_u128)
        .unwrap()
        .try_into()
        .unwrap()
}

/// used to evaluate whether or not we should swap rewards on the serum dex in order
/// to prevent heavy imbalance from occuring on either sides of an lp's constituent assets
pub fn check_serum_swap_skip(
    max_coin_qty: u64,
    limit_price: u64,
    pc_amount: u64,
    decimals: u8,
) -> bool {
    let val: u64 = (max_coin_qty as u128)
        .checked_mul(limit_price as u128)
        .unwrap()
        .checked_div(u128::checked_pow(10, decimals as u32).unwrap())
        .unwrap()
        .try_into()
        .unwrap();
    val < pc_amount
}

/// used to evaluate whether or not we should swap rewards in an lp pair in order
/// to prevent heavy imbalance from occuring on either sides of an lp's constituent assets
/// this should be usable across any tradiditional AMM that uses the X*Y=K formula
/// but should also be usable, although less accurate on stable curves, etc...
///
/// although these are labeled coin/pc, they should be usable as either,
/// as long as you are consistent. for example if swapping on BASIS/USDC
/// and trading USDC -> BASIS, you would use USDC as the "coin"
///
///
/// todo(bonedaddy): should this be renamed as pool_source_amount, pool_dest_amount
///
pub fn check_amm_swap_skip(
    // the amount of coin tokens owned by the pool
    pool_coin_amount: u64,
    // the amount of pc tokens owned by the pool
    pool_pc_amount: u64,
    // the amount of coin tokens owned by the vault
    vault_coin_amount: u64,
    // the amount of pc tokens owned by the vault
    vault_pc_amount: u64,
) -> bool {
    ((((pool_pc_amount as u128)
        .checked_mul(vault_coin_amount as u128)
        .unwrap())
    .checked_div(pool_coin_amount as u128)
    .unwrap()) as u64)
        < vault_pc_amount
}

/// calculates the tulip reward per share in a vault
pub fn reward_per_share(
    tulip_reward_per_share: u128,
    reward_applicable_slot: u64,
    tulip_reward_per_slot: u64,
    last_interaction_slot: u64,
    total_vlp_shares: u64,
) -> u128 {
    if total_vlp_shares == 0 {
        return tulip_reward_per_share;
    }

    (tulip_reward_per_share)
        .checked_add(
            ((((reward_applicable_slot as u128)
                .checked_sub(last_interaction_slot as u128)
                .unwrap())
            .checked_mul(tulip_reward_per_slot as u128)
            .unwrap())
            .checked_mul(u128::checked_pow(10, 18_u32).unwrap())
            .unwrap())
            .checked_div(total_vlp_shares as u128)
            .unwrap(),
        )
        .unwrap()
}

/// calculate reward tulip reward earned currently
pub fn reward_earned(
    user_shares: u64,
    reward_per_share: u128,
    reward_per_share_paid: u128,
    last_pending_reward: u64,
) -> u64 {
    ((((user_shares as u128)
        .checked_mul(
            (reward_per_share)
                .checked_sub(reward_per_share_paid)
                .unwrap(),
        )
        .unwrap())
    .checked_div(u128::checked_pow(10, 18_u32).unwrap())
    .unwrap())
    .checked_add(last_pending_reward as u128)
    .unwrap())
    .try_into()
    .unwrap()
}

pub fn do_swap(
    vault_coin_amount: u64,
    vault_pc_amount: u64,
    pool_coin_amount: u64,
    pool_pc_amount: u64,
) -> bool {
    if ((((pool_pc_amount as u128)
        .checked_mul(vault_coin_amount as u128)
        .unwrap())
    .checked_div(pool_coin_amount as u128)
    .unwrap()) as u64)
        < vault_pc_amount
    {
        return false;
    }

    true
}

pub struct AverageExecutionPriceArgs {
    pub coin_diff: u128,
    pub pc_diff: u128,
    pub coin_decimals: u128,
    pub pc_decimals: u128,
    pub pair_tick_num: u128,
    pub pair_tick_denom: u128,
    /// the best ask/bid price
    pub best_price: u128,
}
/// returns a tuple of (avg_price, price_slippage)
pub fn calculate_average_execution_price_with_slippage(
    args: AverageExecutionPriceArgs,
) -> Option<(u64, u64)> {
    let average_execution_price = match calculate_average_execution_price(
        args.coin_diff,
        args.pc_diff,
        args.coin_decimals,
        args.pc_decimals,
        args.pair_tick_num,
        args.pair_tick_denom,
    ) {
        Some(avg_price) => avg_price,
        None => return None,
    };
    match calculate_execution_price_slippage(args.best_price, average_execution_price) {
        Some(price_slippage) => {
            let avg_exe_price: u64 = match average_execution_price.try_into() {
                Ok(avg_exe_price) => avg_exe_price,
                Err(_) => return None,
            };
            Some((avg_exe_price, price_slippage))
        }
        None => None,
    }
}

pub fn calculate_average_execution_price(
    coin_diff: u128,
    pc_diff: u128,
    coin_decimals: u128,
    pc_decimals: u128,
    pair_tick_num: u128,
    pair_tick_denom: u128,
) -> Option<u128> {
    let pc_diff_num = pc_diff.checked_mul(pair_tick_denom)?;
    let coin_diff_denom = coin_diff.checked_mul(pair_tick_num)?;

    if let Some(a) = pc_diff_num.checked_mul(coin_decimals) {
        if let Some(b) = a.checked_div(pc_decimals) {
            b.checked_div(coin_diff_denom)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn calculate_execution_price_slippage(
    best_price: u128,
    average_execution_price: u128,
) -> Option<u64> {
    if let Some(a) = best_price.checked_sub(average_execution_price) {
        if let Some(b) = a.checked_mul(100) {
            if let Some(c) = b.checked_div(best_price) {
                match c.try_into() {
                    Ok(val) => Some(val),
                    Err(_) => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

/// calculate the amount of shares we need to burn in order to receive
/// `amount` of underlying
pub fn calculate_shares_to_burn(
    amount: u64,
    total_vlp_shares: u64,
    total_vault_balance: u64,
) -> u64 {
    let numerator = (amount as u128)
        .checked_mul(total_vlp_shares as u128)
        .unwrap();
    msg!("amount {}", amount);
    msg!("numerator {}", numerator);
    let denominator = total_vault_balance as u128;
    msg!("denominator {}", denominator);
    let shares_to_burn = numerator.checked_div(denominator).unwrap();
    msg!("shares_to_burn {}", shares_to_burn);

    let casted = shares_to_burn.try_into().unwrap();
    msg!("shares_cast {}", casted);
    casted
}

/// calculate the amount of underlying asset to redeem for the given amount
/// of shares
pub fn calculate_underlying_to_redeem(
    amount: u64,
    total_vlp_shares: u64,
    total_vault_balance: u64,
) -> u64 {
    ((amount as u128)
        .checked_mul(total_vault_balance as u128)
        .unwrap())
    .checked_div(total_vlp_shares as u128)
    .unwrap()
    .try_into()
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_calculate_underlying_to_redeem() {
        let vtd = 999991_u64;
        let vts = 999991000_u64;
        let ru_one = 1000000_u64;
        let ru_two = ru_one / 2;
        let want = 1000;
        let want_two = 500;
        let got = calculate_underlying_to_redeem(vtd, vts, ru_one);
        let got_two = calculate_underlying_to_redeem(vtd, vts, ru_two);
        assert_eq!(got, want);
        assert_eq!(got_two, want_two);
    }
    #[test]
    fn test_calculate_shares_to_burn() {
        let vtd = 999991_u64;
        let vts = 999991000_u64;
        let ru_one = 1000000_u64;
        let ru_two = ru_one / 2;
        let want = 999982000;
        let want_two = 1999964000;
        let got = calculate_shares_to_burn(vtd, vts, ru_one);
        let got_two = calculate_shares_to_burn(vtd, vts, ru_two);
        assert_eq!(got, want);
        assert_eq!(got_two, want_two);
    }
    #[test]
    fn test_calculate_shares_to_give() {
        let vtd = 999991_u64;
        let vts = 999991000_u64;
        let ru_one = 1000000_u64;
        let ru_two = ru_one / 2;
        let want = 999982000;
        let want_two = 1999964000;
        let got = calculate_shares_to_give(vtd, vts, ru_one);
        let got_two = calculate_shares_to_give(vtd, vts, ru_two);
        assert_eq!(got, want);
        assert_eq!(got_two, want_two);
    }
    #[test]
    fn test_swap_a_to_b() {
        // 10 of a
        // 2 of b
        // ratio is 1/1 i.e., 1 a = 1 b
        // 4 a must be swapped to b
        assert_eq!(calculate_amount_a_to_swap_to_b(10, 2, 1, 1), 4)
    }

    #[test]
    fn test_swap_a_to_b_zero() {
        // 5 of a
        // 5 of b
        // ratio is 1/1 i.e., 1 a = 1 b
        // 0 must be swapped
        assert_eq!(calculate_amount_a_to_swap_to_b(5, 5, 1, 1), 0)
    }
    #[test]
    fn test_check_serum_swap_skip() {
        // assert that we skip the swap
        assert!(check_serum_swap_skip(39108057, 12726, 2805046416, 6),);
        // assert that we dont skip the swap
        assert!(!check_serum_swap_skip(561009283200, 12726, 2805046416, 6),)
    }
    #[test]
    fn test_calculate_average_execution_price() {
        let (coin_diff, pc_diff, coin_decimals, pc_decimals, pair_tick_num, pair_tick_denom) = {
            (
                970000000_u128,
                190270681_u128,
                1000000000_u128,
                1000000_u128,
                100000000000_u128,
                100000000000000_u128,
            )
        };
        let avg = calculate_average_execution_price(
            coin_diff,
            pc_diff,
            coin_decimals,
            pc_decimals,
            pair_tick_num,
            pair_tick_denom,
        );
        assert_eq!(avg.unwrap(), 196155);
        match calculate_average_execution_price_with_slippage(AverageExecutionPriceArgs {
            coin_diff,
            pc_diff,
            coin_decimals,
            pc_decimals,
            pair_tick_num,
            pair_tick_denom,
            best_price: 10_000_000_u128,
        }) {
            Some((avg, price_slippage)) => {
                println!("avg {}, price_slip {}", avg, price_slippage);
                assert_eq!(avg, 196155);
                assert_eq!(price_slippage, 98);
            }
            None => panic!("is none"),
        }
    }
    #[test]
    fn test_lp_swap_skip() {
        assert!(!check_amm_swap_skip(2, 4, 1, 1));
        assert!(check_amm_swap_skip(4, 2, 1, 1));
    }
    #[test]
    fn test_calculate_min_amount_out() {
        let result = calculate_min_amount_out(100_000_000_u64, 200);
        assert_eq!(result, 80_000_000_u64);
    }
}

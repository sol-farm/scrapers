use crate::math::common::WAD;
use crate::math::decimal::Decimal;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::{
    DEFAULT_TICKS_PER_SECOND, DEFAULT_TICKS_PER_SLOT, SECONDS_PER_DAY,
};
use solana_program::program_pack::Pack;
use static_pubkey::static_pubkey;
pub mod lending_error;
pub mod lending_last_update;
pub mod lending_obligation;
pub mod lending_reserve;

/// sighash of the staking program update_lock_cooldown function
pub const UPDATE_LOCK_COOLDOWN_SIGHASH: [u8; 8] = [231, 55, 146, 122, 89, 119, 6, 113];

/// Current version of the program and all new accounts created
pub const LENDING_PROGRAM_VERSION: u8 = 1;
/// address of the tulip lending program
pub const LENDING_PROGRAM_ID: Pubkey =
    static_pubkey!("4bcFeLv4nydFrsZqV5CgwCVrPhkQKsXtzfy2KyMz7ozM");
/// address of the tulip pyth program
pub const PYTH_PROGRAM_ID: Pubkey = static_pubkey!("5JQ8Mhdp2wv3HWcfjq9Ts8kwzCAeBADFBDAgBznzRsE4");
/// Accounts are created with data zeroed out, so uninitialized state instances
/// will have the version set to 0.
pub const LENDING_UNINITIALIZED_VERSION: u8 = 0;

/// address of the lending market, in the future there may be more than one lending market
pub const LENDING_MARKET: Pubkey = static_pubkey!("D1cqtVThyebK9KXKGXrCEuiqaNf5L4UfM1vHgCqiJxym");
/// address of the usdc cToken (called pToken on port's website)
pub const USDC_COLLATERAL_MINT: Pubkey =
    static_pubkey!("Amig8TisuLpzun8XyGfC5HJHHGUQEscjLgoTWsCCKihg");
/// address of the usdc reserve liquidity supply token account
pub const USDC_LIQUIDITY_SUPPLY: Pubkey =
    static_pubkey!("64QJd6MYXUjCBvCaZKaqxiKmaMkPUdNonE1KuY1YoGGb");
/// address of the USDC reserve account
pub const USDC_RESERVE_ACCOUNT: Pubkey =
    static_pubkey!("FTkSmGsJ3ZqDSHdcnY7ejN1pWV3Ej7i88MYpZyyaqgGt");
pub const USDC_PYTH_PRICE_ACCOUNT: Pubkey =
    static_pubkey!("ExzpbWgczTgd8J58BrnESndmzBkRVfc6PhFjSGiQXgAB");

/// Number of slots per year
pub const LENDING_SLOTS_PER_YEAR: u64 =
    DEFAULT_TICKS_PER_SECOND / DEFAULT_TICKS_PER_SLOT * SECONDS_PER_DAY * 365;
/// Collateral tokens are initially valued at a ratio of 5:1 (collateral:liquidity)
// @FIXME: restore to 5
pub const LENDING_INITIAL_COLLATERAL_RATIO: u64 = 1;
const LENDING_INITIAL_COLLATERAL_RATE: u64 = LENDING_INITIAL_COLLATERAL_RATIO * WAD;

// Helpers
fn pack_decimal(decimal: Decimal, dst: &mut [u8; 16]) {
    *dst = decimal
        .to_scaled_val()
        .expect("Decimal cannot be packed")
        .to_le_bytes();
}

fn unpack_decimal(src: &[u8; 16]) -> Decimal {
    Decimal::from_scaled_val(u128::from_le_bytes(*src))
}

fn pack_bool(boolean: bool, dst: &mut [u8; 1]) {
    *dst = (boolean as u8).to_le_bytes()
}

fn unpack_bool(src: &[u8; 1]) -> Result<bool> {
    match u8::from_le_bytes(*src) {
        0 => Ok(false),
        1 => Ok(true),
        _ => {
            msg!("Boolean cannot be unpacked");
            Err(ProgramError::InvalidAccountData.into())
        }
    }
}

pub fn load_reserve<'info>(reserve: &AccountInfo<'info>) -> lending_reserve::Reserve {
    lending_reserve::Reserve::unpack_unchecked(&reserve.data.borrow()[..]).unwrap()
}

#[cfg(test)]
mod test {

    use ring::digest::{Context, SHA256};

    #[test]
    fn test_sighashes() {
        let mut context = Context::new(&SHA256);
        // sha256("global:do_something")[..8]
        context.update(b"global:update_lock_cooldown");
        let digest = context.finish();
        println!(
            "pub const UPDATE_LOCK_COOLDOWN_SIGHASH: [u8; 8] = {:?};",
            &digest.as_ref()[0..8]
        );
    }
}

use anchor_lang::{prelude::Pubkey, solana_program};
use static_pubkey::static_pubkey;
use std::str::FromStr;

/// address of the pyth oracle program
pub const PYTH_PROGRAM_ID: Pubkey = static_pubkey!("FsJ3A3u2vn5cTVofAjvy6y5kwABJAqYWpe4975bi2epH");
/// address of the switchboard oracle program
pub const SWITCHBOARD_PROGRAM_ID: Pubkey =
    static_pubkey!("DtmE9D2CSB4L5D6A15mraeEjrGMm6auWVzgaD8hK2tZM");

pub struct LoadSerumMarketSampleArgs {
    pub serum_market: Pubkey,
    pub amm_open_orders: Pubkey,
    pub serum_market_bids: Pubkey,
    pub serum_market_asks: Pubkey,
    pub pool_coin_token_account: Option<Pubkey>,
    pub pool_pc_token_account: Option<Pubkey>,
    pub pool_lp_token_mint: Option<Pubkey>,
    pub amm_id: Option<Pubkey>,
}

#[derive(Debug, Clone, Copy)]
/// the direction to lookup a price for
/// CoinToPC means to return the amount of pc 1 coin eqauls to
/// PCToCoin means to return the amount of coin 1 pc equals to
pub enum LookupDirection {
    CoinToPC,
    PCToCoin,
}

#[derive(Debug, Clone, Copy)]
pub enum LookupType {
    /// indicates the lookup type is
    /// that of lp token price lookup
    LP,
    /// indicates the lookup type is
    /// that of single asset price lookup
    Single,
}

impl ToString for LookupDirection {
    fn to_string(&self) -> String {
        match self {
            LookupDirection::CoinToPC => String::from("coin-to-pc"),
            LookupDirection::PCToCoin => String::from("pc-to-coin"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LookupPlatform {
    /// indicates the lookup platform is serum
    SERUM,
    /// indicates the lookup platform is raydium
    RAYDIUM,
    ORCA,
    SABER,
    ATRIX,
}

impl FromStr for LookupPlatform {
    type Err = std::io::ErrorKind;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if s.eq("SERUM") {
            return Ok(Self::SERUM);
        }
        if s.eq("RAYDIUM") {
            return Ok(Self::RAYDIUM);
        }
        if s.eq("ORCA") {
            return Ok(Self::ORCA);
        }
        if s.eq("SABER") {
            return Ok(Self::SABER);
        }
        if s.eq("ATRIX") {
            return Ok(Self::ATRIX);
        }
        Err(std::io::ErrorKind::InvalidData)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_oracles() {
        assert_eq!(
            &PYTH_PROGRAM_ID.to_string(),
            "FsJ3A3u2vn5cTVofAjvy6y5kwABJAqYWpe4975bi2epH"
        );
        assert_eq!(
            &SWITCHBOARD_PROGRAM_ID.to_string(),
            "DtmE9D2CSB4L5D6A15mraeEjrGMm6auWVzgaD8hK2tZM"
        );
    }
    #[test]
    fn test_lookup_platform() {
        assert_eq!(
            LookupPlatform::from_str("SERUM").unwrap(),
            LookupPlatform::SERUM
        );
        assert_eq!(
            LookupPlatform::from_str("RAYDIUM").unwrap(),
            LookupPlatform::RAYDIUM
        );
        assert_eq!(
            LookupPlatform::from_str("ORCA").unwrap(),
            LookupPlatform::ORCA
        );
        assert_eq!(
            LookupPlatform::from_str("SABER").unwrap(),
            LookupPlatform::SABER
        );
        assert_eq!(
            LookupPlatform::from_str("ATRIX").unwrap(),
            LookupPlatform::ATRIX
        );
    }
}

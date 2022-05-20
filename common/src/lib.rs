#![allow(clippy::needless_lifetimes)]

pub mod account;
pub mod asserts;
pub mod mango;
pub mod math;
pub mod oracle;
pub mod orca;
pub mod port;
pub mod quarry;
pub mod raydium;
pub mod saber;
pub mod serum;
pub mod sighashes;
pub mod solend;
pub mod spl_lending;
pub mod sunny;
pub mod traits;
pub mod tulip;
pub mod vaults;

#[cfg(feature = "v1")]
pub mod v1;

#[allow(unused_imports)]
#[allow(clippy::macro_use_imports)]
use arrform::{arrform, ArrForm};
use solana_program::pubkey::Pubkey;
use static_pubkey::static_pubkey;
/// constant declaration of the  default public key, which allows for optimized usage of the default public key
/// as this is cheaper than repeatedly calling Pubkey::default() within solana programs
pub const DEFAULT_KEY: Pubkey = Pubkey::new_from_array([
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
]);

pub const SERUM_DEX_PROGRAM_ID: Pubkey =
    static_pubkey!("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin");

pub const TULIP_TOKEN_MINT: Pubkey = static_pubkey!("TuLipcqtGVXP9XR62wM8WWCm6a9vhLs7T1uoWBk6FDs");
pub const USDC_TOKEN_MINT: Pubkey = static_pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
pub const USDT_TOKEN_MINT: Pubkey = static_pubkey!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
pub const RAY_TOKEN_MINT: Pubkey = static_pubkey!("4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R");
pub const wSOL_TOKEN_MINT: Pubkey = static_pubkey!("So11111111111111111111111111111111111111112");

/// msg_panic! is a wrapper around the `msg!` and `panic!`
/// macros used to log an error message, and panic in bpf environments
/// which do not actually show a message emitted by a panic macro
#[macro_export]
macro_rules! msg_panic {
    ($($args:tt)+) => {{
        // the actual error message
        msg!("RUNTIME ERROR: {}", format_args!($($args)*));
        // panic to indicate the line the error came from
        // but panicking doesn't log anything in bpf target for solana programs
        panic!("RUNTIME ERROR: {}", format_args!($($args)*));
    }};
}

#[macro_export]
macro_rules! sum {
    // this delcares an exrpession i think :shrug:
    // todo(): explain this more
    ($($args:expr),*) => {{
        let result = 0;
        $(
            // combine the size of each value
            let result = result + std::mem::size_of_val(&$args);
        )*
        // return the size of all arguments
        result
    }}
}
/// msg_trace! is a wrapper around the `msg!` macro, that faciliates logging trace
/// level logs, which include the file and line number the message was emitted in
/// this is potentially very unoptimized, and may not work with long filename paths
/// or big messages.
///
/// state: heavy wip
#[macro_export]
macro_rules! msg_trace {
    ($($args:tt)+) => {
        // get the filename that produce the log, it's less info than the fille path
        // but it saves pace, an when paired with the line number is more than enough debug
        let file_name = std::path::Path::new(file!()).file_name().unwrap().to_string_lossy();
        let input_sizes = sum!($($args)*);
        if input_sizes > 512 {
            // slow path
            msg!("{}", format!("'{}', '{}:{}", format!($($args)*), file_name, line!()).as_str());
        } else {
            let file_info = arrform!(256, "{}:{}", file_name, line!());
            let msg_part = arrform!(512, $($args)*);
            msg!("'{}', {}", msg_part.as_str(), file_info.as_str());
        }
    };
}

/// omsg! is an optimized wrapper around the `msg!` macro which attempts to
/// format messages using stack instead of heap
#[macro_export]
macro_rules! omsg {
    ($($args:tt)+) => {
        // get the filename that produce the log, it's less info than the fille path
        // but it saves pace, an when paired with the line number is more than enough debug
        let input_sizes = sum!($($args)*);
        if input_sizes > 512 {
            // slow path
            msg!("{}", format!("{}", format!($($args)*)));
        } else {
            let msg_part = arrform!(512, $($args)*);
            msg!("{}", msg_part.as_str());
        }
    };
}

pub fn to_u64(bytes: &[u8]) -> u64 {
    let mut amount: [u8; 8] = [0_u8; 8];
    amount.copy_from_slice(bytes);
    u64::from_le_bytes(amount)
}

pub fn to_pubkey(bytes: &[u8]) -> Pubkey {
    let mut key: [u8; 32] = [0_u8; 32];
    key.copy_from_slice(bytes);
    Pubkey::new_from_array(key)
}

#[cfg(test)]
mod test {
    use super::*;
    use solana_program::msg;
    #[test]
    fn test_to_u64() {
        let amount = 420_690_u64;
        let amount_bytes = amount.to_le_bytes();
        let got_amount = to_u64(&amount_bytes);
        assert_eq!(amount, got_amount);
    }
    #[test]
    fn test_to_pubkey() {
        let pubkey = Pubkey::new_unique();
        let pubkey_bytes = pubkey.to_bytes();
        let got_pubkey = to_pubkey(&pubkey_bytes);
        assert_eq!(pubkey, got_pubkey);
    }
    #[test]
    fn default_key() {
        assert_eq!(DEFAULT_KEY, Pubkey::default());
    }
    #[test]
    fn test_size_ofs() {
        println!("{}", sum!("y", "o", "bbbbbb"));
    }
    #[test]
    fn test_trace() {
        msg_trace!("hello world this is {}", "very big message");
    }
}

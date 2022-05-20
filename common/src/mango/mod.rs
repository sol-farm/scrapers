#![allow(clippy::assertions_on_constants)]
#![allow(clippy::too_many_arguments)]

use anchor_lang::prelude::*;
use mango_lib::state::MangoAccount;
use solana_program::{instruction::Instruction, system_program};
use static_pubkey::static_pubkey;

pub use mango as mango_lib;
pub use mango_common;
pub use mango_macro;

pub const MANGO_V3_GROUP_ID: &[u8] = b"mainnet.1";
pub const MANGO_V3_QUOTE_SYMBOL: &[u8] = b"USDC";
pub const MANGO_V3_GROUP_SIGNER_KEY: Pubkey =
    static_pubkey!("9BVcYqEQxyccuwznvxXqDkSJFavvTyheiTYk231T1A8S");
pub const MANGO_V3_USDC_TOKEN_VAULT: Pubkey =
    static_pubkey!("8Vw25ZackDzaJzzBBqcgcpDsCsDfRSkMGgwFQ3gbReWF");
pub const MANGO_V3_PROGRAM_ID: Pubkey =
    static_pubkey!("mv3ekLzLbnVPNxjSKvqBpU3ZeZXPQdEC3bp5MDEBG68");
pub const MANGO_V3_GROUP_KEY: Pubkey =
    static_pubkey!("98pjRuQjK3qA6gXts96PqZT4Ze5QmnCmt3QYjhbUSPue");
pub const MANGO_V3_CACHE: Pubkey = static_pubkey!("EBDRoayCDDUvDgCimta45ajQeXbexv7aKqJubruqpyvu");
// aka root bank
pub const MANGO_V3_MAINNET_ONE_USDC_ROOT_KEY: Pubkey =
    static_pubkey!("AMzanZxMirPCgGcBoH9kw4Jzi9LFMomyUCXbpzDeL2T8");
pub const MANGO_V3_MAINNET_ONE_USDC_NODE_KEYS: [Pubkey; 1] = [static_pubkey!(
    "BGcwkj1WudQwUUjFk78hAjwd1uAm8trh1N4CJSa51euh"
)];

pub const MANGO_ACCONT_SIZE: usize = std::mem::size_of::<MangoAccount>();

/// returns an instruction that is used to create
/// the new mango account pda type
pub fn new_create_mango_account_ix(
    mango_account: Pubkey,
    owner: Pubkey,
    payer: Pubkey,
    account_num: u64,
) -> Instruction {
    mango_lib::instruction::create_mango_account(
        &MANGO_V3_PROGRAM_ID,
        &MANGO_V3_GROUP_KEY,
        &mango_account,
        &owner,
        &system_program::id(),
        &payer,
        account_num,
    )
    .unwrap()
}

/// helper function to derive a mango account pda address
pub fn derive_mango_account_pda(owner: Pubkey, account_num: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            MANGO_V3_GROUP_KEY.as_ref(),
            owner.as_ref(),
            &account_num.to_le_bytes(),
        ],
        &MANGO_V3_PROGRAM_ID,
    )
}

pub fn borrow(
    mango_group: Pubkey,
    mango_account: Pubkey,
    owner: Pubkey,
    mango_cache: Pubkey,
    mango_root_bank: Pubkey,
    mango_node_bank: Pubkey,
    mango_vault: Pubkey,
    token_account: Pubkey,
    signer: Pubkey,
    open_orders: &[Pubkey],
    amount: u64,
) -> Instruction {
    _withdraw(
        mango_group,
        mango_account,
        owner,
        mango_cache,
        mango_root_bank,
        mango_node_bank,
        mango_vault,
        token_account,
        signer,
        open_orders,
        amount,
        true,
    )
}

pub fn withdraw(
    mango_group: Pubkey,
    mango_account: Pubkey,
    owner: Pubkey,
    mango_cache: Pubkey,
    mango_root_bank: Pubkey,
    mango_node_bank: Pubkey,
    mango_vault: Pubkey,
    token_account: Pubkey,
    signer: Pubkey,
    open_orders: &[Pubkey],
    amount: u64,
) -> Instruction {
    _withdraw(
        mango_group,
        mango_account,
        owner,
        mango_cache,
        mango_root_bank,
        mango_node_bank,
        mango_vault,
        token_account,
        signer,
        open_orders,
        amount,
        false,
    )
}

fn _withdraw(
    mango_group: Pubkey,
    mango_account: Pubkey,
    owner: Pubkey,
    mango_cache: Pubkey,
    mango_root_bank: Pubkey,
    mango_node_bank: Pubkey,
    mango_vault: Pubkey,
    token_account: Pubkey,
    signer: Pubkey,
    open_orders: &[Pubkey],
    amount: u64,
    allow_borrow: bool,
) -> Instruction {
    mango_lib::instruction::withdraw(
        &MANGO_V3_PROGRAM_ID,
        &mango_group,
        &mango_account,
        &owner,
        &mango_cache,
        &mango_root_bank,
        &mango_node_bank,
        &mango_vault,
        &token_account,
        &signer,
        open_orders,
        amount,
        allow_borrow,
    )
    .unwrap()
}

#[cfg(test)]
mod test {
    use crate::DEFAULT_KEY;
    use solana_client::rpc_client::RpcClient;
    use solana_program::account_info::IntoAccountInfo;

    use super::*;

    #[test]
    fn test_derive_mango_account_pda() {
        let (got_key_one, got_nonce_one) = derive_mango_account_pda(DEFAULT_KEY, 0);
        assert_eq!(
            got_key_one.to_string(),
            "7VfFV6VWVRqmdzD9AzyWo3bpycp81TV2zttkjK9f8AfK".to_string()
        );
        assert_eq!(got_nonce_one, 255);
        let (got_key_two, got_nonce_two) = derive_mango_account_pda(DEFAULT_KEY, 1);
        assert_eq!(
            got_key_two.to_string(),
            "9KgMCdvJmaGRypXdAxLefM4h8foWbRVdzdVJCpMKcEFU".to_string()
        );
        assert_eq!(got_nonce_two, 255);
    }
    use super::mango_lib::state::MangoGroup;
    pub fn log_mango_group(mango_group: &MangoGroup) {
        let metadata = format!(
            "MetaData {{
                data_type {}
                version {}  
           }}",
            mango_group.meta_data.data_type, mango_group.meta_data.version
        );
        let mut tokens_info = String::new();
        for token in mango_group.tokens.iter() {
            tokens_info.push_str(
                format!(
                    "TokenInfo {{
                      mint {}
                      root_bank {}
                      decimals {}
                }}\n",
                    token.mint, token.root_bank, token.decimals,
                )
                .as_str(),
            );
        }
        println!(
            "MangoGroup {{
                metadata {}
                tokens_info {}
                admin {}
                mango_cache {}
                insurance_vault {}
                srm_vault {}
                msrm_vault {}
                fees_vault {}
                signer_key {}
            }}",
            metadata,
            tokens_info,
            mango_group.admin,
            mango_group.mango_cache,
            mango_group.insurance_vault,
            mango_group.srm_vault,
            mango_group.msrm_vault,
            mango_group.fees_vault,
            mango_group.signer_key,
        )
    }

    #[test]
    fn test_size_of_mango_account() {
        println!("size {}", MANGO_ACCONT_SIZE);
        assert!(MANGO_ACCONT_SIZE == 4296);
    }
    #[test]
    fn test_log_mango_accounts() {
        let rpc = RpcClient::new("http://51.222.241.93:8899".to_string());
        let mango_group_data = rpc.get_account(&MANGO_V3_GROUP_KEY).unwrap();
        let mut mango_group_tuple = (MANGO_V3_GROUP_KEY, mango_group_data);
        let mango_group_info = mango_group_tuple.into_account_info();
        let mango_group =
            mango_lib::state::MangoGroup::load_checked(&mango_group_info, &MANGO_V3_PROGRAM_ID)
                .unwrap();
        // fees vault is likely what we need to deposit
        log_mango_group(&mango_group);
    }
}

/*

/// Create a PDA mango account for a user
///
/// Accounts expected by this instruction (5):
///
/// 0. `[writable]` mango_group_ai - MangoGroup that this mango account is for
/// 1. `[writable]` mango_account_ai - the mango account data
/// 2. `[signer]` owner_ai - Solana account of owner of the mango account
/// 3. `[]` system_prog_ai - System program
/// 4. `[signer, writable]` payer_ai - pays for the PDA creation
CreateMangoAccount {
    account_num: u64,
},*/

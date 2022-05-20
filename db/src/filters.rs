use std::cmp::Ordering;

use into_query::IntoQuery;

use crate::models::V1ObligationLtv;

#[derive(IntoQuery, Default)]
#[table_name = "vault"]
pub struct FindVault {
    pub farm_name: Option<Vec<String>>,
    pub account_address: Option<Vec<String>>,
}

#[derive(IntoQuery, Default)]
#[table_name = "deposit_tracking"]
pub struct FindDepositTracking {
    pub owner_address: Option<Vec<String>>,
    pub account_address: Option<Vec<String>>,
    pub vault_account_address: Option<Vec<String>>,
}

#[derive(IntoQuery, Default)]
#[table_name = "token_price"]
pub struct FindTokenPrice {
    pub asset: Option<Vec<String>>,
    pub asset_identifier: Option<Vec<String>>,
    pub platform: Option<Vec<String>>,
}

#[derive(IntoQuery, Default)]
#[table_name = "interest_rate"]
pub struct FindInterestRate {
    pub asset: Option<Vec<String>>,
    pub platform: Option<Vec<String>>,
}

#[derive(IntoQuery, Default)]
#[table_name = "vault_tvl"]
pub struct FindVaultTvl {
    pub farm_name: Option<Vec<String>>,
}

#[derive(IntoQuery, Default)]
#[table_name = "token_balance"]
pub struct FindTokenBalance {
    pub token_account: Option<Vec<String>>,
    pub identifier: Option<Vec<String>>,
}

#[derive(IntoQuery, Default)]
#[table_name = "interest_rate_curve"]
pub struct FindInterestRateCurve {
    pub platform: Option<Vec<String>>,
    pub asset: Option<Vec<String>>,
    pub rate_name: Option<Vec<String>>,
}

#[derive(IntoQuery, Default)]
#[table_name = "staking_analytic"]
pub struct FindStakingAnalytic;

#[derive(IntoQuery, Default)]
#[table_name = "realize_yield"]
pub struct FindRealizeYield {
    pub vault_address: Option<Vec<String>>,
    pub farm_name: Option<Vec<String>>,
}

#[derive(IntoQuery, Default)]
#[table_name = "lending_optimizer_distribution"]
pub struct FindLendingOptimizerDistribution {
    pub vault_name: Option<Vec<String>>,
}

#[derive(IntoQuery, Default)]
#[table_name = "interest_rate_moving_average"]
pub struct FindInterestRateMovingAverage {
    /// rate_name is a combination of PLATFORM-ASSET
    pub rate_name: Option<Vec<String>>,
    pub asset: Option<Vec<String>>,
    pub platform: Option<Vec<String>>,
}

#[derive(IntoQuery, Default)]
#[table_name = "advertised_yield"]
pub struct FindAdvertisedYield {
    pub farm_name: Option<Vec<String>>,
}

#[derive(IntoQuery, Default)]
#[table_name = "v1_obligation_ltv"]
pub struct FindV1ObligationLtv {
    pub authority: Option<Vec<String>>,
    pub user_farm: Option<Vec<String>>,
    pub account_address: Option<Vec<String>>,
}

#[derive(IntoQuery, Default)]
#[table_name = "v1_user_farm"]
pub struct FindV1UserFarm {
    pub authority: Option<Vec<String>>,
    pub account_address: Option<Vec<String>>,
}

#[derive(IntoQuery, Default)]
#[table_name = "v1_liquidated_position"]
pub struct FindV1LiquidatedPosition {
    pub authority: Option<Vec<String>>,
    pub temp_liquidation_account: Option<Vec<String>>,
    pub user_farm: Option<Vec<String>>,
    pub obligation: Option<Vec<String>>,
    pub liquidation_event_id: Option<Vec<String>>,
}

#[derive(IntoQuery, Default)]
#[table_name = "v1_obligation_account"]
pub struct FindV1ObligationAccount {
    pub account: Option<Vec<String>>,
    pub authority: Option<Vec<String>>,
}

#[derive(IntoQuery, Default)]
#[table_name = "historic_tshare_price"]
pub struct FindHistoricTSharePrice {
    pub farm_name: Option<Vec<String>>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DepositTrackingMatcher {
    Owner(Vec<String>),
    Account(Vec<String>),
    Vault(Vec<String>),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum VaultMatcher {
    FarmName(Vec<String>),
    Account(Vec<String>),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TokenPriceMatcher {
    /// name of the asset
    Asset(Vec<String>),
    /// the platform + asset combination
    AssetIdentifier(Vec<String>),
    AssetAndPlatform(Vec<(String, String)>),
    All,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum InterestRateMatcher {
    /// name of the asset
    Asset(Vec<String>),
    /// name of the platform
    Platform(Vec<String>),
    /// tuple of (asset, platform)
    AssetAndPlatform(Vec<(String, String)>),
    PlatformAndAsset(Vec<(String, String)>),
    /// indicates to return all matching records
    All,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum VaultTvlMatcher {
    FarmName(Vec<String>),
    All,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TokenBalanceMatcher {
    /// token account
    Account(Vec<String>),
    /// balance record identifier
    Identifier(Vec<String>),
    /// indicates to return all matching records
    All,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum StakingAnalyticMatcher {
    All,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RealizeYieldMatcher {
    /// vault account
    Account(Vec<String>),
    /// formattted farm name
    FarmName(Vec<String>),
    /// return all matching records
    All,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum InterestRateCurveMatcher {
    Asset(Vec<String>),
    Platform(Vec<String>),
    PlatformAndAsset(Vec<(String, String)>),
    RateName(Vec<String>),
    All,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LendingOptimizerDistributionMatcher {
    VaultName(Vec<String>),
    All,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum InterestRateMovingAverageMatcher {
    /// name of the asset
    Asset(Vec<String>),
    /// name of the platform
    Platform(Vec<String>),
    /// Platform-Asset combination
    RateName(Vec<String>),
    /// indicates to return all matching records
    All,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum AdvertisedYieldMatcher {
    FarmName(Vec<String>),
    All,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum V1ObligationLtvMatcher {
    AccountAddress(Vec<String>),
    Authority(Vec<String>),
    UserFarm(Vec<String>),
    All,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum V1UserFarmMatcher {
    AccountAddress(Vec<String>),
    Authority(Vec<String>),
    All,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum V1LiquidatedPositionMatcher {
    TempLiquidationAccount(Vec<String>),
    Authority(Vec<String>),
    UserFarm(Vec<String>),
    Obligation(Vec<String>),
    LiquidationEventId(Vec<String>),
    All,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum HistoricTSharePriceMatcher {
    FarmName(Vec<String>),
    All,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum V1ObligationAccountMatcher {
    AccountAddress(Vec<String>),
    Authority(Vec<String>),
    All,
}

impl VaultMatcher {
    /// returns an instance of the vault matcher filter
    pub fn to_filter(&self) -> FindVault {
        let mut ft = FindVault::default();
        match self {
            VaultMatcher::FarmName(farm) => {
                ft.farm_name = Some(farm.clone());
            }
            VaultMatcher::Account(account) => {
                ft.account_address = Some(account.clone());
            }
        }
        ft
    }
}

impl DepositTrackingMatcher {
    /// returns an instance of the deposit tracking filter
    pub fn to_filter(&self) -> FindDepositTracking {
        let mut ft = FindDepositTracking::default();
        match self {
            DepositTrackingMatcher::Owner(owner) => {
                ft.owner_address = Some(owner.clone());
            }
            DepositTrackingMatcher::Account(account) => {
                ft.account_address = Some(account.clone());
            }
            DepositTrackingMatcher::Vault(vault) => {
                ft.vault_account_address = Some(vault.clone());
            }
        }
        ft
    }
}

impl TokenPriceMatcher {
    /// returns an instance of the coingecko filter
    pub fn to_filter(&self) -> FindTokenPrice {
        let mut ft = FindTokenPrice::default();
        match self {
            TokenPriceMatcher::Asset(asset) => {
                ft.asset = Some(asset.clone());
            }
            TokenPriceMatcher::AssetIdentifier(identifier) => {
                ft.asset_identifier = Some(identifier.clone());
            }
            TokenPriceMatcher::AssetAndPlatform(infos) => {
                ft.platform = Some(Vec::with_capacity(infos.len()));
                ft.asset = Some(Vec::with_capacity(infos.len()));
                for (asset, platform) in infos.iter() {
                    ft.asset.as_mut().unwrap().push(asset.clone());
                    ft.platform.as_mut().unwrap().push(platform.clone());
                }
            }
            TokenPriceMatcher::All => (),
        }
        ft
    }
}

impl InterestRateMatcher {
    /// returns an instance of the interest rate filter
    pub fn to_filter(&self) -> FindInterestRate {
        let mut ft = FindInterestRate::default();
        match self {
            InterestRateMatcher::Asset(asset) => {
                ft.asset = Some(
                    asset
                        .iter()
                        .map(|asset| asset.to_ascii_uppercase())
                        .collect(),
                );
            }
            InterestRateMatcher::Platform(platform) => {
                ft.platform = Some(
                    platform
                        .iter()
                        .map(|platform| platform.to_ascii_uppercase())
                        .collect(),
                );
            }
            InterestRateMatcher::AssetAndPlatform(infos) => {
                ft.platform = Some(Vec::with_capacity(infos.len()));
                ft.asset = Some(Vec::with_capacity(infos.len()));
                for (asset, platform) in infos.iter() {
                    ft.platform
                        .as_mut()
                        .unwrap()
                        .push(platform.to_ascii_uppercase());
                    ft.asset.as_mut().unwrap().push(asset.to_ascii_uppercase());
                }
            }
            InterestRateMatcher::PlatformAndAsset(infos) => {
                ft.platform = Some(Vec::with_capacity(infos.len()));
                ft.asset = Some(Vec::with_capacity(infos.len()));
                for (platform, asset) in infos.iter() {
                    ft.platform
                        .as_mut()
                        .unwrap()
                        .push(platform.to_ascii_uppercase());
                    ft.asset.as_mut().unwrap().push(asset.to_ascii_uppercase());
                }
            }
            &InterestRateMatcher::All => (),
        }
        ft
    }
}

impl VaultTvlMatcher {
    pub fn to_filter(&self) -> FindVaultTvl {
        let mut ft = FindVaultTvl::default();
        match self {
            VaultTvlMatcher::FarmName(name) => {
                ft.farm_name = Some(name.clone());
            }
            VaultTvlMatcher::All => (),
        }
        ft
    }
}

impl TokenBalanceMatcher {
    /// returns an instance of the vault matcher filter
    pub fn to_filter(&self) -> FindTokenBalance {
        let mut ft = FindTokenBalance::default();
        match self {
            TokenBalanceMatcher::Account(farm) => {
                ft.token_account = Some(farm.clone());
            }
            TokenBalanceMatcher::Identifier(account) => {
                ft.identifier = Some(account.clone());
            }
            TokenBalanceMatcher::All => (),
        }
        ft
    }
}

impl StakingAnalyticMatcher {
    /// returns an instance of the interest rate filter
    pub fn to_filter(&self) -> FindStakingAnalytic {
        let ft = FindStakingAnalytic::default();
        match self {
            StakingAnalyticMatcher::All => (),
        }
        ft
    }
}

impl RealizeYieldMatcher {
    /// returns an instance of the realize yield filter
    pub fn to_filter(&self) -> FindRealizeYield {
        let mut ft = FindRealizeYield::default();
        match self {
            RealizeYieldMatcher::FarmName(farm) => {
                ft.farm_name = Some(farm.clone());
            }
            RealizeYieldMatcher::Account(account) => {
                ft.vault_address = Some(account.clone());
            }
            RealizeYieldMatcher::All => (),
        }
        ft
    }
}

impl InterestRateCurveMatcher {
    /// returns an instance of the vault matcher filter
    pub fn to_filter(&self) -> FindInterestRateCurve {
        let mut ft = FindInterestRateCurve::default();
        match self {
            InterestRateCurveMatcher::Platform(platform) => {
                // make sure to uppercase it
                ft.platform = Some(
                    platform
                        .iter()
                        .map(|platform| platform.to_ascii_uppercase())
                        .collect(),
                );
            }
            InterestRateCurveMatcher::PlatformAndAsset(infos) => {
                ft.platform = Some(Vec::with_capacity(infos.len()));
                ft.asset = Some(Vec::with_capacity(infos.len()));
                for (platform, asset) in infos.iter() {
                    ft.platform
                        .as_mut()
                        .unwrap()
                        .push(platform.to_ascii_uppercase());
                    ft.asset.as_mut().unwrap().push(asset.to_ascii_uppercase());
                }
            }
            InterestRateCurveMatcher::Asset(asset) => {
                // make sure to uppercase it
                ft.asset = Some(
                    asset
                        .iter()
                        .map(|asset| asset.to_ascii_uppercase())
                        .collect(),
                );
            }
            InterestRateCurveMatcher::RateName(name) => {
                // make sure to uppercase it
                ft.rate_name = Some(name.iter().map(|name| name.to_ascii_uppercase()).collect());
            }
            InterestRateCurveMatcher::All => (),
        }
        ft
    }
}

impl InterestRateMovingAverageMatcher {
    /// returns an instance of the interest rate filter
    pub fn to_filter(&self) -> FindInterestRateMovingAverage {
        let mut ft = FindInterestRateMovingAverage::default();
        match self {
            InterestRateMovingAverageMatcher::Asset(asset) => {
                ft.asset = Some(
                    asset
                        .iter()
                        .map(|asset| asset.to_ascii_uppercase())
                        .collect(),
                );
            }
            InterestRateMovingAverageMatcher::Platform(platform) => {
                ft.platform = Some(
                    platform
                        .iter()
                        .map(|platform| platform.to_ascii_uppercase())
                        .collect(),
                );
            }
            InterestRateMovingAverageMatcher::RateName(rate_name) => {
                ft.rate_name = Some(
                    rate_name
                        .iter()
                        .map(|rate_name| rate_name.to_ascii_uppercase())
                        .collect(),
                );
            }
            &InterestRateMovingAverageMatcher::All => (),
        }
        ft
    }
}

impl LendingOptimizerDistributionMatcher {
    pub fn to_filter(&self) -> FindLendingOptimizerDistribution {
        let mut ft = FindLendingOptimizerDistribution::default();
        match self {
            LendingOptimizerDistributionMatcher::VaultName(name) => {
                ft.vault_name = Some(name.clone());
            }
            LendingOptimizerDistributionMatcher::All => (),
        }
        ft
    }
}

impl AdvertisedYieldMatcher {
    pub fn to_filter(&self) -> FindAdvertisedYield {
        let mut ft = FindAdvertisedYield::default();
        match self {
            AdvertisedYieldMatcher::FarmName(farms) => {
                ft.farm_name = Some(farms.clone());
            }
            AdvertisedYieldMatcher::All => (),
        }
        ft
    }
}

impl V1ObligationLtvMatcher {
    pub fn to_filter(&self) -> FindV1ObligationLtv {
        let mut ft = FindV1ObligationLtv::default();
        match self {
            V1ObligationLtvMatcher::UserFarm(user_farms) => {
                ft.user_farm = Some(user_farms.clone());
            }
            V1ObligationLtvMatcher::Authority(authorities) => {
                ft.authority = Some(authorities.clone());
            }
            V1ObligationLtvMatcher::AccountAddress(addresses) => {
                ft.account_address = Some(addresses.clone());
            }
            V1ObligationLtvMatcher::All => (),
        }
        ft
    }
}

impl V1UserFarmMatcher {
    pub fn to_filter(&self) -> FindV1UserFarm {
        let mut ft = FindV1UserFarm::default();
        match self {
            V1UserFarmMatcher::AccountAddress(accounts) => {
                ft.account_address = Some(accounts.clone());
            }
            V1UserFarmMatcher::Authority(authorities) => {
                ft.authority = Some(authorities.clone());
            }
            V1UserFarmMatcher::All => (),
        }
        ft
    }
}

impl V1LiquidatedPositionMatcher {
    pub fn to_filter(&self) -> FindV1LiquidatedPosition {
        let mut ft = FindV1LiquidatedPosition::default();
        match self {
            V1LiquidatedPositionMatcher::Authority(authorities) => {
                ft.authority = Some(authorities.clone());
            }
            V1LiquidatedPositionMatcher::Obligation(obligations) => {
                ft.obligation = Some(obligations.clone());
            }
            V1LiquidatedPositionMatcher::TempLiquidationAccount(temp_accounts) => {
                ft.temp_liquidation_account = Some(temp_accounts.clone());
            }
            V1LiquidatedPositionMatcher::UserFarm(user_farms) => {
                ft.user_farm = Some(user_farms.clone())
            }
            V1LiquidatedPositionMatcher::LiquidationEventId(event_ids) => {
                ft.liquidation_event_id = Some(event_ids.clone());
            }
            V1LiquidatedPositionMatcher::All => (),
        }
        ft
    }
}

impl HistoricTSharePriceMatcher {
    pub fn to_filter(&self) -> FindHistoricTSharePrice {
        let mut ft = FindHistoricTSharePrice::default();
        match self {
            HistoricTSharePriceMatcher::FarmName(farm_names) => {
                ft.farm_name = Some(farm_names.clone());
            }
            HistoricTSharePriceMatcher::All => (),
        }
        ft
    }
}

impl V1ObligationAccountMatcher {
    pub fn to_filter(&self) -> FindV1ObligationAccount {
        let mut ft = FindV1ObligationAccount::default();
        match self {
            V1ObligationAccountMatcher::Authority(authorities) => {
                ft.authority = Some(authorities.clone());
            }
            V1ObligationAccountMatcher::AccountAddress(accounts) => {
                ft.account = Some(accounts.clone());
            }
            V1ObligationAccountMatcher::All => (),
        }
        ft
    }
}

impl std::fmt::Display for AdvertisedYieldMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdvertisedYieldMatcher::FarmName(farms) => {
                f.write_str(&format!("AdvertisedYieldMatcher::FarmName({:#?})", farms))
            }
            AdvertisedYieldMatcher::All => f.write_str("AdvertisedYieldMatcher::All"),
        }
    }
}

impl std::fmt::Display for DepositTrackingMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DepositTrackingMatcher::Owner(owner) => {
                f.write_str(&format!("DepositTrackingMatcher::Owner({:#?})", owner))
            }
            DepositTrackingMatcher::Account(account) => {
                f.write_str(&format!("DepositTrackingMatcher::Account({:#?})", account))
            }
            DepositTrackingMatcher::Vault(vault) => {
                f.write_str(&format!("DepositTrackingMatcher::Vault({:#?})", vault))
            }
        }
    }
}

impl std::fmt::Display for VaultMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VaultMatcher::FarmName(farm) => {
                f.write_str(&format!("VaultMatcher::FarmName({:#?})", farm))
            }
            VaultMatcher::Account(account) => {
                f.write_str(&format!("VaultMatcher::Account({:#?})", account))
            }
        }
    }
}

impl std::fmt::Display for TokenPriceMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenPriceMatcher::Asset(asset) => {
                f.write_str(&format!("TokenPriceMatcher::Asset({:#?})", asset))
            }
            TokenPriceMatcher::AssetIdentifier(identifier) => f.write_str(&format!(
                "TokenPriceMatcher::AssetIdentifier({:#?})",
                identifier
            )),
            TokenPriceMatcher::AssetAndPlatform(infos) => f.write_str(&format!(
                "TokenPriceMatcher::AssetAndPlatform({:#?})",
                infos
            )),
            TokenPriceMatcher::All => f.write_str(""),
        }
    }
}

impl std::fmt::Display for InterestRateMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterestRateMatcher::Asset(asset) => {
                f.write_str(&format!("InterestRateMatcher::Asset({:#?})", asset))
            }
            InterestRateMatcher::Platform(asset) => {
                f.write_str(&format!("InterestRateMatcher::Asset({:#?})", asset))
            }
            InterestRateMatcher::AssetAndPlatform(assets) => f.write_str(&format!(
                "InterestRateMatcher::AssetAndPlatform({:#?})",
                assets
            )),
            InterestRateMatcher::PlatformAndAsset(assets) => f.write_str(&format!(
                "InterestRateMatcher::PlatformAndAsset({:#?})",
                assets
            )),
            InterestRateMatcher::All => f.write_str(""),
        }
    }
}

impl std::fmt::Display for VaultTvlMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VaultTvlMatcher::FarmName(name) => {
                f.write_str(&format!("VaultTvlMatcher::FarmName({:#?})", name))
            }
            VaultTvlMatcher::All => f.write_str(""),
        }
    }
}

impl std::fmt::Display for RealizeYieldMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RealizeYieldMatcher::FarmName(name) => {
                f.write_str(&format!("RealizeYieldMatcher::FarmName({:#?})", name))
            }
            RealizeYieldMatcher::Account(account) => {
                f.write_str(&format!("RealizeYieldMatcher::Account({:#?})", account))
            }
            RealizeYieldMatcher::All => f.write_str(""),
        }
    }
}

impl std::fmt::Display for InterestRateCurveMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterestRateCurveMatcher::Platform(platform) => f.write_str(&format!(
                "InterestRateCurveMatcher::Platform({:#?})",
                platform
            )),
            InterestRateCurveMatcher::Asset(asset) => {
                f.write_str(&format!("InterestRateCurveMatcher::Asset({:#?})", asset))
            }
            InterestRateCurveMatcher::PlatformAndAsset(platforms) => f.write_str(&format!(
                "InterestRateCurveMatcher::PlatformAndAsset({:#?})",
                platforms,
            )),
            InterestRateCurveMatcher::RateName(name) => {
                f.write_str(&format!("InterestRateCurveMatcher::RateName({:#?})", name))
            }
            InterestRateCurveMatcher::All => f.write_str(""),
        }
    }
}

impl std::fmt::Display for LendingOptimizerDistributionMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LendingOptimizerDistributionMatcher::VaultName(name) => f.write_str(&format!(
                "LendingOptimizerDistribution::VaultName({:#?})",
                name
            )),
            LendingOptimizerDistributionMatcher::All => f.write_str(""),
        }
    }
}

impl std::fmt::Display for InterestRateMovingAverageMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterestRateMovingAverageMatcher::Asset(asset) => f.write_str(&format!(
                "InterestRateMovingAverageMatcher::Asset({:#?})",
                asset
            )),
            InterestRateMovingAverageMatcher::Platform(asset) => f.write_str(&format!(
                "InterestRateMovingAverageMatcher::Asset({:#?})",
                asset
            )),
            InterestRateMovingAverageMatcher::RateName(rate_name) => f.write_str(&format!(
                "InterestRateMovingAverageMatcher::RateName({:#?})",
                rate_name,
            )),
            InterestRateMovingAverageMatcher::All => f.write_str(""),
        }
    }
}

pub fn cmp_ltvs(a: &V1ObligationLtv, b: &V1ObligationLtv) -> Ordering {
    utils::misc::cmp_f64(&a.ltv, &b.ltv)
}

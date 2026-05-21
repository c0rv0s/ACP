use anchor_lang::prelude::*;

use crate::{
    CollateralAssetConfig, CreditFacilityConfig, GovernanceAuthorities, KeeperPermissions,
    MintDistribution, PauseFlags, Regime, SettlementRecipients,
};

#[event]
pub struct ProtocolInitialized {
    pub admin: Pubkey,
    pub agc_mint: Pubkey,
    pub xagc_mint: Pubkey,
    pub usdc_mint: Pubkey,
    pub initial_anchor_price_x18: u128,
}

#[event]
pub struct KeeperPermissionsUpdated {
    pub keeper: Pubkey,
    pub permissions: KeeperPermissions,
}

#[event]
pub struct MarketAdapterAuthorityUpdated {
    pub authority: Pubkey,
}

#[event]
pub struct PythReceiverProgramUpdated {
    pub receiver_program: Pubkey,
}

#[event]
pub struct BuybackUsdcEscrowUpdated {
    pub escrow: Pubkey,
}

#[event]
pub struct AdminTransferStarted {
    pub current_admin: Pubkey,
    pub pending_admin: Pubkey,
}

#[event]
pub struct AdminTransferred {
    pub previous_admin: Pubkey,
    pub new_admin: Pubkey,
}

#[event]
pub struct PauseFlagsUpdated {
    pub pause_flags: PauseFlags,
}

#[event]
pub struct GovernanceAuthoritiesUpdated {
    pub authorities: GovernanceAuthorities,
}

#[event]
pub struct CollateralAssetUpdated {
    pub mint: Pubkey,
    pub config: CollateralAssetConfig,
}

#[event]
pub struct CollateralOraclePriceUpdated {
    pub mint: Pubkey,
    pub price_quote_x18: u128,
    pub confidence_bps: u16,
    pub updated_at: u64,
}

#[event]
pub struct CreditFacilityInitialized {
    pub facility: Pubkey,
    pub facility_id: u64,
    pub collateral_mint: Pubkey,
    pub config: CreditFacilityConfig,
}

#[event]
pub struct CreditFacilityConfigUpdated {
    pub facility: Pubkey,
    pub config: CreditFacilityConfig,
}

#[event]
pub struct UnderwriterAgcDeposited {
    pub facility: Pubkey,
    pub underwriter: Pubkey,
    pub amount: u64,
    pub shares: u64,
}

#[event]
pub struct UnderwriterAgcWithdrawn {
    pub facility: Pubkey,
    pub underwriter: Pubkey,
    pub assets: u64,
    pub shares: u64,
}

#[event]
pub struct CreditLineOpened {
    pub facility: Pubkey,
    pub borrower: Pubkey,
    pub line_id: u64,
    pub credit_limit_agc: u64,
    pub maturity_timestamp: u64,
}

#[event]
pub struct CreditCollateralDeposited {
    pub facility: Pubkey,
    pub borrower: Pubkey,
    pub line: Pubkey,
    pub amount: u64,
}

#[event]
pub struct CreditCollateralWithdrawn {
    pub facility: Pubkey,
    pub borrower: Pubkey,
    pub line: Pubkey,
    pub amount: u64,
}

#[event]
pub struct CreditLineDrawn {
    pub facility: Pubkey,
    pub borrower: Pubkey,
    pub line: Pubkey,
    pub gross_amount: u64,
    pub net_amount: u64,
    pub fee: u64,
}

#[event]
pub struct CreditLineRepaid {
    pub facility: Pubkey,
    pub borrower: Pubkey,
    pub line: Pubkey,
    pub principal_paid: u64,
    pub interest_paid: u64,
}

#[event]
pub struct CreditLineDefaulted {
    pub facility: Pubkey,
    pub borrower: Pubkey,
    pub line: Pubkey,
    pub defaulted_debt: u64,
    pub underwriter_loss: u64,
    pub uncovered_debt: u64,
}

#[event]
pub struct DefaultedCollateralSeized {
    pub facility: Pubkey,
    pub line: Pubkey,
    pub destination: Pubkey,
    pub amount: u64,
}

#[event]
pub struct PolicyParametersUpdated {
    pub normal_band_bps: u16,
    pub stressed_band_bps: u16,
    pub policy_epoch_duration: u64,
}

#[event]
pub struct MintDistributionUpdated {
    pub distribution: MintDistribution,
}

#[event]
pub struct SettlementRecipientsUpdated {
    pub recipients: SettlementRecipients,
}

#[event]
pub struct GrowthProgramsEnabledUpdated {
    pub enabled: bool,
}

#[event]
pub struct ExitFeeUpdated {
    pub exit_fee_bps: u16,
}

#[event]
pub struct XagcDeposited {
    pub caller: Pubkey,
    pub receiver_xagc: Pubkey,
    pub assets: u64,
    pub shares: u64,
}

#[event]
pub struct XagcRedeemed {
    pub caller: Pubkey,
    pub receiver_agc: Pubkey,
    pub shares: u64,
    pub gross_assets: u64,
    pub fee_assets: u64,
    pub net_assets: u64,
}

#[event]
pub struct SwapRecorded {
    pub epoch_id: u64,
    pub agc_amount: u64,
    pub usdc_amount: u64,
    pub price_x18: u128,
    pub agc_to_usdc: bool,
}

#[event]
pub struct EpochSettled {
    pub epoch_id: u64,
    pub regime: Regime,
    pub anchor_next_x18: u128,
    pub mint_budget_acp: u128,
    pub buyback_budget_quote_x18: u128,
}

#[event]
pub struct TreasuryBuybackUsdcReserved {
    pub nonce: u64,
    pub usdc_spent: u64,
    pub pending_treasury_buyback_usdc_after: u64,
}

#[event]
pub struct BuybackCampaignStarted {
    pub campaign: Pubkey,
    pub campaign_id: u64,
    pub total_usdc: u64,
    pub min_total_agc_out: u64,
    pub adapter_usdc_account: Pubkey,
}

#[event]
pub struct BuybackTwapSliceExecuted {
    pub campaign: Pubkey,
    pub campaign_id: u64,
    pub usdc_amount: u64,
    pub agc_burned: u64,
    pub remaining_usdc: u64,
    pub total_agc_burned: u64,
}

#[event]
pub struct BuybackCampaignCancelled {
    pub campaign: Pubkey,
    pub campaign_id: u64,
}

#[event]
pub struct TreasuryAgcBurned {
    pub amount: u64,
}


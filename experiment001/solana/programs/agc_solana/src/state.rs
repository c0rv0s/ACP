use anchor_lang::prelude::*;

#[account]
pub struct ProtocolState {
    pub admin: Pubkey,
    pub pending_admin: Pubkey,
    pub risk_admin: Pubkey,
    pub emergency_admin: Pubkey,
    pub agc_mint: Pubkey,
    pub xagc_mint: Pubkey,
    pub usdc_mint: Pubkey,
    pub treasury_agc: Pubkey,
    pub treasury_usdc: Pubkey,
    pub xagc_vault_agc: Pubkey,
    pub growth_programs_agc: Pubkey,
    pub lp_agc: Pubkey,
    pub integrators_agc: Pubkey,
    pub buyback_usdc_escrow: Pubkey,
    pub market_adapter_authority: Pubkey,
    pub pyth_receiver_program: Pubkey,
    pub state_bump: u8,
    pub mint_authority_bump: u8,
    pub treasury_authority_bump: u8,
    pub xagc_authority_bump: u8,
    pub treasury_agc_bump: u8,
    pub treasury_usdc_bump: u8,
    pub xagc_vault_agc_bump: u8,
    pub agc_decimals: u8,
    pub xagc_decimals: u8,
    pub usdc_decimals: u8,
    pub agc_unit: u64,
    pub quote_scale: u128,
    pub exit_fee_bps: u16,
    pub growth_programs_enabled: bool,
    pub pause_flags: PauseFlags,
    pub policy_params: PolicyParams,
    pub mint_distribution: MintDistribution,
    pub regime: Regime,
    pub anchor_price_x18: u128,
    pub premium_persistence_epochs: u128,
    pub last_gross_buy_quote_x18: u128,
    pub last_coverage_bps: u128,
    pub last_exit_pressure_bps: u128,
    pub last_volatility_bps: u128,
    pub last_premium_bps: u128,
    pub last_locked_share_bps: u128,
    pub last_lock_flow_bps: u128,
    pub last_stable_cash_coverage_bps: u128,
    pub last_liquidity_depth_coverage_bps: u128,
    pub last_reserve_concentration_bps: u128,
    pub last_oracle_confidence_bps: u128,
    pub last_stale_oracle_count: u16,
    pub last_settled_epoch: u64,
    pub last_settlement_timestamp: u64,
    pub recovery_cooldown_epochs_remaining: u64,
    pub mint_window_day: u64,
    pub minted_in_current_day: u128,
    pub pending_treasury_buyback_usdc: u64,
    pub xagc_gross_deposits_total: u128,
    pub xagc_gross_redemptions_total: u128,
    pub xagc_unaccounted_assets: u64,
    pub last_xagc_deposit_total: u128,
    pub last_xagc_redemption_total: u128,
    pub buyback_execution_nonce: u64,
    pub protocol_version: u16,
    pub credit_facility_count: u64,
    pub credit_principal_outstanding_agc: u128,
    pub credit_drawn_agc: u128,
    pub credit_repaid_agc: u128,
    pub credit_interest_paid_agc: u128,
    pub credit_defaulted_agc: u128,
    pub accumulator: EpochAccumulator,
    pub last_epoch_result: EpochResult,
}

impl ProtocolState {
    pub const LEN: usize = 8192;
}

#[account]
pub struct Keeper {
    pub authority: Pubkey,
    pub permissions: KeeperPermissions,
    pub bump: u8,
}

impl Keeper {
    pub const LEN: usize = 32 + KeeperPermissions::LEN + 1;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct KeeperPermissions {
    pub market_reporter: bool,
    pub oracle_reporter: bool,
    pub epoch_settler: bool,
    pub buyback_executor: bool,
    pub treasury_burner: bool,
    pub credit_operator: bool,
}

impl KeeperPermissions {
    pub const LEN: usize = 6;

    pub fn all() -> Self {
        Self {
            market_reporter: true,
            oracle_reporter: true,
            epoch_settler: true,
            buyback_executor: true,
            treasury_burner: true,
            credit_operator: true,
        }
    }

    pub(crate) fn allows(self, required: RequiredKeeperPermission) -> bool {
        match required {
            RequiredKeeperPermission::ReportMarket => self.market_reporter,
            RequiredKeeperPermission::ReportOracle => self.oracle_reporter,
            RequiredKeeperPermission::SettleEpoch => self.epoch_settler,
            RequiredKeeperPermission::ExecuteBuyback => self.buyback_executor,
            RequiredKeeperPermission::BurnTreasury => self.treasury_burner,
            RequiredKeeperPermission::OperateCredit => self.credit_operator,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum RequiredKeeperPermission {
    ReportMarket,
    ReportOracle,
    SettleEpoch,
    ExecuteBuyback,
    BurnTreasury,
    OperateCredit,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct PauseFlags {
    pub xagc_deposits_paused: bool,
    pub xagc_redemptions_paused: bool,
    pub market_reporting_paused: bool,
    pub settlement_paused: bool,
    pub credit_issuance_paused: bool,
    pub collateral_updates_paused: bool,
    pub buybacks_paused: bool,
    pub treasury_burns_paused: bool,
    pub credit_facility_updates_paused: bool,
    pub credit_line_updates_paused: bool,
    pub credit_draws_paused: bool,
    pub credit_repayments_paused: bool,
    pub underwriter_deposits_paused: bool,
    pub underwriter_withdrawals_paused: bool,
    pub liquidations_paused: bool,
}

#[account]
pub struct CollateralAsset {
    pub mint: Pubkey,
    pub mint_decimals: u8,
    pub oracle_source: OracleSource,
    pub oracle_feed: Pubkey,
    pub pyth_price_feed_id: [u8; 32],
    pub reserve_token_account: Pubkey,
    pub asset_class: AssetClass,
    pub reserve_weight_bps: u16,
    pub collateral_factor_bps: u16,
    pub liquidation_threshold_bps: u16,
    pub max_concentration_bps: u16,
    pub max_oracle_staleness_seconds: u64,
    pub max_oracle_confidence_bps: u16,
    pub enabled: bool,
    pub bump: u8,
}

impl CollateralAsset {
    pub const LEN: usize = 32 + 1 + 1 + 32 + 32 + 32 + 1 + 2 + 2 + 2 + 2 + 8 + 2 + 1 + 1 + 64;
}

#[account]
pub struct CollateralOracle {
    pub mint: Pubkey,
    pub oracle_feed: Pubkey,
    pub oracle_source: OracleSource,
    pub pyth_price_feed_id: [u8; 32],
    pub price_quote_x18: u128,
    pub confidence_bps: u16,
    pub updated_at: u64,
    pub publish_time: u64,
    pub bump: u8,
    pub reserved: [u8; 64],
}

impl CollateralOracle {
    pub const LEN: usize = 32 + 32 + 1 + 32 + 16 + 2 + 8 + 8 + 1 + 64;
}

#[account]
pub struct BuybackCampaign {
    pub campaign_id: u64,
    pub status: BuybackCampaignStatus,
    pub total_usdc: u64,
    pub remaining_usdc: u64,
    pub spent_usdc: u64,
    pub min_total_agc_out: u64,
    pub agc_burned: u64,
    pub max_slice_usdc: u64,
    pub slice_interval_seconds: u64,
    pub started_at: u64,
    pub expires_at: u64,
    pub last_slice_at: u64,
    pub slice_count: u64,
    pub adapter_usdc_account: Pubkey,
    pub usdc_escrow: Pubkey,
    pub agc_vault: Pubkey,
    pub bump: u8,
    pub authority_bump: u8,
    pub usdc_escrow_bump: u8,
    pub agc_vault_bump: u8,
    pub reserved: [u8; 128],
}

impl BuybackCampaign {
    pub const LEN: usize = 8 + 1 + (8 * 11) + (32 * 3) + 4 + 128;
}

#[account]
pub struct CreditFacility {
    pub facility_id: u64,
    pub collateral_mint: Pubkey,
    pub collateral_asset: Pubkey,
    pub collateral_vault: Pubkey,
    pub underwriter_vault_agc: Pubkey,
    pub collateral_decimals: u8,
    pub config: CreditFacilityConfig,
    pub status: CreditFacilityStatus,
    pub underwriter_total_shares: u64,
    pub total_principal_debt_agc: u64,
    pub total_underwriter_deposits_agc: u128,
    pub total_underwriter_withdrawals_agc: u128,
    pub total_drawn_agc: u128,
    pub total_repaid_principal_agc: u128,
    pub total_interest_accrued_agc: u128,
    pub total_interest_paid_agc: u128,
    pub total_defaulted_agc: u128,
    pub total_underwriter_loss_agc: u128,
    pub total_collateral_deposited: u128,
    pub total_collateral_seized: u128,
    pub active_credit_lines: u64,
    pub created_at: u64,
    pub bump: u8,
    pub authority_bump: u8,
    pub collateral_vault_bump: u8,
    pub underwriter_vault_bump: u8,
    pub reserved: [u8; 256],
}

impl CreditFacility {
    pub const LEN: usize = 1024;
}

#[account]
pub struct UnderwriterPosition {
    pub facility: Pubkey,
    pub underwriter: Pubkey,
    pub shares: u64,
    pub deposited_agc: u128,
    pub withdrawn_agc: u128,
    pub loss_agc: u128,
    pub bump: u8,
    pub reserved: [u8; 128],
}

impl UnderwriterPosition {
    pub const LEN: usize = 32 + 32 + 8 + 16 + 16 + 16 + 1 + 128;
}

#[account]
pub struct CreditLine {
    pub facility: Pubkey,
    pub borrower: Pubkey,
    pub line_id: u64,
    pub credit_limit_agc: u64,
    pub principal_debt_agc: u64,
    pub accrued_interest_agc: u64,
    pub collateral_amount: u64,
    pub maturity_timestamp: u64,
    pub opened_at: u64,
    pub last_accrued_at: u64,
    pub defaulted_at: u64,
    pub closed_at: u64,
    pub status: CreditLineStatus,
    pub underwriter_loss_agc: u64,
    pub uncovered_default_agc: u64,
    pub collateral_seized: u128,
    pub bump: u8,
    pub reserved: [u8; 128],
}

impl CreditLine {
    pub const LEN: usize =
        32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 1 + 8 + 8 + 16 + 1 + 128;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AssetClass {
    #[default]
    Stable,
    Btc,
    Rwa,
    Other,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CreditFacilityStatus {
    #[default]
    Uninitialized,
    Active,
    Disabled,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CreditLineStatus {
    #[default]
    Uninitialized,
    Active,
    Repaid,
    Defaulted,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum OracleSource {
    #[default]
    Manual,
    Pyth,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BuybackCampaignStatus {
    #[default]
    Uninitialized,
    Active,
    Completed,
    Cancelled,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct CollateralOraclePriceInput {
    pub price_quote_x18: u128,
    pub confidence_bps: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct CreditFacilityConfig {
    pub max_total_debt_agc: u64,
    pub max_line_debt_agc: u64,
    pub min_collateral_health_bps: u16,
    pub liquidation_health_bps: u16,
    pub min_underwriter_reserve_bps: u16,
    pub interest_rate_bps: u16,
    pub origination_fee_bps: u16,
    pub default_grace_seconds: u64,
    pub isolated: bool,
    pub enabled: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct OpenCreditLineArgs {
    pub credit_limit_agc: u64,
    pub maturity_timestamp: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct CollateralAssetConfig {
    pub oracle_source: OracleSource,
    pub oracle_feed: Pubkey,
    pub pyth_price_feed_id: [u8; 32],
    pub reserve_token_account: Pubkey,
    pub asset_class: AssetClass,
    pub reserve_weight_bps: u16,
    pub collateral_factor_bps: u16,
    pub liquidation_threshold_bps: u16,
    pub max_concentration_bps: u16,
    pub max_oracle_staleness_seconds: u64,
    pub max_oracle_confidence_bps: u16,
    pub enabled: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct BuybackCampaignConfig {
    pub total_usdc: u64,
    pub min_total_agc_out: u64,
    pub max_slice_usdc: u64,
    pub slice_interval_seconds: u64,
    pub start_after: u64,
    pub expires_at: u64,
    pub adapter_usdc_account: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct BuybackSliceArgs {
    pub usdc_amount: u64,
    pub agc_amount_to_burn: u64,
    pub min_agc_out: u64,
    pub deadline: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct GovernanceAuthorities {
    pub risk_admin: Pubkey,
    pub emergency_admin: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Regime {
    #[default]
    Neutral,
    Expansion,
    Defense,
    Recovery,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct InitializeProtocolArgs {
    pub initial_anchor_price_x18: u128,
    pub policy_params: PolicyParams,
    pub mint_distribution: MintDistribution,
    pub settlement_recipients: SettlementRecipients,
    pub exit_fee_bps: u16,
    pub growth_programs_enabled: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct SettlementRecipients {
    pub growth_programs_agc: Pubkey,
    pub lp_agc: Pubkey,
    pub integrators_agc: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct MintDistribution {
    pub xagc_bps: u16,
    pub growth_programs_bps: u16,
    pub lp_bps: u16,
    pub integrators_bps: u16,
    pub treasury_bps: u16,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct MintAllocation {
    pub xagc_mint_acp: u128,
    pub growth_programs_mint_acp: u128,
    pub lp_mint_acp: u128,
    pub integrators_mint_acp: u128,
    pub treasury_mint_acp: u128,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct PolicyParams {
    pub normal_band_bps: u16,
    pub stressed_band_bps: u16,
    pub anchor_ema_bps: u16,
    pub max_anchor_crawl_bps: u16,
    pub min_premium_bps: u16,
    pub premium_persistence_required: u16,
    pub min_gross_buy_floor_bps: u16,
    pub min_locked_share_bps: u16,
    pub target_gross_buy_bps: u16,
    pub target_net_buy_bps: u16,
    pub target_lock_flow_bps: u16,
    pub target_buy_growth_bps: u16,
    pub target_locked_share_bps: u16,
    pub expansion_reserve_coverage_bps: u16,
    pub target_reserve_coverage_bps: u16,
    pub neutral_reserve_coverage_bps: u16,
    pub defense_reserve_coverage_bps: u16,
    pub hard_defense_reserve_coverage_bps: u16,
    pub min_stable_cash_coverage_bps: u16,
    pub target_stable_cash_coverage_bps: u16,
    pub defense_stable_cash_coverage_bps: u16,
    pub min_liquidity_depth_coverage_bps: u16,
    pub target_liquidity_depth_coverage_bps: u16,
    pub max_reserve_concentration_bps: u16,
    pub max_oracle_confidence_bps: u16,
    pub max_stale_oracle_count: u16,
    pub max_expansion_volatility_bps: u16,
    pub defense_volatility_bps: u16,
    pub max_expansion_exit_pressure_bps: u16,
    pub defense_exit_pressure_bps: u16,
    pub expansion_kappa_bps: u16,
    pub max_mint_per_epoch_bps: u16,
    pub max_mint_per_day_bps: u16,
    pub buyback_kappa_bps: u16,
    pub mild_defense_spend_bps: u16,
    pub severe_defense_spend_bps: u16,
    pub severe_stress_threshold_bps: u16,
    pub recovery_cooldown_epochs: u16,
    pub policy_epoch_duration: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct EpochAccumulator {
    pub epoch_id: u64,
    pub started_at: u64,
    pub updated_at: u64,
    pub last_observed_at: u64,
    pub observation_count: u64,
    pub gross_buy_volume_quote_x18: u128,
    pub gross_sell_volume_quote_x18: u128,
    pub total_volume_quote_x18: u128,
    pub last_mid_price_x18: u128,
    pub cumulative_mid_price_time_x18: u128,
    pub cumulative_abs_mid_price_change_bps: u128,
    pub total_hook_fees_quote_x18: u128,
    pub total_hook_fees_agc: u128,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct EpochSnapshot {
    pub epoch_id: u64,
    pub started_at: u64,
    pub ended_at: u64,
    pub gross_buy_volume_quote_x18: u128,
    pub gross_sell_volume_quote_x18: u128,
    pub total_volume_quote_x18: u128,
    pub short_twap_price_x18: u128,
    pub realized_volatility_bps: u128,
    pub total_hook_fees_quote_x18: u128,
    pub total_hook_fees_agc: u128,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct ExternalMetrics {
    pub depth_to_target_slippage_quote_x18: u128,
    pub stable_cash_reserve_quote_x18: u128,
    pub risk_weighted_reserve_quote_x18: u128,
    pub liquidity_depth_quote_x18: u128,
    pub largest_collateral_concentration_bps: u16,
    pub oracle_confidence_bps: u16,
    pub stale_oracle_count: u16,
}

#[derive(Clone, Copy, Default)]
pub struct PolicyState {
    pub anchor_price_x18: u128,
    pub premium_persistence_epochs: u128,
    pub last_gross_buy_quote_x18: u128,
    pub minted_today_acp: u128,
    pub last_regime: Regime,
    pub recovery_cooldown_epochs_remaining: u64,
    pub float_supply_acp: u128,
    pub treasury_quote_x18: u128,
    pub treasury_acp: u128,
    pub xagc_total_assets_acp: u128,
}

#[derive(Clone, Copy, Default)]
pub struct VaultFlows {
    pub xagc_deposits_acp: u128,
    pub xagc_gross_redemptions_acp: u128,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct EpochResult {
    pub epoch_id: u64,
    pub regime: Regime,
    pub anchor_price_x18: u128,
    pub anchor_next_x18: u128,
    pub normal_floor_x18: u128,
    pub stressed_floor_x18: u128,
    pub price_twap_x18: u128,
    pub premium_bps: u128,
    pub premium_persistence_epochs: u128,
    pub credit_outstanding_quote_x18: u128,
    pub gross_buy_floor_bps: u128,
    pub net_buy_pressure_bps: u128,
    pub buy_growth_bps: u128,
    pub exit_pressure_bps: u128,
    pub reserve_coverage_bps: u128,
    pub stable_cash_coverage_bps: u128,
    pub liquidity_depth_coverage_bps: u128,
    pub locked_share_bps: u128,
    pub lock_flow_bps: u128,
    pub demand_score_bps: u128,
    pub health_score_bps: u128,
    pub mint_rate_bps: u128,
    pub mint_budget_acp: u128,
    pub buyback_budget_quote_x18: u128,
    pub stress_score_bps: u128,
    pub gross_buy_quote_x18: u128,
    pub gross_sell_quote_x18: u128,
    pub total_volume_quote_x18: u128,
    pub depth_to_target_slippage_quote_x18: u128,
    pub stable_cash_reserve_quote_x18: u128,
    pub risk_weighted_reserve_quote_x18: u128,
    pub liquidity_depth_quote_x18: u128,
    pub largest_collateral_concentration_bps: u16,
    pub oracle_confidence_bps: u16,
    pub stale_oracle_count: u16,
    pub realized_volatility_bps: u128,
    pub xagc_deposits_acp: u128,
    pub xagc_gross_redemptions_acp: u128,
    pub treasury_quote_x18: u128,
    pub treasury_acp: u128,
    pub xagc_total_assets_acp: u128,
    pub mint_allocations: MintAllocation,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct RecordSwapArgs {
    pub agc_amount: u64,
    pub usdc_amount: u64,
    pub price_x18: u128,
    pub agc_to_usdc: bool,
    pub hook_fee_usdc: u64,
    pub hook_fee_agc: u64,
}


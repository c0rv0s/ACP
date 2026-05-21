use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_option::COption;
use anchor_spl::token::Mint;

use crate::{
    AgcError, AssetClass, BuybackCampaign, BuybackCampaignConfig, BuybackCampaignStatus,
    BuybackSliceArgs, CollateralAssetConfig, CreditFacility, CreditFacilityConfig,
    GovernanceAuthorities, MintDistribution, OpenCreditLineArgs, OracleSource, PolicyParams, BPS,
};

pub fn validate_mint_authority(mint: &Account<Mint>, authority: Pubkey) -> Result<()> {
    require!(
        mint.mint_authority == COption::Some(authority),
        AgcError::InvalidMintAuthority
    );
    Ok(())
}

pub fn validate_no_freeze_authority(mint: &Account<Mint>) -> Result<()> {
    require!(
        mint.freeze_authority == COption::None,
        AgcError::InvalidFreezeAuthority
    );
    Ok(())
}

pub fn validate_distribution(distribution: MintDistribution) -> Result<()> {
    let total = distribution.xagc_bps as u32
        + distribution.growth_programs_bps as u32
        + distribution.lp_bps as u32
        + distribution.integrators_bps as u32
        + distribution.treasury_bps as u32;
    require!(
        total == BPS as u32 && distribution.xagc_bps > 0,
        AgcError::InvalidMintDistribution
    );
    Ok(())
}

pub fn validate_governance_authorities(authorities: GovernanceAuthorities) -> Result<()> {
    require!(
        authorities.risk_admin != Pubkey::default()
            && authorities.emergency_admin != Pubkey::default(),
        AgcError::InvalidGovernanceAuthority
    );
    Ok(())
}

pub fn validate_collateral_asset_config(
    config: CollateralAssetConfig,
    pyth_receiver_program: Pubkey,
) -> Result<()> {
    require!(
        config.reserve_weight_bps <= BPS as u16,
        AgcError::InvalidCollateralAssetConfig
    );
    require!(
        config.collateral_factor_bps <= config.liquidation_threshold_bps,
        AgcError::InvalidCollateralAssetConfig
    );
    require!(
        config.liquidation_threshold_bps <= BPS as u16,
        AgcError::InvalidCollateralAssetConfig
    );
    require!(
        config.max_concentration_bps > 0 && config.max_concentration_bps <= BPS as u16,
        AgcError::InvalidCollateralAssetConfig
    );
    require!(
        config.max_oracle_staleness_seconds > 0,
        AgcError::InvalidCollateralAssetConfig
    );
    require!(
        config.max_oracle_confidence_bps <= BPS as u16,
        AgcError::InvalidCollateralAssetConfig
    );
    if config.enabled {
        match config.oracle_source {
            OracleSource::Manual => {
                require!(
                    config.oracle_feed != Pubkey::default(),
                    AgcError::InvalidCollateralAssetConfig
                );
                require!(
                    config.pyth_price_feed_id == [0; 32],
                    AgcError::InvalidCollateralAssetConfig
                );
            }
            OracleSource::Pyth => {
                require!(
                    pyth_receiver_program != Pubkey::default(),
                    AgcError::InvalidCollateralAssetConfig
                );
                require!(
                    config.pyth_price_feed_id != [0; 32],
                    AgcError::InvalidCollateralAssetConfig
                );
            }
        }
        require!(
            config.reserve_token_account != Pubkey::default(),
            AgcError::InvalidCollateralAssetConfig
        );
    }
    Ok(())
}

pub fn validate_buyback_campaign_config(
    config: BuybackCampaignConfig,
    pending_treasury_buyback_usdc: u64,
    now: u64,
) -> Result<()> {
    require!(
        config.total_usdc > 0 && config.total_usdc <= pending_treasury_buyback_usdc,
        AgcError::InvalidBuybackCampaignConfig
    );
    require!(
        config.min_total_agc_out > 0,
        AgcError::InvalidBuybackCampaignConfig
    );
    require!(
        config.adapter_usdc_account != Pubkey::default(),
        AgcError::InvalidBuybackCampaignConfig
    );
    require!(
        config.max_slice_usdc > 0 && config.max_slice_usdc <= config.total_usdc,
        AgcError::InvalidBuybackCampaignConfig
    );
    require!(
        config.slice_interval_seconds > 0,
        AgcError::InvalidBuybackCampaignConfig
    );
    let started_at = if config.start_after == 0 {
        now
    } else {
        config.start_after
    };
    require!(
        config.expires_at > started_at,
        AgcError::InvalidBuybackCampaignConfig
    );
    require!(
        config.expires_at > now,
        AgcError::InvalidBuybackCampaignConfig
    );
    Ok(())
}

pub fn validate_buyback_slice(
    campaign: &BuybackCampaign,
    args: BuybackSliceArgs,
    now: u64,
) -> Result<()> {
    require!(
        campaign.status == BuybackCampaignStatus::Active,
        AgcError::BuybackCampaignInactive
    );
    require!(
        now >= campaign.started_at && now <= campaign.expires_at,
        AgcError::BuybackCampaignNotReady
    );
    require!(args.deadline >= now, AgcError::BuybackDeadlineExpired);
    require!(
        args.usdc_amount > 0 && args.agc_amount_to_burn > 0 && args.min_agc_out > 0,
        AgcError::ZeroAmount
    );
    require!(
        args.usdc_amount <= campaign.max_slice_usdc && args.usdc_amount <= campaign.remaining_usdc,
        AgcError::BuybackSliceTooLarge
    );
    require!(
        args.agc_amount_to_burn >= args.min_agc_out,
        AgcError::InsufficientBuybackOutput
    );
    if campaign.slice_count > 0 {
        let next_slice_at = campaign
            .last_slice_at
            .checked_add(campaign.slice_interval_seconds)
            .ok_or(AgcError::MathOverflow)?;
        require!(now >= next_slice_at, AgcError::BuybackCampaignNotReady);
    }
    Ok(())
}

pub fn validate_credit_facility_config(
    config: CreditFacilityConfig,
    asset_class: AssetClass,
) -> Result<()> {
    if config.enabled {
        require!(
            config.max_total_debt_agc > 0 && config.max_line_debt_agc > 0,
            AgcError::InvalidCreditFacilityConfig
        );
        require!(
            config.min_underwriter_reserve_bps > 0,
            AgcError::InvalidCreditFacilityConfig
        );
    }
    require!(
        config.max_line_debt_agc <= config.max_total_debt_agc,
        AgcError::InvalidCreditFacilityConfig
    );
    require!(
        config.liquidation_health_bps >= BPS as u16,
        AgcError::InvalidCreditFacilityConfig
    );
    require!(
        config.min_collateral_health_bps >= config.liquidation_health_bps,
        AgcError::InvalidCreditFacilityConfig
    );
    require!(
        config.min_underwriter_reserve_bps <= BPS as u16,
        AgcError::InvalidCreditFacilityConfig
    );
    require!(
        config.interest_rate_bps <= BPS as u16 && config.origination_fee_bps < BPS as u16,
        AgcError::InvalidCreditFacilityConfig
    );
    require!(
        config.default_grace_seconds > 0,
        AgcError::InvalidCreditFacilityConfig
    );
    if asset_class == AssetClass::Rwa {
        require!(config.isolated, AgcError::InvalidCreditFacilityConfig);
    }
    Ok(())
}

pub fn validate_open_credit_line_args(
    args: OpenCreditLineArgs,
    facility: &CreditFacility,
) -> Result<()> {
    require!(
        args.credit_limit_agc > 0 && args.credit_limit_agc <= facility.config.max_line_debt_agc,
        AgcError::InvalidCreditLineConfig
    );
    Ok(())
}

pub fn validate_policy_params(params: PolicyParams) -> Result<()> {
    require!(
        params.policy_epoch_duration > 0,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.normal_band_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.stressed_band_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.normal_band_bps <= params.stressed_band_bps,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.anchor_ema_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.max_anchor_crawl_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(params.min_premium_bps > 0, AgcError::InvalidPolicyParams);
    require!(
        params.target_gross_buy_bps > 0,
        AgcError::InvalidPolicyParams
    );
    require!(params.target_net_buy_bps > 0, AgcError::InvalidPolicyParams);
    require!(
        params.target_lock_flow_bps > 0,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.target_buy_growth_bps > 0,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.target_locked_share_bps > 0,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.expansion_reserve_coverage_bps <= params.target_reserve_coverage_bps,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.hard_defense_reserve_coverage_bps <= params.defense_reserve_coverage_bps,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.defense_reserve_coverage_bps <= params.neutral_reserve_coverage_bps,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.neutral_reserve_coverage_bps <= params.expansion_reserve_coverage_bps,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.defense_stable_cash_coverage_bps <= params.min_stable_cash_coverage_bps,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.min_stable_cash_coverage_bps < params.target_stable_cash_coverage_bps,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.min_liquidity_depth_coverage_bps < params.target_liquidity_depth_coverage_bps,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.max_reserve_concentration_bps > 0
            && params.max_reserve_concentration_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.max_oracle_confidence_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.max_expansion_volatility_bps > 0,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.defense_volatility_bps > params.max_expansion_volatility_bps,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.max_expansion_exit_pressure_bps > 0,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.max_expansion_exit_pressure_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.defense_exit_pressure_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.defense_exit_pressure_bps > params.max_expansion_exit_pressure_bps,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.expansion_kappa_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.max_mint_per_epoch_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.max_mint_per_day_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.buyback_kappa_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.mild_defense_spend_bps <= params.severe_defense_spend_bps,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.mild_defense_spend_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.severe_defense_spend_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    require!(
        params.severe_stress_threshold_bps > 0 && params.severe_stress_threshold_bps <= BPS as u16,
        AgcError::InvalidPolicyParams
    );
    Ok(())
}

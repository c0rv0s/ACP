use anchor_lang::prelude::*;

use crate::math::{
    checked_add_u128, checked_div_u128, checked_mul_u128, mul_div, pow10_u128, safe_div,
};
use crate::{
    AgcError, CollateralAsset, CollateralOracle, CreditFacility, CreditFacilityStatus, CreditLine,
    CreditLineStatus, OracleSource, BPS, SECONDS_PER_YEAR,
};

pub fn require_facility_active(facility: &CreditFacility) -> Result<()> {
    require!(
        facility.status == CreditFacilityStatus::Active && facility.config.enabled,
        AgcError::CreditFacilityInactive
    );
    Ok(())
}

pub fn require_credit_line_active(credit_line: &CreditLine) -> Result<()> {
    require!(
        credit_line.status == CreditLineStatus::Active,
        AgcError::CreditLineInactive
    );
    Ok(())
}

pub fn require_credit_line_open_for_repayment(credit_line: &CreditLine) -> Result<()> {
    require_credit_line_active(credit_line)
}

pub fn require_credit_line_allows_collateral_withdrawal(credit_line: &CreditLine) -> Result<()> {
    require!(
        matches!(
            credit_line.status,
            CreditLineStatus::Active | CreditLineStatus::Repaid
        ),
        AgcError::CreditLineInactive
    );
    Ok(())
}

pub fn collateral_withdrawal_needs_health_check(credit_line: &CreditLine) -> Result<bool> {
    require_credit_line_allows_collateral_withdrawal(credit_line)?;
    if credit_line.status == CreditLineStatus::Repaid {
        require!(
            credit_line_total_debt_agc(credit_line)? == 0,
            AgcError::CreditLineInactive
        );
        return Ok(false);
    }
    Ok(credit_line_total_debt_agc(credit_line)? > 0)
}

pub fn validate_oracle_fresh(
    collateral_asset: &CollateralAsset,
    collateral_oracle: &CollateralOracle,
    now: u64,
) -> Result<()> {
    require_keys_eq!(
        collateral_oracle.mint,
        collateral_asset.mint,
        AgcError::InvalidOraclePrice
    );
    require_keys_eq!(
        collateral_oracle.oracle_feed,
        collateral_asset.oracle_feed,
        AgcError::InvalidOraclePrice
    );
    require!(
        collateral_oracle.oracle_source == collateral_asset.oracle_source,
        AgcError::InvalidOraclePrice
    );
    if collateral_asset.oracle_source == OracleSource::Pyth {
        require!(
            collateral_oracle.pyth_price_feed_id == collateral_asset.pyth_price_feed_id,
            AgcError::InvalidOraclePrice
        );
    }
    require!(
        collateral_oracle.price_quote_x18 > 0,
        AgcError::InvalidOraclePrice
    );
    require!(
        collateral_oracle.confidence_bps <= collateral_asset.max_oracle_confidence_bps,
        AgcError::InvalidOraclePrice
    );
    require!(
        now.saturating_sub(collateral_oracle.updated_at)
            <= collateral_asset.max_oracle_staleness_seconds,
        AgcError::InvalidOraclePrice
    );
    Ok(())
}

pub fn credit_line_past_default_grace(
    credit_line: &CreditLine,
    facility: &CreditFacility,
    now: u64,
) -> bool {
    now > credit_line
        .maturity_timestamp
        .saturating_add(facility.config.default_grace_seconds)
}

pub fn validate_credit_line_defaultable(
    credit_line: &CreditLine,
    facility: &CreditFacility,
    collateral_asset: &CollateralAsset,
    collateral_oracle: &CollateralOracle,
    now: u64,
    anchor_price_x18: u128,
    agc_unit: u128,
) -> Result<()> {
    if credit_line_past_default_grace(credit_line, facility, now) {
        return Ok(());
    }

    validate_oracle_fresh(collateral_asset, collateral_oracle, now)?;
    let health_bps = credit_line_health_bps(
        credit_line,
        facility,
        collateral_oracle,
        credit_line.collateral_amount,
        anchor_price_x18,
        agc_unit,
    )?;
    require!(
        health_bps < facility.config.liquidation_health_bps as u128,
        AgcError::CreditLineHealthy
    );
    Ok(())
}

pub fn credit_line_total_debt_agc(credit_line: &CreditLine) -> Result<u64> {
    credit_line
        .principal_debt_agc
        .checked_add(credit_line.accrued_interest_agc)
        .ok_or(error!(AgcError::MathOverflow))
}

pub fn collateral_value_quote_x18(
    collateral_amount: u64,
    collateral_decimals: u8,
    collateral_price_quote_x18: u128,
) -> Result<u128> {
    let collateral_unit = pow10_u128(collateral_decimals)?;
    mul_div(
        collateral_amount as u128,
        collateral_price_quote_x18,
        collateral_unit,
    )
}

pub fn agc_value_quote_x18(amount_agc: u64, anchor_price_x18: u128, agc_unit: u128) -> Result<u128> {
    mul_div(amount_agc as u128, anchor_price_x18, agc_unit)
}

pub fn credit_line_health_bps(
    credit_line: &CreditLine,
    facility: &CreditFacility,
    collateral_oracle: &CollateralOracle,
    collateral_amount: u64,
    anchor_price_x18: u128,
    agc_unit: u128,
) -> Result<u128> {
    let total_debt_agc = credit_line_total_debt_agc(credit_line)?;
    credit_health_bps_for_debt(
        facility,
        collateral_oracle,
        collateral_amount,
        total_debt_agc,
        anchor_price_x18,
        agc_unit,
    )
}

pub fn credit_health_bps_for_debt(
    facility: &CreditFacility,
    collateral_oracle: &CollateralOracle,
    collateral_amount: u64,
    total_debt_agc: u64,
    anchor_price_x18: u128,
    agc_unit: u128,
) -> Result<u128> {
    if total_debt_agc == 0 {
        return Ok(u128::MAX);
    }

    let collateral_value = collateral_value_quote_x18(
        collateral_amount,
        facility.collateral_decimals,
        collateral_oracle.price_quote_x18,
    )?;
    let debt_value = agc_value_quote_x18(total_debt_agc, anchor_price_x18, agc_unit)?;
    safe_div(checked_mul_u128(collateral_value, BPS)?, debt_value)
}

pub fn validate_credit_line_health(
    credit_line: &CreditLine,
    facility: &CreditFacility,
    collateral_oracle: &CollateralOracle,
    collateral_amount: u64,
    anchor_price_x18: u128,
    agc_unit: u128,
    min_health_bps: u16,
) -> Result<()> {
    let health_bps = credit_line_health_bps(
        credit_line,
        facility,
        collateral_oracle,
        collateral_amount,
        anchor_price_x18,
        agc_unit,
    )?;
    require!(
        health_bps >= min_health_bps as u128,
        AgcError::InsufficientCreditHealth
    );
    Ok(())
}

pub fn validate_credit_draw(
    credit_line: &CreditLine,
    facility: &CreditFacility,
    collateral_asset: &CollateralAsset,
    collateral_oracle: &CollateralOracle,
    draw_amount: u64,
    underwriter_vault_assets_agc: u64,
    anchor_price_x18: u128,
    agc_unit: u128,
) -> Result<()> {
    require!(collateral_asset.enabled, AgcError::CollateralDisabled);
    let new_principal_debt = credit_line
        .principal_debt_agc
        .checked_add(draw_amount)
        .ok_or(AgcError::MathOverflow)?;
    let new_total_debt = new_principal_debt
        .checked_add(credit_line.accrued_interest_agc)
        .ok_or(AgcError::MathOverflow)?;
    require!(
        new_total_debt <= credit_line.credit_limit_agc
            && new_total_debt <= facility.config.max_line_debt_agc,
        AgcError::CreditLimitExceeded
    );
    let facility_principal_after = facility
        .total_principal_debt_agc
        .checked_add(draw_amount)
        .ok_or(AgcError::MathOverflow)?;
    require!(
        facility_principal_after <= facility.config.max_total_debt_agc,
        AgcError::CreditLimitExceeded
    );

    let required_underwriter_assets = mul_div(
        facility_principal_after as u128,
        facility.config.min_underwriter_reserve_bps as u128,
        BPS,
    )?;
    require!(
        underwriter_vault_assets_agc as u128 >= required_underwriter_assets,
        AgcError::InsufficientUnderwriterReserve
    );

    let collateral_value = collateral_value_quote_x18(
        credit_line.collateral_amount,
        facility.collateral_decimals,
        collateral_oracle.price_quote_x18,
    )?;
    let borrowable_value = mul_div(
        collateral_value,
        collateral_asset.collateral_factor_bps as u128,
        BPS,
    )?;
    let debt_value = agc_value_quote_x18(new_total_debt, anchor_price_x18, agc_unit)?;
    require!(
        debt_value <= borrowable_value,
        AgcError::InsufficientCollateral
    );

    let health_bps = credit_health_bps_for_debt(
        facility,
        collateral_oracle,
        credit_line.collateral_amount,
        new_total_debt,
        anchor_price_x18,
        agc_unit,
    )?;
    require!(
        health_bps >= facility.config.min_collateral_health_bps as u128,
        AgcError::InsufficientCreditHealth
    );
    Ok(())
}

pub fn accounted_underwriter_assets_agc(facility: &CreditFacility) -> Result<u64> {
    let inflows = checked_add_u128(
        facility.total_underwriter_deposits_agc,
        facility.total_interest_paid_agc,
        AgcError::MathOverflow,
    )?;
    let outflows = checked_add_u128(
        facility.total_underwriter_withdrawals_agc,
        facility.total_underwriter_loss_agc,
        AgcError::MathOverflow,
    )?;
    let assets = inflows.saturating_sub(outflows);
    u64::try_from(assets).map_err(|_| error!(AgcError::AmountTooLarge))
}

pub fn validate_underwriter_reserve(
    facility: &CreditFacility,
    underwriter_vault_assets_agc: u64,
) -> Result<()> {
    let required_underwriter_assets = mul_div(
        facility.total_principal_debt_agc as u128,
        facility.config.min_underwriter_reserve_bps as u128,
        BPS,
    )?;
    require!(
        underwriter_vault_assets_agc as u128 >= required_underwriter_assets,
        AgcError::InsufficientUnderwriterReserve
    );
    Ok(())
}

pub fn accrue_facility_line_interest(
    credit_line: &mut CreditLine,
    facility: &mut CreditFacility,
    now: u64,
) -> Result<()> {
    if credit_line.status != CreditLineStatus::Active {
        return Ok(());
    }
    if now <= credit_line.last_accrued_at {
        return Ok(());
    }
    if credit_line.principal_debt_agc == 0 {
        credit_line.last_accrued_at = now;
        return Ok(());
    }

    let elapsed = now - credit_line.last_accrued_at;
    let annual_interest = checked_mul_u128(
        credit_line.principal_debt_agc as u128,
        facility.config.interest_rate_bps as u128,
    )?;
    let elapsed_interest = checked_div_u128(
        checked_mul_u128(annual_interest, elapsed as u128)?,
        checked_mul_u128(BPS, SECONDS_PER_YEAR)?,
    )?;
    let interest_u64 =
        u64::try_from(elapsed_interest).map_err(|_| error!(AgcError::AmountTooLarge))?;

    if interest_u64 > 0 {
        credit_line.accrued_interest_agc = credit_line
            .accrued_interest_agc
            .checked_add(interest_u64)
            .ok_or(AgcError::MathOverflow)?;
        facility.total_interest_accrued_agc = checked_add_u128(
            facility.total_interest_accrued_agc,
            interest_u64 as u128,
            AgcError::MathOverflow,
        )?;
    }
    credit_line.last_accrued_at = now;
    Ok(())
}

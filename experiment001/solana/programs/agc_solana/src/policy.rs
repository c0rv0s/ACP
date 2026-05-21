use anchor_lang::prelude::*;

use crate::math::{
    checked_add_u128, checked_div_u128, checked_mul_u128, max_u128, min_u128, mul_div,
    positive_delta, safe_div,
};
use crate::{
    AgcError, EpochAccumulator, EpochResult, EpochSnapshot, ExternalMetrics, MintAllocation,
    MintDistribution, PolicyParams, PolicyState, ProtocolState, Regime, VaultFlows, BPS,
    SECONDS_PER_DAY,
};

pub fn quote_to_x18(state: &ProtocolState, raw_usdc: u64) -> Result<u128> {
    checked_mul_u128(raw_usdc as u128, state.quote_scale)
}

pub fn quote_from_x18(state: &ProtocolState, quote_x18: u128) -> Result<u64> {
    u64::try_from(quote_x18 / state.quote_scale).map_err(|_| error!(AgcError::AmountTooLarge))
}

pub fn circulating_float(total_supply: u64, treasury_agc: u64, xagc_assets: u64) -> u64 {
    total_supply.saturating_sub(treasury_agc.saturating_add(xagc_assets))
}

pub fn accounted_assets(total_assets: u64, unaccounted_assets: u64) -> u64 {
    total_assets.saturating_sub(unaccounted_assets)
}

pub fn convert_to_shares(
    assets: u64,
    share_supply: u64,
    total_assets: u64,
    unaccounted_assets: u64,
) -> Result<u64> {
    if share_supply == 0 {
        return Ok(assets);
    }

    let assets_before = accounted_assets(total_assets, unaccounted_assets);
    require!(assets_before > 0, AgcError::ZeroAmount);
    u64::try_from(mul_div(
        assets as u128,
        share_supply as u128,
        assets_before as u128,
    )?)
    .map_err(|_| error!(AgcError::AmountTooLarge))
}

pub fn convert_to_assets(
    shares: u64,
    share_supply: u64,
    total_assets: u64,
    unaccounted_assets: u64,
) -> Result<u64> {
    if share_supply == 0 {
        return Ok(shares);
    }

    let assets_before = accounted_assets(total_assets, unaccounted_assets);
    require!(assets_before > 0, AgcError::ZeroAmount);
    u64::try_from(mul_div(
        shares as u128,
        assets_before as u128,
        share_supply as u128,
    )?)
    .map_err(|_| error!(AgcError::AmountTooLarge))
}

pub fn observe_mid_price(
    state: &mut ProtocolState,
    current_mid_price_x18: u128,
    now: u64,
) -> Result<()> {
    let acc = &mut state.accumulator;
    if acc.last_observed_at == 0 {
        acc.updated_at = now;
        acc.last_observed_at = now;
        acc.last_mid_price_x18 = current_mid_price_x18;
        if acc.observation_count == 0 {
            acc.observation_count = 1;
        }
        return Ok(());
    }

    if now > acc.last_observed_at && acc.last_mid_price_x18 > 0 {
        let elapsed = now - acc.last_observed_at;
        acc.cumulative_mid_price_time_x18 = checked_add_u128(
            acc.cumulative_mid_price_time_x18,
            checked_mul_u128(acc.last_mid_price_x18, elapsed as u128)?,
            AgcError::MathOverflow,
        )?;
        let price_change_bps = if current_mid_price_x18 > acc.last_mid_price_x18 {
            checked_div_u128(
                checked_mul_u128(current_mid_price_x18 - acc.last_mid_price_x18, BPS)?,
                acc.last_mid_price_x18,
            )?
        } else {
            checked_div_u128(
                checked_mul_u128(acc.last_mid_price_x18 - current_mid_price_x18, BPS)?,
                acc.last_mid_price_x18,
            )?
        };
        acc.cumulative_abs_mid_price_change_bps = checked_add_u128(
            acc.cumulative_abs_mid_price_change_bps,
            price_change_bps,
            AgcError::MathOverflow,
        )?;
        acc.observation_count = acc
            .observation_count
            .checked_add(1)
            .ok_or(AgcError::MathOverflow)?;
        acc.last_observed_at = now;
    }

    acc.updated_at = now;
    acc.last_mid_price_x18 = current_mid_price_x18;
    Ok(())
}

pub fn volatility_bps(acc: &EpochAccumulator) -> u128 {
    if acc.observation_count <= 1 {
        return 0;
    }
    acc.cumulative_abs_mid_price_change_bps / (acc.observation_count - 1) as u128
}

pub fn preview_epoch_snapshot(acc: &EpochAccumulator, now: u64) -> Result<EpochSnapshot> {
    let mut cumulative_mid_price_time_x18 = acc.cumulative_mid_price_time_x18;
    if now > acc.last_observed_at && acc.last_mid_price_x18 > 0 {
        cumulative_mid_price_time_x18 = checked_add_u128(
            cumulative_mid_price_time_x18,
            checked_mul_u128(acc.last_mid_price_x18, (now - acc.last_observed_at) as u128)?,
            AgcError::MathOverflow,
        )?;
    }

    let epoch_elapsed = now.saturating_sub(acc.started_at);
    let short_twap_price_x18 = if epoch_elapsed == 0 {
        acc.last_mid_price_x18
    } else if cumulative_mid_price_time_x18 == 0 && acc.observation_count == 0 {
        acc.last_mid_price_x18
    } else {
        cumulative_mid_price_time_x18 / epoch_elapsed as u128
    };

    Ok(EpochSnapshot {
        epoch_id: acc.epoch_id,
        started_at: acc.started_at,
        ended_at: now,
        gross_buy_volume_quote_x18: acc.gross_buy_volume_quote_x18,
        gross_sell_volume_quote_x18: acc.gross_sell_volume_quote_x18,
        total_volume_quote_x18: acc.total_volume_quote_x18,
        short_twap_price_x18,
        realized_volatility_bps: volatility_bps(acc),
        total_hook_fees_quote_x18: acc.total_hook_fees_quote_x18,
        total_hook_fees_agc: acc.total_hook_fees_agc,
    })
}

pub fn compute_anchor_next(
    anchor_price_x18: u128,
    price_twap_x18: u128,
    anchor_ema_bps: u16,
    max_anchor_crawl_bps: u16,
) -> Result<u128> {
    let ema = checked_div_u128(
        checked_add_u128(
            checked_mul_u128(anchor_price_x18, BPS - anchor_ema_bps as u128)?,
            checked_mul_u128(price_twap_x18, anchor_ema_bps as u128)?,
            AgcError::MathOverflow,
        )?,
        BPS,
    )?;
    let min_anchor = checked_div_u128(
        checked_mul_u128(anchor_price_x18, BPS - max_anchor_crawl_bps as u128)?,
        BPS,
    )?;
    let max_anchor = checked_div_u128(
        checked_mul_u128(anchor_price_x18, BPS + max_anchor_crawl_bps as u128)?,
        BPS,
    )?;

    Ok(ema.clamp(min_anchor, max_anchor))
}

pub fn evaluate_epoch(
    snapshot: EpochSnapshot,
    external_metrics: ExternalMetrics,
    state: PolicyState,
    flows: VaultFlows,
    policy_params: PolicyParams,
    agc_unit: u128,
) -> Result<EpochResult> {
    let price_twap_x18 = snapshot.short_twap_price_x18;
    let gross_buy_quote_x18 = snapshot.gross_buy_volume_quote_x18;
    let gross_sell_quote_x18 = snapshot.gross_sell_volume_quote_x18;
    let total_volume_quote_x18 = snapshot.total_volume_quote_x18;
    let xagc_net_deposits_acp =
        flows.xagc_deposits_acp as i128 - flows.xagc_gross_redemptions_acp as i128;

    let credit_outstanding_quote_x18 =
        mul_div(state.float_supply_acp, state.anchor_price_x18, agc_unit)?;
    let gross_buy_floor_bps = safe_div(
        checked_mul_u128(gross_buy_quote_x18, BPS)?,
        credit_outstanding_quote_x18,
    )?;
    let net_buy_quote_x18 = gross_buy_quote_x18.saturating_sub(gross_sell_quote_x18);
    let net_buy_pressure_bps = safe_div(
        checked_mul_u128(net_buy_quote_x18, BPS)?,
        credit_outstanding_quote_x18,
    )?;
    let buy_growth_bps = if state.last_gross_buy_quote_x18 == 0 {
        0
    } else {
        safe_div(
            checked_mul_u128(
                positive_delta(gross_buy_quote_x18, state.last_gross_buy_quote_x18),
                BPS,
            )?,
            state.last_gross_buy_quote_x18,
        )?
    };
    let exit_pressure_bps = safe_div(
        checked_mul_u128(gross_sell_quote_x18, BPS)?,
        total_volume_quote_x18,
    )?;
    let stable_cash_reserve_quote_x18 = if external_metrics.stable_cash_reserve_quote_x18 > 0 {
        external_metrics.stable_cash_reserve_quote_x18
    } else {
        state.treasury_quote_x18
    };
    let risk_weighted_reserve_quote_x18 = if external_metrics.risk_weighted_reserve_quote_x18 > 0 {
        external_metrics.risk_weighted_reserve_quote_x18
    } else {
        stable_cash_reserve_quote_x18
    };
    let liquidity_depth_quote_x18 = if external_metrics.liquidity_depth_quote_x18 > 0 {
        external_metrics.liquidity_depth_quote_x18
    } else {
        external_metrics.depth_to_target_slippage_quote_x18
    };
    let reserve_coverage_bps = safe_div(
        checked_mul_u128(risk_weighted_reserve_quote_x18, BPS)?,
        credit_outstanding_quote_x18,
    )?;
    let stable_cash_coverage_bps = safe_div(
        checked_mul_u128(stable_cash_reserve_quote_x18, BPS)?,
        credit_outstanding_quote_x18,
    )?;
    let liquidity_depth_coverage_bps = safe_div(
        checked_mul_u128(liquidity_depth_quote_x18, BPS)?,
        credit_outstanding_quote_x18,
    )?;
    let locked_share_bps = safe_div(
        checked_mul_u128(state.xagc_total_assets_acp, BPS)?,
        state.float_supply_acp,
    )?;
    let lock_flow_bps = if xagc_net_deposits_acp <= 0 {
        0
    } else {
        safe_div(
            checked_mul_u128(xagc_net_deposits_acp as u128, BPS)?,
            state.float_supply_acp,
        )?
    };
    let premium_bps = if price_twap_x18 > state.anchor_price_x18 && state.anchor_price_x18 > 0 {
        checked_div_u128(
            checked_mul_u128(price_twap_x18 - state.anchor_price_x18, BPS)?,
            state.anchor_price_x18,
        )?
    } else {
        0
    };
    let premium_persistence_epochs = if premium_bps >= policy_params.min_premium_bps as u128 {
        state
            .premium_persistence_epochs
            .checked_add(1)
            .ok_or(AgcError::MathOverflow)?
    } else {
        0
    };

    let normal_floor_x18 = checked_div_u128(
        checked_mul_u128(
            state.anchor_price_x18,
            BPS - policy_params.normal_band_bps as u128,
        )?,
        BPS,
    )?;
    let stressed_floor_x18 = checked_div_u128(
        checked_mul_u128(
            state.anchor_price_x18,
            BPS - policy_params.stressed_band_bps as u128,
        )?,
        BPS,
    )?;
    let anchor_next_x18 = compute_anchor_next(
        state.anchor_price_x18,
        price_twap_x18,
        policy_params.anchor_ema_bps,
        policy_params.max_anchor_crawl_bps,
    )?;

    let oracle_health_blocked = external_metrics.oracle_confidence_bps
        > policy_params.max_oracle_confidence_bps
        || external_metrics.stale_oracle_count > policy_params.max_stale_oracle_count;
    let concentration_blocked = external_metrics.largest_collateral_concentration_bps
        > policy_params.max_reserve_concentration_bps;

    let in_defense = price_twap_x18 < stressed_floor_x18
        || reserve_coverage_bps < policy_params.defense_reserve_coverage_bps as u128
        || stable_cash_coverage_bps < policy_params.defense_stable_cash_coverage_bps as u128
        || oracle_health_blocked
        || snapshot.realized_volatility_bps >= policy_params.defense_volatility_bps as u128
        || exit_pressure_bps >= policy_params.defense_exit_pressure_bps as u128;

    let can_expand = premium_bps >= policy_params.min_premium_bps as u128
        && premium_persistence_epochs >= policy_params.premium_persistence_required as u128
        && gross_buy_floor_bps >= policy_params.min_gross_buy_floor_bps as u128
        && net_buy_pressure_bps > 0
        && lock_flow_bps > 0
        && locked_share_bps >= policy_params.min_locked_share_bps as u128
        && reserve_coverage_bps >= policy_params.expansion_reserve_coverage_bps as u128
        && stable_cash_coverage_bps >= policy_params.min_stable_cash_coverage_bps as u128
        && liquidity_depth_coverage_bps >= policy_params.min_liquidity_depth_coverage_bps as u128
        && !concentration_blocked
        && !oracle_health_blocked
        && snapshot.realized_volatility_bps <= policy_params.max_expansion_volatility_bps as u128
        && exit_pressure_bps <= policy_params.max_expansion_exit_pressure_bps as u128
        && buy_growth_bps > 0;

    let in_recovery = !in_defense
        && state.recovery_cooldown_epochs_remaining > 0
        && (state.last_regime == Regime::Defense || state.last_regime == Regime::Recovery);

    let next_regime = if in_defense {
        Regime::Defense
    } else if in_recovery {
        Regime::Recovery
    } else if can_expand {
        Regime::Expansion
    } else {
        Regime::Neutral
    };

    let mut demand_score_bps = 0;
    let mut health_score_bps = 0;
    let mut mint_rate_bps = 0;
    let mut mint_budget_acp = 0;

    if next_regime == Regime::Expansion {
        let premium_score_bps = min_u128(
            safe_div(
                checked_mul_u128(
                    positive_delta(premium_bps, policy_params.min_premium_bps as u128),
                    BPS,
                )?,
                policy_params.min_premium_bps as u128,
            )?,
            BPS,
        );
        let buy_score_bps = min_u128(
            safe_div(
                checked_mul_u128(gross_buy_floor_bps, BPS)?,
                policy_params.target_gross_buy_bps as u128,
            )?,
            BPS,
        );
        let net_buy_score_bps = min_u128(
            safe_div(
                checked_mul_u128(net_buy_pressure_bps, BPS)?,
                policy_params.target_net_buy_bps as u128,
            )?,
            BPS,
        );
        let lock_flow_score_bps = min_u128(
            safe_div(
                checked_mul_u128(lock_flow_bps, BPS)?,
                policy_params.target_lock_flow_bps as u128,
            )?,
            BPS,
        );
        let buy_growth_score_bps = min_u128(
            safe_div(
                checked_mul_u128(buy_growth_bps, BPS)?,
                policy_params.target_buy_growth_bps as u128,
            )?,
            BPS,
        );

        demand_score_bps = min_u128(
            premium_score_bps,
            min_u128(
                buy_score_bps,
                min_u128(
                    net_buy_score_bps,
                    min_u128(lock_flow_score_bps, buy_growth_score_bps),
                ),
            ),
        );

        let reserve_health_bps = if reserve_coverage_bps
            <= policy_params.expansion_reserve_coverage_bps as u128
        {
            0
        } else {
            min_u128(
                safe_div(
                    checked_mul_u128(
                        reserve_coverage_bps - policy_params.expansion_reserve_coverage_bps as u128,
                        BPS,
                    )?,
                    (policy_params.target_reserve_coverage_bps
                        - policy_params.expansion_reserve_coverage_bps) as u128,
                )?,
                BPS,
            )
        };
        let stable_cash_health_bps = if stable_cash_coverage_bps
            <= policy_params.min_stable_cash_coverage_bps as u128
        {
            0
        } else {
            min_u128(
                safe_div(
                    checked_mul_u128(
                        stable_cash_coverage_bps
                            - policy_params.min_stable_cash_coverage_bps as u128,
                        BPS,
                    )?,
                    (policy_params.target_stable_cash_coverage_bps
                        - policy_params.min_stable_cash_coverage_bps) as u128,
                )?,
                BPS,
            )
        };
        let liquidity_health_bps = if liquidity_depth_coverage_bps
            <= policy_params.min_liquidity_depth_coverage_bps as u128
        {
            0
        } else {
            min_u128(
                safe_div(
                    checked_mul_u128(
                        liquidity_depth_coverage_bps
                            - policy_params.min_liquidity_depth_coverage_bps as u128,
                        BPS,
                    )?,
                    (policy_params.target_liquidity_depth_coverage_bps
                        - policy_params.min_liquidity_depth_coverage_bps)
                        as u128,
                )?,
                BPS,
            )
        };
        let volatility_health_bps = if snapshot.realized_volatility_bps
            >= policy_params.max_expansion_volatility_bps as u128
        {
            0
        } else {
            checked_div_u128(
                checked_mul_u128(
                    policy_params.max_expansion_volatility_bps as u128
                        - snapshot.realized_volatility_bps,
                    BPS,
                )?,
                policy_params.max_expansion_volatility_bps as u128,
            )?
        };
        let exit_health_bps =
            if exit_pressure_bps >= policy_params.max_expansion_exit_pressure_bps as u128 {
                0
            } else {
                checked_div_u128(
                    checked_mul_u128(
                        policy_params.max_expansion_exit_pressure_bps as u128 - exit_pressure_bps,
                        BPS,
                    )?,
                    policy_params.max_expansion_exit_pressure_bps as u128,
                )?
            };
        let locked_share_health_bps = min_u128(
            safe_div(
                checked_mul_u128(locked_share_bps, BPS)?,
                policy_params.target_locked_share_bps as u128,
            )?,
            BPS,
        );

        health_score_bps = min_u128(
            reserve_health_bps,
            min_u128(
                stable_cash_health_bps,
                min_u128(
                    liquidity_health_bps,
                    min_u128(
                        volatility_health_bps,
                        min_u128(exit_health_bps, locked_share_health_bps),
                    ),
                ),
            ),
        );

        let raw_mint_rate_bps = checked_div_u128(
            checked_mul_u128(
                checked_div_u128(
                    checked_mul_u128(policy_params.expansion_kappa_bps as u128, demand_score_bps)?,
                    BPS,
                )?,
                health_score_bps,
            )?,
            BPS,
        )?;
        mint_rate_bps = min_u128(
            raw_mint_rate_bps,
            policy_params.max_mint_per_epoch_bps as u128,
        );

        let remaining_daily_mint_acp = positive_delta(
            checked_div_u128(
                checked_mul_u128(
                    state.float_supply_acp,
                    policy_params.max_mint_per_day_bps as u128,
                )?,
                BPS,
            )?,
            state.minted_today_acp,
        );
        mint_budget_acp = min_u128(
            checked_div_u128(
                checked_mul_u128(state.float_supply_acp, mint_rate_bps)?,
                BPS,
            )?,
            remaining_daily_mint_acp,
        );
    }

    let price_stress_bps = if price_twap_x18 < stressed_floor_x18 && state.anchor_price_x18 > 0 {
        checked_div_u128(
            checked_mul_u128(stressed_floor_x18 - price_twap_x18, BPS)?,
            state.anchor_price_x18,
        )?
    } else {
        0
    };
    let coverage_stress_bps = positive_delta(
        policy_params.defense_reserve_coverage_bps as u128,
        reserve_coverage_bps,
    );
    let stable_cash_stress_bps = positive_delta(
        policy_params.defense_stable_cash_coverage_bps as u128,
        stable_cash_coverage_bps,
    );
    let concentration_stress_bps = positive_delta(
        external_metrics.largest_collateral_concentration_bps as u128,
        policy_params.max_reserve_concentration_bps as u128,
    );
    let oracle_stress_bps = positive_delta(
        external_metrics.oracle_confidence_bps as u128,
        policy_params.max_oracle_confidence_bps as u128,
    );
    let exit_stress_bps = positive_delta(
        exit_pressure_bps,
        policy_params.defense_exit_pressure_bps as u128,
    );
    let volatility_stress_bps = positive_delta(
        snapshot.realized_volatility_bps,
        policy_params.defense_volatility_bps as u128,
    );
    let mut stress_score_bps = max_u128(
        price_stress_bps,
        max_u128(
            coverage_stress_bps,
            max_u128(
                stable_cash_stress_bps,
                max_u128(
                    concentration_stress_bps,
                    max_u128(
                        oracle_stress_bps,
                        max_u128(exit_stress_bps, volatility_stress_bps),
                    ),
                ),
            ),
        ),
    );
    if reserve_coverage_bps < policy_params.hard_defense_reserve_coverage_bps as u128 {
        stress_score_bps = max_u128(
            stress_score_bps,
            policy_params.severe_stress_threshold_bps as u128,
        );
    }

    let mut buyback_budget_quote_x18 = 0;
    if next_regime == Regime::Defense {
        let buyback_cap_bps =
            if stress_score_bps >= policy_params.severe_stress_threshold_bps as u128 {
                policy_params.severe_defense_spend_bps
            } else {
                policy_params.mild_defense_spend_bps
            };
        let buyback_spend_rate_bps = min_u128(
            checked_div_u128(
                checked_mul_u128(policy_params.buyback_kappa_bps as u128, stress_score_bps)?,
                BPS,
            )?,
            buyback_cap_bps as u128,
        );
        buyback_budget_quote_x18 = checked_div_u128(
            checked_mul_u128(state.treasury_quote_x18, buyback_spend_rate_bps)?,
            BPS,
        )?;
    }

    Ok(EpochResult {
        epoch_id: snapshot.epoch_id,
        regime: next_regime,
        anchor_price_x18: state.anchor_price_x18,
        anchor_next_x18,
        normal_floor_x18,
        stressed_floor_x18,
        price_twap_x18,
        premium_bps,
        premium_persistence_epochs,
        credit_outstanding_quote_x18,
        gross_buy_floor_bps,
        net_buy_pressure_bps,
        buy_growth_bps,
        exit_pressure_bps,
        reserve_coverage_bps,
        stable_cash_coverage_bps,
        liquidity_depth_coverage_bps,
        locked_share_bps,
        lock_flow_bps,
        demand_score_bps,
        health_score_bps,
        mint_rate_bps,
        mint_budget_acp,
        buyback_budget_quote_x18,
        stress_score_bps,
        gross_buy_quote_x18,
        gross_sell_quote_x18,
        total_volume_quote_x18,
        depth_to_target_slippage_quote_x18: liquidity_depth_quote_x18,
        stable_cash_reserve_quote_x18,
        risk_weighted_reserve_quote_x18,
        liquidity_depth_quote_x18,
        largest_collateral_concentration_bps: external_metrics.largest_collateral_concentration_bps,
        oracle_confidence_bps: external_metrics.oracle_confidence_bps,
        stale_oracle_count: external_metrics.stale_oracle_count,
        realized_volatility_bps: snapshot.realized_volatility_bps,
        xagc_deposits_acp: flows.xagc_deposits_acp,
        xagc_gross_redemptions_acp: flows.xagc_gross_redemptions_acp,
        treasury_quote_x18: state.treasury_quote_x18,
        treasury_acp: state.treasury_acp,
        xagc_total_assets_acp: state.xagc_total_assets_acp,
        mint_allocations: MintAllocation::default(),
    })
}

pub fn allocate_mint(mint_budget_acp: u128, distribution: MintDistribution) -> MintAllocation {
    let xagc_mint_acp = mint_budget_acp * distribution.xagc_bps as u128 / BPS;
    let growth_programs_mint_acp = mint_budget_acp * distribution.growth_programs_bps as u128 / BPS;
    let lp_mint_acp = mint_budget_acp * distribution.lp_bps as u128 / BPS;
    let integrators_mint_acp = mint_budget_acp * distribution.integrators_bps as u128 / BPS;
    let treasury_mint_acp = mint_budget_acp
        .saturating_sub(xagc_mint_acp)
        .saturating_sub(growth_programs_mint_acp)
        .saturating_sub(lp_mint_acp)
        .saturating_sub(integrators_mint_acp);

    MintAllocation {
        xagc_mint_acp,
        growth_programs_mint_acp,
        lp_mint_acp,
        integrators_mint_acp,
        treasury_mint_acp,
    }
}

pub fn validate_settlement_window(state: &ProtocolState, now: u64) -> Result<()> {
    let next_allowed = state
        .accumulator
        .started_at
        .checked_add(state.policy_params.policy_epoch_duration)
        .ok_or(AgcError::MathOverflow)?;
    require!(now >= next_allowed, AgcError::EpochTooSoon);
    Ok(())
}

pub fn refresh_mint_window(state: &mut ProtocolState, now: u64) {
    let current_day = now / SECONDS_PER_DAY;
    if state.mint_window_day != current_day {
        state.mint_window_day = current_day;
        state.minted_in_current_day = 0;
    }
}

pub fn persist_epoch_settlement(
    state: &mut ProtocolState,
    snapshot: EpochSnapshot,
    result: EpochResult,
    raw_buyback_budget: u64,
    now: u64,
) -> Result<()> {
    let next_epoch_id = snapshot
        .epoch_id
        .checked_add(1)
        .ok_or(AgcError::MathOverflow)?;
    state.anchor_price_x18 = result.anchor_next_x18;
    state.premium_persistence_epochs = result.premium_persistence_epochs;
    state.last_gross_buy_quote_x18 = result.gross_buy_quote_x18;
    state.regime = result.regime;

    state.recovery_cooldown_epochs_remaining = match result.regime {
        Regime::Defense => state.policy_params.recovery_cooldown_epochs as u64,
        Regime::Recovery => state.recovery_cooldown_epochs_remaining.saturating_sub(1),
        _ => 0,
    };

    if raw_buyback_budget > 0 {
        state.pending_treasury_buyback_usdc = state
            .pending_treasury_buyback_usdc
            .checked_add(raw_buyback_budget)
            .ok_or(AgcError::MathOverflow)?;
    }

    state.last_settled_epoch = snapshot.epoch_id;
    state.last_settlement_timestamp = now;
    state.last_coverage_bps = result.reserve_coverage_bps;
    state.last_exit_pressure_bps = result.exit_pressure_bps;
    state.last_volatility_bps = result.realized_volatility_bps;
    state.last_premium_bps = result.premium_bps;
    state.last_locked_share_bps = result.locked_share_bps;
    state.last_lock_flow_bps = result.lock_flow_bps;
    state.last_stable_cash_coverage_bps = result.stable_cash_coverage_bps;
    state.last_liquidity_depth_coverage_bps = result.liquidity_depth_coverage_bps;
    state.last_reserve_concentration_bps = result.largest_collateral_concentration_bps as u128;
    state.last_oracle_confidence_bps = result.oracle_confidence_bps as u128;
    state.last_stale_oracle_count = result.stale_oracle_count;
    state.last_xagc_deposit_total = state.xagc_gross_deposits_total;
    state.last_xagc_redemption_total = state.xagc_gross_redemptions_total;
    state.last_epoch_result = result;

    let current_mid_price_x18 = state.accumulator.last_mid_price_x18;
    state.accumulator = EpochAccumulator {
        epoch_id: next_epoch_id,
        started_at: now,
        updated_at: now,
        last_observed_at: now,
        observation_count: if current_mid_price_x18 > 0 { 1 } else { 0 },
        gross_buy_volume_quote_x18: 0,
        gross_sell_volume_quote_x18: 0,
        total_volume_quote_x18: 0,
        last_mid_price_x18: current_mid_price_x18,
        cumulative_mid_price_time_x18: 0,
        cumulative_abs_mid_price_change_bps: 0,
        total_hook_fees_quote_x18: 0,
        total_hook_fees_agc: 0,
    };

    Ok(())
}

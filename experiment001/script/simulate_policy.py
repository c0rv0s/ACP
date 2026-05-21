#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any


BPS = 10_000
WAD = 10**18


def coerce_numbers(value: Any) -> Any:
    if isinstance(value, dict):
        return {key: coerce_numbers(item) for key, item in value.items()}
    if isinstance(value, list):
        return [coerce_numbers(item) for item in value]
    if isinstance(value, str):
        stripped = value.strip()
        if stripped.startswith("-"):
            digits = stripped[1:]
        else:
            digits = stripped
        if digits.isdigit():
            return int(stripped)
    return value


def load_json(path: Path) -> dict[str, Any]:
    return coerce_numbers(json.loads(path.read_text()))


def clamp(value: int, low: int, high: int) -> int:
    return min(max(value, low), high)


def pct_from_bps(value: int) -> str:
    return f"{value / 100:.2f}%"


def format_wad(value: int) -> str:
    return f"{value / WAD:,.2f}"


def format_price(value: int) -> str:
    return f"{value / WAD:,.4f}"


def format_ratio_x(numerator: int, denominator: int) -> str:
    if denominator <= 0:
        return "n/a"
    return f"{numerator / denominator:.2f}x"


def safe_div(numerator: int, denominator: int) -> int:
    if denominator == 0:
        return 0
    return numerator // denominator


def compute_anchor_next(anchor_price_x18: int, price_twap_x18: int, anchor_ema_bps: int, max_anchor_crawl_bps: int) -> int:
    ema = (anchor_price_x18 * (BPS - anchor_ema_bps) + price_twap_x18 * anchor_ema_bps) // BPS
    anchor_min = anchor_price_x18 * (BPS - max_anchor_crawl_bps) // BPS
    anchor_max = anchor_price_x18 * (BPS + max_anchor_crawl_bps) // BPS
    return clamp(ema, anchor_min, anchor_max)


def allocate_mint(mint_budget_agc: int, distribution_bps: dict[str, int], growth_programs_enabled: bool) -> dict[str, int]:
    xagc_mint = mint_budget_agc * distribution_bps["xagc"] // BPS
    growth_programs = mint_budget_agc * distribution_bps["growthPrograms"] // BPS
    lp = mint_budget_agc * distribution_bps["lp"] // BPS
    integrators = mint_budget_agc * distribution_bps["integrators"] // BPS
    treasury = mint_budget_agc - xagc_mint - growth_programs - lp - integrators

    if not growth_programs_enabled:
        treasury += growth_programs
        growth_programs = 0

    return {
        "xagcMintAgc": xagc_mint,
        "growthProgramsMintAgc": growth_programs,
        "lpMintAgc": lp,
        "integratorsMintAgc": integrators,
        "treasuryMintAgc": treasury,
    }


def validate_model(model: dict[str, Any]) -> None:
    total_distribution = sum(model["distributionBps"].values())
    if total_distribution != BPS:
        raise ValueError(f"distributionBps must sum to {BPS}, got {total_distribution}")
    required_sections = (
        "reserveCoverageBps",
        "stableCashCoverageBps",
        "liquidityDepthCoverageBps",
        "concentrationBps",
        "oracle",
    )
    for section in required_sections:
        if section not in model:
            raise ValueError(f"model missing required section: {section}")


def simulate_epoch(
    model: dict[str, Any],
    state: dict[str, Any],
    epoch: dict[str, Any],
    growth_programs_enabled: bool,
) -> dict[str, Any]:
    expansion = model["expansion"]
    defense = model["defense"]
    xagc_cfg = model["xagc"]
    anchor_params = model["anchor"]
    bands = model["bandsBps"]
    reserve_coverage = model["reserveCoverageBps"]
    stable_cash_coverage_cfg = model["stableCashCoverageBps"]
    liquidity_depth_coverage_cfg = model["liquidityDepthCoverageBps"]
    concentration_cfg = model["concentrationBps"]
    oracle_cfg = model["oracle"]
    distribution_bps = model["distributionBps"]

    price_twap_x18 = epoch["priceTwapX18"]
    gross_buy_quote_x18 = epoch["grossBuyQuoteX18"]
    gross_sell_quote_x18 = epoch["grossSellQuoteX18"]
    total_volume_quote_x18 = epoch["totalVolumeQuoteX18"]
    depth_to_target_slippage_quote_x18 = epoch["depthToTargetSlippageQuoteX18"]
    realized_volatility_bps = epoch["realizedVolatilityBps"]
    xagc_deposits_agc = epoch.get("xagcDepositsAgc", 0)
    xagc_gross_redemptions_agc = epoch.get("xagcGrossRedemptionsAgc", 0)
    treasury_quote_inflow_x18 = epoch.get("treasuryQuoteInflowX18", 0)

    anchor_price_x18 = state["anchorPriceX18"]
    float_supply_agc = state["floatSupplyAgc"]
    treasury_quote_x18 = state["treasuryQuoteX18"]
    treasury_agc = state["treasuryAgc"]
    xagc_total_assets_agc = state["xagcTotalAssetsAgc"]

    # External-metric defaulting mirrors the on-chain evaluate_epoch:
    # if an explicit external feed is not provided, fall back to derived values.
    stable_cash_reserve_quote_x18 = epoch.get("stableCashReserveQuoteX18") or treasury_quote_x18
    risk_weighted_reserve_quote_x18 = (
        epoch.get("riskWeightedReserveQuoteX18") or stable_cash_reserve_quote_x18
    )
    liquidity_depth_quote_x18 = (
        epoch.get("liquidityDepthQuoteX18") or depth_to_target_slippage_quote_x18
    )
    largest_collateral_concentration_bps = epoch.get("largestCollateralConcentrationBps", 0)
    oracle_confidence_bps = epoch.get("oracleConfidenceBps", 0)
    stale_oracle_count = epoch.get("staleOracleCount", 0)

    xagc_exit_fee_agc = xagc_gross_redemptions_agc * xagc_cfg["exitFeeBps"] // BPS
    xagc_net_redemption_agc = xagc_gross_redemptions_agc - xagc_exit_fee_agc
    xagc_net_deposits_agc = xagc_deposits_agc - xagc_gross_redemptions_agc

    if xagc_gross_redemptions_agc > xagc_total_assets_agc + xagc_deposits_agc:
        raise ValueError(
            f"gross xAGC redemptions exceed vault assets in epoch '{epoch.get('label', 'unknown')}'"
        )
    if xagc_deposits_agc > float_supply_agc:
        raise ValueError(
            f"xAGC deposits exceed float supply in epoch '{epoch.get('label', 'unknown')}'"
        )

    credit_outstanding_quote_x18 = float_supply_agc * anchor_price_x18 // WAD
    gross_buy_floor_bps = safe_div(gross_buy_quote_x18 * BPS, credit_outstanding_quote_x18)
    net_buy_quote_x18 = max(gross_buy_quote_x18 - gross_sell_quote_x18, 0)
    net_buy_pressure_bps = safe_div(net_buy_quote_x18 * BPS, credit_outstanding_quote_x18)
    buy_growth_bps = 0
    if state["lastGrossBuyQuoteX18"] > 0:
        buy_growth_bps = safe_div(
            max(gross_buy_quote_x18 - state["lastGrossBuyQuoteX18"], 0) * BPS,
            state["lastGrossBuyQuoteX18"],
        )
    exit_pressure_bps = safe_div(gross_sell_quote_x18 * BPS, total_volume_quote_x18)
    reserve_coverage_bps = safe_div(
        risk_weighted_reserve_quote_x18 * BPS, credit_outstanding_quote_x18
    )
    stable_cash_coverage_bps = safe_div(
        stable_cash_reserve_quote_x18 * BPS, credit_outstanding_quote_x18
    )
    liquidity_depth_coverage_bps = safe_div(
        liquidity_depth_quote_x18 * BPS, credit_outstanding_quote_x18
    )
    locked_share_bps = safe_div(xagc_total_assets_agc * BPS, float_supply_agc)
    lock_flow_bps = safe_div(max(xagc_net_deposits_agc, 0) * BPS, float_supply_agc)
    premium_bps = 0
    if price_twap_x18 > anchor_price_x18 and anchor_price_x18 > 0:
        premium_bps = (price_twap_x18 - anchor_price_x18) * BPS // anchor_price_x18

    premium_persistence_epochs = (
        state["premiumPersistenceEpochs"] + 1
        if premium_bps >= expansion["minPremiumBps"]
        else 0
    )

    normal_floor_x18 = anchor_price_x18 * (BPS - bands["normal"]) // BPS
    stressed_floor_x18 = anchor_price_x18 * (BPS - bands["stressed"]) // BPS
    anchor_next_x18 = compute_anchor_next(
        anchor_price_x18,
        price_twap_x18,
        anchor_params["anchorEmaBps"],
        anchor_params["maxAnchorCrawlBps"],
    )

    oracle_health_blocked = (
        oracle_confidence_bps > oracle_cfg["maxConfidenceBps"]
        or stale_oracle_count > oracle_cfg["maxStaleCount"]
    )
    concentration_blocked = (
        largest_collateral_concentration_bps > concentration_cfg["max"]
    )

    in_defense = (
        price_twap_x18 < stressed_floor_x18
        or reserve_coverage_bps < reserve_coverage["defense"]
        or stable_cash_coverage_bps < stable_cash_coverage_cfg["defense"]
        or oracle_health_blocked
        or realized_volatility_bps >= defense["defenseVolatilityBps"]
        or exit_pressure_bps >= defense["defenseExitPressureBps"]
    )

    can_expand = (
        premium_bps >= expansion["minPremiumBps"]
        and premium_persistence_epochs >= expansion["premiumPersistenceRequired"]
        and gross_buy_floor_bps >= expansion["minGrossBuyFloorBps"]
        and net_buy_pressure_bps > 0
        and lock_flow_bps > 0
        and locked_share_bps >= expansion["minLockedShareBps"]
        and reserve_coverage_bps >= reserve_coverage["expansionMin"]
        and stable_cash_coverage_bps >= stable_cash_coverage_cfg["expansionMin"]
        and liquidity_depth_coverage_bps >= liquidity_depth_coverage_cfg["expansionMin"]
        and not concentration_blocked
        and not oracle_health_blocked
        and realized_volatility_bps <= expansion["maxVolatilityBps"]
        and exit_pressure_bps <= expansion["maxExitPressureBps"]
        and buy_growth_bps > 0
    )

    in_recovery = (
        not in_defense
        and state["recoveryCooldownEpochsRemaining"] > 0
        and state["lastRegime"] in {"Defense", "Recovery"}
    )

    if in_defense:
        regime = "Defense"
    elif in_recovery:
        regime = "Recovery"
    elif can_expand:
        regime = "Expansion"
    else:
        regime = "Neutral"

    demand_score_bps = 0
    health_score_bps = 0
    mint_budget_agc = 0
    mint_rate_bps = 0
    mint_allocations = {
        "xagcMintAgc": 0,
        "growthProgramsMintAgc": 0,
        "lpMintAgc": 0,
        "integratorsMintAgc": 0,
        "treasuryMintAgc": 0,
    }

    if regime == "Expansion":
        premium_score_bps = min(
            safe_div(max(premium_bps - expansion["minPremiumBps"], 0) * BPS, expansion["minPremiumBps"]),
            BPS,
        )
        buy_score_bps = min(
            safe_div(gross_buy_floor_bps * BPS, expansion["targetGrossBuyBps"]),
            BPS,
        )
        net_buy_score_bps = min(
            safe_div(net_buy_pressure_bps * BPS, expansion["targetNetBuyBps"]),
            BPS,
        )
        lock_flow_score_bps = min(
            safe_div(lock_flow_bps * BPS, expansion["targetLockFlowBps"]),
            BPS,
        )
        buy_growth_score_bps = min(
            safe_div(max(buy_growth_bps, 0) * BPS, expansion["targetBuyGrowthBps"]),
            BPS,
        )
        demand_score_bps = min(
            premium_score_bps,
            buy_score_bps,
            net_buy_score_bps,
            lock_flow_score_bps,
            buy_growth_score_bps,
        )

        reserve_health_bps = 0
        if reserve_coverage_bps > reserve_coverage["expansionMin"]:
            reserve_health_bps = min(
                safe_div(
                    (reserve_coverage_bps - reserve_coverage["expansionMin"]) * BPS,
                    reserve_coverage["target"] - reserve_coverage["expansionMin"],
                ),
                BPS,
            )
        stable_cash_health_bps = 0
        if stable_cash_coverage_bps > stable_cash_coverage_cfg["expansionMin"]:
            stable_cash_health_bps = min(
                safe_div(
                    (stable_cash_coverage_bps - stable_cash_coverage_cfg["expansionMin"]) * BPS,
                    stable_cash_coverage_cfg["target"] - stable_cash_coverage_cfg["expansionMin"],
                ),
                BPS,
            )
        liquidity_depth_health_bps = 0
        if liquidity_depth_coverage_bps > liquidity_depth_coverage_cfg["expansionMin"]:
            liquidity_depth_health_bps = min(
                safe_div(
                    (liquidity_depth_coverage_bps - liquidity_depth_coverage_cfg["expansionMin"]) * BPS,
                    liquidity_depth_coverage_cfg["target"] - liquidity_depth_coverage_cfg["expansionMin"],
                ),
                BPS,
            )
        volatility_health_bps = 0
        if realized_volatility_bps < expansion["maxVolatilityBps"]:
            volatility_health_bps = (
                (expansion["maxVolatilityBps"] - realized_volatility_bps) * BPS
                // expansion["maxVolatilityBps"]
            )
        exit_health_bps = 0
        if exit_pressure_bps < expansion["maxExitPressureBps"]:
            exit_health_bps = (
                (expansion["maxExitPressureBps"] - exit_pressure_bps) * BPS
                // expansion["maxExitPressureBps"]
            )
        locked_share_health_bps = min(
            safe_div(locked_share_bps * BPS, expansion["targetLockedShareBps"]),
            BPS,
        )
        health_score_bps = min(
            reserve_health_bps,
            stable_cash_health_bps,
            liquidity_depth_health_bps,
            volatility_health_bps,
            exit_health_bps,
            locked_share_health_bps,
        )

        raw_mint_rate_bps = (
            expansion["expansionKappaBps"] * demand_score_bps // BPS * health_score_bps // BPS
        )
        mint_rate_bps = min(raw_mint_rate_bps, expansion["maxMintPerEpochBps"])
        remaining_daily_mint_agc = max(
            float_supply_agc * expansion["maxMintPerDayBps"] // BPS - state["mintedTodayAgc"],
            0,
        )
        mint_budget_agc = min(
            float_supply_agc * mint_rate_bps // BPS,
            remaining_daily_mint_agc,
        )
        mint_allocations = allocate_mint(
            mint_budget_agc, distribution_bps, growth_programs_enabled
        )

    price_stress_bps = 0
    if price_twap_x18 < stressed_floor_x18 and anchor_price_x18 > 0:
        price_stress_bps = (stressed_floor_x18 - price_twap_x18) * BPS // anchor_price_x18
    coverage_stress_bps = max(reserve_coverage["defense"] - reserve_coverage_bps, 0)
    stable_cash_stress_bps = max(
        stable_cash_coverage_cfg["defense"] - stable_cash_coverage_bps, 0
    )
    concentration_stress_bps = max(
        largest_collateral_concentration_bps - concentration_cfg["max"], 0
    )
    oracle_stress_bps = max(
        oracle_confidence_bps - oracle_cfg["maxConfidenceBps"], 0
    )
    exit_stress_bps = max(exit_pressure_bps - defense["defenseExitPressureBps"], 0)
    vol_stress_bps = max(realized_volatility_bps - defense["defenseVolatilityBps"], 0)
    stress_score_bps = max(
        price_stress_bps,
        coverage_stress_bps,
        stable_cash_stress_bps,
        concentration_stress_bps,
        oracle_stress_bps,
        exit_stress_bps,
        vol_stress_bps,
    )
    if reserve_coverage_bps < reserve_coverage["hardDefense"]:
        stress_score_bps = max(stress_score_bps, defense["severeStressThresholdBps"])

    buyback_budget_quote_x18 = 0
    buyback_burn_agc = 0
    if regime == "Defense":
        buyback_cap_bps = (
            defense["severeDefenseSpendBps"]
            if stress_score_bps >= defense["severeStressThresholdBps"]
            else defense["mildDefenseSpendBps"]
        )
        buyback_spend_rate_bps = min(
            defense["buybackKappaBps"] * stress_score_bps // BPS,
            buyback_cap_bps,
        )
        buyback_budget_quote_x18 = treasury_quote_x18 * buyback_spend_rate_bps // BPS
        if price_twap_x18 > 0:
            buyback_burn_agc = buyback_budget_quote_x18 * WAD // price_twap_x18

    treasury_quote_next_x18 = treasury_quote_x18 + treasury_quote_inflow_x18 - buyback_budget_quote_x18
    treasury_agc_next = (
        treasury_agc
        + mint_allocations["treasuryMintAgc"]
        + xagc_exit_fee_agc
    )
    xagc_total_assets_next_agc = (
        xagc_total_assets_agc
        + xagc_deposits_agc
        - xagc_gross_redemptions_agc
        + mint_allocations["xagcMintAgc"]
    )
    float_supply_next_agc = (
        float_supply_agc
        - xagc_deposits_agc
        + xagc_net_redemption_agc
        + mint_allocations["growthProgramsMintAgc"]
        + mint_allocations["lpMintAgc"]
        + mint_allocations["integratorsMintAgc"]
        - buyback_burn_agc
    )

    if treasury_quote_next_x18 < 0 or xagc_total_assets_next_agc < 0 or float_supply_next_agc < 0:
        raise ValueError(f"epoch '{epoch.get('label', 'unknown')}' produced negative state")

    if regime == "Defense":
        recovery_cooldown_next = defense["recoveryCooldownEpochs"]
    elif regime == "Recovery":
        recovery_cooldown_next = max(state["recoveryCooldownEpochsRemaining"] - 1, 0)
    else:
        recovery_cooldown_next = 0

    next_state = {
        **state,
        "anchorPriceX18": anchor_next_x18,
        "premiumPersistenceEpochs": premium_persistence_epochs,
        "lastGrossBuyQuoteX18": gross_buy_quote_x18,
        "mintedTodayAgc": state["mintedTodayAgc"] + mint_budget_agc,
        "lastRegime": regime,
        "recoveryCooldownEpochsRemaining": recovery_cooldown_next,
        "floatSupplyAgc": float_supply_next_agc,
        "treasuryQuoteX18": treasury_quote_next_x18,
        "treasuryAgc": treasury_agc_next,
        "xagcTotalAssetsAgc": xagc_total_assets_next_agc,
    }

    return {
        "label": epoch.get("label"),
        "regime": regime,
        "anchorPriceX18": anchor_price_x18,
        "anchorNextX18": anchor_next_x18,
        "normalFloorX18": normal_floor_x18,
        "stressedFloorX18": stressed_floor_x18,
        "priceTwapX18": price_twap_x18,
        "premiumBps": premium_bps,
        "premiumPersistenceEpochs": premium_persistence_epochs,
        "creditOutstandingQuoteX18": credit_outstanding_quote_x18,
        "grossBuyFloorBps": gross_buy_floor_bps,
        "netBuyPressureBps": net_buy_pressure_bps,
        "buyGrowthBps": buy_growth_bps,
        "exitPressureBps": exit_pressure_bps,
        "reserveCoverageBps": reserve_coverage_bps,
        "stableCashCoverageBps": stable_cash_coverage_bps,
        "liquidityDepthCoverageBps": liquidity_depth_coverage_bps,
        "largestCollateralConcentrationBps": largest_collateral_concentration_bps,
        "oracleConfidenceBps": oracle_confidence_bps,
        "staleOracleCount": stale_oracle_count,
        "lockedShareBps": locked_share_bps,
        "lockFlowBps": lock_flow_bps,
        "demandScoreBps": demand_score_bps,
        "healthScoreBps": health_score_bps,
        "mintRateBps": mint_rate_bps,
        "mintBudgetAgc": mint_budget_agc,
        "buybackBudgetQuoteX18": buyback_budget_quote_x18,
        "buybackBurnAgc": buyback_burn_agc,
        "stressScoreBps": stress_score_bps,
        "grossBuyQuoteX18": gross_buy_quote_x18,
        "grossSellQuoteX18": gross_sell_quote_x18,
        "totalVolumeQuoteX18": total_volume_quote_x18,
        "depthToTargetSlippageQuoteX18": depth_to_target_slippage_quote_x18,
        "stableCashReserveQuoteX18": stable_cash_reserve_quote_x18,
        "riskWeightedReserveQuoteX18": risk_weighted_reserve_quote_x18,
        "liquidityDepthQuoteX18": liquidity_depth_quote_x18,
        "realizedVolatilityBps": realized_volatility_bps,
        "xagcDepositsAgc": xagc_deposits_agc,
        "xagcGrossRedemptionsAgc": xagc_gross_redemptions_agc,
        "xagcExitFeeAgc": xagc_exit_fee_agc,
        "treasuryQuoteInflowX18": treasury_quote_inflow_x18,
        "mintAllocations": mint_allocations,
        "stateBefore": {
            "floatSupplyAgc": float_supply_agc,
            "treasuryQuoteX18": treasury_quote_x18,
            "treasuryAgc": treasury_agc,
            "xagcTotalAssetsAgc": xagc_total_assets_agc,
        },
        "stateAfter": {
            "floatSupplyAgc": float_supply_next_agc,
            "treasuryQuoteX18": treasury_quote_next_x18,
            "treasuryAgc": treasury_agc_next,
            "xagcTotalAssetsAgc": xagc_total_assets_next_agc,
        },
        "nextState": next_state,
    }


def simulate_scenario(model: dict[str, Any], scenario: dict[str, Any]) -> dict[str, Any]:
    state = {**model["initialState"], **scenario.get("initialState", {})}
    epochs_per_day = max(1, 86_400 // model["meta"]["epochDurationSeconds"])
    growth_programs_enabled = scenario.get("growthProgramsEnabled", True)
    results: list[dict[str, Any]] = []

    for index, epoch in enumerate(scenario["epochs"]):
        if index > 0 and index % epochs_per_day == 0:
            state["mintedTodayAgc"] = 0
        result = simulate_epoch(model, state, epoch, growth_programs_enabled)
        results.append(result)
        state = result["nextState"]

    return {
        "name": scenario["name"],
        "description": scenario.get("description"),
        "growthProgramsEnabled": growth_programs_enabled,
        "epochs": results,
        "finalState": state,
    }


def render_text_report(simulation: dict[str, Any]) -> str:
    lines = [f"\n[{simulation['name']}]"]
    if simulation.get("description"):
        lines.append(simulation["description"])
    header = (
        "epoch  regime     price   anchor  prem   r_cov  sc_cov ld_cov conc   oconf  buys     sells    lock    mint     burn     xagc     t_quote"
    )
    lines.append(header)
    lines.append("-" * len(header))

    for epoch in simulation["epochs"]:
        lines.append(
            f"{(epoch.get('label') or '-'):>4}  "
            f"{epoch['regime']:<9} "
            f"{format_price(epoch['priceTwapX18']):>6} "
            f"{format_price(epoch['anchorPriceX18']):>6} "
            f"{pct_from_bps(epoch['premiumBps']):>6} "
            f"{pct_from_bps(epoch['reserveCoverageBps']):>6} "
            f"{pct_from_bps(epoch['stableCashCoverageBps']):>6} "
            f"{pct_from_bps(epoch['liquidityDepthCoverageBps']):>6} "
            f"{pct_from_bps(epoch['largestCollateralConcentrationBps']):>6} "
            f"{pct_from_bps(epoch['oracleConfidenceBps']):>6} "
            f"{format_wad(epoch['grossBuyQuoteX18']):>8} "
            f"{format_wad(epoch['grossSellQuoteX18']):>8} "
            f"{pct_from_bps(epoch['lockFlowBps']):>7} "
            f"{format_wad(epoch['mintBudgetAgc']):>8} "
            f"{format_wad(epoch['buybackBurnAgc']):>8} "
            f"{format_wad(epoch['stateAfter']['xagcTotalAssetsAgc']):>8} "
            f"{format_wad(epoch['stateAfter']['treasuryQuoteX18']):>8}"
        )

    final_state = simulation["finalState"]
    lines.append("")
    lines.append(
        "final: "
        f"anchor={format_price(final_state['anchorPriceX18'])} "
        f"float={format_wad(final_state['floatSupplyAgc'])} "
        f"xagc_assets={format_wad(final_state['xagcTotalAssetsAgc'])} "
        f"treasury_quote={format_wad(final_state['treasuryQuoteX18'])} "
        f"treasury_agc={format_wad(final_state['treasuryAgc'])}"
    )
    return "\n".join(lines)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Simulate AGC policy scenarios from JSON models.")
    root = Path(__file__).resolve().parent.parent
    parser.add_argument(
        "--model",
        type=Path,
        default=root / "configs" / "policy" / "launch-model.json",
        help="Path to the launch policy model JSON.",
    )
    parser.add_argument(
        "--scenarios",
        type=Path,
        default=root / "configs" / "policy" / "scenarios.json",
        help="Path to the scenarios JSON.",
    )
    parser.add_argument(
        "--scenario",
        action="append",
        default=[],
        help="Limit output to one or more named scenarios.",
    )
    parser.add_argument(
        "--format",
        choices=("text", "json"),
        default="text",
        help="Output format.",
    )
    argv = sys.argv[1:]
    if argv and argv[0] == "--":
        argv = argv[1:]
    return parser.parse_args(argv)


def main() -> int:
    args = parse_args()
    model = load_json(args.model)
    validate_model(model)
    scenario_file = load_json(args.scenarios)

    scenarios = scenario_file["scenarios"]
    selected = set(args.scenario)
    if selected:
        scenarios = [scenario for scenario in scenarios if scenario["name"] in selected]
        if not scenarios:
            raise SystemExit("No scenarios matched --scenario selection")

    simulations = [simulate_scenario(model, scenario) for scenario in scenarios]

    if args.format == "json":
        json.dump(simulations, sys.stdout, indent=2)
        sys.stdout.write("\n")
        return 0

    report = "\n".join(render_text_report(simulation) for simulation in simulations)
    print(report)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

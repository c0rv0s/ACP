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


def allocate_mint(mint_budget_acp: int, distribution_bps: dict[str, int], growth_programs_enabled: bool) -> dict[str, int]:
    xacp = mint_budget_acp * distribution_bps["xacp"] // BPS
    growth_programs = mint_budget_acp * distribution_bps["growthPrograms"] // BPS
    lp = mint_budget_acp * distribution_bps["lp"] // BPS
    integrators = mint_budget_acp * distribution_bps["integrators"] // BPS
    treasury = mint_budget_acp - xacp - growth_programs - lp - integrators

    if not growth_programs_enabled:
        treasury += growth_programs
        growth_programs = 0

    return {
        "xacpMintAcp": xacp,
        "growthProgramsMintAcp": growth_programs,
        "lpMintAcp": lp,
        "integratorsMintAcp": integrators,
        "treasuryMintAcp": treasury,
    }


def validate_model(model: dict[str, Any]) -> None:
    total_distribution = sum(model["distributionBps"].values())
    if total_distribution != BPS:
        raise ValueError(f"distributionBps must sum to {BPS}, got {total_distribution}")


def simulate_epoch(
    model: dict[str, Any],
    state: dict[str, Any],
    epoch: dict[str, Any],
    growth_programs_enabled: bool,
) -> dict[str, Any]:
    expansion = model["expansion"]
    defense = model["defense"]
    xacp = model["xacp"]
    anchor_params = model["anchor"]
    bands = model["bandsBps"]
    reserve_coverage = model["reserveCoverageBps"]
    distribution_bps = model["distributionBps"]

    price_twap_x18 = epoch["priceTwapX18"]
    gross_buy_quote_x18 = epoch["grossBuyQuoteX18"]
    gross_sell_quote_x18 = epoch["grossSellQuoteX18"]
    total_volume_quote_x18 = epoch["totalVolumeQuoteX18"]
    depth_to_target_slippage_quote_x18 = epoch["depthToTargetSlippageQuoteX18"]
    realized_volatility_bps = epoch["realizedVolatilityBps"]
    xacp_deposits_acp = epoch.get("xacpDepositsAcp", 0)
    xacp_gross_redemptions_acp = epoch.get("xacpGrossRedemptionsAcp", 0)
    treasury_quote_inflow_x18 = epoch.get("treasuryQuoteInflowX18", 0)

    anchor_price_x18 = state["anchorPriceX18"]
    float_supply_acp = state["floatSupplyAcp"]
    treasury_quote_x18 = state["treasuryQuoteX18"]
    treasury_acp = state["treasuryAcp"]
    xacp_total_assets_acp = state["xacpTotalAssetsAcp"]

    xacp_exit_fee_acp = xacp_gross_redemptions_acp * xacp["exitFeeBps"] // BPS
    xacp_net_redemption_acp = xacp_gross_redemptions_acp - xacp_exit_fee_acp
    xacp_net_deposits_acp = xacp_deposits_acp - xacp_gross_redemptions_acp

    if xacp_gross_redemptions_acp > xacp_total_assets_acp + xacp_deposits_acp:
        raise ValueError(
            f"gross xACP redemptions exceed vault assets in epoch '{epoch.get('label', 'unknown')}'"
        )
    if xacp_deposits_acp > float_supply_acp:
        raise ValueError(
            f"xACP deposits exceed float supply in epoch '{epoch.get('label', 'unknown')}'"
        )

    credit_outstanding_quote_x18 = float_supply_acp * anchor_price_x18 // WAD
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
        depth_to_target_slippage_quote_x18 * BPS, credit_outstanding_quote_x18
    )
    locked_share_bps = safe_div(xacp_total_assets_acp * BPS, float_supply_acp)
    lock_flow_bps = safe_div(max(xacp_net_deposits_acp, 0) * BPS, float_supply_acp)
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

    in_defense = (
        price_twap_x18 < stressed_floor_x18
        or reserve_coverage_bps < reserve_coverage["defense"]
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
    mint_budget_acp = 0
    mint_rate_bps = 0
    mint_allocations = {
        "xacpMintAcp": 0,
        "growthProgramsMintAcp": 0,
        "lpMintAcp": 0,
        "integratorsMintAcp": 0,
        "treasuryMintAcp": 0,
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
            volatility_health_bps,
            exit_health_bps,
            locked_share_health_bps,
        )

        raw_mint_rate_bps = (
            expansion["expansionKappaBps"] * demand_score_bps // BPS * health_score_bps // BPS
        )
        mint_rate_bps = min(raw_mint_rate_bps, expansion["maxMintPerEpochBps"])
        remaining_daily_mint_acp = max(
            float_supply_acp * expansion["maxMintPerDayBps"] // BPS - state["mintedTodayAcp"],
            0,
        )
        mint_budget_acp = min(
            float_supply_acp * mint_rate_bps // BPS,
            remaining_daily_mint_acp,
        )
        mint_allocations = allocate_mint(
            mint_budget_acp, distribution_bps, growth_programs_enabled
        )

    price_stress_bps = 0
    if price_twap_x18 < stressed_floor_x18 and anchor_price_x18 > 0:
        price_stress_bps = (stressed_floor_x18 - price_twap_x18) * BPS // anchor_price_x18
    coverage_stress_bps = max(reserve_coverage["defense"] - reserve_coverage_bps, 0)
    exit_stress_bps = max(exit_pressure_bps - defense["defenseExitPressureBps"], 0)
    vol_stress_bps = max(realized_volatility_bps - defense["defenseVolatilityBps"], 0)
    stress_score_bps = max(price_stress_bps, coverage_stress_bps, exit_stress_bps, vol_stress_bps)
    if reserve_coverage_bps < reserve_coverage["hardDefense"]:
        stress_score_bps = max(stress_score_bps, defense["severeStressThresholdBps"])

    buyback_budget_quote_x18 = 0
    buyback_burn_acp = 0
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
            buyback_burn_acp = buyback_budget_quote_x18 * WAD // price_twap_x18

    treasury_quote_next_x18 = treasury_quote_x18 + treasury_quote_inflow_x18 - buyback_budget_quote_x18
    treasury_acp_next = (
        treasury_acp
        + mint_allocations["treasuryMintAcp"]
        + xacp_exit_fee_acp
    )
    xacp_total_assets_next_acp = (
        xacp_total_assets_acp
        + xacp_deposits_acp
        - xacp_gross_redemptions_acp
        + mint_allocations["xacpMintAcp"]
    )
    float_supply_next_acp = (
        float_supply_acp
        - xacp_deposits_acp
        + xacp_net_redemption_acp
        + mint_allocations["growthProgramsMintAcp"]
        + mint_allocations["lpMintAcp"]
        + mint_allocations["integratorsMintAcp"]
        - buyback_burn_acp
    )

    if treasury_quote_next_x18 < 0 or xacp_total_assets_next_acp < 0 or float_supply_next_acp < 0:
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
        "mintedTodayAcp": state["mintedTodayAcp"] + mint_budget_acp,
        "lastRegime": regime,
        "recoveryCooldownEpochsRemaining": recovery_cooldown_next,
        "floatSupplyAcp": float_supply_next_acp,
        "treasuryQuoteX18": treasury_quote_next_x18,
        "treasuryAcp": treasury_acp_next,
        "xacpTotalAssetsAcp": xacp_total_assets_next_acp,
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
        "lockedShareBps": locked_share_bps,
        "lockFlowBps": lock_flow_bps,
        "demandScoreBps": demand_score_bps,
        "healthScoreBps": health_score_bps,
        "mintRateBps": mint_rate_bps,
        "mintBudgetAcp": mint_budget_acp,
        "buybackBudgetQuoteX18": buyback_budget_quote_x18,
        "buybackBurnAcp": buyback_burn_acp,
        "stressScoreBps": stress_score_bps,
        "grossBuyQuoteX18": gross_buy_quote_x18,
        "grossSellQuoteX18": gross_sell_quote_x18,
        "totalVolumeQuoteX18": total_volume_quote_x18,
        "depthToTargetSlippageQuoteX18": depth_to_target_slippage_quote_x18,
        "realizedVolatilityBps": realized_volatility_bps,
        "xacpDepositsAcp": xacp_deposits_acp,
        "xacpGrossRedemptionsAcp": xacp_gross_redemptions_acp,
        "xacpExitFeeAcp": xacp_exit_fee_acp,
        "treasuryQuoteInflowX18": treasury_quote_inflow_x18,
        "mintAllocations": mint_allocations,
        "stateBefore": {
            "floatSupplyAcp": float_supply_acp,
            "treasuryQuoteX18": treasury_quote_x18,
            "treasuryAcp": treasury_acp,
            "xacpTotalAssetsAcp": xacp_total_assets_acp,
        },
        "stateAfter": {
            "floatSupplyAcp": float_supply_next_acp,
            "treasuryQuoteX18": treasury_quote_next_x18,
            "treasuryAcp": treasury_acp_next,
            "xacpTotalAssetsAcp": xacp_total_assets_next_acp,
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
            state["mintedTodayAcp"] = 0
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
        "epoch  regime     price   anchor  prem   cov    buys     sells    lock    mint     burn     xacp     t_quote  eff"
    )
    lines.append(header)
    lines.append("-" * len(header))

    for epoch in simulation["epochs"]:
        credit_outstanding = epoch["creditOutstandingQuoteX18"]
        depth = epoch["depthToTargetSlippageQuoteX18"]
        lines.append(
            f"{(epoch.get('label') or '-'):>4}  "
            f"{epoch['regime']:<9} "
            f"{format_price(epoch['priceTwapX18']):>6} "
            f"{format_price(epoch['anchorPriceX18']):>6} "
            f"{pct_from_bps(epoch['premiumBps']):>6} "
            f"{pct_from_bps(epoch['reserveCoverageBps']):>6} "
            f"{format_wad(epoch['grossBuyQuoteX18']):>8} "
            f"{format_wad(epoch['grossSellQuoteX18']):>8} "
            f"{pct_from_bps(epoch['lockFlowBps']):>7} "
            f"{format_wad(epoch['mintBudgetAcp']):>8} "
            f"{format_wad(epoch['buybackBurnAcp']):>8} "
            f"{format_wad(epoch['stateAfter']['xacpTotalAssetsAcp']):>8} "
            f"{format_wad(epoch['stateAfter']['treasuryQuoteX18']):>8} "
            f"{format_ratio_x(credit_outstanding, depth):>5}"
        )

    final_state = simulation["finalState"]
    lines.append("")
    lines.append(
        "final: "
        f"anchor={format_price(final_state['anchorPriceX18'])} "
        f"float={format_wad(final_state['floatSupplyAcp'])} "
        f"xacp_assets={format_wad(final_state['xacpTotalAssetsAcp'])} "
        f"treasury_quote={format_wad(final_state['treasuryQuoteX18'])} "
        f"treasury_acp={format_wad(final_state['treasuryAcp'])}"
    )
    return "\n".join(lines)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Simulate ACP policy scenarios.")
    root = Path(__file__).resolve().parent.parent
    parser.add_argument(
        "--model",
        type=Path,
        default=root / "configs" / "policy" / "acp-launch-model.json",
        help="Path to the ACP launch model JSON.",
    )
    parser.add_argument(
        "--scenarios",
        type=Path,
        default=root / "configs" / "policy" / "acp-scenarios.json",
        help="Path to the ACP scenario JSON.",
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

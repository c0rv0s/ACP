# AGC Current System Context

Last updated from repository state on 2026-05-17.

This document is a handoff brief for taking Agent Credit Protocol from hackathon prototype to a rigorous whitepaper and accelerator-ready company narrative. It describes what exists in this repository now: the product concept, protocol mechanics, on-chain surfaces, formulas, configuration, simulations, demo scripts, and known gaps.

## 1. One-Paragraph Summary

Agent Credit Protocol (AGC) is a Solana-native credit machine for autonomous markets. `AGC` is intended to be liquid working capital for agents, apps, borrowers, and users. `xAGC` is a non-rebasing vault share that owns the long-duration expansion layer. The system does not target a hard dollar peg. Instead, it uses an epoch policy controller to keep AGC in a stable operating range, expand supply only when demand and balance-sheet conditions justify it, and defend the system with halted expansion, treasury buybacks, burns, pause controls, and credit risk limits when conditions weaken.

Core loop:

```text
AGC demand rises
-> reserves and liquidity deepen
-> safe credit capacity increases
-> borrowers and agents use credit
-> fees, repayments, and reserve strength grow
-> xAGC becomes more valuable
-> confidence and AGC demand increase
```

The intended framing is "reserve-efficient agent credit" or "private credit for autonomous markets," not "another stablecoin."

## 2. Repository Map

- `solana/programs/agc_solana/`: Anchor program implementing the protocol.
- `script/simulate_policy.py`: deterministic Python simulator for policy epochs.
- `configs/policy/launch-model.json`: launch parameter model used by the simulator.
- `configs/policy/scenarios.json`: scenario pack used by the simulator.
- `script/devnet-bootstrap.ts`: one-shot devnet bootstrap for mints, protocol state, BTC collateral, oracle, and a BTC credit facility.
- `script/demo/*.ts`: devnet demo scripts for underwrite, borrow, expansion settlement, reset, and smoke test.
- `web/`: Vite/React frontend with live Solana dashboard, wallet actions, hosted docs, and AI-readable docs.
- `docs/`: design, economics, policy, deployment, risk, user-example, buyback-adapter, and migration docs.

Primary source of truth for coded mechanics:

- `solana/programs/agc_solana/src/lib.rs`
- `solana/programs/agc_solana/src/policy.rs`
- `solana/programs/agc_solana/src/credit.rs`
- `solana/programs/agc_solana/src/state.rs`
- `solana/programs/agc_solana/src/validation.rs`

## 3. What Problem AGC Is Trying To Solve

The conceptual bet is that autonomous software agents, automated markets, and onchain applications need a native credit inventory layer. Existing onchain assets tend to be one of:

- fully reserved stablecoins that settle payments but do not expand credit,
- volatile collateral assets,
- governance/speculation tokens,
- lending markets that re-lend existing liquidity.

AGC tries to be different: a floating credit asset whose supply can expand against reserves, collateral, useful credit demand, and protocol revenue. The system aims to maximize safe credit outstanding, not peg fidelity.

High-level objective:

```text
CreditOutstanding = circulating AGC * anchor price
ReserveBase = USDC exit liquidity at target slippage
ReserveEfficiency = CreditOutstanding / ReserveBase
```

The protocol wants `ReserveEfficiency` to be as high as possible while preserving:

- price stability around a soft anchor,
- orderly exits,
- acceptable volatility,
- sufficient stable cash and risk-weighted reserve coverage,
- confidence that normal withdrawals can be absorbed.

This is closer to a fractional-reserve private-credit system than to a fully backed stablecoin.

## 4. Core Assets

### AGC

`AGC` is the liquid credit inventory token.

Intended uses:

- working capital for agents,
- borrower draw asset in credit facilities,
- liquidity and settlement inventory,
- input token for xAGC deposits,
- unit whose float is managed by epoch policy.

It is explicitly not:

- a hard 1:1 dollar claim,
- a fully backed stablecoin,
- a rebasing receipt.

### xAGC

`xAGC` is the savings/upside layer. It is a normal SPL token representing shares in a program-owned AGC vault.

Mechanics:

- users deposit AGC into the xAGC vault,
- the program mints xAGC shares,
- policy-directed expansion can mint AGC directly into the xAGC vault,
- xAGC share count does not rebase,
- the AGC-per-xAGC exchange rate rises when the vault receives expansion mints.

Redemptions:

- user burns xAGC shares,
- vault calculates gross AGC claim,
- protocol charges an exit fee in AGC,
- net AGC returns to the user,
- fee AGC goes to treasury.

Current coded launch config uses `exitFeeBps = 100`, or 1%. Some economic prose in older docs recommends 3%; the current simulator and code use the configured value.

### Stable Reserves

`USDC` and `USDT` are defensive cash reserves, not full backing. The current devnet bootstrap uses a mock USDC mint. Mainnet production would need live reserve accounts and issuer/concentration risk controls.

### BTC Wrappers

BTC wrappers are strategic reserve/collateral candidates. They receive haircuts because they are volatile and have wrapper/custody/liquidity risks. The devnet bootstrap creates a mock BTC mint and configures it as BTC collateral with a manual oracle at $100,000/BTC.

### RWAs / Tokenized Stocks

RWAs are future isolated collateral candidates. The design says they should start disabled globally and only graduate after issuer, legal, oracle, market-hours, redemption, and liquidation mechanics are proven.

## 5. Deployed Devnet Prototype

The repo includes a devnet deployment.

- Program ID: `H1n8VTp6pMY5WFfVfi4MNkQ9q5szkMpVWcHQ21JRETXC`
- Protocol state: `2y2au7Fo1MEaXZzP8TknDrDfX2uezojkqaCxUBj6fMQS`
- AGC mint: `BgEmfYvG48d93QHw5aBrRszdDobXTVPTdagg9fXwaP9D`
- xAGC mint: `8N3f2iQVzUxh4k8CB2ZDzBy2HUn4wy1YMKVck63Hbbrz`
- Mock USDC mint: `BCAw89QFbg1Zv7ZquAeKaCsF4t2V6FVCTQvDeK6Aawz9`
- Mock BTC mint: `DgqjKgCh3SnPhCEvEzpeLAviQLBk2VWsPSQEbekY87G2`
- BTC credit facility: `ASJXw3NmVdPpkHVdrf6NXfRpChGDUeaHskEapNuHbmmq`

Bootstrap facts:

- 10,000,000 AGC are pre-minted to the deployer wallet for demos.
- AGC mint authority is transferred to the program PDA.
- xAGC mint authority is the same program PDA from inception.
- Protocol initializes treasury AGC, treasury USDC, and xAGC vault token accounts.
- BTC collateral is configured with manual oracle source.
- The BTC facility has a 1,000,000 AGC max total debt cap and 100,000 AGC max line debt cap.
- Devnet upgrade authority remains the deployer wallet; production migration to multisig is pending.

Important: devnet bootstrap uses an initial anchor of `1.0e18`, while the simulator launch model uses `0.5e18`. The difference is a demo/staging artifact, not an economic requirement.

## 6. Governance and Authorities

The on-chain program separates authority lanes:

- `admin`: high-level protocol initialization, keeper setup, authority changes, adapter/program configuration.
- `risk_admin`: policy parameters, collateral settings, mint distribution, settlement recipients, exit fee, facility configuration.
- `emergency_admin`: pause flags and emergency response.
- `keeper` accounts: scoped operational permissions.
- Solana upgrade authority: external program upgrade control, intended to sit behind a high-threshold multisig in production.

Keeper permissions are scoped:

- market reporting,
- oracle reporting,
- epoch settlement,
- buyback execution,
- treasury burn,
- credit operation.

Admin transfer is two-step:

```text
transfer_admin(next_admin)
-> pending_admin set
-> accept_admin() by pending admin
-> admin changes and roles still held by old admin migrate
```

Pause flags can independently halt:

- xAGC deposits and redemptions,
- market reporting,
- settlement,
- credit issuance,
- collateral updates,
- buybacks,
- treasury burns,
- credit facility updates,
- credit line updates,
- credit draws,
- credit repayments,
- underwriter deposits,
- underwriter withdrawals,
- liquidations.

## 7. Units

Simulator conventions:

- AGC amounts use `1e18`,
- quote notionals use normalized `1e18`,
- prices use X18,
- percentages use basis points (`BPS = 10,000`).

On-chain current SPL token conventions:

- AGC mint has 9 decimals in devnet bootstrap,
- xAGC mint has 9 decimals,
- mock USDC has 6 decimals,
- mock BTC has 8 decimals.

On-chain quote normalization:

```text
quote_scale = 10^(18 - usdc_decimals)
quote_x18 = raw_usdc * quote_scale
raw_usdc = quote_x18 / quote_scale
```

## 8. xAGC Mechanics

The vault uses share accounting. The coded helper uses `xagc_unaccounted_assets` to ignore assets that were already in the vault before the first xAGC share mint.

Definitions:

```text
accounted_assets = total_assets - unaccounted_assets
```

Deposit share conversion:

```text
if share_supply == 0:
    shares = assets
else:
    shares = assets * share_supply / accounted_assets_before
```

Redemption asset conversion:

```text
if share_supply == 0:
    gross_assets = shares
else:
    gross_assets = shares * accounted_assets_before / share_supply

fee_assets = gross_assets * exit_fee_bps / BPS
net_assets = gross_assets - fee_assets
```

Deposit flow:

```text
depositor AGC -> xAGC vault
xAGC shares minted to receiver
xagc_gross_deposits_total += assets
```

Redemption flow:

```text
owner xAGC shares burned
fee_assets AGC transferred from vault to treasury_agc
net_assets AGC transferred from vault to receiver
xagc_gross_redemptions_total += gross_assets
```

Policy settlement computes current epoch vault flows by subtracting last-settlement watermarks:

```text
xagc_deposits_this_epoch = xagc_gross_deposits_total - last_xagc_deposit_total
xagc_redemptions_this_epoch = xagc_gross_redemptions_total - last_xagc_redemption_total
```

## 9. Policy Inputs

The policy engine is epoch-based. Market actions record telemetry; the controller settles later.

Epoch snapshot inputs from market accumulator:

- `gross_buy_volume_quote_x18`
- `gross_sell_volume_quote_x18`
- `total_volume_quote_x18`
- `short_twap_price_x18`
- `realized_volatility_bps`
- hook fees in quote and AGC, currently telemetry.

External metrics passed to settlement today:

- `depth_to_target_slippage_quote_x18`
- `stable_cash_reserve_quote_x18`
- `risk_weighted_reserve_quote_x18`
- `liquidity_depth_quote_x18`
- `largest_collateral_concentration_bps`
- `oracle_confidence_bps`
- `stale_oracle_count`

Current important shortcut: reserve and liquidity metrics are passed into `settle_epoch` as external metrics. Production still needs on-chain aggregation from configured reserve accounts and controlled market/oracle sources.

## 10. Derived Metrics

Let:

```text
BPS = 10,000
WAD = 1e18
```

Credit outstanding:

```text
credit_outstanding_quote_x18 = float_supply_agc * anchor_price_x18 / WAD
```

On-chain `float_supply` is computed from real token balances:

```text
float_supply = agc_mint.supply - treasury_agc.amount - xagc_vault_agc.amount
```

Demand metrics:

```text
gross_buy_floor_bps = gross_buy_quote_x18 * BPS / credit_outstanding_quote_x18
net_buy_quote_x18 = max(gross_buy_quote_x18 - gross_sell_quote_x18, 0)
net_buy_pressure_bps = net_buy_quote_x18 * BPS / credit_outstanding_quote_x18

buy_growth_bps =
    if last_gross_buy_quote_x18 == 0:
        0
    else:
        max(gross_buy_quote_x18 - last_gross_buy_quote_x18, 0) * BPS / last_gross_buy_quote_x18

exit_pressure_bps =
    if total_volume_quote_x18 == 0:
        0
    else:
        gross_sell_quote_x18 * BPS / total_volume_quote_x18
```

Coverage metrics:

```text
reserve_coverage_bps = risk_weighted_reserve_quote_x18 * BPS / credit_outstanding_quote_x18
stable_cash_coverage_bps = stable_cash_reserve_quote_x18 * BPS / credit_outstanding_quote_x18
liquidity_depth_coverage_bps = liquidity_depth_quote_x18 * BPS / credit_outstanding_quote_x18
```

Lock metrics:

```text
locked_share_bps = xagc_total_assets_agc * BPS / float_supply_agc
xagc_net_deposits = xagc_deposits - xagc_gross_redemptions
lock_flow_bps = max(xagc_net_deposits, 0) * BPS / float_supply_agc
```

Premium:

```text
premium_bps =
    if price_twap_x18 > anchor_price_x18:
        (price_twap_x18 - anchor_price_x18) * BPS / anchor_price_x18
    else:
        0
```

## 11. Anchor, Bands, and Regimes

Anchor update:

```text
ema = (anchor * (BPS - anchor_ema_bps) + price_twap * anchor_ema_bps) / BPS
anchor_min = anchor * (BPS - max_anchor_crawl_bps) / BPS
anchor_max = anchor * (BPS + max_anchor_crawl_bps) / BPS
anchor_next = clamp(ema, anchor_min, anchor_max)
```

Band floors:

```text
normal_floor = anchor * (BPS - normal_band_bps) / BPS
stressed_floor = anchor * (BPS - stressed_band_bps) / BPS
```

Current launch-model bands:

- normal band: 300 bps, or +/-3%,
- stressed band: 700 bps, or +/-7%,
- anchor EMA: 500 bps,
- max anchor crawl: 100 bps per epoch.

### Defense

Enter Defense if any are true:

```text
price_twap < stressed_floor
reserve_coverage_bps < defense_reserve_coverage_bps
stable_cash_coverage_bps < defense_stable_cash_coverage_bps
oracle_confidence_bps > max_oracle_confidence_bps
stale_oracle_count > max_stale_oracle_count
realized_volatility_bps >= defense_volatility_bps
exit_pressure_bps >= defense_exit_pressure_bps
```

### Recovery

Enter Recovery if:

```text
not Defense
and recovery_cooldown_epochs_remaining > 0
and last_regime in {Defense, Recovery}
```

### Expansion

Enter Expansion only if all are true:

```text
premium_bps >= min_premium_bps
premium_persistence_epochs >= premium_persistence_required
gross_buy_floor_bps >= min_gross_buy_floor_bps
net_buy_pressure_bps > 0
lock_flow_bps > 0
locked_share_bps >= min_locked_share_bps
reserve_coverage_bps >= expansion_reserve_coverage_bps
stable_cash_coverage_bps >= min_stable_cash_coverage_bps
liquidity_depth_coverage_bps >= min_liquidity_depth_coverage_bps
largest_collateral_concentration_bps <= max_reserve_concentration_bps
oracle_confidence_bps <= max_oracle_confidence_bps
stale_oracle_count <= max_stale_oracle_count
realized_volatility_bps <= max_expansion_volatility_bps
exit_pressure_bps <= max_expansion_exit_pressure_bps
buy_growth_bps > 0
```

Everything else is Neutral.

## 12. Expansion Scoring and Mint Formula

Demand component scores:

```text
premium_score_bps = min(max(premium_bps - min_premium_bps, 0) * BPS / min_premium_bps, BPS)
buy_score_bps = min(gross_buy_floor_bps * BPS / target_gross_buy_bps, BPS)
net_buy_score_bps = min(net_buy_pressure_bps * BPS / target_net_buy_bps, BPS)
lock_flow_score_bps = min(lock_flow_bps * BPS / target_lock_flow_bps, BPS)
buy_growth_score_bps = min(max(buy_growth_bps, 0) * BPS / target_buy_growth_bps, BPS)

demand_score_bps = min(
    premium_score_bps,
    buy_score_bps,
    net_buy_score_bps,
    lock_flow_score_bps,
    buy_growth_score_bps
)
```

Health component scores:

```text
reserve_health_bps =
    if reserve_coverage_bps <= expansion_reserve_coverage_bps:
        0
    else:
        min(
            (reserve_coverage_bps - expansion_reserve_coverage_bps) * BPS
            / (target_reserve_coverage_bps - expansion_reserve_coverage_bps),
            BPS
        )

stable_cash_health_bps =
    if stable_cash_coverage_bps <= min_stable_cash_coverage_bps:
        0
    else:
        min(
            (stable_cash_coverage_bps - min_stable_cash_coverage_bps) * BPS
            / (target_stable_cash_coverage_bps - min_stable_cash_coverage_bps),
            BPS
        )

liquidity_depth_health_bps =
    if liquidity_depth_coverage_bps <= min_liquidity_depth_coverage_bps:
        0
    else:
        min(
            (liquidity_depth_coverage_bps - min_liquidity_depth_coverage_bps) * BPS
            / (target_liquidity_depth_coverage_bps - min_liquidity_depth_coverage_bps),
            BPS
        )

volatility_health_bps =
    if realized_volatility_bps >= max_expansion_volatility_bps:
        0
    else:
        (max_expansion_volatility_bps - realized_volatility_bps) * BPS / max_expansion_volatility_bps

exit_health_bps =
    if exit_pressure_bps >= max_expansion_exit_pressure_bps:
        0
    else:
        (max_expansion_exit_pressure_bps - exit_pressure_bps) * BPS / max_expansion_exit_pressure_bps

locked_share_health_bps = min(locked_share_bps * BPS / target_locked_share_bps, BPS)

health_score_bps = min(
    reserve_health_bps,
    stable_cash_health_bps,
    liquidity_depth_health_bps,
    volatility_health_bps,
    exit_health_bps,
    locked_share_health_bps
)
```

Mint rate and budget:

```text
raw_mint_rate_bps = expansion_kappa_bps * demand_score_bps / BPS * health_score_bps / BPS
mint_rate_bps = min(raw_mint_rate_bps, max_mint_per_epoch_bps)

remaining_daily_mint = max(float_supply * max_mint_per_day_bps / BPS - minted_today, 0)
mint_budget = min(float_supply * mint_rate_bps / BPS, remaining_daily_mint)
```

Mint budget is zero unless regime is Expansion. On-chain settlement also zeros the policy mint if `credit_issuance_paused` is true.

Current launch-model caps:

- `maxMintPerEpochBps = 100` or 1% of float,
- `maxMintPerDayBps = 250` or 2.5% of float,
- `expansionKappaBps = 1000` or 10%.

## 13. Mint Distribution

Current coded and simulated launch distribution:

```text
xAGC:           3000 bps = 30%
growthPrograms: 2000 bps = 20%
LP:             2000 bps = 20%
integrators:    1000 bps = 10%
treasury:       2000 bps = 20%
```

Allocation formula:

```text
xagc_mint = mint_budget * xagc_bps / BPS
growth_mint = mint_budget * growth_programs_bps / BPS
lp_mint = mint_budget * lp_bps / BPS
integrators_mint = mint_budget * integrators_bps / BPS
treasury_mint = mint_budget - xagc_mint - growth_mint - lp_mint - integrators_mint
```

If growth programs are disabled, the growth allocation rolls to treasury.

If xAGC mint supply is zero, the xAGC allocation rolls to treasury because there are no xAGC holders to benefit from direct vault expansion.

Note for next-stage work: some economics docs recommend `50/20/10/5/15` with xAGC as the largest bucket. The current implementation/config uses `30/20/20/10/20`. A serious whitepaper should resolve this discrepancy.

## 14. Defense and Buybacks

Stress components:

```text
price_stress_bps =
    if price_twap < stressed_floor:
        (stressed_floor - price_twap) * BPS / anchor
    else:
        0

coverage_stress_bps = max(defense_reserve_coverage_bps - reserve_coverage_bps, 0)
stable_cash_stress_bps = max(defense_stable_cash_coverage_bps - stable_cash_coverage_bps, 0)
concentration_stress_bps = max(largest_collateral_concentration_bps - max_reserve_concentration_bps, 0)
oracle_stress_bps = max(oracle_confidence_bps - max_oracle_confidence_bps, 0)
exit_stress_bps = max(exit_pressure_bps - defense_exit_pressure_bps, 0)
volatility_stress_bps = max(realized_volatility_bps - defense_volatility_bps, 0)

stress_score_bps = max(
    price_stress_bps,
    coverage_stress_bps,
    stable_cash_stress_bps,
    concentration_stress_bps,
    oracle_stress_bps,
    exit_stress_bps,
    volatility_stress_bps
)
```

Severe override:

```text
if reserve_coverage_bps < hard_defense_reserve_coverage_bps:
    stress_score_bps = max(stress_score_bps, severe_stress_threshold_bps)
```

Buyback budget:

```text
buyback_cap_bps =
    if stress_score_bps >= severe_stress_threshold_bps:
        severe_defense_spend_bps
    else:
        mild_defense_spend_bps

buyback_spend_rate_bps = min(buyback_kappa_bps * stress_score_bps / BPS, buyback_cap_bps)
buyback_budget_quote_x18 = treasury_quote_x18 * buyback_spend_rate_bps / BPS
```

The simulator directly computes burn amount:

```text
buyback_burn_agc = buyback_budget_quote_x18 * WAD / price_twap_x18
```

On-chain settlement does not directly swap or burn. It converts the quote budget to raw USDC units and adds it to `pending_treasury_buyback_usdc`. A keeper then starts a constrained `BuybackCampaign`.

Buyback campaign invariant:

```text
USDC leaves campaign escrow only after matching AGC is already in the campaign AGC vault and burned in the same instruction.
```

Campaign constraints:

- total USDC cannot exceed pending treasury buyback budget,
- max slice USDC caps each execution,
- slice interval enforces TWAP cadence,
- deadline prevents stale execution,
- min AGC out protects execution quality,
- final slice must satisfy min total AGC out.

`reserve_treasury_buyback_usdc` exists but is deprecated and returns `DeprecatedBuybackPath`.

## 15. Collateral and Oracle Mechanics

Collateral registry account per accepted mint:

- mint,
- mint decimals,
- oracle source: `Manual` or `Pyth`,
- oracle feed or Pyth receiver program,
- Pyth price feed id,
- reserve token account,
- asset class: Stable, BTC, RWA, Other,
- reserve weight bps,
- collateral factor bps,
- liquidation threshold bps,
- max concentration bps,
- max oracle staleness seconds,
- max oracle confidence bps,
- enabled flag.

Config validation:

- reserve weight cannot exceed 100%,
- collateral factor must be <= liquidation threshold,
- liquidation threshold cannot exceed 100%,
- concentration cap must be between 1 and 100%,
- staleness must be positive,
- confidence limit cannot exceed 100%,
- enabled Manual oracles need non-default oracle feed and empty Pyth id,
- enabled Pyth oracles need configured Pyth receiver and non-zero feed id,
- enabled assets need a reserve token account.

Manual oracle updates:

```text
price_quote_x18 > 0
confidence_bps <= collateral_asset.max_oracle_confidence_bps
```

Pyth oracle refresh:

- verifies owner is configured Pyth Receiver program,
- checks Pyth price-update discriminator,
- checks feed id,
- checks verification level,
- checks staleness,
- converts Pyth price/exponent to X18 quote price,
- converts confidence to bps,
- rejects if confidence exceeds asset limit.

Launch collateral presets in docs:

| Asset | Reserve weight | Collateral factor | Liquidation threshold | Concentration cap | Max oracle age | Max confidence |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| USDC | 9,900 bps | 9,000 bps | 9,500 bps | 4,500 bps | 60 sec | 50 bps |
| USDT | 9,700 bps | 8,500 bps | 9,250 bps | 3,500 bps | 60 sec | 75 bps |
| BTC wrapper | 6,000 bps | 4,500 bps | 6,000 bps | 3,000 bps | 90 sec | 150 bps |
| RWA | 0 bps global | 0 bps global | per-facility only | 0 bps global | n/a | n/a |

Devnet BTC differs slightly for demo:

- reserve weight: 6000 bps,
- collateral factor: 5000 bps,
- liquidation threshold: 6500 bps,
- concentration cap: 4000 bps,
- max oracle age: 86400 seconds,
- max confidence: 100 bps,
- manual oracle price: $100,000/BTC.

## 16. Credit Facility Mechanics

Credit facilities are the borrower side of AGC. They are controlled credit sleeves, not open lending pools.

Facility state:

- one collateral mint,
- one collateral vault,
- one AGC underwriter vault,
- max total debt,
- max line debt,
- min collateral health,
- liquidation health,
- min underwriter reserve,
- interest rate,
- origination fee,
- default grace period,
- isolated flag,
- enabled flag.

Underwriters:

- deposit AGC into a facility-specific underwriter vault,
- receive facility shares,
- earn interest paid by borrowers,
- are first-loss capital on default.

Underwriter share math:

```text
if underwriter_total_shares == 0:
    shares = amount
else:
    shares = amount * underwriter_total_shares / underwriter_vault_assets

assets = shares * underwriter_vault_assets / underwriter_total_shares
```

Underwriter withdrawals are rejected if remaining vault assets would violate required reserve:

```text
required_underwriter_assets = total_principal_debt_agc * min_underwriter_reserve_bps / BPS
underwriter_vault_assets >= required_underwriter_assets
```

Borrower flow:

```text
risk admin opens credit line for borrower
borrower deposits collateral
borrower draws AGC if collateral, debt caps, health, oracle, and underwriter reserve pass
borrower repays AGC
principal repayment burns AGC
interest repayment transfers AGC to underwriter vault
```

Credit line total debt:

```text
total_debt_agc = principal_debt_agc + accrued_interest_agc
```

Collateral value:

```text
collateral_value_quote_x18 =
    collateral_amount * collateral_price_quote_x18 / 10^collateral_decimals
```

AGC debt value:

```text
debt_value_quote_x18 =
    debt_agc * anchor_price_x18 / agc_unit
```

Health:

```text
health_bps = collateral_value_quote_x18 * BPS / debt_value_quote_x18
```

Draw validation:

```text
new_total_debt <= credit_line.credit_limit_agc
new_total_debt <= facility.max_line_debt_agc
facility_principal_after <= facility.max_total_debt_agc
underwriter_vault_assets >= facility_principal_after * min_underwriter_reserve_bps / BPS
debt_value <= collateral_value * collateral_factor_bps / BPS
health_bps >= facility.min_collateral_health_bps
oracle fresh and confidence-safe
facility active and collateral enabled
```

Draw minting:

```text
fee = draw_amount * origination_fee_bps / BPS
net_amount = draw_amount - fee

mint net_amount AGC to borrower
mint fee AGC to treasury_agc
principal_debt += draw_amount
facility total principal += draw_amount
protocol credit principal outstanding += draw_amount
```

Interest accrual:

```text
elapsed = now - last_accrued_at
annual_interest = principal_debt_agc * interest_rate_bps
elapsed_interest = annual_interest * elapsed / (BPS * SECONDS_PER_YEAR)
accrued_interest += elapsed_interest
facility.total_interest_accrued += elapsed_interest
```

Repayment:

```text
repay_amount = min(amount, principal + accrued_interest)
interest_paid = min(repay_amount, accrued_interest)
principal_paid = repay_amount - interest_paid

interest_paid transfers to underwriter vault
principal_paid burns AGC from payer
if debt becomes zero, line becomes Repaid
```

Default:

Default is allowed if either:

- now is past maturity plus facility grace period, or
- health is below liquidation threshold using fresh oracle data.

Default flow:

```text
defaulted_debt = principal + accrued_interest
underwriter_loss = min(defaulted_debt, underwriter_vault_agc.amount)
burn underwriter_loss from underwriter vault
uncovered_debt = defaulted_debt - underwriter_loss
mark line Defaulted
reduce facility principal by principal debt
record default/loss accounting
zero principal and accrued interest on line
```

Seizure:

```text
credit operator transfers collateral from facility vault to collateral_asset.reserve_token_account
line collateral_amount decreases
facility total_collateral_seized increases
```

## 17. Current Policy Parameters

From `configs/policy/launch-model.json`:

```text
initial anchor: 0.50
float supply: 1,000,000 AGC
treasury quote reserve: 150,000
xAGC assets: 150,000 AGC
epoch duration: 3600 seconds

normal band: 300 bps
stressed band: 700 bps
anchor EMA: 500 bps
max anchor crawl: 100 bps

target reserve coverage: 8000 bps
expansion reserve coverage min: 3000 bps
neutral reserve lower: 2000 bps
defense reserve coverage: 1500 bps
hard defense reserve coverage: 800 bps

target stable cash coverage: 2500 bps
expansion stable cash min: 1200 bps
defense stable cash: 800 bps

target liquidity depth coverage: 5000 bps
expansion liquidity depth min: 2000 bps

max reserve concentration: 6000 bps
max oracle confidence: 150 bps
max stale oracle count: 0

min premium: 100 bps
premium persistence required: 2 epochs
min gross buy floor: 50 bps
min locked share: 1000 bps
target gross buy: 500 bps
target net buy: 250 bps
target lock flow: 100 bps
target buy growth: 500 bps
target locked share: 3000 bps

max expansion volatility: 300 bps
max expansion exit pressure: 3000 bps
defense volatility: 1000 bps
defense exit pressure: 7000 bps

expansion kappa: 1000 bps
max mint per epoch: 100 bps
max mint per day: 250 bps

buyback kappa: 5000 bps
mild defense spend: 500 bps
severe defense spend: 1500 bps
severe stress threshold: 1000 bps
recovery cooldown: 2 epochs

xAGC exit fee: 100 bps
mint distribution: 30/20/20/10/20
```

## 18. Python Policy Simulator

Script: `script/simulate_policy.py`

Command:

```bash
pnpm simulate:epochs
```

The simulator:

- reads `configs/policy/launch-model.json`,
- reads `configs/policy/scenarios.json`,
- coerces integer strings into Python integers,
- simulates each epoch in a scenario,
- applies the same core formulas as on-chain `evaluate_epoch`,
- renders either text or JSON.

Supported CLI flags:

```bash
python3 script/simulate_policy.py --format text
python3 script/simulate_policy.py --format json
python3 script/simulate_policy.py --scenario persistent-bid
python3 script/simulate_policy.py --model path/to/model.json --scenarios path/to/scenarios.json
```

Simulator state transitions:

```text
anchorPriceX18 = anchorNextX18
lastGrossBuyQuoteX18 = grossBuyQuoteX18
treasuryQuoteX18 = treasuryQuoteX18 + treasuryQuoteInflowX18 - buybackBudgetQuoteX18
treasuryAgc = treasuryAgc + treasuryMintAgc + xagcExitFeeAgc
xagcTotalAssetsAgc = xagcTotalAssetsAgc + xagcDepositsAgc - xagcGrossRedemptionsAgc + xagcMintAgc
floatSupplyAgc =
    floatSupplyAgc
    - xagcDepositsAgc
    + xagcNetRedemptionAgc
    + growthProgramsMintAgc
    + lpMintAgc
    + integratorsMintAgc
    - buybackBurnAgc
```

Note: simulator directly adjusts float and treasury state. On-chain behavior is distributed over token balances, settlement mints, pending buyback budget, and later buyback-campaign burns.

## 19. Simulator Results

I ran:

```bash
pnpm simulate:epochs
python3 script/simulate_policy.py --format json
```

Both completed successfully.

### Summary Table

| Scenario | Regime path | Total minted | Total burned | Buyback budget | Final anchor | Final float | Final xAGC assets | Final treasury quote | Final treasury AGC |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `persistent-bid` | Neutral -> Neutral -> Expansion -> Expansion -> Expansion -> Expansion | 12,063.76 AGC | 0.00 AGC | 0.00 | 0.5028 | 958,031.88 AGC | 201,619.13 AGC | 159,000.00 | 2,412.75 AGC |
| `bank-run` | Neutral -> Neutral -> Defense -> Recovery -> Recovery | 0.00 AGC | 12,251.11 AGC | 5,537.50 | 0.4937 | 1,023,858.89 AGC | 183,500.00 AGC | 218,262.50 | 390.00 AGC |
| `false-breakout` | Neutral -> Neutral -> Neutral -> Neutral -> Neutral | 0.00 AGC | 0.00 AGC | 0.00 | 0.5018 | 999,500.00 AGC | 150,500.00 AGC | 155,000.00 | 0.00 AGC |
| `oracle-degradation` | Neutral -> Neutral -> Expansion -> Defense -> Recovery | 9,890.00 AGC | 1,397.95 AGC | 715.75 | 0.5018 | 975,047.05 AGC | 231,467.00 AGC | 206,784.25 | 1,978.00 AGC |

### persistent-bid

Purpose: sustained premium, growing gross buy pressure, positive net buys, and positive xAGC lock flow.

Key behavior:

- Epochs 1-2 remain Neutral because premium persistence and buy-growth prerequisites are still building.
- Epoch 3 enters Expansion once premium persistence reaches 2 epochs and buy growth is positive.
- Epochs 3-6 mint increasing amounts as demand/health scores improve.
- No buybacks occur.

Important epoch details:

```text
e1 Neutral:   price 0.5030, premium 0.60%, reserve coverage 30.00%, lock flow 0.60%, mint 0
e2 Neutral:   price 0.5070, premium 1.36%, reserve coverage 30.47%, lock flow 0.70%, mint 0
e3 Expansion: price 0.5100, premium 1.89%, reserve coverage 30.97%, lock flow 0.81%, mint 1,480.50
e4 Expansion: price 0.5120, premium 2.20%, reserve coverage 31.47%, lock flow 0.86%, mint 2,449.35
e5 Expansion: price 0.5140, premium 2.48%, reserve coverage 31.98%, lock flow 0.92%, mint 3,500.87
e6 Expansion: price 0.5150, premium 2.56%, reserve coverage 32.49%, lock flow 0.98%, mint 4,633.03
```

Interpretation: the controller expands only after delayed confirmation. Float falls despite mints because xAGC deposits remove more AGC from liquid float than growth/LP/integrator allocations add back.

### bank-run

Purpose: falling price, rising sell pressure, thinning liquidity, and xAGC redemptions.

Key behavior:

- Epochs 1-2 are Neutral despite stress building because price remains above stressed floor and hard defense conditions are not met.
- Epoch 3 enters Defense when price falls below the stressed floor.
- Defense queues a buyback budget and burns 12,251.11 AGC in the simulator.
- Epochs 4-5 enter Recovery after price recovers above stressed floor and stress clears.

Important epoch details:

```text
e1 Neutral:  price 0.4990, exit pressure 50.00%, liquidity depth coverage 18.00%, mint 0
e2 Neutral:  price 0.4780, exit pressure 69.76%, liquidity depth coverage 11.01%, mint 0
e3 Defense:  price 0.4520, exit pressure 75.00%, liquidity depth coverage 6.96%, burn 12,251.11
e4 Recovery: price 0.4630, exit pressure 66.67%, burn 0
e5 Recovery: price 0.4720, exit pressure 61.11%, burn 0
```

Interpretation: the system does not mint during a sell-heavy event, triggers defense when price breaches the stressed band, and then forces a cooldown before expansion can resume.

### false-breakout

Purpose: premium and buy bursts exist, but almost no xAGC commitment.

Key behavior:

- All epochs stay Neutral.
- Premium persists for several epochs, and gross buys are meaningful.
- Expansion remains blocked because `lockFlowBps` is zero or too small and later because net buys/buy growth fade.
- No mint and no buyback occur.

Important epoch details:

```text
e1 Neutral: price 0.5060, premium 1.20%, lock flow 0.00%, mint 0
e2 Neutral: price 0.5090, premium 1.73%, lock flow 0.00%, mint 0
e3 Neutral: price 0.5110, premium 2.04%, lock flow 0.05%, mint 0
e4 Neutral: price 0.5080, premium 1.34%, lock flow 0.00%, mint 0
e5 Neutral: price 0.5050, premium 0.68%, lock flow 0.00%, mint 0
```

Interpretation: the model intentionally rejects price-only momentum. New supply requires committed long-duration demand through xAGC.

### oracle-degradation

Purpose: otherwise healthy expansion path until oracle confidence exceeds the maximum.

Key behavior:

- Epochs 1-2 are Neutral while persistence builds.
- Epoch 3 enters Expansion and mints 9,890 AGC.
- Epoch 4 enters Defense even though price, reserves, liquidity, and demand still look healthy, because oracle confidence rises to 220 bps while max is 150 bps.
- Epoch 5 enters Recovery after oracle confidence returns to 60 bps.

Important epoch details:

```text
e1 Neutral:   oracle confidence 0.50%, concentration 45.00%, mint 0
e2 Neutral:   oracle confidence 0.80%, concentration 45.00%, mint 0
e3 Expansion: oracle confidence 1.20%, mint 9,890.00
e4 Defense:   oracle confidence 2.20%, buyback burn 1,397.95
e5 Recovery:  oracle confidence 0.60%, mint 0
```

Interpretation: oracle health is a hard safety gate. The system can block expansion and enter defense due to bad data quality even when price action is favorable.

## 20. Demo and Bootstrap Scripts

### `script/devnet-bootstrap.ts`

One-shot devnet setup. It:

1. loads the deployer/admin keypair from `~/.config/solana/id.json`,
2. creates AGC mint with temporary admin mint authority,
3. mints 10,000,000 AGC to the deployer wallet,
4. transfers AGC mint authority to the program `mint-authority` PDA,
5. creates xAGC mint with PDA mint authority from inception,
6. creates mock USDC and BTC mints,
7. mints 1 BTC to deployer,
8. creates growth/LP/integrator AGC token accounts,
9. calls `initialize_protocol`,
10. configures BTC collateral and manual oracle,
11. initializes BTC credit facility,
12. writes all important addresses to `deployments/devnet.json`.

Not safe to rerun against the same program ID because the protocol `state` PDA is one-shot.

### `script/demo/underwrite.ts`

Deposits AGC into the BTC facility underwriter vault.

Default:

```bash
pnpm exec tsx script/demo/underwrite.ts
```

Default amount is 50,000 AGC. It prints the underwriter vault balance before and after and a devnet explorer transaction URL.

### `script/demo/borrow.ts`

Opens a credit line, deposits BTC collateral, and draws AGC.

Default:

```bash
pnpm exec tsx script/demo/borrow.ts
```

Defaults:

- collateral: 0.5 BTC,
- draw: 5,000 AGC,
- line id: 1,
- line limit: 100,000 AGC,
- maturity: 30 days.

It prints borrower AGC delta and principal debt.

### `script/demo/expansion-cycle.ts`

Drives a full expansion settlement demo on devnet.

It:

1. lowers policy epoch duration to 2 seconds,
2. relaxes several targets so the devnet state can hit Expansion gates,
3. records cycle 1 swaps at elevated price,
4. waits and settles cycle 1, expected Neutral because initial buy growth is zero,
5. deposits 50 AGC into xAGC to seed positive lock flow for cycle 2,
6. records cycle 2 swaps with higher gross buy volume,
7. waits and settles cycle 2, expected Expansion,
8. prints mint distribution deltas for xAGC vault, growth, LP, integrators, and treasury.

Note: `docs/deployment-guide.md` mentions 10,000 AGC deposit for positive lock flow, while the current script deposits 50 AGC after a reset-drained float. The current script is the executable source of truth.

### `script/demo/full-run.ts`

Pre-recording smoke test for dashboard actions. It runs:

1. `depositXagc` for 100 AGC,
2. `depositUnderwriterAgc` for 1,000 AGC,
3. `depositCreditCollateral` for 0.1 BTC,
4. `drawCreditLine` for 200 AGC,
5. `repayCreditLine` for 100 AGC.

It assumes line id 1 already exists from bootstrap/borrow flow and skips epoch settlement.

### `script/demo/reset.ts`

Pre-recording reset. It:

1. redeems all deployer xAGC,
2. drains deployer AGC down to 2,000 AGC by transferring excess to treasury,
3. tries to settle an empty epoch so the regime returns to Neutral.

It may fail with `EpochTooSoon`; the script tells the user to retry later.

## 21. Frontend

The `web/` app is a Vite/React Solana dashboard. It has:

- AGC console hero and telemetry,
- live protocol state polling via `web/src/useProtocolState.ts`,
- wallet connection,
- xAGC deposit/redeem actions,
- underwrite, deposit collateral, draw, and repay actions,
- mainnet-only Jupiter swap panel,
- devnet program/address explorer links,
- hosted docs and `llms.txt`/`llms-full.txt`.

Frontend reads:

- protocol regime,
- anchor price,
- last settled epoch,
- reserve coverage,
- stable cash coverage,
- premium,
- locked share,
- exit pressure,
- volatility,
- credit outstanding/drawn/repaid/interest/default counters,
- treasury AGC/USDC,
- xAGC vault AGC,
- underwriter vault AGC,
- AGC/xAGC mint supplies,
- connected wallet AGC/xAGC/BTC balances.

Frontend transaction handlers build Anchor instructions in `web/src/transactions.ts` and submit through the injected Solana wallet.

## 22. Tests and Verification

I ran:

```bash
cargo test --manifest-path solana/programs/agc_solana/Cargo.toml --lib
```

Result:

```text
31 passed; 0 failed
```

Pure Rust tests cover:

- anchor crawl clamp,
- expansion mints,
- defense buyback budgeting,
- xAGC share math,
- invalid policy parameter rejection,
- collateral asset config guardrails,
- Pyth config/price/decoder checks,
- buyback campaign config checks,
- buyback slice cadence and output checks,
- credit facility config checks,
- admin migration behavior,
- credit draw collateral and underwriter reserve checks,
- disabled collateral and debt cap failures,
- underwriter withdrawal reserve protection,
- interest accrual,
- oracle freshness enforcement,
- repaid-line collateral withdrawal behavior,
- matured and immature default logic,
- stable cash/oracle expansion blocking,
- concentration expansion blocking,
- keeper permission scoping,
- settlement window checks,
- mint day-window reset,
- settlement state roll-forward,
- recovery cooldown countdown,
- daily mint cap.

Anchor local-validator integration tests exist in `solana/tests/agc_solana.ts` and cover:

- rejecting externally freezable AGC mints,
- initialized protocol account/mint/vault/admin records,
- credit lifecycle deposit -> draw -> repay -> withdraw,
- xAGC deposit -> share mint -> redeem with exit fee,
- credit default and collateral seizure,
- epoch settlement and expansion mint distribution.

I did not run the local-validator Anchor test suite in this pass.

## 23. Current Production Gaps and Hackathon Shortcuts

Important gaps:

- Reserve aggregation is not fully on-chain; settlement currently accepts reserve/liquidity/oracle/concentration external metrics.
- Devnet uses mock USDC and mock BTC mints.
- Devnet BTC oracle is manual, not Pyth.
- Devnet authorities are deployer-centric; production multisig migration is pending.
- Upgrade authority is still deployer wallet in staging.
- There is no mainnet AGC liquidity yet; Jupiter panel is hidden off mainnet.
- Buyback execution adapter is documented but not implemented as a production Raydium/Orca/Jupiter adapter in this repo.
- RWA collateral is conceptual/disabled, not productionized.
- Facility detail pages and richer credit risk UI are pending.
- Operational monitoring for policy settlement, oracle freshness, reserves, credit health, and pause events is pending.
- Additional adversarial tests are listed as TODOs: stale oracle data, wrong mint accounts, underwriter reserve drain, overdraw, uncovered default accounting, collateral seizure routing edge cases.
- Distribution mismatch should be resolved: current config uses 30% xAGC expansion allocation, while economics docs recommend xAGC as the largest allocation at 50%.
- Exit fee mismatch should be resolved: current config is 1%, while some docs recommend 3%.
- Launch-model initial anchor is 0.50; devnet bootstrap anchor is 1.00.

## 24. Suggested Questions for Next-Stage Whitepaper Work

These are the questions a rigorous academic/product pass should answer.

1. Does "reserve-efficient agent credit" make sense as a real market category, or is the demand thesis too speculative?
2. What exactly is the legal/economic status of AGC if it is neither a stablecoin nor a claim on reserves?
3. What reserve-efficiency ratio is defensible at launch, and how should it evolve with live data?
4. Should xAGC receive 30%, 50%, or another share of expansion, and how does that affect holder incentives and float stability?
5. Is an exit fee enough for xAGC, or should redemptions have a cooldown/queue under stress?
6. Should Defense trigger on oracle degradation alone, or should it block expansion without spending buyback budget unless price/reserve stress exists?
7. What market data sources are robust enough for `grossBuy`, `grossSell`, `TWAP`, liquidity depth, and exit pressure on Solana?
8. How should the protocol prevent manipulation of lock flow, buy volume, and liquidity depth around epoch boundaries?
9. How should treasury stable cash be acquired in production: primary issuance, fees, underwriter premiums, LP revenue, external market-making, or some mix?
10. What should happen in severe insolvency-style scenarios where buybacks and halted issuance are insufficient?
11. Is underwriter first-loss AGC capital enough protection if AGC price itself is impaired during defaults?
12. Should credit facilities mint AGC at anchor value, TWAP value, or risk-adjusted value?
13. What are the correct collateral factors for BTC wrappers and future RWAs under Solana liquidity conditions?
14. What should be governed by multisig, what should be immutable, and what should require timelock?
15. What is the accelerator pitch: agent credit network, programmable private credit, autonomous-market money layer, or something else?

## 25. Useful One-Liners for Pitch/Whitepaper Drafting

- AGC is liquid credit inventory for autonomous markets.
- xAGC owns the long-duration expansion layer.
- The system expands against balance-sheet strength, not hype volume.
- AGC is not a hard-peg stablecoin; it is a policy-managed credit asset.
- Stablecoins are defense cash, BTC wrappers are risk-weighted strategic reserves, and RWAs start isolated.
- Borrowers receive AGC only when collateral, underwriter reserve, oracle freshness, and debt caps pass.
- Underwriters are first-loss capital and earn credit spread.
- Principal repayment burns AGC; interest repayment flows to underwriters.
- Defense stops issuance and queues constrained buyback campaigns.
- Buyback USDC cannot leave escrow unless AGC is delivered and burned.
- The central research problem is whether a floating, reserve-efficient credit asset can become useful working capital for agent economies.


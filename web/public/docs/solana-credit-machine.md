# AGC Solana Credit Machine

AGC is a reserve-efficient credit machine on Solana. It expands circulating AGC only when the protocol gains stablecoin reserves, risk-weighted BTC exposure, isolated RWA collateral, real protocol revenue, or high-quality credit claims.

The important question for a holder is not whether the protocol can print. It is whether every expansion leaves the system stronger than it was before. AGC is built around that standard.

## Overview

- `AGC` is the circulating credit unit.
- `xAGC` is the savings and upside share.
- USDC and USDT are defensive cash reserves.
- BTC wrappers are strategic reserve collateral with haircuts.
- Tokenized stocks and RWAs start in isolated buckets.

The core rule:

```text
Every expansion leaves the protocol with more assets, more revenue, or better credit claims than before.
```

## Reserve Model

For holders, reserves are what make expansion credible. AGC can accept several reserve or collateral assets, but they do not count equally.

### Cash bucket

USDC and USDT support immediate defense, buybacks, and AGC/USDC liquidity. They receive the highest reserve weights, with concentration, depeg, issuer, and oracle controls around them.

### BTC bucket

BTC wrappers can create upside and strategic reserve strength, but each wrapper is a separate risk object. cbBTC, tBTC, Wormhole WBTC, zBTC, and future wrappers each have their own oracle, reserve weight, concentration cap, and liquidity assumptions.

### RWA bucket

Tokenized stocks, treasuries, funds, and other RWAs start isolated. They carry issuer risk, market-hours gaps, legal restrictions, oracle risk, and liquidity cliffs. They become part of global reserve capacity only after those risks are bounded.

## Policy Model

AGC expansion is deliberately hard to unlock. Capacity is the minimum of independent limits:

```text
mint_capacity = min(
  stable_cash_capacity,
  risk_weighted_reserve_capacity,
  liquidity_depth_capacity,
  credit_demand_capacity,
  oracle_safe_capacity,
  epoch_cap,
  daily_cap
)
```

Expansion requires healthy stable cash, risk-weighted reserve coverage, AGC/USDC liquidity depth, oracle confidence, collateral concentration, low volatility, low exit pressure, xAGC lock flow, and persistent demand.

Defense starts when price, reserve coverage, stable cash coverage, oracle health, volatility, or exit pressure breaks configured limits.

## Credit Facilities

Credit facilities are the borrower side of AGC. Each facility is tied to one collateral mint, one collateral vault, one AGC underwriter vault, debt caps, health thresholds, oracle limits, and pause controls.

The flow:

```text
risk governance opens a facility
-> underwriters deposit AGC as first-loss capital
-> an approved borrower opens a credit line
-> the borrower deposits collateral
-> the borrower draws AGC inside collateral, facility, and reserve limits
-> repayment burns principal and sends interest to underwriters
-> default burns underwriter reserve and routes seized collateral to the configured reserve account
```

This is how AGC creates credit without pretending collateral does not matter. A draw creates AGC, but the other side of the balance sheet is visible: collateral in a PDA vault, underwriter AGC behind the line, interest owed by the borrower, and liquidation rights if the line breaks.

## Governance

AGC governance is structured for a live credit protocol: fast enough to reduce risk, constrained enough to avoid arbitrary control.

- High-level control sits behind Solana multisigs rather than one hot wallet.
- Admin authority covers migration and high-level configuration.
- Risk authority sets policy parameters, collateral configs, reserve weights, mint caps, and distributions inside hard limits.
- Emergency authority can pause minting, settlement, collateral updates, buybacks, and vault operations.
- Upgrade authority is held behind a higher-threshold multisig.

Risk-reducing actions can happen quickly. Risk-increasing changes are operationally slower and bounded by the program.

## Demand Path

AGC is not just another borrow asset. Demand comes from scarce access to monetary capacity and from ownership of a system that can grow when its balance sheet improves.

- Users buy AGC because they expect reserve and credit capacity to grow.
- Agents use AGC as working capital.
- xAGC holders receive most expansion flow and later protocol revenue.
- Underwriters can back credit pools for spread with real first-loss risk.
- BTC reserve appreciation can increase risk-weighted expansion capacity.

The loop:

```text
AGC demand rises
-> reserves and liquidity deepen
-> credit capacity increases
-> agents and borrowers use credit
-> fees and repayments grow
-> xAGC becomes more valuable
-> confidence and AGC demand increase
```

## FOMO Mechanics

Healthy FOMO comes from scarce capacity, not fake yield.

Good mechanics:

- capped epoch issuance
- rising primary issuance curve
- xAGC fee and expansion index
- credit access tiers
- underwriter tranches
- BTC reserve upside
- repaid-credit mining

Bad mechanics:

- emissions-only APY
- minting because price pumped
- unbounded rebases
- fake buybacks
- wash-volume rewards
- opaque admin risk

## Solana Architecture

Solana public pools can be traded through Jupiter, Phantom, wallets, and bots. Therefore, optional adapter flow cannot be treated as complete global demand.

Under the hood, AGC does the following:

- AGC/USDC remains the primary quote market.
- Public DEX trading remains open through wallets, aggregators, and bots.
- Adapter flow is official-venue telemetry, not complete demand truth.
- Issuance depends on reserves, collateral, oracles, liquidity depth, and credit quality.
- PDAs control mint authority, treasury authority, and vault authority.
- Collateral is registered per mint with weight, factor, threshold, concentration, and oracle controls.
- Credit facilities use PDA collateral vaults, PDA underwriter AGC vaults, borrower credit-line accounts, interest accrual, repayment burns, default accounting, and collateral seizure.

## Implementation Status

The Anchor program includes the core Solana protocol: xAGC vaults, policy settlement, collateral registry, oracle cache accounts, buyback escrow reservations, role-scoped keepers, separated governance authorities, and the credit facility layer.

Production integration adds direct oracle adapters, reserve aggregation, a venue-specific atomic buyback executor, deployed IDL/client wiring, and isolated RWA onboarding.

The program is upgradeable through the Solana upgradeable loader. Production upgrade authority sits behind a higher-threshold multisig, and the main protocol accounts include reserved space for future evolution.

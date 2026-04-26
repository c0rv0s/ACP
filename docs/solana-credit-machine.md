# AGC Solana Credit Machine

This document explains the Solana version of AGC for people evaluating how the system works and what protects it.

AGC is not another generic lending market. It is a reserve-efficient credit machine: a protocol that can expand circulating AGC quickly when the balance sheet, collateral base, liquidity, and credit demand justify it, then slow or defend when those conditions weaken.

## 1. Protocol Thesis

AGC expands only when the protocol receives or creates an asset on the other side of the balance sheet.

Acceptable expansion sources:

- stablecoin reserves
- risk-weighted BTC reserves
- isolated RWA or stock-token collateral
- performing credit lines
- protocol revenue
- primary issuance proceeds

Unsafe expansion sources:

- raw hype volume by itself
- keeper-reported buy pressure as the only trigger
- unbounded emissions
- non-atomic buybacks
- admin discretion outside hard-coded parameter bounds

Short form:

```text
AGC is the circulating credit unit.
xAGC is the long-duration upside and savings share.
Reserves and collateral are the balance-sheet base.
Credit facilities create AGC against approved collateral and underwriter reserve.
Governance chooses parameters inside hard protocol guardrails.
```

For a holder, the key diligence question is simple: does new AGC leave the protocol with more reserves, more revenue, or better credit claims than before? The system is built so the answer has to be yes before expansion becomes available.

## 2. Reserve Buckets

AGC supports more than USDC, but assets do not count equally. A dollar stablecoin, a wrapped BTC mint, and a tokenized stock all strengthen the balance sheet in different ways, so the protocol treats them differently.

### Cash Reserve Bucket

Purpose:

- immediate defense
- buybacks
- redemption and exit liquidity support
- primary AGC/USDC market depth

Assets:

- USDC
- USDT

How the protocol treats this bucket:

- USDC reserve weight: 98% to 100%
- USDT reserve weight: 95% to 99%
- concentration caps per issuer and mint
- depeg and oracle guards
- ability to rebalance USDT into USDC

### Strategic Reserve Bucket

Purpose:

- long-term collateral strength
- upside from BTC appreciation
- expansion capacity during healthy markets

Assets:

- BTC wrappers on Solana such as cbBTC, tBTC, Wormhole WBTC, zBTC, or later wrappers

How the protocol treats this bucket:

- each wrapper onboarded separately
- reserve weight: 50% to 70% at launch
- strict concentration cap
- oracle confidence and staleness limits
- no assumption that wrapped BTC equals native BTC

### Experimental RWA Bucket

Purpose:

- future support for tokenized stocks, treasuries, funds, or other real-world assets
- isolated credit and collateral markets before global reserve inclusion

How the protocol treats this bucket:

- start disabled
- onboard asset by asset
- isolated caps first
- lower reserve weights
- account for market-hours gaps, issuer risk, legal restrictions, and liquidity cliffs

## 3. Policy Inputs

The Solana policy does not depend on every AGC swap passing through a protocol adapter. Public DEX pools can be traded through Jupiter, Phantom, bots, and other aggregators. Optional adapter flow is not treated as global truth.

Policy inputs are balance-sheet first:

```text
risk_weighted_reserve_value
stable_cash_reserve_value
liquidity_depth
oracle_confidence
oracle_staleness
largest_collateral_concentration
credit_demand
repayment_quality
protocol_revenue
xAGC lock flow
price TWAP
exit pressure
volatility
```

Market observations can still be useful, but they are advisory unless they are derived from a controlled venue or verified pool/oracle accounts.

## 4. Mint Capacity

The policy computes expansion capacity as a minimum across independent limits.

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

This makes AGC a balance-sheet printer rather than a price-action printer.

Expansion requires:

- risk-weighted reserve coverage above the expansion threshold
- stable cash coverage above the stable-cash threshold
- sufficient AGC/USDC liquidity depth
- collateral concentration below cap
- fresh oracle data with acceptable confidence
- low volatility
- low exit pressure
- positive xAGC lock flow
- persistent premium or demand

Defense triggers:

- price below stressed floor
- risk-weighted reserve coverage below defense threshold
- stable cash coverage below defense threshold
- stale or low-confidence oracle data
- volatility above defense threshold
- exit pressure above defense threshold

## 5. Credit Facilities

Credit facilities are the controlled borrower side of AGC. They are not open-ended lending pools. Each facility is tied to one collateral mint, one collateral vault, one AGC underwriter vault, hard debt caps, health thresholds, oracle limits, and pause controls.

The flow:

```text
risk governance opens a facility
-> underwriters deposit AGC into the first-loss vault
-> an approved borrower opens a credit line
-> the borrower deposits collateral
-> the borrower draws AGC inside collateral, facility, and reserve limits
-> repayment burns principal and sends interest to underwriters
-> default burns underwriter reserve and routes seized collateral to the configured reserve account
```

This is the part of the protocol that makes AGC more than a passive vault token. Credit can be created when another balance-sheet claim exists: collateral in the vault, underwriter AGC behind the line, interest owed by the borrower, and liquidation rights if the line breaks.

Important mechanics:

- Principal draw mints AGC directly to the borrower, with origination fees minted to treasury.
- Principal repayment burns AGC, reducing outstanding credit.
- Interest repayment flows into the underwriter vault.
- Underwriters are first-loss capital. Default burns available underwriter AGC before collateral recovery.
- Borrower collateral is valued through the configured collateral oracle cache.
- Draws fail if the line exceeds its credit limit, the facility exceeds total debt caps, health falls below the minimum, or underwriter reserve falls below the required percentage.
- Matured or unhealthy lines can be marked defaulted by credit operators or governance, and seized collateral routes to the configured reserve account for that collateral mint.

## 6. Governance

AGC governance is structured for a live credit protocol: fast enough to reduce risk, constrained enough to avoid arbitrary control. The launch model uses Solana multisigs rather than token voting or a single founder wallet.

The authority lanes are separated:

- Admin multisig: migrates authorities and configures high-level protocol addresses.
- Risk multisig: sets risk parameters, collateral configs, policy params, and mint distribution inside hard limits.
- Emergency guardian: pauses minting, settlement, collateral updates, buybacks, and vault operations.
- Upgrade authority: controls program upgrades behind a higher-threshold multisig.

Risk-reducing actions can happen quickly:

- disable collateral
- lower reserve weights
- lower collateral factors
- lower mint caps
- pause credit issuance
- pause buybacks or settlement

Risk-increasing actions are operationally slower:

- raise mint caps
- add new collateral
- raise reserve weights
- raise collateral factors
- lower required coverage

The program enforces hard-coded bounds so governance tunes inside a safe box rather than choosing arbitrary values.

## 7. Demand Path

AGC demand comes from credit capacity and ownership of the growth machine, not from passive emissions.

Demand loops:

- Users buy AGC because they expect reserve and credit capacity to grow.
- Agents and applications use AGC as working capital.
- Borrowers or agents need AGC/xAGC staking, fees, or credit access to unlock better limits.
- xAGC holders receive the majority of expansion and later protocol revenue.
- Underwriters back credit pools and earn spread for taking real risk.
- Protocol-owned BTC and other collateral can create expansion capacity when they appreciate.

The reflexive loop:

```text
AGC demand rises
-> primary issuance or market demand adds reserves
-> reserves and liquidity deepen
-> credit capacity increases
-> borrowers and agents use credit
-> fees and repayments grow
-> xAGC becomes more valuable
-> confidence and AGC demand increase
```

This is the constructive "money printer" loop: every expansion leaves the protocol with more assets, revenue, or high-quality credit claims than before.

## 8. FOMO Mechanics

Healthy FOMO comes from scarce access to capacity.

Good mechanics:

- limited epoch issuance
- rising primary issuance curve
- xAGC fee and expansion index
- credit access tiers
- underwriter tranches
- BTC reserve upside
- rewards for repaid credit volume

Bad mechanics:

- emissions-only APY
- minting because spot price pumped
- unbounded rebases
- fake buybacks
- rewards for wash volume
- opaque admin-controlled risk changes

## 9. Solana Implementation Notes

Solana public pools can be traded through wallets, aggregators, and bots. If users trade through normal DEX pools, AGC cannot assume the program sees all swaps.

Therefore:

- AGC/USDC remains the primary quote market.
- Jupiter/Phantom/DEX trading stays open.
- Policy does not require every swap to pass through AGC.
- Adapter-reported swap flow is official-venue telemetry, not global demand.
- Critical expansion math uses reserves, collateral, oracles, liquidity depth, and credit quality.

## 10. Current Implementation

The Anchor program now contains the core Solana protocol surfaces:

- PDA-owned AGC mint authority, treasury authority, xAGC authority, and credit-facility authority.
- xAGC deposit and redemption accounting.
- Role-scoped keepers for market reporting, oracle reporting, settlement, buybacks, treasury burns, and credit operations.
- Separate admin, risk, emergency, and upgrade authority model.
- Freeze-authority rejection for AGC and xAGC mints at initialization.
- Two-step admin transfer.
- Emergency pause flags across market, settlement, vault, collateral, buyback, and credit surfaces.
- Collateral asset registry with reserve weight, collateral factor, liquidation threshold, concentration cap, oracle staleness, and oracle confidence controls.
- Collateral oracle cache accounts keyed by collateral mint.
- Credit facilities with collateral vaults, underwriter AGC vaults, borrower credit-line accounts, draw caps, interest accrual, repayment, default, and collateral seizure.
- Policy settlement with stable cash coverage, risk-weighted reserve coverage, liquidity depth coverage, oracle confidence, oracle staleness, and concentration inputs.
- Buyback budget reservation to a configured escrow instead of arbitrary token accounts.

Production integration work remains:

- Direct Pyth or Switchboard adapter validation for the collateral oracle cache.
- On-chain reserve aggregation from configured reserve token accounts.
- A venue-specific atomic buyback executor that swaps escrowed USDC and burns AGC in one controlled flow.
- Frontend IDL/client wiring for live Solana transactions after deployment addresses exist.
- Isolated RWA onboarding with issuer, legal, oracle, and liquidity constraints.

The program is upgradeable through the Solana upgradeable loader. Production upgrade authority belongs behind a higher-threshold multisig, and the main protocol accounts include reserved space so the protocol can evolve without replacing the entire account model.

## 11. Non-Goals For Now

- no Token-2022 transfer hooks at launch
- no assumption that all swaps route through AGC
- no token voting governance requirement
- no unbounded minting
- no full global RWA reserve basket before isolated testing

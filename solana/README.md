# AGC Solana Program

This folder contains the Anchor implementation of Agent Credit Protocol.

The Solana program owns the core protocol surfaces:

- SPL `AGC` mint with this program's mint-authority PDA.
- Non-rebasing `xAGC` SPL share mint backed by the program-owned AGC vault token account.
- Program-owned treasury AGC and stablecoin token accounts.
- Collateral registry accounts for USDC, USDT, BTC wrappers, and later RWA/tokenized-stock mints.
- Credit facility accounts with collateral vaults, AGC underwriter vaults, borrower credit lines, draw limits, repayment, default, and collateral seizure.
- Market telemetry instructions for official venue adapters.
- On-chain epoch settlement, regime selection, expansion mint distribution, and treasury buyback budgeting.

The actual AMM swap path is intentionally not embedded in this program. Public Solana pools can be traded through Jupiter, Phantom, wallets, and bots, so optional adapter flow is not treated as complete global demand. A Raydium, Orca, Phoenix, Jupiter, or custom venue adapter can still call the market-recording instructions with official-venue flow data, but production mint policy remains balance-sheet first.

## Current Solana Economic Model

The target model is a risk-weighted reserve and credit-capacity system:

- USDC and USDT are defensive cash reserves.
- BTC wrappers are strategic reserve collateral with haircuts and concentration caps.
- Tokenized stocks and RWAs can be onboarded later as isolated collateral.
- Market reporting is telemetry unless it comes from a controlled venue adapter.
- Epoch settlement uses risk-weighted reserves, stable cash coverage, liquidity depth, oracle confidence, oracle staleness, and collateral concentration in addition to market flow.
- Credit facilities mint AGC only against approved collateral, fresh oracle cache data, facility debt caps, line debt caps, and required underwriter reserves.

## Build

```bash
cd solana
anchor build
```

For the pure Rust policy tests:

```bash
cargo test --manifest-path programs/agc_solana/Cargo.toml --lib
```

## Program Accounts

- `ProtocolState`: global protocol configuration, policy parameters, regime state, epoch accumulator, xAGC vault counters, and settlement recipients.
- `Keeper`: optional authorization account with scoped permissions for off-admin market reporting, epoch settlement, buyback execution, and treasury burns.
- `CollateralAsset`: per-mint registry account for accepted collateral or reserve assets, including oracle feed, asset class, reserve weight, collateral factor, liquidation threshold, concentration cap, oracle staleness, oracle confidence, and enabled flag.
- `CollateralOracle`: cached per-mint collateral price with confidence and freshness metadata.
- `CreditFacility`: per-collateral credit sleeve with collateral vault, underwriter AGC vault, risk config, principal debt, interest, default, and collateral recovery accounting.
- `UnderwriterPosition`: share accounting for an underwriter's AGC deposited into a facility.
- `CreditLine`: borrower line with credit limit, principal debt, accrued interest, collateral amount, maturity, default state, and seized collateral accounting.
- `treasury_agc`: program-owned AGC token account.
- `treasury_usdc`: program-owned USDC token account.
- `xagc_vault_agc`: program-owned AGC token account backing xAGC shares.
- `buyback_usdc_escrow`: admin-configured USDC token account that receives reserved buyback budget.

## Governance

The program separates authority surfaces:

- `admin`: migration and high-level configuration.
- `risk_admin`: policy parameters, mint distribution, settlement recipients, exit fee, and collateral asset configs.
- `emergency_admin`: pause flags.
- scoped keepers: operational settlement, market reporting, oracle reporting, credit operations, buyback execution, and treasury burns.

The admin can migrate governance through the existing two-step `transfer_admin` / `accept_admin` flow. Production authorities are Squads multisigs or equivalent Solana multisig accounts, not a single hot wallet.

## Token Authorities

The initializer must provide existing SPL mints whose mint authority is the program PDA derived from `["mint-authority"]`.

AGC and xAGC use the same mint-authority PDA and must not have an external freeze authority. Treasury and xAGC vault token accounts are owned by separate PDAs so the program can transfer vault assets, burn treasury AGC, and mint policy allocations without any external private key.

## Hardening Notes

- Market-reporting instructions can be called by admin, a keeper with market-reporting permission, or the configured `market_adapter_authority`. In production, `market_adapter_authority` is a PDA controlled by the canonical venue adapter so swap reporting happens through that adapter CPI.
- Treasury buyback reservations can only transfer USDC to the configured `buyback_usdc_escrow`. A production venue adapter executes the swap and burn atomically, or holds funds in a nonce/deadline-constrained escrow until the burn leg settles.
- Admin transfer is two-step through `transfer_admin` and `accept_admin`.
- Pause flags can independently halt xAGC deposits, xAGC redemptions, market reporting, epoch settlement, credit issuance, collateral updates, credit facility updates, credit line updates, credit draws, credit repayments, underwriter deposits, underwriter withdrawals, liquidations, buyback reservations, and treasury burns.
- Collateral updates are risk-admin gated and validated against hard-coded guardrails.
- Expansion checks stable cash coverage, risk-weighted reserve coverage, liquidity depth coverage, oracle confidence, oracle staleness, and collateral concentration.
- Credit draws enforce collateral factors, minimum health, total facility caps, line caps, and underwriter reserve requirements.
- Principal repayment burns AGC. Interest repayment flows into the facility underwriter vault. Default burns underwriter AGC first and leaves collateral available for recovery.
- Program accounts reserve extra space for upgrades, and production upgrade authority belongs behind a higher-threshold multisig.

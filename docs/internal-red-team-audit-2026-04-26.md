# Internal Red-Team Audit - 2026-04-26

Scope: `solana/programs/agc_solana/src/lib.rs`.

Angle: assume a compromised keeper, compromised risk multisig, confused governance migration, hostile borrower, hostile underwriter, and stressed oracle/liquidity conditions.

## Executive Result

The math and lifecycle foundation is improving. This pass removed the highest-risk oracle reporter path for Pyth collateral assets and replaced the raw buyback escrow transfer with a constrained campaign primitive. The largest open production trust assumption is settlement-time reserve and liquidity reality, which is still keeper-supplied.

## Fixed During This Pass

### Admin migration left old risk and emergency authority live

`accept_admin` previously moved only `admin`. If the previous admin also held `risk_admin` or `emergency_admin`, those roles stayed with the old key after migration. A compromised old admin key could still alter risk parameters or pause surfaces after a migration that looked complete.

Fix:

- `accept_admin` now routes through `accept_admin_inner`.
- If `risk_admin` or `emergency_admin` still point to the previous admin, they migrate to the new admin.
- Explicitly separate risk/emergency roles are preserved.

### Repaid collateral invariant tightened

The borrower collateral-withdrawal path now calls the withdrawal-status helper before branching, so an impossible `Repaid` line with nonzero debt is rejected rather than relying only on the status enum.

### Pyth collateral oracle path added

Collateral assets can now be configured with `OracleSource::Pyth`. The refresh instruction validates that the price-update account is owned by the configured Pyth receiver program, has the expected Anchor `PriceUpdateV2` discriminator, is fully verified, matches the configured feed id, is fresh, has acceptable confidence, and converts into quote x18 on-chain. The old manual price setter is rejected for Pyth-backed assets.

### Buyback defense moved into campaign escrow

The legacy `reserve_treasury_buyback_usdc` path is disabled. Defense budgets now fund PDA-owned campaign escrows. A TWAP slice releases USDC to the configured adapter account only after AGC has been delivered into the campaign AGC vault and burned in the same instruction. Slices enforce max size, cadence, deadline, per-slice minimum output, and total campaign output.

## High-Risk Findings Still Open

### Keeper-reported reserve reality can drive issuance

`settle_epoch` accepts `ExternalMetrics` directly from the settler. Those values drive stable cash coverage, risk-weighted reserve coverage, liquidity depth, oracle confidence, stale-oracle count, and concentration. A compromised settler can make the balance sheet look healthy and mint up to the configured epoch/day caps.

Required fix:

- Replace keeper-supplied reserve and liquidity metrics with on-chain aggregation from configured reserve accounts, verified oracle feeds, and verified pool/liquidity accounts.
- Keep manual metrics only as telemetry, never as expansion authority.

### Risk governance can redirect minted allocations and seized collateral

`set_settlement_recipients` stores raw recipient pubkeys, and settlement only checks that passed accounts match those keys and have the AGC mint. Risk governance can route growth, LP, or integrator mints to arbitrary AGC accounts. Separately, `set_collateral_asset` stores a raw `reserve_token_account`, and seized collateral routes to that configured account.

Required fix:

- Put recipient changes behind delay and public pending state.
- Require recipient token accounts to be registry-owned or PDA-owned where possible.
- Use per-collateral reserve PDAs for seized collateral, with governed withdrawal paths instead of raw destination accounts.

## Failure Modes To Model Next

- Oracle outage while borrowers are close to liquidation.
- Sudden reserve depeg where stable cash coverage remains numerically high until oracle updates.
- BTC wrapper halt, bridge freeze, or redemption impairment.
- A risk multisig mistake that sets aggressive mint caps and redirects recipients in the same governance window.
- Underwriter exit rush after credit quality worsens.
- Jupiter/DEX liquidity disappears while policy still reports stale healthy liquidity.
- Campaign executor delivers AGC through an off-chain route at poor execution quality; campaign min-output and slice caps should be monitored against live liquidity.

## Verification

```bash
cargo test --manifest-path solana/programs/agc_solana/Cargo.toml --lib
```

Passed with 31 tests after this pass.

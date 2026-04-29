# Internal Protocol Audit - 2026-04-26

Scope: `solana/programs/agc_solana/src/lib.rs`.

This pass reviewed the current Anchor implementation, account constraints, credit lifecycle, xAGC accounting, policy settlement, keeper permissions, pause flags, and pure Rust test coverage. It did not run a local validator integration suite because the repo does not currently include one.

## Result

The foundation is stronger after this pass, but it is not yet production complete. The core policy and credit math now have better adversarial coverage, two credit-lifecycle bugs were fixed, Pyth-backed collateral oracle refresh is implemented, and buyback defense now runs through constrained campaign escrows. The largest remaining risks are integration-level: on-chain reserve aggregation, local-validator coverage, deployed client wiring, and external security review.

## Fixed Findings

### Repaid credit lines trapped borrower collateral

`withdraw_credit_collateral` previously required `CreditLineStatus::Active`. When a borrower fully repaid, `repay_credit_line` moved the line to `Repaid`, which made any remaining collateral impossible to withdraw through the normal borrower path.

Fix:

- `withdraw_credit_collateral` now accepts both `Active` and `Repaid` lines.
- Active lines with debt still require fresh oracle data and post-withdrawal health.
- Active lines with no debt and repaid lines can withdraw collateral without an unnecessary oracle dependency.

### Matured defaults were blocked by stale oracle data

`mark_credit_line_default` previously validated oracle freshness before checking whether the line had matured past its grace period. During an oracle outage, a matured unpaid borrower could not be marked defaulted, which blocked underwriter loss accounting and collateral recovery.

Fix:

- Matured-past-grace defaults no longer require a fresh oracle.
- Immature defaults still require fresh oracle data and health below liquidation threshold.

### Collateral oracle prices were keeper-reported

Pyth collateral assets no longer accept arbitrary keeper price numbers. `refresh_collateral_oracle_from_pyth` validates the configured Pyth receiver program, `PriceUpdateV2` account discriminator, full verification level, feed id, publish time, staleness window, confidence, and quote x18 conversion before updating the cache used by credit draws.

### Defense buybacks were raw USDC transfers

The legacy reserve-transfer instruction is disabled. Buyback budgets now move into PDA-owned campaign escrows. Each TWAP slice burns delivered AGC and only then releases the corresponding USDC to the configured adapter account, with slice size, cadence, deadline, and min-output checks.

## Adversarial Tests Added

- Disabled collateral cannot support credit draws.
- Line-level and facility-level debt caps reject overdraw attempts.
- Underwriter withdrawals cannot leave the facility below required first-loss reserve.
- Repaid credit lines can release collateral without oracle health checks.
- Defaulted lines cannot use the borrower collateral-withdrawal path.
- Matured defaults proceed even if the oracle is stale.
- Immature defaults require both fresh oracle data and bad health.
- Reserve concentration blocks expansion even when demand signals are strong.
- Pyth collateral config requires a receiver program and feed id.
- Pyth price conversion, confidence math, and account discriminator checks are covered.
- Buyback campaign config rejects zero budgets, overfunding, bad slice limits, and expired windows.
- Buyback slices enforce cadence, deadlines, max size, and AGC burn output.

## Existing Coverage Revalidated

- Policy parameter guardrails.
- Collateral asset config guardrails.
- Credit facility config guardrails.
- Credit draw collateral and underwriter reserve checks.
- Interest accrual accounting.
- Oracle freshness checks.
- Stable-cash and oracle-health expansion blocks.
- Keeper permission scoping.
- Daily mint cap enforcement.
- Mint window reset across UTC day boundaries.
- Recovery cooldown behavior.
- Epoch state rollover.
- xAGC share math.

## Remaining Production Blockers

- On-chain reserve aggregation from configured reserve token accounts.
- Optional venue-specific CPI adapters around the buyback campaign primitive.
- Local-validator integration tests for SPL token transfers, burns, mints, PDA signer paths, account constraints, and pause flags.
- Deployment-runbook tests for multisig authority migration and emergency operations.
- External Solana security review after the integration suite exists.

## Verification

```bash
cargo test --manifest-path solana/programs/agc_solana/Cargo.toml --lib
env PATH=/Users/nate/.cargo/bin:/Users/nate/.local/share/solana/install/active_release/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin anchor build
```

Both passed. `anchor build` still emits the local cargo-build-sbf undefined-symbol warning that has appeared in prior builds, while exiting successfully.

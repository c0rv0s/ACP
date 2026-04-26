# Internal Protocol Audit - 2026-04-26

Scope: `solana/programs/agc_solana/src/lib.rs`.

This pass reviewed the current Anchor implementation, account constraints, credit lifecycle, xAGC accounting, policy settlement, keeper permissions, pause flags, and pure Rust test coverage. It did not run a local validator integration suite because the repo does not currently include one.

## Result

The foundation is stronger after this pass, but it is not yet production complete. The core policy and credit math now have better adversarial coverage, and two credit-lifecycle bugs were fixed. The largest remaining risks are integration-level: verified oracle adapters, on-chain reserve aggregation, atomic buyback execution, and validator-backed tests against real SPL token accounts.

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

## Adversarial Tests Added

- Disabled collateral cannot support credit draws.
- Line-level and facility-level debt caps reject overdraw attempts.
- Underwriter withdrawals cannot leave the facility below required first-loss reserve.
- Repaid credit lines can release collateral without oracle health checks.
- Defaulted lines cannot use the borrower collateral-withdrawal path.
- Matured defaults proceed even if the oracle is stale.
- Immature defaults require both fresh oracle data and bad health.
- Reserve concentration blocks expansion even when demand signals are strong.

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

- Direct Pyth or Switchboard validation for collateral oracle cache updates.
- On-chain reserve aggregation from configured reserve token accounts.
- Atomic buyback executor that swaps escrowed USDC and burns AGC in one controlled flow.
- Local-validator integration tests for SPL token transfers, burns, mints, PDA signer paths, account constraints, and pause flags.
- Deployment-runbook tests for multisig authority migration and emergency operations.
- External Solana security review after the integration suite exists.

## Verification

```bash
cargo test --manifest-path solana/programs/agc_solana/Cargo.toml --lib
env PATH=/Users/nate/.cargo/bin:/Users/nate/.local/share/solana/install/active_release/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin anchor build
```

Both passed. `anchor build` still emits the local cargo-build-sbf undefined-symbol warning that has appeared in prior builds, while exiting successfully.

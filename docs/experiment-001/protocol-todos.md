# AGC Protocol TODOs

Remaining work checklist toward production launch of the Solana program and web surface.

## Solana Program

- [x] Add direct Pyth validation for collateral oracle cache updates.
- [ ] Aggregate reserve value from actual configured reserve token accounts instead of passing reserve metrics manually into settlement.
- [x] Add a constrained buyback campaign executor that releases USDC slices only after AGC is delivered and burned.
- [x] Document the Raydium-first buyback adapter MVP around the constrained campaign primitive.
- [x] Add local-validator Anchor tests for initialization and freeze-authority rejection across real SPL mint/token accounts.
- [x] Add deeper local-validator tests for credit draw, repayment, default, and collateral seizure across real SPL token accounts. Covers credit lifecycle (deposit → draw → repay), xAGC round-trip with exit fee, and default + collateral seize.
- [x] Split monolithic `programs/agc_solana/src/lib.rs` (6.8k lines) into modules: `state`, `events`, `policy`, `credit`, `validation`, `auth`, `math`, `pyth`, `cpi`, `tests`. lib.rs is now 2.8k lines holding only `#[program]` handlers, `#[derive(Accounts)]` contexts, and `#[error_code]`.
- [x] Fix SBF stack-frame overflows in `DrawCreditLine.try_accounts` and `StartBuybackCampaign.try_accounts`. Both produced runtime `Access violation` failures pre-fix; resolved by relocating selected `seeds`/`address` constraints into handler-level `require_keys_eq!` checks.
- [ ] Add migration playbooks for future account-version upgrades.
- [ ] Decide final launch multisig structure for admin, risk, emergency, and upgrade authorities.
- [x] Define launch parameter presets for USDC, USDT, the first BTC wrapper, and disabled RWA assets.

## Frontend

- [x] Wire the dashboard to the deployed Solana IDL/client. ([`web/src/useProtocolState.ts`](../web/src/useProtocolState.ts) fetches `ProtocolState` + token balances on a 5s interval.)
- [x] Read live `ProtocolState`, treasury token accounts, xAGC vault, and AGC/xAGC mint supplies in the hero tiles + operating metrics. Collateral registry / per-facility detail pages still pending.
- [x] Hide the Jupiter swap panel on non-mainnet clusters (it requires mainnet liquidity to be useful). Re-enables automatically when `VITE_SOLANA_CLUSTER=mainnet-beta`.
- [x] Submit live xAGC deposit/redeem transactions from the dashboard buttons. ([`web/src/transactions.ts`](../web/src/transactions.ts) builds instructions and signs via the injected Phantom/Solflare provider.)
- [x] Submit live credit facility transactions: underwrite, deposit collateral, draw, repay. Withdraw still pending. Caller-side prerequisites (admin-opened line, posted collateral) are documented in [`deployment-guide.md`](deployment-guide.md).
- [ ] Add facility detail pages for collateral health, maturity, debt, reserve coverage, underwriter reserve, and default state.
- [ ] Re-enable Jupiter swap panel once AGC has live mainnet liquidity.

## Docs

- [x] Add a deployment guide with exact Solana accounts, PDA seeds, authority setup, and environment variables. ([`deployment-guide.md`](deployment-guide.md))
- [x] Add a risk-parameter reference for stablecoins, BTC wrappers, and isolated RWAs.
- [x] Add user-facing examples for xAGC, credit borrowers, underwriters, and liquidations.
- [x] Add an upgrade/migration runbook explaining production upgrade controls.
- [x] Reconcile `policy-sheet.md §15` recommended launch values with `risk-parameter-presets.md` (single source of truth for launch parameters).
- [ ] Keep `/llms.txt` and `/llms-full.txt` in sync with product docs after every protocol change.

## Tooling & Simulator

- [x] Extend Python policy simulator (`script/simulate_policy.py`) with stable cash coverage, liquidity depth coverage, oracle confidence/staleness, and collateral concentration gates so it mirrors on-chain `evaluate_epoch`.
- [x] Update `configs/policy/launch-model.json` schema with the new policy parameter sections.
- [x] Add `oracle-degradation` scenario to `configs/policy/scenarios.json` exercising the oracle health gate (Neutral → Expansion → Defense → Recovery).
- [x] Remove dead EVM-era `script/simulatePolicy.mjs` and its `simulate:policy` `package.json` script.

## Launch Readiness

- [ ] Complete an external Solana security review. Specifically include the relocated constraint checks in `DrawCreditLine` (credit_line seeds → handler `constraint`) and `StartBuybackCampaign` (treasury/mint `address` constraints → handler `require_keys_eq!`).
- [ ] Extend adversarial test coverage: stale oracle data, wrong mint accounts, underwriter reserve drain, overdraw beyond collateral factor, underwriter loss accounting on uncovered defaults, collateral seizure routing edge cases.
- [x] Deploy program to devnet and publish program ID + on-chain IDL. Program ID `H1n8VTp6pMY5WFfVfi4MNkQ9q5szkMpVWcHQ21JRETXC`.
- [x] Run `initialize_protocol` + seed BTC collateral + open BTC credit facility on devnet via [`script/devnet-bootstrap.ts`](../script/devnet-bootstrap.ts). Full address list in [`docs/deployment-guide.md`](deployment-guide.md). Multisig migration still pending — upgrade authority remains the deployer wallet for staging.
- [x] Drive demo flows from `script/demo/*.ts` (underwrite, borrow, expansion-cycle). The expansion script lands a cycle-2 Expansion settlement and mints across all five recipients per `mint_distribution` bps.
- [ ] Define operational monitoring for policy settlement, oracle freshness, reserve balances, credit health, and emergency pause events.

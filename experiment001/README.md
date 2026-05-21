# Agent Credit Protocol

Agent Credit Protocol is a Solana-native credit machine for autonomous markets. AGC is liquid credit inventory; xAGC owns the long-duration expansion layer.

Source repository: [github.com/c0rv0s/agc](https://github.com/c0rv0s/agc).

The system is built around a reserve and credit asset set:

- `AGC`: liquid credit inventory agents, apps, borrowers, and users hold as working capital
- `xAGC`: the non-rebasing expansion share that captures most expansion
- `USDC` / `USDT`: defensive stablecoin reserve and settlement assets
- BTC wrappers: strategic reserve collateral with haircuts
- RWAs / tokenized stocks: later isolated collateral candidates

The protocol does not target a hard peg. Policy keeps `AGC` inside a stable operating range, expands supply when balance-sheet conditions are strong, and defends with pauses, fees, and treasury buybacks when conditions weaken.

The architecture is framed as a balance-sheet credit machine, not as a swap-volume printer:

```text
AGC demand rises
-> reserves and liquidity deepen
-> credit capacity increases
-> agents and borrowers use credit
-> fees and repayments grow
-> xAGC becomes more valuable
-> confidence and AGC demand increase
```

## Current Architecture

- [`solana/programs/agc_solana/src/lib.rs`](solana/programs/agc_solana/src/lib.rs)
  Anchor program for AGC mint authority, xAGC vault accounting, treasury accounts, collateral registry, credit facilities, policy settlement, buyback budgeting, and governance roles.
- [`solana/README.md`](solana/README.md)
  Solana program build, account, governance, and hardening notes.
- [`web/`](web/)
  Solana product site, AGC console, hosted docs, and AI-readable docs.

## Policy Model

The controller evaluates epochs using market and balance-sheet observables rather than route-specific payment labels:

- premium over anchor
- premium persistence
- gross buy floor
- net buy pressure
- buy growth
- reserve coverage
- lock flow into `xAGC`
- realized volatility
- exit pressure

Expansion mints are split across:

- `xAGC`
- growth programs
- LP incentives
- integrators
- treasury

Contraction is handled by:

- halted expansion
- defense fees
- treasury buybacks and burns

There are no negative rebases in the normal path.

## Website

The AGC console in [`web/`](web/) is the Solana user surface for:

- AGC market entry through Jupiter
- xAGC deposits and redemptions
- credit facility monitoring and transaction surfaces
- policy telemetry
- reserve and regime monitoring
- hosted protocol docs

It also shows live protocol state:

- anchor price
- regime
- premium
- reserve coverage
- stable cash / risk reserve model notes
- exit pressure
- volatility
- locked share
- treasury inventory
- `xAGC` assets and exchange rate
- current epoch flow
- credit principal and facility state after deployment telemetry is wired
- human docs at `/docs`
- AI-readable docs at `/llms.txt` and `/llms-full.txt`

## Local Development

```bash
pnpm build:web
cd solana && anchor build
cargo test --manifest-path solana/programs/agc_solana/Cargo.toml --lib
```

## Planning Docs

- Solana design: [`docs/solana-credit-machine.md`](docs/solana-credit-machine.md)
- Economics and framing: [`docs/economics-spec.md`](docs/economics-spec.md)
- Policy sheet: [`docs/policy-sheet.md`](docs/policy-sheet.md)
- Launch model config: [`configs/policy/launch-model.json`](configs/policy/launch-model.json)
- Scenario pack: [`configs/policy/scenarios.json`](configs/policy/scenarios.json)
- Epoch simulator (Python): [`script/simulate_policy.py`](script/simulate_policy.py)

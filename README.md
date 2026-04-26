# Agent Credit Protocol

Agent Credit Protocol is a Solana-native credit machine for autonomous markets. AGC is liquid credit inventory; xAGC owns the long-duration expansion layer.

The system is built around a reserve and credit asset set:

- `AGC`: liquid credit inventory agents, apps, borrowers, and users hold as working capital
- `xAGC`: the non-rebasing expansion share that captures most expansion
- `USDC` / `USDT`: defensive stablecoin reserve and settlement assets
- BTC wrappers: strategic reserve collateral with haircuts
- RWAs / tokenized stocks: later isolated collateral candidates

The protocol does not target a hard peg. Policy keeps `AGC` inside a stable operating range, expands supply when balance-sheet conditions are strong, and defends with pauses, fees, and treasury buybacks when conditions weaken.

The current Solana target is a balance-sheet credit machine rather than a swap-volume printer:

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

- [`/Users/nate/Desktop/agc/solana/programs/agc_solana/src/lib.rs`](/Users/nate/Desktop/agc/solana/programs/agc_solana/src/lib.rs)
  Anchor program for AGC mint authority, xAGC vault accounting, treasury accounts, collateral registry, credit facilities, policy settlement, buyback budgeting, and governance roles.
- [`/Users/nate/Desktop/agc/solana/README.md`](/Users/nate/Desktop/agc/solana/README.md)
  Solana program build, account, governance, and hardening notes.
- [`/Users/nate/Desktop/agc/web`](/Users/nate/Desktop/agc/web)
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

The AGC console in [`/Users/nate/Desktop/agc/web`](/Users/nate/Desktop/agc/web) is the Solana user surface for:

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

- Solana credit-machine design: [`/Users/nate/Desktop/agc/docs/solana-credit-machine.md`](/Users/nate/Desktop/agc/docs/solana-credit-machine.md)
- Rewrite spec: [`/Users/nate/Desktop/agc/docs/rewrite-spec.md`](/Users/nate/Desktop/agc/docs/rewrite-spec.md)
- Policy sheet: [`/Users/nate/Desktop/agc/docs/policy-sheet.md`](/Users/nate/Desktop/agc/docs/policy-sheet.md)
- Launch model config: [`/Users/nate/Desktop/agc/configs/policy/acp-launch-model.json`](/Users/nate/Desktop/agc/configs/policy/acp-launch-model.json)
- Scenario pack: [`/Users/nate/Desktop/agc/configs/policy/acp-scenarios.json`](/Users/nate/Desktop/agc/configs/policy/acp-scenarios.json)
- Python simulator: [`/Users/nate/Desktop/agc/script/simulate_acp.py`](/Users/nate/Desktop/agc/script/simulate_acp.py)

## Legacy Note

The in-repo facilitator and reward-receipt path is legacy planning residue and is no longer part of the active v1 contract or web flow.

# Agent Credit Protocol

Agent Credit Protocol is a pre-launch v1 implementation of reserve-efficient onchain credit for autonomous markets.

The system is built around three assets:

- `AGC`: the liquid credit asset agents hold as working capital
- `xAGC`: the non-rebasing savings share that captures most expansion
- `USDC`: the reserve and settlement asset

The protocol does not target a hard peg. Policy tries to keep `AGC` inside a stable operating range, expand supply when demand and reserve conditions are strong, and defend with fees plus treasury buybacks when conditions weaken.

## Current V1 Architecture

- [`/Users/nate/Desktop/agc/src/AGCToken.sol`](/Users/nate/Desktop/agc/src/AGCToken.sol)
  Role-gated `AGC` token with mint and burn permissions.
- [`/Users/nate/Desktop/agc/src/XAGCVault.sol`](/Users/nate/Desktop/agc/src/XAGCVault.sol)
  Non-rebasing vault-share token for locked `AGC`, with an exit fee routed to treasury.
- [`/Users/nate/Desktop/agc/src/StabilityVault.sol`](/Users/nate/Desktop/agc/src/StabilityVault.sol)
  Treasury inventory for `USDC` and protocol-held `AGC`.
- [`/Users/nate/Desktop/agc/src/AGCHook.sol`](/Users/nate/Desktop/agc/src/AGCHook.sol)
  Canonical Uniswap v4 hook tracking buy volume, sell volume, volatility, hook fees, and LP behavior.
- [`/Users/nate/Desktop/agc/src/PolicyEngine.sol`](/Users/nate/Desktop/agc/src/PolicyEngine.sol)
  Pure epoch math for expansion, defense, recovery, anchor crawl, and buyback budgets.
- [`/Users/nate/Desktop/agc/src/PolicyController.sol`](/Users/nate/Desktop/agc/src/PolicyController.sol)
  Epoch settlement, mint distribution, queued treasury buybacks, and regime state.
- [`/Users/nate/Desktop/agc/src/SettlementRouter.sol`](/Users/nate/Desktop/agc/src/SettlementRouter.sol)
  Direct user buy/sell flow against the canonical `AGC/USDC` pool, plus controller-driven treasury buybacks.

## Policy Model

The controller evaluates epochs using generic market observables rather than route-specific payment labels:

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

The dashboard in [`/Users/nate/Desktop/agc/web`](/Users/nate/Desktop/agc/web) is aligned to the current contracts and supports the full user interaction surface:

- approve `USDC` for router buys
- buy `AGC`
- approve `AGC` for router sells
- sell `AGC`
- approve `AGC` for `xAGC`
- deposit into `xAGC`
- redeem `xAGC`

It also shows live protocol state:

- anchor price
- regime
- premium
- reserve coverage
- exit pressure
- volatility
- locked share
- treasury inventory
- `xAGC` assets and exchange rate
- current hook epoch flow

## Local Development

```bash
pnpm test
pnpm generate:abis
pnpm build:web
pnpm deploy:local
```

`pnpm deploy:local`:

- deploys the v1 contract set
- mines a valid Uniswap v4 hook address
- initializes the canonical pool around the `0.50` launch anchor
- seeds treasury `USDC`, user inventory, and initial `xAGC`
- writes [`/Users/nate/Desktop/agc/deployments/local.json`](/Users/nate/Desktop/agc/deployments/local.json)
- writes [`/Users/nate/Desktop/agc/web/.env.local`](/Users/nate/Desktop/agc/web/.env.local)

## Planning Docs

- Rewrite spec: [`/Users/nate/Desktop/agc/docs/rewrite-spec.md`](/Users/nate/Desktop/agc/docs/rewrite-spec.md)
- Policy sheet: [`/Users/nate/Desktop/agc/docs/policy-sheet.md`](/Users/nate/Desktop/agc/docs/policy-sheet.md)
- Launch model config: [`/Users/nate/Desktop/agc/configs/policy/acp-launch-model.json`](/Users/nate/Desktop/agc/configs/policy/acp-launch-model.json)
- Scenario pack: [`/Users/nate/Desktop/agc/configs/policy/acp-scenarios.json`](/Users/nate/Desktop/agc/configs/policy/acp-scenarios.json)
- Python simulator: [`/Users/nate/Desktop/agc/script/simulate_acp.py`](/Users/nate/Desktop/agc/script/simulate_acp.py)

## Legacy Note

The in-repo facilitator and reward-receipt path is legacy planning residue and is no longer part of the active v1 contract or web flow.

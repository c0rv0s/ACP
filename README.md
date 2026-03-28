# Agent Credit Protocol

Foundry + Uniswap v4 MVP for the Agent Credit Protocol, with a small wagmi/viem dashboard under [`/Users/nate/Desktop/agc/web`](/Users/nate/Desktop/agc/web).

The repo implements the narrow v1 described in the spec:

- `AGCToken` with role-gated mint/burn
- canonical `AGC/USDC` v4 pool hook with dynamic fees, productive-flow receipts, oracle-style epoch accounting, and LP anti-JIT penalties
- `PolicyController` for epoch settlement, mint caps, defense buybacks, and reward routing
- `StabilityVault`, `RewardDistributor`, and `SettlementRouter`
- local deployment script that mines a valid v4 hook address, initializes the pool, seeds liquidity, and writes frontend env vars

## Commands

Install:

```bash
pnpm install
forge install
```

Build contracts:

```bash
pnpm build:contracts
```

Run tests:

```bash
pnpm test
```

Build the frontend:

```bash
pnpm build:web
```

Generate ABI bindings for the frontend:

```bash
pnpm generate:abis
```

## Local deploy

Start Anvil, then run:

```bash
export PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
pnpm deploy:local
pnpm build:web
```

The deploy script writes:

- [`/Users/nate/Desktop/agc/deployments/local.json`](/Users/nate/Desktop/agc/deployments/local.json)
- [`/Users/nate/Desktop/agc/web/.env.local`](/Users/nate/Desktop/agc/web/.env.local)

Those outputs give the wagmi app the deployed contract addresses for a local Anvil session.

## Foundry note

This codebase currently needs split compilation:

- `AGCHook` compiles cleanly in the regular pipeline
- `PolicyController`, router-facing test suites, and reward/policy suites are run with `--via-ir`

The package scripts already encode the working build and test matrix, so use those instead of plain `forge build` / `forge test` at the repo root. Local deployment is handled by the viem script in [`/Users/nate/Desktop/agc/script/deployLocal.mjs`](/Users/nate/Desktop/agc/script/deployLocal.mjs), which queries bytecode and ABIs from `forge inspect`.

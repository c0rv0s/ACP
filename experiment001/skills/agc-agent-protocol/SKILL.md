---
name: agc-agent-protocol
description: Explain how Agent Credit Protocol works on Solana — AGC and xAGC roles, reserve and policy model, credit facilities, treasury buyback campaigns, and where to read authoritative source. Use when users ask for protocol walkthroughs, integration guidance, Solana program behavior, or how the on-chain design relates to the written spec.
---

# AGC Agent Protocol (Solana)

## Overview

Default to the **Solana program** and the docs listed below as the integration surface.

Authoritative sources:

- [`solana/programs/agc_solana/src/lib.rs`](../../solana/programs/agc_solana/src/lib.rs) — Anchor program (mint authority, vaults, collateral registry, credit facilities, epoch settlement, buyback campaigns).
- [`solana/README.md`](../../solana/README.md) — Build, accounts, governance, operational notes.
- [`docs/solana-credit-machine.md`](../../docs/solana-credit-machine.md) — Mechanism design on Solana.
- [`README.md`](../../README.md) — Repository map and planning-doc index.

## Workflow

### 1. Identify the caller’s role

Tailor to agent operators, underwriters, borrowers, integrators, or governance/multisig operators.

### 2. Describe assets clearly

- **AGC** — Liquid credit inventory (SPL mint; program-held mint authority).
- **xAGC** — Non-rebasing expansion share (SPL mint; backed by program vault accounting).
- **USDC / USDT** — Defensive reserves; BTC wrappers and RWAs are collateral classes with guards in the program and docs.

### 3. Explain demand and policy without a single-router story

Swaps can happen via Jupiter, wallets, and bots. Optional venue/market reporting is **telemetry**, not the sole source of global demand. Epoch policy uses balance-sheet observables (reserves, liquidity, oracles, credit quality, etc.) as described in `docs/solana-credit-machine.md` and `docs/policy-sheet.md`.

### 4. Credit facilities

Credit lines draw **AGC** against approved collateral, oracle-cache freshness, facility and line caps, and underwriter reserve rules. Point implementers at the program’s credit instructions and account layout in `solana/README.md`.

### 5. Treasury buybacks

Defense paths budget buybacks; execution uses **campaign-style escrows** that release USDC only under the enforced on-chain invariant (AGC delivered and burned). See `docs/raydium-buyback-adapter.md` for the execution-client model around that primitive.

### 6. Trust and status

- Governance is explicit (admin / risk / emergency / keeper scopes); production intent is multisig-held authorities.
- Be honest about what is fully wired in the **web console** versus **program-complete**: check `docs/protocol-todos.md` and the frontend for live IDL wiring.

## Spec versus implementation

If the user asks for the broader economic thesis, use `docs/economics-spec.md` and `docs/solana-credit-machine.md`, and call out anything not yet reflected in the deployed program or UI.

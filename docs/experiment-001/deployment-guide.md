# AGC Deployment Guide

How to build, deploy, and bootstrap the Agent Credit Protocol on a Solana cluster.

## Live deployments

| Cluster | Program ID | ProgramData | Upgrade Authority | Notes |
| --- | --- | --- | --- | --- |
| devnet | `H1n8VTp6pMY5WFfVfi4MNkQ9q5szkMpVWcHQ21JRETXC` | `2N7Q6hNieanqQdRWox47C2fV7xXwV2c7hFtSWqXZSHw6` | `HDm9RuTnD3dhHq6xBykvWvoJdX1tAyirywYqjpBikrXV` (deployer wallet) | Sized `--max-len 1100000`; protocol initialized + seeded with a BTC credit facility |

### Devnet protocol state (from `script/devnet-bootstrap.ts`)

| Account | Address |
| --- | --- |
| `state` | `2y2au7Fo1MEaXZzP8TknDrDfX2uezojkqaCxUBj6fMQS` |
| AGC mint | `BgEmfYvG48d93QHw5aBrRszdDobXTVPTdagg9fXwaP9D` |
| xAGC mint | `8N3f2iQVzUxh4k8CB2ZDzBy2HUn4wy1YMKVck63Hbbrz` |
| USDC mint (mock) | `BCAw89QFbg1Zv7ZquAeKaCsF4t2V6FVCTQvDeK6Aawz9` |
| BTC mint (mock collateral) | `DgqjKgCh3SnPhCEvEzpeLAviQLBk2VWsPSQEbekY87G2` |
| `treasury_agc` | `CfWtYbg4oJL3yWWCJscGL86U9Q5a57QMkHFmowxjdJ5R` |
| `treasury_usdc` | `J17zi2jdk4ManR47YQyT7t6Hxpq2xacrKrSsq2UH5o1T` |
| `xagc_vault_agc` | `HJzwXgP3LJR8WbpxeVQDkK8ZBqmMrjoGk7zS8LRMUuSc` |
| BTC `collateral_asset` | `MpBeEH2t1k6wfRjcKcvZucapTx3nc9bivTtDnJ8iNVq` |
| BTC `collateral_oracle` | `H5rGTPuDJoCVLkfUjqBJNN1yGsjxC1uQd2KYSdiacG65` |
| BTC `credit_facility` | `ASJXw3NmVdPpkHVdrf6NXfRpChGDUeaHskEapNuHbmmq` |
| BTC `underwriter_vault_agc` | `3oV1UfH4wxQYcowk3MmfhwWLCBuJNmK1QKbKTSMfSoaV` |

Initial state: 10M AGC pre-minted to the deployer wallet; BTC oracle at $100k; credit facility limits 100k AGC per line, 1M AGC total. The complete address list (including PDAs and settlement-recipient ATAs) lives in `deployments/devnet.json` after running the bootstrap.

### Dashboard buttons (interactive demo)

The dashboard at `pnpm dev:web` wires all six action buttons to live program calls via Anchor + the injected Solana wallet:

| Button | Instruction | Caller needs |
| --- | --- | --- |
| `Deposit AGC` | `deposit_xagc` | AGC in the connected wallet |
| `Redeem xAGC` | `redeem_xagc` | xAGC shares in the connected wallet |
| `Underwrite AGC` | `deposit_underwriter_agc` | AGC in the connected wallet |
| `Deposit collateral` | `deposit_credit_collateral` | BTC + an admin-opened line for the wallet |
| `Draw AGC` | `draw_credit_line` | An admin-opened line + posted collateral |
| `Repay AGC` | `repay_credit_line` | An admin-opened line with debt + AGC to repay with |

**Demo-wallet setup for the video:** import the deployer keypair (`~/.config/solana/id.json`) into Phantom or Solflare and switch the wallet to **Devnet**. That wallet already holds 10M AGC + ~1 BTC from the bootstrap and has a pre-opened credit line (`line_id=1`) from the `borrow.ts` demo run. With it connected, all six buttons work directly.

For visitors that connect with an empty wallet, the xAGC and underwrite buttons will return "insufficient funds" — the credit-side buttons additionally need a line opened by the admin first. There is no public faucet; if you want to give a visitor demo tokens, transfer AGC/BTC from the deployer wallet and run `script/demo/borrow.ts` against their pubkey (or generalize that script with a `--borrower` flag).

### Demo scripts (`script/demo/*.ts`)

Run with `pnpm exec tsx script/demo/<name>.ts`. All drive transactions against the devnet program above using the deployer wallet.

- `underwrite.ts [amount-agc]` — deposit AGC underwriter capital into the BTC facility. The dashboard's "Underwriter vault" tile fills in.
- `borrow.ts [collateral-btc] [draw-agc] [line-id]` — open a credit line, post BTC collateral, draw AGC. "Credit drawn (cumulative)" tile moves; borrower AGC balance grows.
- `expansion-cycle.ts` — lowers policy params to a 2-second epoch, records two cycles of buy/sell swaps with growth between them, deposits 10k AGC to xAGC for positive `lock_flow`, settles both. Cycle 2 lands in Expansion and AGC is minted to all five recipients per the configured bps. Dashboard's Regime tile flips, AGC supply jumps.

## Cluster Choice

Solana exposes four public clusters. For staging this protocol use **devnet** — it is the convention for application testing, accepts airdrop requests, and is what wallets (Phantom, etc.) speak to when set to "Devnet". Solana's `testnet` cluster is primarily for validator software, not application deployment; mention of "testnet" in protocol contexts almost always means devnet.

## Program ID

The program ID is committed to the repo as `solana/target/deploy/agc_solana-keypair.json`. Its pubkey is `H1n8VTp6pMY5WFfVfi4MNkQ9q5szkMpVWcHQ21JRETXC` and is hard-coded into `declare_id!(...)` at the top of `solana/programs/agc_solana/src/lib.rs`. Do not regenerate the keypair unless you intend to deploy under a new ID — Anchor will refuse to deploy if the keypair and the `declare_id!` diverge, and external integrators will lose any pinned references.

## Build

```bash
cd solana
anchor build
```

This produces:

- `target/deploy/agc_solana.so` — the BPF program binary (~1.1 MB)
- `target/idl/agc_solana.json` — the Anchor IDL (frontends and SDKs read this)
- `target/types/agc_solana.ts` — TypeScript types for the IDL

`solana/target/` is gitignored, so after every program rebuild that changes the IDL run `pnpm sync:idl` from the repo root. That copies the two files into `web/src/agc/` (committed) so Vercel can bundle them — the frontend imports from there, not from `solana/target/`.

## Deploy (devnet)

```bash
# 1. Point Solana CLI at devnet.
solana config set --url https://api.devnet.solana.com

# 2. Fund the upgrade authority wallet (~3 SOL is comfortable for ~1 MB program).
solana airdrop 2
solana airdrop 2     # repeat if needed
solana balance

# 3. Deploy.
anchor deploy --provider.cluster devnet

# 4. (Optional) Upload the IDL on-chain so clients can fetch it via the program ID.
anchor idl init --provider.cluster devnet \
  --filepath target/idl/agc_solana.json \
  H1n8VTp6pMY5WFfVfi4MNkQ9q5szkMpVWcHQ21JRETXC
```

If `anchor deploy` fails partway through and leaves a buffer account behind, recover the SOL with:

```bash
solana program close --buffers --keypair ~/.config/solana/id.json
```

## Initialize the protocol

Before any other instruction can run, the admin must call `initialize_protocol` once. The accounts and flow are identical to the integration tests in [`solana/tests/agc_solana.ts`](../solana/tests/agc_solana.ts). The high-level sequence:

1. Admin creates the AGC mint with admin as the temporary mint authority (so admin can pre-mint launch supply to integrators, LPs, growth programs).
2. Admin pre-mints whatever initial supply is appropriate (see launch-model.json `initialState`).
3. Admin transfers AGC mint authority to the `mint-authority` PDA derived from the program ID.
4. Admin creates the xAGC mint with the `mint-authority` PDA as the mint authority from inception.
5. Admin creates ATAs for the growth, LP, and integrators settlement-recipient accounts (these are the destinations the policy controller will mint into during expansion settlements).
6. Admin calls `initialize_protocol` with the policy parameters from `configs/policy/launch-model.json` and the settlement-recipient pubkeys. The program PDA-creates `state`, `treasury_agc`, `treasury_usdc`, and `xagc_vault_agc`.

## Post-init operational notes

### Warm up the price accumulator immediately after init

`initialize_protocol` creates the epoch accumulator with `last_mid_price_x18 = initial_anchor_price_x18` and `last_observed_at = now`. If no `record_swap` or `record_market_observation` lands for an extended period, that idle period at the initial-anchor price gets folded into the first epoch's TWAP. With a stale accumulator, the first `settle_epoch` can see an artificially high or low TWAP — driving the controller into Defense (volatility spike, price below stressed floor) even when the live market is calm.

**Mitigation:** the keeper that will be reporting market flow should call `record_market_observation` (or its first `record_swap`) within the same epoch as `initialize_protocol`. If the deployment workflow has a gap between init and first observation, call `record_market_observation` once with the initial anchor price to reset `last_observed_at` cleanly before any real swaps land.

### Configure a Pyth receiver before enabling Pyth-source collateral

`set_collateral_asset` with `oracle_source = Pyth` requires `state.pyth_receiver_program` to be a real Pyth Receiver program ID. Call `set_pyth_receiver_program` before configuring any Pyth-source collateral assets; Manual-source collateral does not need it.

### Lock down upgrade authority before launch

The default upgrade authority is the deployer wallet. For production, transfer upgrade authority to the launch multisig (Squads or equivalent) via:

```bash
solana program set-upgrade-authority H1n8VTp6pMY5WFfVfi4MNkQ9q5szkMpVWcHQ21JRETXC --new-upgrade-authority <MULTISIG>
```

For devnet/staging it is fine to leave it on the deployer wallet.

## Verifying a devnet deployment

```bash
# Program exists and is executable
solana program show H1n8VTp6pMY5WFfVfi4MNkQ9q5szkMpVWcHQ21JRETXC --url devnet

# IDL fetchable
anchor idl fetch --provider.cluster devnet H1n8VTp6pMY5WFfVfi4MNkQ9q5szkMpVWcHQ21JRETXC > /tmp/idl.json
```

## Re-running the integration tests against a deployed cluster

The default test script in `Anchor.toml` spins up a local validator. To exercise the same suite against a deployed devnet program, set `provider.cluster` to devnet and skip the local-validator harness:

```bash
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com \
ANCHOR_WALLET=~/.config/solana/id.json \
pnpm exec ts-mocha -p solana/tsconfig.json -t 1000000 solana/tests/**/*.ts
```

Note that the integration tests `init`-the protocol each run, so they can only run once against any given deployed instance unless the state PDA is closed between runs (which Anchor does not provide out of the box).

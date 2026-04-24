# AGC Solana Program

This folder contains an Anchor port of the portable Agent Credit Protocol core.

The Solana version maps the EVM contracts as follows:

- `AGCToken.sol`: SPL `AGC` mint with this program's mint-authority PDA.
- `XAGCVault.sol`: non-rebasing `xAGC` SPL share mint backed by the program-owned AGC vault token account.
- `StabilityVault.sol`: program-owned treasury AGC and USDC token accounts.
- `AGCHook.sol`: `record_swap` / `record_market_observation` adapter boundary for a Solana DEX integration.
- `PolicyEngine.sol` and `PolicyController.sol`: on-chain epoch settlement, regime selection, expansion mint distribution, and treasury buyback budgeting.

The actual AMM swap path is intentionally not hard-wired into this program. A Raydium, Orca, Phoenix, or custom venue adapter should call the market-recording instructions with canonical AGC/USDC flow data, then use the queued treasury buyback budget to execute venue-specific swaps.

## Build

```bash
cd solana
anchor build
```

For the pure Rust policy tests:

```bash
cargo test --manifest-path programs/agc_solana/Cargo.toml --lib
```

## Program Accounts

- `ProtocolState`: global protocol configuration, policy parameters, regime state, epoch accumulator, xAGC vault counters, and settlement recipients.
- `Keeper`: optional authorization account with scoped permissions for off-admin market reporting, epoch settlement, buyback execution, and treasury burns.
- `treasury_agc`: program-owned AGC token account.
- `treasury_usdc`: program-owned USDC token account.
- `xagc_vault_agc`: program-owned AGC token account backing xAGC shares.
- `buyback_usdc_escrow`: admin-configured USDC token account that receives reserved buyback budget.

## Token Authorities

The initializer must provide existing SPL mints whose mint authority is the program PDA derived from `["mint-authority"]`.

AGC and xAGC use the same mint-authority PDA and must not have an external freeze authority. Treasury and xAGC vault token accounts are owned by separate PDAs so the program can transfer vault assets, burn treasury AGC, and mint policy allocations without any external private key.

## Hardening Notes

- Market-reporting instructions can be called by admin, a keeper with market-reporting permission, or the configured `market_adapter_authority`. For production, set `market_adapter_authority` to a PDA controlled by the canonical venue adapter so swap reporting must happen through that adapter CPI.
- Treasury buyback reservations can only transfer USDC to the configured `buyback_usdc_escrow`. A production venue adapter should still execute the swap and burn atomically, or hold funds in a nonce/deadline-constrained escrow until the burn leg settles.
- Admin transfer is two-step through `transfer_admin` and `accept_admin`.
- Pause flags can independently halt xAGC deposits, xAGC redemptions, market reporting, epoch settlement, buyback reservations, and treasury burns.

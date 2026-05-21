# Raydium Buyback Adapter MVP

The onchain buyback campaign is the hard safety primitive. It escrows USDC, accepts AGC into the campaign vault, burns that AGC, and only then releases the matching USDC slice.

The Raydium adapter sits around that primitive. It is an execution client that routes each slice through Raydium liquidity, then calls `execute_buyback_twap_slice` after the AGC has reached the campaign vault.

## Why The Adapter Is Offchain First

A fully hard-coded onchain route is easy to reason about but poor execution. Large public swaps along a fixed path invite slippage and sandwiching. The MVP keeps the route selected by the operator while the program enforces the part that matters: USDC cannot leave the campaign unless AGC is delivered and burned.

## Slice Flow

1. Operator selects an active campaign and a USDC slice size.
2. Adapter checks campaign limits, interval, deadline, and remaining budget.
3. Adapter obtains a Raydium route for USDC -> AGC.
4. Adapter sets a minimum AGC output and expiry.
5. Swap output is sent to the campaign AGC vault.
6. Adapter calls `execute_buyback_twap_slice`.
7. Program burns AGC from the campaign vault and releases the USDC slice to the configured adapter destination.

## MVP Constraints

- Only the configured campaign adapter destination can receive USDC.
- `max_slice_usdc` limits per-slice execution.
- `slice_interval_seconds` prevents one campaign from draining in a single burst.
- `min_agc_out` and `min_total_agc_out` enforce execution quality.
- `deadline` prevents stale execution.

## Production Path

The first production adapter should target the deepest AGC venue at launch. Raydium is the expected first venue if AGC launches with the best liquidity there. If Jupiter or Orca becomes the better execution path, the adapter client can route there without weakening the onchain campaign invariant.

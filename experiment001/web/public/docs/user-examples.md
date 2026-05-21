# User Examples

These examples explain how people interact with AGC from the outside. They are written for users deciding whether the system is understandable enough to trust with capital.

## Holder

Maya wants exposure to the AGC credit network. She buys AGC through a normal Solana swap route, then decides how much to keep liquid and how much to lock into xAGC.

Her liquid AGC is useful inventory. Her xAGC is the longer-duration position. If the protocol earns expansion, xAGC receives a large share of new AGC and the vault exchange rate rises.

The risk is straightforward: AGC is not a guaranteed-dollar stablecoin. It is a managed credit asset. Maya is betting that credit demand, reserves, and policy discipline keep the network growing.

## Underwriter

Jordan deposits AGC into a facility reserve. That reserve is first-loss capital behind borrower debt. In return, Jordan earns the spread assigned to that facility.

The upside is recurring credit income. The risk is that defaults consume underwriter reserve before the system fully recovers value from seized collateral.

This is not passive staking. It is underwriting a specific credit sleeve.

## Borrower

An automated trading service wants AGC working capital. It deposits approved collateral, opens a credit line, and draws AGC inside the facility limit.

The borrower keeps the line healthy by maintaining collateral value and repaying principal plus interest. If collateral falls or the line matures unpaid, the facility can move into liquidation/default flow.

The useful part is speed: the borrower receives native Solana credit inventory without selling long-term collateral.

## Defense Event

AGC trades below its stressed band while liquidity thins. Expansion shuts off. The protocol queues a buyback campaign using defensive cash.

The campaign does not release USDC as a blind transfer. AGC must arrive in the campaign vault and be burned before each USDC slice leaves escrow. Operators can run the campaign in slices so the defense does not advertise one giant predictable swap.

Holders can inspect the campaign state, remaining USDC, burned AGC, deadlines, and whether the campaign completed.

## Liquidation

A credit line becomes unsafe or defaults. The protocol can seize collateral into the configured reserve path and burn underwriter AGC reserve according to the facility rules.

That outcome is bad for the borrower and underwriters, but it is the point of the structure: losses are absorbed inside the facility before they become an unlimited liability for the whole network.

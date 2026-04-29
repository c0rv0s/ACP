# Risk Parameter Presets

These presets are the launch baseline for the first collateral set. They are intentionally conservative: the protocol can grow faster after live liquidity, oracle behavior, borrow demand, and reserve operations are proven under load.

## Launch Assets

AGC starts with three reserve/collateral families:

- USDC: primary defensive cash and the main AGC market quote asset.
- USDT: secondary defensive cash with its own concentration cap.
- BTC wrapper: strategic reserve collateral with a larger haircut because the asset is volatile and wrapper risk matters.

The BTC wrapper must be selected from the most liquid Solana venue set at deployment time. cbBTC is the current default candidate because Coinbase has launched cbBTC on Solana and it has broad ecosystem routing support, but the final mint is chosen from live Jupiter/Raydium/Orca liquidity and Pyth feed availability during deployment prep.

## Stablecoin Presets

| Asset | Reserve weight | Collateral factor | Liquidation threshold | Concentration cap | Max oracle age | Max confidence |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| USDC | 9,900 bps | 9,000 bps | 9,500 bps | 4,500 bps | 60 sec | 50 bps |
| USDT | 9,700 bps | 8,500 bps | 9,250 bps | 3,500 bps | 60 sec | 75 bps |

USDC receives the cleanest treatment because it is the primary quote market and defense asset. USDT is useful capacity, but it does not replace USDC as the operating cash layer.

## BTC Wrapper Preset

| Asset | Reserve weight | Collateral factor | Liquidation threshold | Concentration cap | Max oracle age | Max confidence |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| BTC wrapper | 6,000 bps | 4,500 bps | 6,000 bps | 3,000 bps | 90 sec | 150 bps |

BTC creates balance-sheet upside when it appreciates, but that upside is not treated like cash. The haircut protects the protocol from drawdowns, bridge/custody risk, and thin exit liquidity during stress.

## RWA Preset

New RWAs launch isolated:

| Asset | Reserve weight | Collateral factor | Liquidation threshold | Concentration cap | Default state |
| --- | ---: | ---: | ---: | ---: | --- |
| Tokenized stock/RWA | 0 bps global | 0 bps global | Per-facility only | 0 bps global | Disabled |

An RWA graduates only after issuer risk, oracle coverage, venue liquidity, redemption mechanics, legal wrapper, market-hours behavior, and liquidation process are understood in production-like conditions.

## Policy Preset

| Parameter | Launch value |
| --- | ---: |
| Max mint per epoch | 100 bps |
| Max mint per day | 250 bps |
| Target reserve coverage | 8,000 bps |
| Expansion reserve coverage floor | 3,000 bps |
| Target stable cash coverage | 2,500 bps |
| Defense stable cash floor | 800 bps |
| Max reserve concentration | 6,000 bps |
| Max expansion volatility | 300 bps |
| Defense volatility | 1,000 bps |
| Recovery cooldown | 2 epochs |

The preset favors a live system that can survive mistakes. Growth parameters are loosened only after devnet testing, external review, and real market telemetry support the change.

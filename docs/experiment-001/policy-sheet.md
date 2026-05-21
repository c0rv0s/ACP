# AGC Policy Sheet

This document is the compact policy definition for AGC.

Use it as the source of truth for simulation and later implementation.

Companion files:

- framing and product language: [`economics-spec.md`](economics-spec.md)
- simulator parameter model: [`../configs/policy/launch-model.json`](../configs/policy/launch-model.json)

## 1. Units

Simulator convention:

- AGC amounts use `1e18`
- quote notionals use normalized `1e18`
- prices use `X18`
- percentages use basis points

If real market data comes in raw USDC `6` decimals, normalize to `1e18` before running the model.

## 2. State

Carry these between epochs:

- `anchorPriceX18`
- `premiumPersistenceEpochs`
- `lastGrossBuyQuoteX18`
- `mintedTodayAgc`
- `lastRegime`
- `recoveryCooldownEpochsRemaining`
- `floatSupplyAgc`
- `treasuryQuoteX18`
- `treasuryAgc`
- `xagcTotalAssetsAgc`

## 3. Epoch Inputs

Per epoch the simulator needs:

- `priceTwapX18`
- `grossBuyQuoteX18`
- `grossSellQuoteX18`
- `totalVolumeQuoteX18`
- `depthToTargetSlippageQuoteX18`
- `stableCashReserveQuoteX18`
- `riskWeightedReserveQuoteX18`
- `liquidityDepthQuoteX18`
- `largestCollateralConcentrationBps`
- `oracleConfidenceBps`
- `staleOracleCount`
- `realizedVolatilityBps`
- `xagcDepositsAgc`
- `xagcGrossRedemptionsAgc`
- `treasuryQuoteInflowX18`

## 4. Derived Metrics

Definitions:

- `BPS = 10_000`
- `creditOutstandingQuoteX18 = floatSupplyAgc * anchorPriceX18 / 1e18`
- `grossBuyFloorBps = grossBuyQuoteX18 * BPS / creditOutstandingQuoteX18`
- `netBuyQuoteX18 = max(grossBuyQuoteX18 - grossSellQuoteX18, 0)`
- `netBuyPressureBps = netBuyQuoteX18 * BPS / creditOutstandingQuoteX18`
- `buyGrowthBps = lastGrossBuyQuoteX18 == 0 ? 0 : max(grossBuyQuoteX18 - lastGrossBuyQuoteX18, 0) * BPS / lastGrossBuyQuoteX18`
- `exitPressureBps = totalVolumeQuoteX18 == 0 ? 0 : grossSellQuoteX18 * BPS / totalVolumeQuoteX18`
- `reserveCoverageBps = creditOutstandingQuoteX18 == 0 ? 0 : riskWeightedReserveQuoteX18 * BPS / creditOutstandingQuoteX18`
- `stableCashCoverageBps = creditOutstandingQuoteX18 == 0 ? 0 : stableCashReserveQuoteX18 * BPS / creditOutstandingQuoteX18`
- `liquidityDepthCoverageBps = creditOutstandingQuoteX18 == 0 ? 0 : liquidityDepthQuoteX18 * BPS / creditOutstandingQuoteX18`
- `lockedShareBps = floatSupplyAgc == 0 ? 0 : xagcTotalAssetsAgc * BPS / floatSupplyAgc`
- `xagcExitFeeAgc = xagcGrossRedemptionsAgc * xagcExitFeeBps / BPS`
- `xagcNetRedemptionAgc = xagcGrossRedemptionsAgc - xagcExitFeeAgc`
- `xagcNetDepositsAgc = xagcDepositsAgc - xagcGrossRedemptionsAgc`
- `lockFlowBps = floatSupplyAgc == 0 ? 0 : max(xagcNetDepositsAgc, 0) * BPS / floatSupplyAgc`
- `premiumBps = priceTwapX18 > anchorPriceX18 ? (priceTwapX18 - anchorPriceX18) * BPS / anchorPriceX18 : 0`

## 5. Anchor

Inputs:

- `normalBandBps`
- `stressedBandBps`

Launch anchor:

- `anchorPriceX18 = initialAnchorPriceX18`

Update rule:

- `anchorNextX18 = ema(anchorPriceX18, priceTwapX18, anchorEmaBps)`
- clamp anchor movement by `maxAnchorCrawlBps` per epoch

Reference implementation:

- `ema = (anchorPriceX18 * (BPS - anchorEmaBps) + priceTwapX18 * anchorEmaBps) / BPS`
- `anchorMin = anchorPriceX18 * (BPS - maxAnchorCrawlBps) / BPS`
- `anchorMax = anchorPriceX18 * (BPS + maxAnchorCrawlBps) / BPS`
- `anchorNextX18 = min(max(ema, anchorMin), anchorMax)`

Band edges:

- `normalFloorX18 = anchorPriceX18 * (BPS - normalBandBps) / BPS`
- `stressedFloorX18 = anchorPriceX18 * (BPS - stressedBandBps) / BPS`

## 6. Regimes

### 6.1 Defense

Enter `Defense` if any are true:

- `priceTwapX18 < stressedFloorX18`
- `reserveCoverageBps < defenseReserveCoverageBps`
- `stableCashCoverageBps < defenseStableCashCoverageBps`
- `oracleConfidenceBps > maxOracleConfidenceBps`
- `staleOracleCount > maxStaleOracleCount`
- `realizedVolatilityBps >= defenseVolatilityBps`
- `exitPressureBps >= defenseExitPressureBps`

### 6.2 Recovery

Enter `Recovery` if:

- prior regime was `Defense` and
- `recoveryCooldownEpochsRemaining > 0` and
- current epoch is not still `Defense`

### 6.3 Expansion

Enter `Expansion` only if all are true:

- `premiumBps >= minPremiumBps`
- `premiumPersistenceEpochs >= premiumPersistenceRequired`
- `grossBuyFloorBps >= minGrossBuyFloorBps`
- `netBuyPressureBps > 0`
- `lockFlowBps > 0`
- `lockedShareBps >= minLockedShareBps`
- `reserveCoverageBps >= expansionReserveCoverageBps`
- `stableCashCoverageBps >= minStableCashCoverageBps`
- `liquidityDepthCoverageBps >= minLiquidityDepthCoverageBps`
- `largestCollateralConcentrationBps <= maxReserveConcentrationBps`
- `oracleConfidenceBps <= maxOracleConfidenceBps`
- `staleOracleCount <= maxStaleOracleCount`
- `realizedVolatilityBps <= maxExpansionVolatilityBps`
- `exitPressureBps <= maxExpansionExitPressureBps`
- `buyGrowthBps > 0`

### 6.4 Neutral

Everything else is `Neutral`.

Interpretation:

- `reserveCoverageBps` between `neutralReserveCoverageBps` and `expansionReserveCoverageBps` means the system can operate, but it does not print new credit

## 7. Premium Persistence

Update each epoch:

- if `premiumBps >= minPremiumBps`, increment `premiumPersistenceEpochs`
- else reset `premiumPersistenceEpochs = 0`

## 8. Expansion Score

### 8.1 Demand score

Definitions:

- `premiumScoreBps = min(max(premiumBps - minPremiumBps, 0) * BPS / minPremiumBps, BPS)`
- `buyScoreBps = min(grossBuyFloorBps * BPS / targetGrossBuyBps, BPS)`
- `netBuyScoreBps = min(netBuyPressureBps * BPS / targetNetBuyBps, BPS)`
- `lockFlowScoreBps = min(lockFlowBps * BPS / targetLockFlowBps, BPS)`
- `buyGrowthScoreBps = min(max(buyGrowthBps, 0) * BPS / targetBuyGrowthBps, BPS)`

Demand score:

- `demandScoreBps = min(premiumScoreBps, buyScoreBps, netBuyScoreBps, lockFlowScoreBps, buyGrowthScoreBps)`

### 8.2 Health score

Definitions:

- `reserveHealthBps = reserveCoverageBps <= expansionReserveCoverageBps ? 0 : min((reserveCoverageBps - expansionReserveCoverageBps) * BPS / (targetReserveCoverageBps - expansionReserveCoverageBps), BPS)`
- `stableCashHealthBps = stableCashCoverageBps <= minStableCashCoverageBps ? 0 : min((stableCashCoverageBps - minStableCashCoverageBps) * BPS / (targetStableCashCoverageBps - minStableCashCoverageBps), BPS)`
- `liquidityDepthHealthBps = liquidityDepthCoverageBps <= minLiquidityDepthCoverageBps ? 0 : min((liquidityDepthCoverageBps - minLiquidityDepthCoverageBps) * BPS / (targetLiquidityDepthCoverageBps - minLiquidityDepthCoverageBps), BPS)`
- `volatilityHealthBps = realizedVolatilityBps >= maxExpansionVolatilityBps ? 0 : (maxExpansionVolatilityBps - realizedVolatilityBps) * BPS / maxExpansionVolatilityBps`
- `exitHealthBps = exitPressureBps >= maxExpansionExitPressureBps ? 0 : (maxExpansionExitPressureBps - exitPressureBps) * BPS / maxExpansionExitPressureBps`
- `lockedShareHealthBps = min(lockedShareBps * BPS / targetLockedShareBps, BPS)`

Health score:

- `healthScoreBps = min(reserveHealthBps, stableCashHealthBps, liquidityDepthHealthBps, volatilityHealthBps, exitHealthBps, lockedShareHealthBps)`

## 9. Minting

Raw mint rate:

- `rawMintRateBps = expansionKappaBps * demandScoreBps / BPS * healthScoreBps / BPS`

Final mint rate:

- `mintRateBps = min(rawMintRateBps, maxMintPerEpochBps)`
- `remainingDailyMintAgc = max(floatSupplyAgc * maxMintPerDayBps / BPS - mintedTodayAgc, 0)`
- `mintBudgetAgc = min(floatSupplyAgc * mintRateBps / BPS, remainingDailyMintAgc)`

Mint budget is zero unless regime is `Expansion`.

## 10. Mint Distribution

Split newly minted AGC as:

- `xagcBps`
- `growthProgramsBps`
- `lpBps`
- `integratorsBps`
- `treasuryBps`

Launch default:

- `xagcBps = 5000`
- `growthProgramsBps = 2000`
- `lpBps = 1000`
- `integratorsBps = 500`
- `treasuryBps = 1500`

Rules:

- `xAGC` share is minted directly into the `xAGC` vault
- `treasury` share is minted to treasury inventory
- if growth programs are disabled, their budget rolls to treasury
- `growthPrograms`, `lp`, and `integrators` are assumed liquid and increase `floatSupplyAgc`

## 11. Defense Score

Definitions:

- `priceStressBps = priceTwapX18 < stressedFloorX18 ? (stressedFloorX18 - priceTwapX18) * BPS / anchorPriceX18 : 0`
- `coverageStressBps = reserveCoverageBps < defenseReserveCoverageBps ? defenseReserveCoverageBps - reserveCoverageBps : 0`
- `stableCashStressBps = stableCashCoverageBps < defenseStableCashCoverageBps ? defenseStableCashCoverageBps - stableCashCoverageBps : 0`
- `concentrationStressBps = largestCollateralConcentrationBps > maxReserveConcentrationBps ? largestCollateralConcentrationBps - maxReserveConcentrationBps : 0`
- `oracleStressBps = oracleConfidenceBps > maxOracleConfidenceBps ? oracleConfidenceBps - maxOracleConfidenceBps : 0`
- `exitStressBps = exitPressureBps > defenseExitPressureBps ? exitPressureBps - defenseExitPressureBps : 0`
- `volStressBps = realizedVolatilityBps > defenseVolatilityBps ? realizedVolatilityBps - defenseVolatilityBps : 0`
- `stressScoreBps = max(priceStressBps, coverageStressBps, stableCashStressBps, concentrationStressBps, oracleStressBps, exitStressBps, volStressBps)`

Severe stress override:

- if `reserveCoverageBps < hardDefenseReserveCoverageBps`, set `stressScoreBps = max(stressScoreBps, severeStressThresholdBps)`

## 12. Buybacks

Definitions:

- `buybackCapBps = stressScoreBps >= severeStressThresholdBps ? severeDefenseSpendBps : mildDefenseSpendBps`
- `buybackSpendRateBps = min(buybackKappaBps * stressScoreBps / BPS, buybackCapBps)`
- `buybackBudgetQuoteX18 = treasuryQuoteX18 * buybackSpendRateBps / BPS`
- `buybackBurnAgc = priceTwapX18 == 0 ? 0 : buybackBudgetQuoteX18 * 1e18 / priceTwapX18`

Buyback budget is zero unless regime is `Defense`.

## 13. xAGC Redemption

Definitions:

- `xagcExitFeeBps`

On redemption:

- `grossRedeemAgc = shares * vaultAssets / totalShares`
- `feeAgc = grossRedeemAgc * xagcExitFeeBps / BPS`
- `netRedeemAgc = grossRedeemAgc - feeAgc`

Rules:

- `feeAgc` goes to treasury
- treasury does not aggressively sell fee AGC during stress

For simulation, it is acceptable to input `xagcGrossRedemptionsAgc` directly instead of share counts.

## 14. State Transitions

At the end of each epoch:

- `anchorPriceX18 = anchorNextX18`
- `lastGrossBuyQuoteX18 = grossBuyQuoteX18`
- `treasuryQuoteX18 = treasuryQuoteX18 + treasuryQuoteInflowX18 - buybackBudgetQuoteX18`
- `treasuryAgc = treasuryAgc + treasuryMintAgc + xagcExitFeeAgc`
- `xagcTotalAssetsAgc = xagcTotalAssetsAgc + xagcDepositsAgc - xagcGrossRedemptionsAgc + xagcMintAgc`
- `floatSupplyAgc = floatSupplyAgc - xagcDepositsAgc + xagcNetRedemptionAgc + growthProgramsMintAgc + lpMintAgc + integratorsMintAgc - buybackBurnAgc`

Treasury AGC inventory does not count toward `floatSupplyAgc` until it is distributed or sold back into circulation.

## 15. Recommended Launch Values

These are the conservative launch baseline. The collateral-side counterparts (reserve weight, collateral factor, liquidation threshold, concentration cap, oracle staleness) live in [`risk-parameter-presets.md`](risk-parameter-presets.md).

- `initialAnchorPriceX18 = 0.50e18`
- `normalBandBps = 300`
- `stressedBandBps = 700`
- `anchorEmaBps = 500`
- `maxAnchorCrawlBps = 100`
- `minPremiumBps = 100`
- `premiumPersistenceRequired = 2`
- `minGrossBuyFloorBps = 50`
- `targetGrossBuyBps = 500`
- `targetNetBuyBps = 250`
- `targetLockFlowBps = 100`
- `targetBuyGrowthBps = 500`
- `minLockedShareBps = 1000`
- `targetLockedShareBps = 3000`
- `expansionReserveCoverageBps = 3000`
- `targetReserveCoverageBps = 8000`
- `neutralReserveCoverageBps = 2000`
- `defenseReserveCoverageBps = 1500`
- `hardDefenseReserveCoverageBps = 800`
- `minStableCashCoverageBps = 1200`
- `targetStableCashCoverageBps = 2500`
- `defenseStableCashCoverageBps = 800`
- `minLiquidityDepthCoverageBps = 2000`
- `targetLiquidityDepthCoverageBps = 5000`
- `maxReserveConcentrationBps = 6000`
- `maxOracleConfidenceBps = 150`
- `maxStaleOracleCount = 0`
- `maxExpansionVolatilityBps = 300`
- `defenseVolatilityBps = 1000`
- `maxExpansionExitPressureBps = 3000`
- `defenseExitPressureBps = 7000`
- `expansionKappaBps = 1000`
- `maxMintPerEpochBps = 100`
- `maxMintPerDayBps = 250`
- `buybackKappaBps = 5000`
- `mildDefenseSpendBps = 500`
- `severeDefenseSpendBps = 1500`
- `severeStressThresholdBps = 1000`
- `recoveryCooldownEpochs = 2`
- `xagcExitFeeBps = 100`

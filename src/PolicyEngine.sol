// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import { SafeCast } from "@openzeppelin/contracts/utils/math/SafeCast.sol";
import { AGCDataTypes } from "./libraries/AGCDataTypes.sol";

contract PolicyEngine {
    function evaluateEpoch(
        AGCDataTypes.EpochSnapshot memory snapshot,
        AGCDataTypes.ExternalMetrics memory externalMetrics,
        AGCDataTypes.PolicyState memory state,
        AGCDataTypes.VaultFlows memory flows,
        AGCDataTypes.PolicyParams memory policyParams
    ) public pure returns (AGCDataTypes.EpochResult memory result) {
        uint256 priceTwapX18 = snapshot.shortTwapPriceX18;
        uint256 grossBuyQuoteX18 = snapshot.grossBuyVolumeQuoteX18;
        uint256 grossSellQuoteX18 = snapshot.grossSellVolumeQuoteX18;
        uint256 totalVolumeQuoteX18 = snapshot.totalVolumeQuoteX18;

        int256 xagcNetDepositsAcp =
            int256(flows.xagcDepositsAcp) - int256(flows.xagcGrossRedemptionsAcp);

        uint256 creditOutstandingQuoteX18 =
            state.floatSupplyAcp * state.anchorPriceX18 / 1e18;
        uint256 grossBuyFloorBps =
            _safeDiv(grossBuyQuoteX18 * AGCDataTypes.BPS, creditOutstandingQuoteX18);
        uint256 netBuyQuoteX18 = grossBuyQuoteX18 > grossSellQuoteX18
            ? grossBuyQuoteX18 - grossSellQuoteX18
            : 0;
        uint256 netBuyPressureBps =
            _safeDiv(netBuyQuoteX18 * AGCDataTypes.BPS, creditOutstandingQuoteX18);
        uint256 buyGrowthBps = state.lastGrossBuyQuoteX18 == 0
            ? 0
            : _safeDiv(
                _positiveDelta(grossBuyQuoteX18, state.lastGrossBuyQuoteX18) * AGCDataTypes.BPS,
                state.lastGrossBuyQuoteX18
            );
        uint256 exitPressureBps =
            _safeDiv(grossSellQuoteX18 * AGCDataTypes.BPS, totalVolumeQuoteX18);
        uint256 reserveCoverageBps = _safeDiv(
            externalMetrics.depthToTargetSlippageQuoteX18 * AGCDataTypes.BPS,
            creditOutstandingQuoteX18
        );
        uint256 lockedShareBps =
            _safeDiv(state.xagcTotalAssetsAcp * AGCDataTypes.BPS, state.floatSupplyAcp);
        uint256 lockFlowBps = xagcNetDepositsAcp <= 0
            ? 0
            : _safeDiv(
                SafeCast.toUint256(xagcNetDepositsAcp) * AGCDataTypes.BPS, state.floatSupplyAcp
            );
        uint256 premiumBps = priceTwapX18 > state.anchorPriceX18 && state.anchorPriceX18 > 0
            ? (priceTwapX18 - state.anchorPriceX18) * AGCDataTypes.BPS / state.anchorPriceX18
            : 0;
        uint256 premiumPersistenceEpochs = premiumBps >= policyParams.minPremiumBps
            ? state.premiumPersistenceEpochs + 1
            : 0;

        uint256 normalFloorX18 =
            state.anchorPriceX18 * (AGCDataTypes.BPS - policyParams.normalBandBps)
                / AGCDataTypes.BPS;
        uint256 stressedFloorX18 =
            state.anchorPriceX18 * (AGCDataTypes.BPS - policyParams.stressedBandBps)
                / AGCDataTypes.BPS;
        uint256 anchorNextX18 = computeAnchorNext(
            state.anchorPriceX18,
            priceTwapX18,
            policyParams.anchorEmaBps,
            policyParams.maxAnchorCrawlBps
        );

        bool inDefense = priceTwapX18 < stressedFloorX18
            || reserveCoverageBps < policyParams.defenseReserveCoverageBps
            || snapshot.realizedVolatilityBps >= policyParams.defenseVolatilityBps
            || exitPressureBps >= policyParams.defenseExitPressureBps;

        bool canExpand = premiumBps >= policyParams.minPremiumBps
            && premiumPersistenceEpochs >= policyParams.premiumPersistenceRequired
            && grossBuyFloorBps >= policyParams.minGrossBuyFloorBps
            && netBuyPressureBps > 0 && lockFlowBps > 0
            && lockedShareBps >= policyParams.minLockedShareBps
            && reserveCoverageBps >= policyParams.expansionReserveCoverageBps
            && snapshot.realizedVolatilityBps <= policyParams.maxExpansionVolatilityBps
            && exitPressureBps <= policyParams.maxExpansionExitPressureBps
            && buyGrowthBps > 0;

        bool inRecovery = !inDefense && state.recoveryCooldownEpochsRemaining > 0
            && (
                state.lastRegime == AGCDataTypes.Regime.Defense
                    || state.lastRegime == AGCDataTypes.Regime.Recovery
            );

        AGCDataTypes.Regime nextRegime = AGCDataTypes.Regime.Neutral;
        if (inDefense) {
            nextRegime = AGCDataTypes.Regime.Defense;
        } else if (inRecovery) {
            nextRegime = AGCDataTypes.Regime.Recovery;
        } else if (canExpand) {
            nextRegime = AGCDataTypes.Regime.Expansion;
        }

        uint256 demandScoreBps;
        uint256 healthScoreBps;
        uint256 mintRateBps;
        uint256 mintBudgetAcp;

        if (nextRegime == AGCDataTypes.Regime.Expansion) {
            uint256 premiumScoreBps = _min(
                _safeDiv(
                    _positiveDelta(premiumBps, policyParams.minPremiumBps) * AGCDataTypes.BPS,
                    policyParams.minPremiumBps
                ),
                AGCDataTypes.BPS
            );
            uint256 buyScoreBps = _min(
                _safeDiv(
                    grossBuyFloorBps * AGCDataTypes.BPS, policyParams.targetGrossBuyBps
                ),
                AGCDataTypes.BPS
            );
            uint256 netBuyScoreBps = _min(
                _safeDiv(
                    netBuyPressureBps * AGCDataTypes.BPS, policyParams.targetNetBuyBps
                ),
                AGCDataTypes.BPS
            );
            uint256 lockFlowScoreBps = _min(
                _safeDiv(lockFlowBps * AGCDataTypes.BPS, policyParams.targetLockFlowBps),
                AGCDataTypes.BPS
            );
            uint256 buyGrowthScoreBps = _min(
                _safeDiv(buyGrowthBps * AGCDataTypes.BPS, policyParams.targetBuyGrowthBps),
                AGCDataTypes.BPS
            );

            demandScoreBps = _min(
                premiumScoreBps,
                _min(
                    buyScoreBps,
                    _min(netBuyScoreBps, _min(lockFlowScoreBps, buyGrowthScoreBps))
                )
            );

            uint256 reserveHealthBps = reserveCoverageBps <= policyParams.expansionReserveCoverageBps
                ? 0
                : _min(
                    _safeDiv(
                        (reserveCoverageBps - policyParams.expansionReserveCoverageBps)
                            * AGCDataTypes.BPS,
                        policyParams.targetReserveCoverageBps
                            - policyParams.expansionReserveCoverageBps
                    ),
                    AGCDataTypes.BPS
                );
            uint256 volatilityHealthBps =
                snapshot.realizedVolatilityBps >= policyParams.maxExpansionVolatilityBps
                ? 0
                : (policyParams.maxExpansionVolatilityBps - snapshot.realizedVolatilityBps)
                    * AGCDataTypes.BPS / policyParams.maxExpansionVolatilityBps;
            uint256 exitHealthBps = exitPressureBps >= policyParams.maxExpansionExitPressureBps
                ? 0
                : (policyParams.maxExpansionExitPressureBps - exitPressureBps)
                    * AGCDataTypes.BPS / policyParams.maxExpansionExitPressureBps;
            uint256 lockedShareHealthBps = _min(
                _safeDiv(
                    lockedShareBps * AGCDataTypes.BPS, policyParams.targetLockedShareBps
                ),
                AGCDataTypes.BPS
            );

            healthScoreBps = _min(
                reserveHealthBps,
                _min(volatilityHealthBps, _min(exitHealthBps, lockedShareHealthBps))
            );

            uint256 rawMintRateBps = policyParams.expansionKappaBps * demandScoreBps
                / AGCDataTypes.BPS * healthScoreBps / AGCDataTypes.BPS;
            mintRateBps = _min(rawMintRateBps, policyParams.maxMintPerEpochBps);

            uint256 remainingDailyMintAcp = _positiveDelta(
                state.floatSupplyAcp * policyParams.maxMintPerDayBps / AGCDataTypes.BPS,
                state.mintedTodayAcp
            );
            mintBudgetAcp = _min(
                state.floatSupplyAcp * mintRateBps / AGCDataTypes.BPS, remainingDailyMintAcp
            );
        }

        uint256 priceStressBps = priceTwapX18 < stressedFloorX18 && state.anchorPriceX18 > 0
            ? (stressedFloorX18 - priceTwapX18) * AGCDataTypes.BPS / state.anchorPriceX18
            : 0;
        uint256 coverageStressBps = reserveCoverageBps < policyParams.defenseReserveCoverageBps
            ? policyParams.defenseReserveCoverageBps - reserveCoverageBps
            : 0;
        uint256 exitStressBps = exitPressureBps > policyParams.defenseExitPressureBps
            ? exitPressureBps - policyParams.defenseExitPressureBps
            : 0;
        uint256 volatilityStressBps =
            snapshot.realizedVolatilityBps > policyParams.defenseVolatilityBps
            ? snapshot.realizedVolatilityBps - policyParams.defenseVolatilityBps
            : 0;
        uint256 stressScoreBps = _max(
            priceStressBps,
            _max(coverageStressBps, _max(exitStressBps, volatilityStressBps))
        );
        if (reserveCoverageBps < policyParams.hardDefenseReserveCoverageBps) {
            stressScoreBps = _max(stressScoreBps, policyParams.severeStressThresholdBps);
        }

        uint256 buybackBudgetQuoteX18;
        if (nextRegime == AGCDataTypes.Regime.Defense) {
            uint256 buybackCapBps = stressScoreBps >= policyParams.severeStressThresholdBps
                ? policyParams.severeDefenseSpendBps
                : policyParams.mildDefenseSpendBps;
            uint256 buybackSpendRateBps = _min(
                policyParams.buybackKappaBps * stressScoreBps / AGCDataTypes.BPS, buybackCapBps
            );
            buybackBudgetQuoteX18 =
                state.treasuryQuoteX18 * buybackSpendRateBps / AGCDataTypes.BPS;
        }

        result = AGCDataTypes.EpochResult({
            epochId: snapshot.epochId,
            regime: nextRegime,
            anchorPriceX18: state.anchorPriceX18,
            anchorNextX18: anchorNextX18,
            normalFloorX18: normalFloorX18,
            stressedFloorX18: stressedFloorX18,
            priceTwapX18: priceTwapX18,
            premiumBps: premiumBps,
            premiumPersistenceEpochs: premiumPersistenceEpochs,
            creditOutstandingQuoteX18: creditOutstandingQuoteX18,
            grossBuyFloorBps: grossBuyFloorBps,
            netBuyPressureBps: netBuyPressureBps,
            buyGrowthBps: buyGrowthBps,
            exitPressureBps: exitPressureBps,
            reserveCoverageBps: reserveCoverageBps,
            lockedShareBps: lockedShareBps,
            lockFlowBps: lockFlowBps,
            demandScoreBps: demandScoreBps,
            healthScoreBps: healthScoreBps,
            mintRateBps: mintRateBps,
            mintBudgetAcp: mintBudgetAcp,
            buybackBudgetQuoteX18: buybackBudgetQuoteX18,
            stressScoreBps: stressScoreBps,
            grossBuyQuoteX18: grossBuyQuoteX18,
            grossSellQuoteX18: grossSellQuoteX18,
            totalVolumeQuoteX18: totalVolumeQuoteX18,
            depthToTargetSlippageQuoteX18: externalMetrics.depthToTargetSlippageQuoteX18,
            realizedVolatilityBps: snapshot.realizedVolatilityBps,
            xagcDepositsAcp: flows.xagcDepositsAcp,
            xagcGrossRedemptionsAcp: flows.xagcGrossRedemptionsAcp,
            treasuryQuoteX18: state.treasuryQuoteX18,
            treasuryAcp: state.treasuryAcp,
            xagcTotalAssetsAcp: state.xagcTotalAssetsAcp,
            mintAllocations: AGCDataTypes.MintAllocation(0, 0, 0, 0, 0)
        });
    }

    function computeAnchorNext(
        uint256 anchorPriceX18,
        uint256 priceTwapX18,
        uint256 anchorEmaBps,
        uint256 maxAnchorCrawlBps
    ) public pure returns (uint256 nextAnchorPriceX18) {
        uint256 ema = (anchorPriceX18 * (AGCDataTypes.BPS - anchorEmaBps))
            + (priceTwapX18 * anchorEmaBps);
        ema /= AGCDataTypes.BPS;

        uint256 minAnchor =
            anchorPriceX18 * (AGCDataTypes.BPS - maxAnchorCrawlBps) / AGCDataTypes.BPS;
        uint256 maxAnchor =
            anchorPriceX18 * (AGCDataTypes.BPS + maxAnchorCrawlBps) / AGCDataTypes.BPS;

        if (ema < minAnchor) return minAnchor;
        if (ema > maxAnchor) return maxAnchor;
        return ema;
    }

    function _safeDiv(
        uint256 numerator,
        uint256 denominator
    ) internal pure returns (uint256) {
        return denominator == 0 ? 0 : numerator / denominator;
    }

    function _positiveDelta(
        uint256 lhs,
        uint256 rhs
    ) internal pure returns (uint256) {
        return lhs > rhs ? lhs - rhs : 0;
    }

    function _min(
        uint256 lhs,
        uint256 rhs
    ) internal pure returns (uint256) {
        return lhs < rhs ? lhs : rhs;
    }

    function _max(
        uint256 lhs,
        uint256 rhs
    ) internal pure returns (uint256) {
        return lhs > rhs ? lhs : rhs;
    }
}

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {AGCDataTypes} from "./libraries/AGCDataTypes.sol";

contract PolicyEngine {
    function deriveMetrics(
        AGCDataTypes.EpochSnapshot memory snapshot,
        AGCDataTypes.ExternalMetrics memory externalMetrics,
        uint256 floatSupply
    ) public pure returns (AGCDataTypes.DerivedMetrics memory metrics) {
        metrics.floatSupply = floatSupply;
        metrics.price = snapshot.shortTwapPriceX18;
        metrics.productiveUsageBps = snapshot.totalVolume == 0
            ? 0
            : snapshot.productiveVolume * AGCDataTypes.BPS / snapshot.totalVolume;
        metrics.coverageBps =
            floatSupply == 0 ? 0 : externalMetrics.depthTo1Pct * AGCDataTypes.BPS / floatSupply;
        metrics.exitPressureBps = snapshot.totalVolume == 0 || snapshot.netExitVolume <= 0
            ? 0
            : uint256(snapshot.netExitVolume) * AGCDataTypes.BPS / snapshot.totalVolume;
        metrics.repeatUserBps = snapshot.productiveUsers == 0
            ? 0
            : uint256(snapshot.repeatUsers) * AGCDataTypes.BPS / snapshot.productiveUsers;
        metrics.volatilityBps = snapshot.realizedVolatilityBps;
    }

    function selectRegime(
        uint256 price,
        uint256 anchorPriceX18,
        uint256 bandWidthBps,
        AGCDataTypes.PolicyParams memory policyParams,
        AGCDataTypes.DerivedMetrics memory metrics,
        int256 productiveGrowthBps
    ) public pure returns (AGCDataTypes.Regime nextRegime) {
        uint256 floorPrice = anchorPriceX18 * (AGCDataTypes.BPS - bandWidthBps) / AGCDataTypes.BPS;

        bool inDefense = price < floorPrice || metrics.volatilityBps >= policyParams.criticalVolatilityBps
            || metrics.coverageBps < policyParams.criticalCoverageBps
            || metrics.exitPressureBps >= policyParams.criticalExitBps;

        if (inDefense) return AGCDataTypes.Regime.Defense;

        bool canExpand = metrics.productiveUsageBps >= policyParams.minProductiveUsageBps
            && metrics.coverageBps >= policyParams.minCoverageBps
            && metrics.volatilityBps <= policyParams.maxExpansionVolatilityBps
            && metrics.exitPressureBps <= policyParams.maxExpansionExitBps
            && productiveGrowthBps > 0
            && price >= anchorPriceX18;

        if (canExpand) return AGCDataTypes.Regime.Expansion;
        return AGCDataTypes.Regime.Neutral;
    }

    function updateAnchor(
        AGCDataTypes.EpochSnapshot memory snapshot,
        uint256 anchorPriceX18,
        AGCDataTypes.PolicyParams memory policyParams
    ) public pure returns (uint256 nextAnchorPriceX18) {
        uint256 referencePrice = snapshot.productiveSettlementPriceX18 > 0
            ? snapshot.productiveSettlementPriceX18
            : snapshot.shortTwapPriceX18;

        if (referencePrice == 0) {
            return anchorPriceX18;
        }

        uint256 emaPrice = (anchorPriceX18 * (AGCDataTypes.BPS - policyParams.anchorEmaBps))
            + (referencePrice * policyParams.anchorEmaBps);
        emaPrice /= AGCDataTypes.BPS;

        uint256 minAnchor =
            anchorPriceX18 * (AGCDataTypes.BPS - policyParams.maxAnchorCrawlBps) / AGCDataTypes.BPS;
        uint256 maxAnchor =
            anchorPriceX18 * (AGCDataTypes.BPS + policyParams.maxAnchorCrawlBps) / AGCDataTypes.BPS;

        if (emaPrice < minAnchor) return minAnchor;
        if (emaPrice > maxAnchor) return maxAnchor;
        return emaPrice;
    }

    function mintBudget(
        AGCDataTypes.DerivedMetrics memory metrics,
        int256 productiveGrowthBps,
        AGCDataTypes.PolicyParams memory policyParams,
        uint256 remainingDailyCapacity
    ) public pure returns (uint256 budget) {
        uint256 mintRateBps = policyParams.expansionKappaBps
            * _healthBps(metrics.productiveUsageBps, metrics.coverageBps, productiveGrowthBps, policyParams)
            / AGCDataTypes.BPS;
        if (mintRateBps > policyParams.maxMintPerEpochBps) {
            mintRateBps = policyParams.maxMintPerEpochBps;
        }

        budget = metrics.floatSupply * mintRateBps / AGCDataTypes.BPS;
        if (budget > remainingDailyCapacity) {
            budget = remainingDailyCapacity;
        }
    }

    function _healthBps(
        uint256 productiveUsageBps,
        uint256 coverageBps,
        int256 productiveGrowthBps,
        AGCDataTypes.PolicyParams memory policyParams
    ) internal pure returns (uint256 healthBps) {
        uint256 usageHeadroom = productiveUsageBps > policyParams.minProductiveUsageBps
            ? productiveUsageBps - policyParams.minProductiveUsageBps
            : 0;
        uint256 coverageHeadroom = coverageBps > policyParams.minCoverageBps
            ? coverageBps - policyParams.minCoverageBps
            : 0;
        uint256 growthBps = productiveGrowthBps > 0 ? uint256(productiveGrowthBps) : 0;

        healthBps = usageHeadroom;
        if (coverageHeadroom < healthBps) healthBps = coverageHeadroom;
        if (growthBps < healthBps) healthBps = growthBps;
        if (healthBps > AGCDataTypes.BPS) healthBps = AGCDataTypes.BPS;
    }

    function buybackBudget(
        uint256 price,
        uint256 anchorPriceX18,
        uint256 bandWidthBps,
        AGCDataTypes.DerivedMetrics memory metrics,
        uint256 treasuryUsdc,
        AGCDataTypes.PolicyParams memory policyParams
    ) public pure returns (uint256 budget) {
        uint256 floorPrice = anchorPriceX18 * (AGCDataTypes.BPS - bandWidthBps) / AGCDataTypes.BPS;
        uint256 priceStressBps = price < floorPrice ? (floorPrice - price) * AGCDataTypes.BPS / anchorPriceX18 : 0;
        uint256 coverageStressBps = metrics.coverageBps < policyParams.criticalCoverageBps
            ? policyParams.criticalCoverageBps - metrics.coverageBps
            : 0;
        uint256 exitStressBps = metrics.exitPressureBps > policyParams.criticalExitBps
            ? metrics.exitPressureBps - policyParams.criticalExitBps
            : 0;
        uint256 volStressBps = metrics.volatilityBps > policyParams.criticalVolatilityBps
            ? metrics.volatilityBps - policyParams.criticalVolatilityBps
            : 0;

        uint256 stressBps = priceStressBps;
        if (coverageStressBps > stressBps) stressBps = coverageStressBps;
        if (exitStressBps > stressBps) stressBps = exitStressBps;
        if (volStressBps > stressBps) stressBps = volStressBps;
        if (stressBps > AGCDataTypes.BPS) stressBps = AGCDataTypes.BPS;

        uint256 targetSpendRateBps = stressBps > policyParams.criticalVolatilityBps
            ? policyParams.severeDefenseSpendBps
            : policyParams.mildDefenseSpendBps;
        uint256 stressSpend =
            treasuryUsdc * policyParams.buybackKappaBps * stressBps / AGCDataTypes.BPS / AGCDataTypes.BPS;
        uint256 cap = treasuryUsdc * targetSpendRateBps / AGCDataTypes.BPS;
        budget = stressSpend > cap ? cap : stressSpend;
    }
}

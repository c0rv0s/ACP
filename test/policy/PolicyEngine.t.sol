// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import "forge-std/Test.sol";
import { PolicyEngine } from "../../src/PolicyEngine.sol";
import { AGCDataTypes } from "../../src/libraries/AGCDataTypes.sol";

contract PolicyEngineTest is Test {
    PolicyEngine internal engine;

    function setUp() public {
        engine = new PolicyEngine();
    }

    function testComputeAnchorNextClampsLargeMoves() public view {
        assertEq(engine.computeAnchorNext(1e18, 2e18, 300, 10), 1.001e18);
        assertEq(engine.computeAnchorNext(1e18, 5e17, 300, 10), 0.999e18);
    }

    function testEvaluateEpochExpansionMintIsCappedByRemainingDailyLimit() public view {
        AGCDataTypes.PolicyParams memory params = _policyParams();
        AGCDataTypes.PolicyState memory state = AGCDataTypes.PolicyState({
            anchorPriceX18: 5e17,
            premiumPersistenceEpochs: 2,
            lastGrossBuyQuoteX18: 20_000e18,
            mintedTodayAcp: 49_000e18,
            lastRegime: AGCDataTypes.Regime.Neutral,
            recoveryCooldownEpochsRemaining: 0,
            floatSupplyAcp: 1_000_000e18,
            treasuryQuoteX18: 150_000e18,
            treasuryAcp: 0,
            xagcTotalAssetsAcp: 200_000e18
        });
        AGCDataTypes.VaultFlows memory flows = AGCDataTypes.VaultFlows({
            xagcDepositsAcp: 50_000e18,
            xagcGrossRedemptionsAcp: 0
        });
        AGCDataTypes.EpochSnapshot memory snapshot = AGCDataTypes.EpochSnapshot({
            epochId: 3,
            startedAt: 0,
            endedAt: 0,
            grossBuyVolumeQuoteX18: 50_000e18,
            grossSellVolumeQuoteX18: 10_000e18,
            totalVolumeQuoteX18: 70_000e18,
            shortTwapPriceX18: 55e16,
            realizedVolatilityBps: 80,
            totalHookFeesQuoteX18: 0,
            totalHookFeesAgc: 0
        });
        AGCDataTypes.ExternalMetrics memory metrics =
            AGCDataTypes.ExternalMetrics({ depthToTargetSlippageQuoteX18: 150_000e18 });

        AGCDataTypes.EpochResult memory result =
            engine.evaluateEpoch(snapshot, metrics, state, flows, params);

        assertEq(uint8(result.regime), uint8(AGCDataTypes.Regime.Expansion));
        assertEq(result.premiumPersistenceEpochs, 3);
        assertEq(result.mintRateBps, 30);
        assertEq(result.mintBudgetAcp, 1_000e18);
        assertEq(result.buybackBudgetQuoteX18, 0);
    }

    function testEvaluateEpochRecoveryTakesPrecedenceOverFreshExpansion() public view {
        AGCDataTypes.PolicyParams memory params = _policyParams();
        AGCDataTypes.PolicyState memory state = AGCDataTypes.PolicyState({
            anchorPriceX18: 5e17,
            premiumPersistenceEpochs: 2,
            lastGrossBuyQuoteX18: 20_000e18,
            mintedTodayAcp: 0,
            lastRegime: AGCDataTypes.Regime.Defense,
            recoveryCooldownEpochsRemaining: 1,
            floatSupplyAcp: 1_000_000e18,
            treasuryQuoteX18: 150_000e18,
            treasuryAcp: 0,
            xagcTotalAssetsAcp: 200_000e18
        });
        AGCDataTypes.VaultFlows memory flows = AGCDataTypes.VaultFlows({
            xagcDepositsAcp: 50_000e18,
            xagcGrossRedemptionsAcp: 0
        });
        AGCDataTypes.EpochSnapshot memory snapshot = AGCDataTypes.EpochSnapshot({
            epochId: 4,
            startedAt: 0,
            endedAt: 0,
            grossBuyVolumeQuoteX18: 50_000e18,
            grossSellVolumeQuoteX18: 10_000e18,
            totalVolumeQuoteX18: 70_000e18,
            shortTwapPriceX18: 55e16,
            realizedVolatilityBps: 80,
            totalHookFeesQuoteX18: 0,
            totalHookFeesAgc: 0
        });
        AGCDataTypes.ExternalMetrics memory metrics =
            AGCDataTypes.ExternalMetrics({ depthToTargetSlippageQuoteX18: 150_000e18 });

        AGCDataTypes.EpochResult memory result =
            engine.evaluateEpoch(snapshot, metrics, state, flows, params);

        assertEq(uint8(result.regime), uint8(AGCDataTypes.Regime.Recovery));
        assertEq(result.mintBudgetAcp, 0);
        assertEq(result.buybackBudgetQuoteX18, 0);
    }

    function testEvaluateEpochDefenseBuybackUsesSevereCap() public view {
        AGCDataTypes.PolicyParams memory params = _policyParams();
        AGCDataTypes.PolicyState memory state = AGCDataTypes.PolicyState({
            anchorPriceX18: 5e17,
            premiumPersistenceEpochs: 0,
            lastGrossBuyQuoteX18: 10_000e18,
            mintedTodayAcp: 0,
            lastRegime: AGCDataTypes.Regime.Neutral,
            recoveryCooldownEpochsRemaining: 0,
            floatSupplyAcp: 1_000_000e18,
            treasuryQuoteX18: 200_000e18,
            treasuryAcp: 0,
            xagcTotalAssetsAcp: 200_000e18
        });
        AGCDataTypes.VaultFlows memory flows = AGCDataTypes.VaultFlows({
            xagcDepositsAcp: 0,
            xagcGrossRedemptionsAcp: 0
        });
        AGCDataTypes.EpochSnapshot memory snapshot = AGCDataTypes.EpochSnapshot({
            epochId: 7,
            startedAt: 0,
            endedAt: 0,
            grossBuyVolumeQuoteX18: 5_000e18,
            grossSellVolumeQuoteX18: 90_000e18,
            totalVolumeQuoteX18: 100_000e18,
            shortTwapPriceX18: 4e17,
            realizedVolatilityBps: 600,
            totalHookFeesQuoteX18: 0,
            totalHookFeesAgc: 0
        });
        AGCDataTypes.ExternalMetrics memory metrics =
            AGCDataTypes.ExternalMetrics({ depthToTargetSlippageQuoteX18: 20_000e18 });

        AGCDataTypes.EpochResult memory result =
            engine.evaluateEpoch(snapshot, metrics, state, flows, params);

        assertEq(uint8(result.regime), uint8(AGCDataTypes.Regime.Defense));
        assertEq(result.stressScoreBps, 5_500);
        assertEq(result.buybackBudgetQuoteX18, 20_000e18);
        assertEq(result.mintBudgetAcp, 0);
    }

    function _policyParams() internal pure returns (AGCDataTypes.PolicyParams memory params) {
        return AGCDataTypes.PolicyParams({
            normalBandBps: 300,
            stressedBandBps: 700,
            anchorEmaBps: 300,
            maxAnchorCrawlBps: 10,
            minPremiumBps: 100,
            premiumPersistenceRequired: 3,
            minGrossBuyFloorBps: 100,
            minLockedShareBps: 1000,
            targetGrossBuyBps: 500,
            targetNetBuyBps: 100,
            targetLockFlowBps: 100,
            targetBuyGrowthBps: 200,
            targetLockedShareBps: 2500,
            expansionReserveCoverageBps: 2000,
            targetReserveCoverageBps: 3000,
            neutralReserveCoverageBps: 1200,
            defenseReserveCoverageBps: 1200,
            hardDefenseReserveCoverageBps: 800,
            maxExpansionVolatilityBps: 150,
            defenseVolatilityBps: 400,
            maxExpansionExitPressureBps: 1800,
            defenseExitPressureBps: 3500,
            expansionKappaBps: 150,
            maxMintPerEpochBps: 100,
            maxMintPerDayBps: 500,
            buybackKappaBps: 2500,
            mildDefenseSpendBps: 200,
            severeDefenseSpendBps: 1000,
            severeStressThresholdBps: 1000,
            recoveryCooldownEpochs: 2,
            policyEpochDuration: 1 hours
        });
    }
}

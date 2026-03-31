// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import "forge-std/Test.sol";
import { AGCToken } from "../../src/AGCToken.sol";
import { PolicyController } from "../../src/PolicyController.sol";
import { PolicyEngine } from "../../src/PolicyEngine.sol";
import { IAGCHook } from "../../src/interfaces/IAGCHook.sol";
import { IRewardDistributor } from "../../src/interfaces/IRewardDistributor.sol";
import { ISettlementRouter } from "../../src/interfaces/ISettlementRouter.sol";
import { IStabilityVault } from "../../src/interfaces/IStabilityVault.sol";
import { MockUSDC } from "../../src/mocks/MockUSDC.sol";
import { AGCDataTypes } from "../../src/libraries/AGCDataTypes.sol";
import { MockHookAdapter } from "../mocks/MockHookAdapter.sol";
import { MockRewardDistributor } from "../mocks/MockRewardDistributor.sol";
import { MockSettlementRouter } from "../mocks/MockSettlementRouter.sol";
import { MockVault } from "../mocks/MockVault.sol";

contract PolicyControllerTest is Test {
    bytes32 internal constant MINTER_ROLE = keccak256("MINTER_ROLE");

    AGCToken internal agc;
    MockUSDC internal usdc;
    MockHookAdapter internal hook;
    MockVault internal vault;
    MockRewardDistributor internal distributor;
    MockSettlementRouter internal router;
    PolicyEngine internal engine;
    PolicyController internal controller;

    function setUp() public {
        vm.warp(1 days);

        agc = new AGCToken(address(this));
        usdc = new MockUSDC(address(this));
        hook = new MockHookAdapter();
        vault = new MockVault(usdc, agc);
        distributor = new MockRewardDistributor();
        router = new MockSettlementRouter();
        engine = new PolicyEngine();

        AGCDataTypes.PolicyParams memory params = AGCDataTypes.PolicyParams({
            baseBandBps: 200,
            stressedBandBps: 400,
            anchorEmaBps: 500,
            maxAnchorCrawlBps: 10,
            minProductiveUsageBps: 3000,
            minCoverageBps: 1000,
            criticalCoverageBps: 500,
            maxExpansionVolatilityBps: 200,
            criticalVolatilityBps: 400,
            maxExpansionExitBps: 2000,
            criticalExitBps: 4000,
            maxMintPerEpochBps: 500,
            maxMintPerDayBps: 5000,
            expansionKappaBps: 200,
            buybackKappaBps: 4000,
            mildDefenseSpendBps: 2500,
            severeDefenseSpendBps: 8000,
            recoveryCooldownEpochs: 2,
            policyEpochDuration: 1 hours,
            treasuryLockDuration: 1 days
        });

        AGCDataTypes.RewardSplit memory split = AGCDataTypes.RewardSplit({
            agentBps: 3000, lpBps: 2000, integratorBps: 2000, treasuryBps: 2000, reserveBps: 1000
        });

        controller = new PolicyController(
            address(this),
            PolicyController.Dependencies({
                agcToken: agc,
                hookContract: IAGCHook(address(hook)),
                stabilityVault: IStabilityVault(address(vault)),
                rewardDistributor: IRewardDistributor(address(distributor)),
                router: ISettlementRouter(address(router)),
                policyEngine: engine
            }),
            1e18,
            params,
            split
        );

        agc.grantRole(MINTER_ROLE, address(controller));
        agc.grantRole(MINTER_ROLE, address(this));
        agc.mint(address(0x1111), 1_000e18);
        usdc.mint(address(vault), 1_000_000e6);
    }

    function testPreviewAndSettleExpansionEpochMatch() public {
        hook.setNextSnapshot(_healthyExpansionSnapshot(1));
        AGCDataTypes.ExternalMetrics memory metrics = _healthyExternalMetrics(200e18, 5_000, 0);

        AGCDataTypes.EpochResult memory preview = controller.previewEpoch(metrics);
        AGCDataTypes.EpochResult memory settled = controller.settleEpoch(metrics);

        assertEq(uint8(preview.regime), uint8(AGCDataTypes.Regime.Expansion));
        assertEq(preview.mintBudget, 2e18);
        assertEq(preview.buybackBudget, 0);
        assertEq(preview.anchorPriceX18, 1000500000000000000);

        assertEq(preview.mintBudget, settled.mintBudget);
        assertEq(preview.buybackBudget, settled.buybackBudget);
        assertEq(preview.anchorPriceX18, settled.anchorPriceX18);
        assertEq(preview.productiveUsageBps, settled.productiveUsageBps);
        assertEq(preview.coverageBps, settled.coverageBps);
        assertEq(preview.productiveGrowthBps, settled.productiveGrowthBps);

        AGCDataTypes.EpochResult memory recorded = controller.lastEpochResult();
        assertEq(recorded.mintBudget, settled.mintBudget);
        assertEq(distributor.lastEpochId(), 1);
        assertEq(agc.balanceOf(address(distributor)), 14e17);
        assertEq(agc.balanceOf(address(vault)), 6e17);
    }

    function testDefenseEpochTriggersDeterministicBuyback() public {
        hook.setNextSnapshot(
            AGCDataTypes.EpochSnapshot({
                epochId: 1,
                startedAt: uint64(block.timestamp - 1 hours),
                endedAt: uint64(block.timestamp),
                productiveVolume: 0,
                totalVolume: 100e6,
                netExitVolume: int256(100e6),
                shortTwapPriceX18: 95e16,
                productiveSettlementPriceX18: 95e16,
                realizedVolatilityBps: 600,
                productiveSettlementCount: 0,
                productiveUsers: 0,
                repeatUsers: 0,
                totalHookFeesUsdc: 0,
                totalHookFeesAgc: 0
            })
        );

        AGCDataTypes.ExternalMetrics memory metrics = AGCDataTypes.ExternalMetrics({
            depthTo1Pct: 20e18,
            depthTo2Pct: 30e18,
            productiveGrowthBps: -500,
            lpStabilityBps: 4_000,
            idleShareBps: 2_000,
            buybackMinAgcOut: 50e18
        });

        AGCDataTypes.EpochResult memory settled = controller.settleEpoch(metrics);

        assertEq(uint8(settled.regime), uint8(AGCDataTypes.Regime.Defense));
        assertEq(settled.bandWidthBps, 400);
        assertGt(settled.buybackBudget, 0);
        assertEq(router.lastBuybackBudget(), settled.buybackBudget);
        assertEq(router.lastMinAgcOut(), 50e18);
    }

    function testCooldownForcesRecoveryAndZeroMint() public {
        hook.setNextSnapshot(
            AGCDataTypes.EpochSnapshot({
                epochId: 1,
                startedAt: uint64(block.timestamp - 1 hours),
                endedAt: uint64(block.timestamp),
                productiveVolume: 0,
                totalVolume: 100e6,
                netExitVolume: int256(100e6),
                shortTwapPriceX18: 95e16,
                productiveSettlementPriceX18: 95e16,
                realizedVolatilityBps: 600,
                productiveSettlementCount: 0,
                productiveUsers: 0,
                repeatUsers: 0,
                totalHookFeesUsdc: 0,
                totalHookFeesAgc: 0
            })
        );
        controller.settleEpoch(
            AGCDataTypes.ExternalMetrics({
                depthTo1Pct: 20e18,
                depthTo2Pct: 30e18,
                productiveGrowthBps: -500,
                lpStabilityBps: 4_000,
                idleShareBps: 2_000,
                buybackMinAgcOut: 50e18
            })
        );

        vm.warp(block.timestamp + 1 hours);
        hook.setNextSnapshot(_healthyExpansionSnapshot(2));
        AGCDataTypes.EpochResult memory settled =
            controller.settleEpoch(_healthyExternalMetrics(200e18, 5_000, 0));

        assertEq(uint8(settled.regime), uint8(AGCDataTypes.Regime.Recovery));
        assertEq(settled.bandWidthBps, 400);
        assertEq(settled.mintBudget, 0);
    }

    function testEmissionsForceDisabledKeepsExpansionButZeroMint() public {
        controller.setEmissionsForceDisabled(true);
        hook.setNextSnapshot(_healthyExpansionSnapshot(1));

        AGCDataTypes.EpochResult memory preview =
            controller.previewEpoch(_healthyExternalMetrics(200e18, 5_000, 0));
        assertEq(uint8(preview.regime), uint8(AGCDataTypes.Regime.Expansion));
        assertEq(preview.mintBudget, 0);
    }

    function testControllerCanScheduleLpAndIntegratorRewardStreams() public {
        hook.setNextSnapshot(_healthyExpansionSnapshot(1));
        controller.settleEpoch(_healthyExternalMetrics(200e18, 5_000, 0));

        PolicyController.RewardStreamRequest[] memory lpRequests =
            new PolicyController.RewardStreamRequest[](1);
        lpRequests[0] = PolicyController.RewardStreamRequest({
            beneficiary: address(0x2222), amount: 2e17, duration: 7 days, source: keccak256("lp-1")
        });

        uint256[] memory lpStreamIds = controller.scheduleLpRewardStreams(1, lpRequests);
        assertEq(lpStreamIds.length, 1);
        assertEq(lpStreamIds[0], 1);

        PolicyController.RewardStreamRequest[] memory integratorRequests =
            new PolicyController.RewardStreamRequest[](1);
        integratorRequests[0] = PolicyController.RewardStreamRequest({
            beneficiary: address(0x3333),
            amount: 2e17,
            duration: 14 days,
            source: keccak256("integrator-1")
        });

        uint256[] memory integratorStreamIds =
            controller.scheduleIntegratorRewardStreams(1, integratorRequests);
        assertEq(integratorStreamIds.length, 1);
        assertEq(integratorStreamIds[0], 2);
        assertEq(distributor.scheduleCalls(), 2);
    }

    function testScheduleRewardStreamsRejectsInvalidRequest() public {
        hook.setNextSnapshot(_healthyExpansionSnapshot(1));
        controller.settleEpoch(_healthyExternalMetrics(200e18, 5_000, 0));

        PolicyController.RewardStreamRequest[] memory requests =
            new PolicyController.RewardStreamRequest[](1);
        requests[0] = PolicyController.RewardStreamRequest({
            beneficiary: address(0), amount: 1e18, duration: 7 days, source: keccak256("lp-invalid")
        });

        vm.expectRevert(PolicyController.InvalidRewardRequest.selector);
        controller.scheduleLpRewardStreams(1, requests);
    }

    function _healthyExpansionSnapshot(
        uint64 epochId
    ) internal view returns (AGCDataTypes.EpochSnapshot memory) {
        return AGCDataTypes.EpochSnapshot({
            epochId: epochId,
            startedAt: uint64(block.timestamp - 1 hours),
            endedAt: uint64(block.timestamp),
            productiveVolume: 500_000e6,
            totalVolume: 1_000_000e6,
            netExitVolume: 0,
            shortTwapPriceX18: 102e16,
            productiveSettlementPriceX18: 101e16,
            realizedVolatilityBps: 100,
            productiveSettlementCount: 10,
            productiveUsers: 5,
            repeatUsers: 3,
            totalHookFeesUsdc: 0,
            totalHookFeesAgc: 0
        });
    }

    function _healthyExternalMetrics(
        uint256 depthTo1Pct,
        int256 productiveGrowthBps,
        uint256 buybackMinAgcOut
    ) internal pure returns (AGCDataTypes.ExternalMetrics memory) {
        return AGCDataTypes.ExternalMetrics({
            depthTo1Pct: depthTo1Pct,
            depthTo2Pct: depthTo1Pct * 2,
            productiveGrowthBps: productiveGrowthBps,
            lpStabilityBps: 8_000,
            idleShareBps: 1_500,
            buybackMinAgcOut: buybackMinAgcOut
        });
    }
}

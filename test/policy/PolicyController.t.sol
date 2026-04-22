// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import "forge-std/Test.sol";
import { SafeCast } from "@openzeppelin/contracts/utils/math/SafeCast.sol";
import { AGCToken } from "../../src/AGCToken.sol";
import { PolicyController } from "../../src/PolicyController.sol";
import { PolicyEngine } from "../../src/PolicyEngine.sol";
import { IAGCHook } from "../../src/interfaces/IAGCHook.sol";
import { ISettlementRouter } from "../../src/interfaces/ISettlementRouter.sol";
import { IStabilityVault } from "../../src/interfaces/IStabilityVault.sol";
import { IXAGCVault } from "../../src/interfaces/IXAGCVault.sol";
import { XAGCVault } from "../../src/XAGCVault.sol";
import { StabilityVault } from "../../src/StabilityVault.sol";
import { MockUSDC } from "../../src/mocks/MockUSDC.sol";
import { AGCDataTypes } from "../../src/libraries/AGCDataTypes.sol";
import { MockHookAdapter } from "../mocks/MockHookAdapter.sol";
import { MockSettlementRouter } from "../mocks/MockSettlementRouter.sol";

contract PolicyControllerTest is Test {
    bytes32 internal constant MINTER_ROLE = keccak256("MINTER_ROLE");

    AGCToken internal agc;
    MockUSDC internal usdc;
    MockHookAdapter internal hook;
    StabilityVault internal treasuryVault;
    XAGCVault internal xagcVault;
    MockSettlementRouter internal router;
    PolicyEngine internal engine;
    PolicyController internal controller;

    address internal staker = address(0x1111);
    address internal trader = address(0x2222);
    address internal growthPrograms = address(0x3333);
    address internal lpRewards = address(0x4444);
    address internal integrators = address(0x5555);

    function setUp() public {
        vm.warp(1 days);

        agc = new AGCToken(address(this));
        usdc = new MockUSDC(address(this));
        hook = new MockHookAdapter();
        treasuryVault = new StabilityVault(address(this), agc, usdc);
        xagcVault = new XAGCVault(address(this), agc, address(treasuryVault), 300);
        router = new MockSettlementRouter();
        engine = new PolicyEngine();

        AGCDataTypes.PolicyParams memory params = AGCDataTypes.PolicyParams({
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

        AGCDataTypes.MintDistribution memory distribution = AGCDataTypes.MintDistribution({
            xagcBps: 5000,
            growthProgramsBps: 2000,
            lpBps: 1000,
            integratorsBps: 500,
            treasuryBps: 1500
        });

        controller = new PolicyController(
            address(this),
            PolicyController.Dependencies({
                agcToken: agc,
                hookContract: IAGCHook(address(hook)),
                stabilityVault: IStabilityVault(address(treasuryVault)),
                xagcVault: IXAGCVault(address(xagcVault)),
                router: ISettlementRouter(address(router)),
                policyEngine: engine
            }),
            5e17,
            params,
            distribution
        );

        controller.setSettlementRecipients(
            AGCDataTypes.SettlementRecipients({
                growthPrograms: growthPrograms,
                lp: lpRewards,
                integrators: integrators
            })
        );

        treasuryVault.setPolicyController(address(controller));
        treasuryVault.setSettlementRouter(address(router));

        agc.grantRole(MINTER_ROLE, address(controller));
        agc.grantRole(MINTER_ROLE, address(this));

        agc.mint(staker, 200_000e18);
        agc.mint(trader, 800_000e18);
        usdc.mint(address(treasuryVault), 150_000e6);

        vm.startPrank(staker);
        agc.approve(address(xagcVault), type(uint256).max);
        xagcVault.deposit(150_000e18, staker);
        vm.stopPrank();
    }

    function testExpansionRequiresPremiumPersistenceAndPositiveLockFlow() public {
        vm.prank(staker);
        xagcVault.deposit(10_000e18, staker);
        hook.setNextSnapshot(
            _expansionSnapshot(1, 1 days, 506000000000000000, 15_000e18, 4_000e18, 22_000e18)
        );
        AGCDataTypes.EpochResult memory first =
            controller.settleEpoch(AGCDataTypes.ExternalMetrics({ depthToTargetSlippageQuoteX18: 170_000e18 }));
        assertEq(uint8(first.regime), uint8(AGCDataTypes.Regime.Neutral));
        assertEq(first.mintBudgetAcp, 0);

        vm.warp(controller.lastSettlementTimestamp() + 2 hours);
        vm.prank(staker);
        xagcVault.deposit(10_000e18, staker);
        hook.setNextSnapshot(
            _expansionSnapshot(2, block.timestamp, 508000000000000000, 20_000e18, 5_000e18, 28_000e18)
        );
        AGCDataTypes.EpochResult memory second =
            controller.settleEpoch(AGCDataTypes.ExternalMetrics({ depthToTargetSlippageQuoteX18: 175_000e18 }));
        assertEq(uint8(second.regime), uint8(AGCDataTypes.Regime.Neutral));
        assertEq(second.mintBudgetAcp, 0);

        vm.warp(controller.lastSettlementTimestamp() + 2 hours);
        vm.prank(staker);
        xagcVault.deposit(10_000e18, staker);
        hook.setNextSnapshot(
            _expansionSnapshot(3, block.timestamp, 510000000000000000, 25_000e18, 6_000e18, 34_000e18)
        );
        AGCDataTypes.EpochResult memory third =
            controller.settleEpoch(AGCDataTypes.ExternalMetrics({ depthToTargetSlippageQuoteX18: 180_000e18 }));

        assertEq(uint8(third.regime), uint8(AGCDataTypes.Regime.Expansion));
        assertEq(third.premiumPersistenceEpochs, 3);
        assertGt(third.mintBudgetAcp, 0);
        assertEq(agc.balanceOf(growthPrograms), third.mintAllocations.growthProgramsMintAcp);
        assertEq(agc.balanceOf(lpRewards), third.mintAllocations.lpMintAcp);
        assertEq(agc.balanceOf(integrators), third.mintAllocations.integratorsMintAcp);
        assertEq(agc.balanceOf(address(treasuryVault)), third.mintAllocations.treasuryMintAcp);
        assertEq(
            xagcVault.totalAssets(),
            180_000e18 + third.mintAllocations.xagcMintAcp
        );
    }

    function testDefenseQueuesBuybackAndRecoveryBlocksFreshExpansion() public {
        hook.setNextSnapshot(
            AGCDataTypes.EpochSnapshot({
                epochId: 1,
                startedAt: uint64(block.timestamp - 1 hours),
                endedAt: uint64(block.timestamp),
                grossBuyVolumeQuoteX18: 5_000e18,
                grossSellVolumeQuoteX18: 40_000e18,
                totalVolumeQuoteX18: 48_000e18,
                shortTwapPriceX18: 45e16,
                realizedVolatilityBps: 550,
                totalHookFeesQuoteX18: 0,
                totalHookFeesAgc: 0
            })
        );
        router.setNextBurnAmount(10e18);

        AGCDataTypes.EpochResult memory defense =
            controller.settleEpoch(AGCDataTypes.ExternalMetrics({ depthToTargetSlippageQuoteX18: 20_000e18 }));

        assertEq(uint8(defense.regime), uint8(AGCDataTypes.Regime.Defense));
        assertGt(defense.buybackBudgetQuoteX18, 0);
        assertGt(controller.pendingTreasuryBuybackUsdc(), 0);

        uint256 chunk = controller.pendingTreasuryBuybackUsdc() / 2;
        uint256 burned = controller.executePendingTreasuryBuyback(chunk, 1e18, 0);
        assertEq(burned, 10e18);
        assertEq(router.lastBuybackBudget(), chunk);

        vm.warp(block.timestamp + 1 hours);
        vm.prank(staker);
        xagcVault.deposit(10_000e18, staker);
        hook.setNextSnapshot(
            _expansionSnapshot(2, block.timestamp, 510000000000000000, 30_000e18, 6_000e18, 38_000e18)
        );

        AGCDataTypes.EpochResult memory recovery =
            controller.settleEpoch(AGCDataTypes.ExternalMetrics({ depthToTargetSlippageQuoteX18: 180_000e18 }));

        assertEq(uint8(recovery.regime), uint8(AGCDataTypes.Regime.Recovery));
        assertEq(recovery.mintBudgetAcp, 0);
    }

    function _expansionSnapshot(
        uint64 epochId,
        uint256 endedAt,
        uint256 priceTwapX18,
        uint256 grossBuyQuoteX18,
        uint256 grossSellQuoteX18,
        uint256 totalVolumeQuoteX18
    ) internal pure returns (AGCDataTypes.EpochSnapshot memory) {
        return AGCDataTypes.EpochSnapshot({
            epochId: epochId,
            startedAt: SafeCast.toUint64(endedAt - 1 hours),
            endedAt: SafeCast.toUint64(endedAt),
            grossBuyVolumeQuoteX18: grossBuyQuoteX18,
            grossSellVolumeQuoteX18: grossSellQuoteX18,
            totalVolumeQuoteX18: totalVolumeQuoteX18,
            shortTwapPriceX18: priceTwapX18,
            realizedVolatilityBps: 90,
            totalHookFeesQuoteX18: 0,
            totalHookFeesAgc: 0
        });
    }
}

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import "forge-std/Test.sol";
import {AGCToken} from "../../src/AGCToken.sol";
import {PolicyController} from "../../src/PolicyController.sol";
import {IAGCHook} from "../../src/interfaces/IAGCHook.sol";
import {IRewardDistributor} from "../../src/interfaces/IRewardDistributor.sol";
import {ISettlementRouter} from "../../src/interfaces/ISettlementRouter.sol";
import {IStabilityVault} from "../../src/interfaces/IStabilityVault.sol";
import {MockUSDC} from "../../src/mocks/MockUSDC.sol";
import {AGCDataTypes} from "../../src/libraries/AGCDataTypes.sol";
import {MockHookAdapter} from "../mocks/MockHookAdapter.sol";
import {MockRewardDistributor} from "../mocks/MockRewardDistributor.sol";
import {MockSettlementRouter} from "../mocks/MockSettlementRouter.sol";
import {MockVault} from "../mocks/MockVault.sol";

contract PolicyControllerTest is Test {
    bytes32 internal constant MINTER_ROLE = keccak256("MINTER_ROLE");

    AGCToken internal agc;
    MockUSDC internal usdc;
    MockHookAdapter internal hook;
    MockVault internal vault;
    MockRewardDistributor internal distributor;
    MockSettlementRouter internal router;
    PolicyController internal controller;

    function setUp() public {
        vm.warp(1 days);

        agc = new AGCToken(address(this));
        usdc = new MockUSDC(address(this));
        hook = new MockHookAdapter();
        vault = new MockVault(usdc, agc);
        distributor = new MockRewardDistributor();
        router = new MockSettlementRouter();

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
            agentBps: 3000,
            lpBps: 2000,
            integratorBps: 2000,
            treasuryBps: 2000,
            reserveBps: 1000
        });

        controller = new PolicyController(
            address(this),
            PolicyController.Dependencies({
                agcToken: agc,
                hookContract: IAGCHook(address(hook)),
                stabilityVault: IStabilityVault(address(vault)),
                rewardDistributor: IRewardDistributor(address(distributor)),
                router: ISettlementRouter(address(router))
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

    function testExpansionEpochMintsWithinCap() public {
        hook.setNextSnapshot(
            AGCDataTypes.EpochSnapshot({
                epochId: 1,
                startedAt: uint64(block.timestamp - 1 hours),
                endedAt: uint64(block.timestamp),
                productiveVolume: 500_000e6,
                totalVolume: 1_000_000e6,
                netExitVolume: 0,
                shortTwapPriceX18: 1e18,
                productiveSettlementPriceX18: 1e18,
                realizedVolatilityBps: 100,
                productiveSettlementCount: 10,
                productiveUsers: 5,
                repeatUsers: 3,
                totalHookFeesUsdc: 0,
                totalHookFeesAgc: 0
            })
        );

        controller.settleEpoch(
            PolicyController.EpochCommand({
                epochId: 1,
                regime: AGCDataTypes.Regime.Expansion,
                anchorPriceX18: 1e18,
                bandWidthBps: 200,
                mintBudget: 50e18,
                buybackBudget: 0,
                productiveUsageBps: 5000,
                coverageBps: 2000,
                exitPressureBps: 0,
                volatilityBps: 100,
                buybackMinAgcOut: 0
            })
        );

        assertEq(distributor.lastEpochId(), 1);
        assertEq(agc.balanceOf(address(distributor)), 35e18);
        assertEq(agc.balanceOf(address(vault)), 15e18);
    }

    function testMintWhileWeakReverts() public {
        hook.setNextSnapshot(
            AGCDataTypes.EpochSnapshot({
                epochId: 1,
                startedAt: uint64(block.timestamp - 1 hours),
                endedAt: uint64(block.timestamp),
                productiveVolume: 1,
                totalVolume: 1,
                netExitVolume: 0,
                shortTwapPriceX18: 9e17,
                productiveSettlementPriceX18: 9e17,
                realizedVolatilityBps: 100,
                productiveSettlementCount: 1,
                productiveUsers: 1,
                repeatUsers: 0,
                totalHookFeesUsdc: 0,
                totalHookFeesAgc: 0
            })
        );

        vm.expectRevert(PolicyController.MintForbiddenWhileWeak.selector);
        controller.settleEpoch(
            PolicyController.EpochCommand({
                epochId: 1,
                regime: AGCDataTypes.Regime.Expansion,
                anchorPriceX18: 1e18,
                bandWidthBps: 200,
                mintBudget: 10e18,
                buybackBudget: 0,
                productiveUsageBps: 10_000,
                coverageBps: 10_000,
                exitPressureBps: 0,
                volatilityBps: 100,
                buybackMinAgcOut: 0
            })
        );
    }

    function testDefenseEpochTriggersBuyback() public {
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
            PolicyController.EpochCommand({
                epochId: 1,
                regime: AGCDataTypes.Regime.Defense,
                anchorPriceX18: 999e15,
                bandWidthBps: 400,
                mintBudget: 0,
                buybackBudget: 100e6,
                productiveUsageBps: 0,
                coverageBps: 400,
                exitPressureBps: 10_000,
                volatilityBps: 600,
                buybackMinAgcOut: 50e18
            })
        );

        assertEq(router.lastBuybackBudget(), 100e6);
        assertEq(router.lastMinAgcOut(), 50e18);
    }
}

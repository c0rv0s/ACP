// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import "forge-std/Test.sol";
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
import { MockPolicyEngine } from "../mocks/MockPolicyEngine.sol";

contract PolicyControllerZeroSupplyTest is Test {
    bytes32 internal constant MINTER_ROLE = keccak256("MINTER_ROLE");

    AGCToken internal agc;
    MockUSDC internal usdc;
    MockHookAdapter internal hook;
    StabilityVault internal treasuryVault;
    XAGCVault internal xagcVault;
    MockSettlementRouter internal router;
    MockPolicyEngine internal engine;
    PolicyController internal controller;

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
        engine = new MockPolicyEngine();

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
                policyEngine: PolicyEngine(address(engine))
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

        hook.setNextSnapshot(
            AGCDataTypes.EpochSnapshot({
                epochId: 1,
                startedAt: uint64(block.timestamp - 1 hours),
                endedAt: uint64(block.timestamp),
                grossBuyVolumeQuoteX18: 0,
                grossSellVolumeQuoteX18: 0,
                totalVolumeQuoteX18: 0,
                shortTwapPriceX18: 5e17,
                realizedVolatilityBps: 0,
                totalHookFeesQuoteX18: 0,
                totalHookFeesAgc: 0
            })
        );

        engine.setNextResult(
            AGCDataTypes.EpochResult({
                epochId: 1,
                regime: AGCDataTypes.Regime.Expansion,
                anchorPriceX18: 5e17,
                anchorNextX18: 5e17,
                normalFloorX18: 0,
                stressedFloorX18: 0,
                priceTwapX18: 5e17,
                premiumBps: 0,
                premiumPersistenceEpochs: 0,
                creditOutstandingQuoteX18: 0,
                grossBuyFloorBps: 0,
                netBuyPressureBps: 0,
                buyGrowthBps: 0,
                exitPressureBps: 0,
                reserveCoverageBps: 0,
                lockedShareBps: 0,
                lockFlowBps: 0,
                demandScoreBps: 0,
                healthScoreBps: 0,
                mintRateBps: 0,
                mintBudgetAcp: 100e18,
                buybackBudgetQuoteX18: 0,
                stressScoreBps: 0,
                grossBuyQuoteX18: 0,
                grossSellQuoteX18: 0,
                totalVolumeQuoteX18: 0,
                depthToTargetSlippageQuoteX18: 0,
                realizedVolatilityBps: 0,
                xagcDepositsAcp: 0,
                xagcGrossRedemptionsAcp: 0,
                treasuryQuoteX18: 0,
                treasuryAcp: 0,
                xagcTotalAssetsAcp: 0,
                mintAllocations: AGCDataTypes.MintAllocation({
                    xagcMintAcp: 50e18,
                    growthProgramsMintAcp: 20e18,
                    lpMintAcp: 10e18,
                    integratorsMintAcp: 5e18,
                    treasuryMintAcp: 15e18
                })
            })
        );
    }

    function testExpansionMintRedirectsXagcAllocationWhenVaultHasNoShares() public {
        AGCDataTypes.EpochResult memory result =
            controller.settleEpoch(AGCDataTypes.ExternalMetrics({ depthToTargetSlippageQuoteX18: 0 }));

        assertEq(result.mintAllocations.xagcMintAcp, 0);
        assertEq(result.mintAllocations.treasuryMintAcp, 65e18);
        assertEq(agc.balanceOf(address(xagcVault)), 0);
        assertEq(agc.balanceOf(growthPrograms), 20e18);
        assertEq(agc.balanceOf(lpRewards), 10e18);
        assertEq(agc.balanceOf(integrators), 5e18);
        assertEq(agc.balanceOf(address(treasuryVault)), 65e18);
    }
}

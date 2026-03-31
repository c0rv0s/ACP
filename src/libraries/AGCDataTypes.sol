// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import { Currency } from "v4-core/types/Currency.sol";

library AGCDataTypes {
    uint256 internal constant BPS = 10_000;
    uint256 internal constant FEE_UNITS = 1_000_000;

    enum Regime {
        Neutral,
        Expansion,
        Defense,
        Recovery
    }

    enum FlowClass {
        Unknown,
        ProductivePayment,
        InventoryRebalance,
        SpeculativeTrade,
        LiquidityManagement,
        StressExit
    }

    enum RewardCategory {
        Agent,
        LP,
        Integrator
    }

    struct PoolConfig {
        Currency agcCurrency;
        Currency usdcCurrency;
        uint24 lpFee;
        int24 tickSpacing;
        uint8 agcDecimals;
        uint8 usdcDecimals;
    }

    struct HookFeeConfig {
        uint24 baseLPFee;
        uint24 volatilityFeeSlope;
        uint24 imbalanceFeeSlope;
        uint24 productiveDiscount;
        uint24 inventoryDiscount;
        uint24 speculativeSurcharge;
        uint24 defenseLpSurcharge;
        uint24 productiveHookFee;
        uint24 inventoryHookFee;
        uint24 speculativeHookFee;
        uint24 unknownHookFee;
        uint24 defenseExitHookFee;
        uint24 earlyWithdrawalFee;
        uint40 minLpHoldTime;
    }

    struct HookMetadata {
        address originalSender;
        address beneficiary;
        bytes32 intentHash;
        FlowClass flowClass;
        uint16 qualityScoreBps;
        bytes32 routeHash;
    }

    struct RewardReceipt {
        address beneficiary;
        address originalSender;
        bytes32 intentHash;
        FlowClass flowClass;
        uint64 epochId;
        uint64 createdAt;
        uint16 qualityScoreBps;
        uint256 agcAmount;
        uint256 usdcAmount;
        bool consumed;
    }

    struct EpochAccumulator {
        uint64 epochId;
        uint64 startedAt;
        uint64 updatedAt;
        uint64 lastObservedAt;
        uint64 observationCount;
        uint64 productiveSettlementCount;
        uint64 productiveUsers;
        uint64 repeatUsers;
        uint256 productiveVolume;
        uint256 totalVolume;
        int256 netExitVolume;
        uint256 lastMidPriceX18;
        uint256 cumulativeMidPriceTimeX18;
        uint256 cumulativeProductivePriceVolumeX18;
        uint256 cumulativeAbsMidPriceChangeBps;
        uint256 totalHookFeesUsdc;
        uint256 totalHookFeesAgc;
    }

    struct EpochSnapshot {
        uint64 epochId;
        uint64 startedAt;
        uint64 endedAt;
        uint256 productiveVolume;
        uint256 totalVolume;
        int256 netExitVolume;
        uint256 shortTwapPriceX18;
        uint256 productiveSettlementPriceX18;
        uint256 realizedVolatilityBps;
        uint64 productiveSettlementCount;
        uint64 productiveUsers;
        uint64 repeatUsers;
        uint256 totalHookFeesUsdc;
        uint256 totalHookFeesAgc;
    }

    struct RewardSplit {
        uint16 agentBps;
        uint16 lpBps;
        uint16 integratorBps;
        uint16 treasuryBps;
        uint16 reserveBps;
    }

    struct PolicyParams {
        uint16 baseBandBps;
        uint16 stressedBandBps;
        uint16 anchorEmaBps;
        uint16 maxAnchorCrawlBps;
        uint16 minProductiveUsageBps;
        uint16 minCoverageBps;
        uint16 criticalCoverageBps;
        uint16 maxExpansionVolatilityBps;
        uint16 criticalVolatilityBps;
        uint16 maxExpansionExitBps;
        uint16 criticalExitBps;
        uint16 maxMintPerEpochBps;
        uint16 maxMintPerDayBps;
        uint16 expansionKappaBps;
        uint16 buybackKappaBps;
        uint16 mildDefenseSpendBps;
        uint16 severeDefenseSpendBps;
        uint16 recoveryCooldownEpochs;
        uint64 policyEpochDuration;
        uint64 treasuryLockDuration;
    }

    struct ExternalMetrics {
        uint256 depthTo1Pct;
        uint256 depthTo2Pct;
        int256 productiveGrowthBps;
        uint256 lpStabilityBps;
        uint256 idleShareBps;
        uint256 buybackMinAgcOut;
    }

    struct EpochResult {
        uint64 epochId;
        Regime regime;
        uint256 anchorPriceX18;
        uint256 bandWidthBps;
        uint256 shortTwapPriceX18;
        uint256 productiveSettlementPriceX18;
        uint256 productiveUsageBps;
        uint256 coverageBps;
        uint256 exitPressureBps;
        uint256 volatilityBps;
        uint256 repeatUserBps;
        uint256 mintBudget;
        uint256 buybackBudget;
        uint256 floatSupply;
        uint256 depthTo1Pct;
        int256 productiveGrowthBps;
        uint256 depthTo2Pct;
        uint256 lpStabilityBps;
        uint256 idleShareBps;
        uint256 buybackMinAgcOut;
    }

    struct DerivedMetrics {
        uint256 floatSupply;
        uint256 price;
        uint256 productiveUsageBps;
        uint256 coverageBps;
        uint256 exitPressureBps;
        uint256 repeatUserBps;
        uint256 volatilityBps;
    }

    struct RewardBudget {
        uint256 agentBudget;
        uint256 lpBudget;
        uint256 integratorBudget;
        uint256 agentRemaining;
        uint256 lpRemaining;
        uint256 integratorRemaining;
        bool funded;
    }

    struct RewardStream {
        address beneficiary;
        RewardCategory category;
        uint64 startTime;
        uint64 endTime;
        uint128 totalAmount;
        uint128 claimedAmount;
        bytes32 source;
    }
}

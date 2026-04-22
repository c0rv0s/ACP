// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import { Currency } from "v4-core/types/Currency.sol";

library AGCDataTypes {
    uint256 internal constant BPS = 10_000;
    uint256 internal constant FEE_UNITS = 1_000_000;
    uint256 internal constant QUOTE_SCALE = 1e12;

    enum Regime {
        Neutral,
        Expansion,
        Defense,
        Recovery
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
        uint24 defenseLpSurcharge;
        uint24 buyHookFee;
        uint24 sellHookFee;
        uint24 defenseExitHookFee;
        uint24 earlyWithdrawalFee;
        uint40 minLpHoldTime;
    }

    struct EpochAccumulator {
        uint64 epochId;
        uint64 startedAt;
        uint64 updatedAt;
        uint64 lastObservedAt;
        uint64 observationCount;
        uint256 grossBuyVolumeQuoteX18;
        uint256 grossSellVolumeQuoteX18;
        uint256 totalVolumeQuoteX18;
        uint256 lastMidPriceX18;
        uint256 cumulativeMidPriceTimeX18;
        uint256 cumulativeAbsMidPriceChangeBps;
        uint256 totalHookFeesQuoteX18;
        uint256 totalHookFeesAgc;
    }

    struct EpochSnapshot {
        uint64 epochId;
        uint64 startedAt;
        uint64 endedAt;
        uint256 grossBuyVolumeQuoteX18;
        uint256 grossSellVolumeQuoteX18;
        uint256 totalVolumeQuoteX18;
        uint256 shortTwapPriceX18;
        uint256 realizedVolatilityBps;
        uint256 totalHookFeesQuoteX18;
        uint256 totalHookFeesAgc;
    }

    struct PolicyParams {
        uint16 normalBandBps;
        uint16 stressedBandBps;
        uint16 anchorEmaBps;
        uint16 maxAnchorCrawlBps;
        uint16 minPremiumBps;
        uint16 premiumPersistenceRequired;
        uint16 minGrossBuyFloorBps;
        uint16 minLockedShareBps;
        uint16 targetGrossBuyBps;
        uint16 targetNetBuyBps;
        uint16 targetLockFlowBps;
        uint16 targetBuyGrowthBps;
        uint16 targetLockedShareBps;
        uint16 expansionReserveCoverageBps;
        uint16 targetReserveCoverageBps;
        uint16 neutralReserveCoverageBps;
        uint16 defenseReserveCoverageBps;
        uint16 hardDefenseReserveCoverageBps;
        uint16 maxExpansionVolatilityBps;
        uint16 defenseVolatilityBps;
        uint16 maxExpansionExitPressureBps;
        uint16 defenseExitPressureBps;
        uint16 expansionKappaBps;
        uint16 maxMintPerEpochBps;
        uint16 maxMintPerDayBps;
        uint16 buybackKappaBps;
        uint16 mildDefenseSpendBps;
        uint16 severeDefenseSpendBps;
        uint16 severeStressThresholdBps;
        uint16 recoveryCooldownEpochs;
        uint64 policyEpochDuration;
    }

    struct MintDistribution {
        uint16 xagcBps;
        uint16 growthProgramsBps;
        uint16 lpBps;
        uint16 integratorsBps;
        uint16 treasuryBps;
    }

    struct SettlementRecipients {
        address growthPrograms;
        address lp;
        address integrators;
    }

    struct ExternalMetrics {
        uint256 depthToTargetSlippageQuoteX18;
    }

    struct PolicyState {
        uint256 anchorPriceX18;
        uint256 premiumPersistenceEpochs;
        uint256 lastGrossBuyQuoteX18;
        uint256 mintedTodayAcp;
        Regime lastRegime;
        uint64 recoveryCooldownEpochsRemaining;
        uint256 floatSupplyAcp;
        uint256 treasuryQuoteX18;
        uint256 treasuryAcp;
        uint256 xagcTotalAssetsAcp;
    }

    struct VaultFlows {
        uint256 xagcDepositsAcp;
        uint256 xagcGrossRedemptionsAcp;
    }

    struct MintAllocation {
        uint256 xagcMintAcp;
        uint256 growthProgramsMintAcp;
        uint256 lpMintAcp;
        uint256 integratorsMintAcp;
        uint256 treasuryMintAcp;
    }

    struct DerivedMetrics {
        uint256 creditOutstandingQuoteX18;
        uint256 grossBuyFloorBps;
        uint256 netBuyPressureBps;
        uint256 buyGrowthBps;
        uint256 exitPressureBps;
        uint256 reserveCoverageBps;
        uint256 lockedShareBps;
        uint256 xagcExitFeeAcp;
        uint256 xagcNetRedemptionAcp;
        uint256 lockFlowBps;
        uint256 premiumBps;
        uint256 premiumPersistenceEpochs;
        uint256 demandScoreBps;
        uint256 healthScoreBps;
        uint256 stressScoreBps;
    }

    struct EpochResult {
        uint64 epochId;
        Regime regime;
        uint256 anchorPriceX18;
        uint256 anchorNextX18;
        uint256 normalFloorX18;
        uint256 stressedFloorX18;
        uint256 priceTwapX18;
        uint256 premiumBps;
        uint256 premiumPersistenceEpochs;
        uint256 creditOutstandingQuoteX18;
        uint256 grossBuyFloorBps;
        uint256 netBuyPressureBps;
        uint256 buyGrowthBps;
        uint256 exitPressureBps;
        uint256 reserveCoverageBps;
        uint256 lockedShareBps;
        uint256 lockFlowBps;
        uint256 demandScoreBps;
        uint256 healthScoreBps;
        uint256 mintRateBps;
        uint256 mintBudgetAcp;
        uint256 buybackBudgetQuoteX18;
        uint256 stressScoreBps;
        uint256 grossBuyQuoteX18;
        uint256 grossSellQuoteX18;
        uint256 totalVolumeQuoteX18;
        uint256 depthToTargetSlippageQuoteX18;
        uint256 realizedVolatilityBps;
        uint256 xagcDepositsAcp;
        uint256 xagcGrossRedemptionsAcp;
        uint256 treasuryQuoteX18;
        uint256 treasuryAcp;
        uint256 xagcTotalAssetsAcp;
        MintAllocation mintAllocations;
    }
}

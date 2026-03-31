// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import { Ownable } from "@openzeppelin/contracts/access/Ownable.sol";
import { Ownable2Step } from "@openzeppelin/contracts/access/Ownable2Step.sol";
import { AGCToken } from "./AGCToken.sol";
import { IAGCHook } from "./interfaces/IAGCHook.sol";
import { IRewardDistributor } from "./interfaces/IRewardDistributor.sol";
import { ISettlementRouter } from "./interfaces/ISettlementRouter.sol";
import { IStabilityVault } from "./interfaces/IStabilityVault.sol";
import { AGCDataTypes } from "./libraries/AGCDataTypes.sol";
import { PolicyEngine } from "./PolicyEngine.sol";

contract PolicyController is Ownable2Step {
    error Unauthorized();
    error InvalidRewardSplit();
    error EpochTooSoon();
    error InvalidEpoch();
    error InvalidRewardRequest();

    event KeeperUpdated(address indexed keeper, bool allowed);
    event PolicyParametersUpdated(uint16 baseBandBps, uint16 stressedBandBps, uint64 epochDuration);
    event RewardSplitUpdated(
        uint16 agentBps, uint16 lpBps, uint16 integratorBps, uint16 treasuryBps, uint16 reserveBps
    );
    event EmissionsForceDisabledUpdated(bool disabled);
    event EpochSettled(
        uint64 indexed epochId,
        AGCDataTypes.Regime indexed regime,
        uint256 anchorPriceX18,
        uint256 mintBudget,
        uint256 buybackBudget
    );
    event RewardBudgetStreamScheduled(
        uint64 indexed epochId,
        AGCDataTypes.RewardCategory indexed category,
        address indexed beneficiary,
        uint256 amount,
        uint64 duration,
        bytes32 source,
        uint256 streamId
    );

    struct Dependencies {
        AGCToken agcToken;
        IAGCHook hookContract;
        IStabilityVault stabilityVault;
        IRewardDistributor rewardDistributor;
        ISettlementRouter router;
        PolicyEngine policyEngine;
    }

    struct RewardStreamRequest {
        address beneficiary;
        uint256 amount;
        uint64 duration;
        bytes32 source;
    }

    AGCToken public immutable agc;
    IAGCHook public immutable hook;
    IStabilityVault public immutable vault;
    IRewardDistributor public immutable distributor;
    ISettlementRouter public immutable settlementRouter;
    PolicyEngine public immutable policyEngine;

    AGCDataTypes.PolicyParams public policyParams;
    AGCDataTypes.RewardSplit public rewardSplit;
    AGCDataTypes.Regime public regime;

    uint256 public anchorPriceX18;
    uint256 public bandWidthBps;
    uint256 public lastProductiveUsageBps;
    uint256 public lastCoverageBps;
    uint256 public lastExitPressureBps;
    uint256 public lastVolatilityBps;

    uint64 public lastSettledEpoch;
    uint64 public lastSettlementTimestamp;
    uint64 public recoveryCooldownUntilEpoch;
    uint64 public mintWindowDay;

    uint256 public mintedInCurrentDay;
    bool public emissionsForceDisabled;

    mapping(address keeper => bool allowed) public keepers;
    mapping(uint64 epochId => AGCDataTypes.EpochResult result) public epochResult;

    AGCDataTypes.EpochResult internal _lastEpochResult;

    constructor(
        address admin,
        Dependencies memory deps,
        uint256 initialAnchorPriceX18,
        AGCDataTypes.PolicyParams memory params,
        AGCDataTypes.RewardSplit memory split
    ) Ownable(admin) {
        agc = deps.agcToken;
        hook = deps.hookContract;
        vault = deps.stabilityVault;
        distributor = deps.rewardDistributor;
        settlementRouter = deps.router;
        policyEngine = deps.policyEngine;

        anchorPriceX18 = initialAnchorPriceX18;
        bandWidthBps = params.baseBandBps;
        policyParams = params;
        regime = AGCDataTypes.Regime.Neutral;

        _setRewardSplit(split);
    }

    modifier onlyKeeperOrOwner() {
        if (msg.sender != owner() && !keepers[msg.sender]) revert Unauthorized();
        _;
    }

    function setKeeper(
        address keeper,
        bool allowed
    ) external onlyOwner {
        keepers[keeper] = allowed;
        emit KeeperUpdated(keeper, allowed);
    }

    function setPolicyParams(
        AGCDataTypes.PolicyParams calldata params
    ) external onlyOwner {
        policyParams = params;
        emit PolicyParametersUpdated(
            params.baseBandBps, params.stressedBandBps, params.policyEpochDuration
        );
    }

    function setRewardSplit(
        AGCDataTypes.RewardSplit calldata split
    ) external onlyOwner {
        _setRewardSplit(split);
    }

    function setEmissionsForceDisabled(
        bool disabled
    ) external onlyOwner {
        emissionsForceDisabled = disabled;
        emit EmissionsForceDisabledUpdated(disabled);
    }

    function previewEpoch(
        AGCDataTypes.ExternalMetrics calldata externalMetrics
    ) external view returns (AGCDataTypes.EpochResult memory result) {
        return _deriveEpochResult(hook.previewEpochSnapshot(), externalMetrics);
    }

    function settleEpoch(
        AGCDataTypes.ExternalMetrics calldata externalMetrics
    ) external onlyKeeperOrOwner returns (AGCDataTypes.EpochResult memory result) {
        _validateSettlementWindow();
        _refreshMintWindow();

        AGCDataTypes.EpochSnapshot memory snapshot = hook.consumeEpochSnapshot();
        if (snapshot.epochId <= lastSettledEpoch) revert InvalidEpoch();

        result = _deriveEpochResult(snapshot, externalMetrics);

        anchorPriceX18 = result.anchorPriceX18;
        bandWidthBps = result.bandWidthBps;
        regime = result.regime;

        if (result.regime == AGCDataTypes.Regime.Defense) {
            recoveryCooldownUntilEpoch = snapshot.epochId + policyParams.recoveryCooldownEpochs;
        }

        hook.setRegime(result.regime);

        if (result.mintBudget > 0) {
            mintedInCurrentDay += result.mintBudget;
            _distributeExpansionMint(snapshot.epochId, result.mintBudget);
        }

        if (result.buybackBudget > 0 && externalMetrics.buybackMinAgcOut > 0) {
            settlementRouter.executeTreasuryBuyback(
                result.buybackBudget,
                externalMetrics.buybackMinAgcOut,
                keccak256(abi.encodePacked("buyback", snapshot.epochId, block.timestamp))
            );
        }

        lastSettledEpoch = snapshot.epochId;
        lastSettlementTimestamp = uint64(block.timestamp);
        lastProductiveUsageBps = result.productiveUsageBps;
        lastCoverageBps = result.coverageBps;
        lastExitPressureBps = result.exitPressureBps;
        lastVolatilityBps = result.volatilityBps;
        epochResult[snapshot.epochId] = result;
        _lastEpochResult = result;

        emit EpochSettled(
            snapshot.epochId,
            result.regime,
            result.anchorPriceX18,
            result.mintBudget,
            result.buybackBudget
        );
    }

    function lastEpochResult() external view returns (AGCDataTypes.EpochResult memory result) {
        return _lastEpochResult;
    }

    function scheduleLpRewardStreams(
        uint64 epochId,
        RewardStreamRequest[] calldata requests
    ) external onlyKeeperOrOwner returns (uint256[] memory streamIds) {
        return _scheduleRewardStreams(epochId, AGCDataTypes.RewardCategory.LP, requests);
    }

    function scheduleIntegratorRewardStreams(
        uint64 epochId,
        RewardStreamRequest[] calldata requests
    ) external onlyKeeperOrOwner returns (uint256[] memory streamIds) {
        return _scheduleRewardStreams(epochId, AGCDataTypes.RewardCategory.Integrator, requests);
    }

    function circulatingFloat() public view returns (uint256) {
        uint256 totalSupply = agc.totalSupply();
        uint256 sequestered = agc.balanceOf(address(vault)) + agc.balanceOf(address(distributor));
        return totalSupply > sequestered ? totalSupply - sequestered : 0;
    }

    function _deriveEpochResult(
        AGCDataTypes.EpochSnapshot memory snapshot,
        AGCDataTypes.ExternalMetrics calldata externalMetrics
    ) internal view returns (AGCDataTypes.EpochResult memory result) {
        uint256 floatSupply = circulatingFloat();
        AGCDataTypes.DerivedMetrics memory metrics =
            policyEngine.deriveMetrics(snapshot, externalMetrics, floatSupply);

        AGCDataTypes.Regime nextRegime = policyEngine.selectRegime(
            metrics.price,
            anchorPriceX18,
            policyParams.baseBandBps,
            policyParams,
            metrics,
            externalMetrics.productiveGrowthBps
        );

        if (
            nextRegime != AGCDataTypes.Regime.Defense
                && snapshot.epochId < recoveryCooldownUntilEpoch
        ) {
            nextRegime = AGCDataTypes.Regime.Recovery;
        }

        uint256 nextBandWidthBps = nextRegime == AGCDataTypes.Regime.Defense
            || nextRegime == AGCDataTypes.Regime.Recovery
            ? policyParams.stressedBandBps
            : policyParams.baseBandBps;
        uint256 nextAnchorPriceX18 =
            policyEngine.updateAnchor(snapshot, anchorPriceX18, policyParams);

        uint256 mintBudget;
        if (nextRegime == AGCDataTypes.Regime.Expansion && !emissionsForceDisabled) {
            mintBudget = policyEngine.mintBudget(
                metrics,
                externalMetrics.productiveGrowthBps,
                policyParams,
                _remainingDailyMintCapacity(floatSupply)
            );
            uint256 maxMintBudget = _maxMintBudget(floatSupply);
            if (mintBudget > maxMintBudget) {
                mintBudget = maxMintBudget;
            }
        }

        uint256 buybackBudget;
        if (nextRegime == AGCDataTypes.Regime.Defense) {
            buybackBudget = policyEngine.buybackBudget(
                metrics.price,
                nextAnchorPriceX18,
                nextBandWidthBps,
                metrics,
                vault.availableUsdc(),
                policyParams
            );
            uint256 maxBuybackBudget = _maxBuybackBudget();
            if (buybackBudget > maxBuybackBudget) {
                buybackBudget = maxBuybackBudget;
            }
        }

        result = AGCDataTypes.EpochResult({
            epochId: snapshot.epochId,
            regime: nextRegime,
            anchorPriceX18: nextAnchorPriceX18,
            bandWidthBps: nextBandWidthBps,
            shortTwapPriceX18: snapshot.shortTwapPriceX18,
            productiveSettlementPriceX18: snapshot.productiveSettlementPriceX18,
            productiveUsageBps: metrics.productiveUsageBps,
            coverageBps: metrics.coverageBps,
            exitPressureBps: metrics.exitPressureBps,
            volatilityBps: metrics.volatilityBps,
            repeatUserBps: metrics.repeatUserBps,
            mintBudget: mintBudget,
            buybackBudget: buybackBudget,
            floatSupply: floatSupply,
            depthTo1Pct: externalMetrics.depthTo1Pct,
            productiveGrowthBps: externalMetrics.productiveGrowthBps,
            depthTo2Pct: externalMetrics.depthTo2Pct,
            lpStabilityBps: externalMetrics.lpStabilityBps,
            idleShareBps: externalMetrics.idleShareBps,
            buybackMinAgcOut: externalMetrics.buybackMinAgcOut
        });
    }

    function _refreshMintWindow() internal {
        uint64 currentDay = uint64(block.timestamp / 1 days);
        if (mintWindowDay != currentDay) {
            mintWindowDay = currentDay;
            mintedInCurrentDay = 0;
        }
    }

    function _validateSettlementWindow() internal view {
        if (
            lastSettlementTimestamp != 0
                && block.timestamp < lastSettlementTimestamp + policyParams.policyEpochDuration
        ) revert EpochTooSoon();
    }

    function _remainingDailyMintCapacity(
        uint256 floatSupply
    ) internal view returns (uint256) {
        uint256 dayCap = floatSupply * policyParams.maxMintPerDayBps / AGCDataTypes.BPS;
        return dayCap > mintedInCurrentDay ? dayCap - mintedInCurrentDay : 0;
    }

    function _maxMintBudget(
        uint256 floatSupply
    ) internal view returns (uint256) {
        uint256 epochCap = floatSupply * policyParams.maxMintPerEpochBps / AGCDataTypes.BPS;
        uint256 remainingDayCap = _remainingDailyMintCapacity(floatSupply);
        return epochCap < remainingDayCap ? epochCap : remainingDayCap;
    }

    function _maxBuybackBudget() internal view returns (uint256) {
        return vault.availableUsdc() * policyParams.severeDefenseSpendBps / AGCDataTypes.BPS;
    }

    function _setRewardSplit(
        AGCDataTypes.RewardSplit memory split
    ) internal {
        uint256 total = uint256(split.agentBps) + split.lpBps + split.integratorBps
            + split.treasuryBps + split.reserveBps;
        if (total != AGCDataTypes.BPS) revert InvalidRewardSplit();

        rewardSplit = split;
        emit RewardSplitUpdated(
            split.agentBps, split.lpBps, split.integratorBps, split.treasuryBps, split.reserveBps
        );
    }

    function _distributeExpansionMint(
        uint64 epochId,
        uint256 mintBudget
    ) internal {
        uint256 agentBudget = mintBudget * rewardSplit.agentBps / AGCDataTypes.BPS;
        uint256 lpBudget = mintBudget * rewardSplit.lpBps / AGCDataTypes.BPS;
        uint256 integratorBudget = mintBudget * rewardSplit.integratorBps / AGCDataTypes.BPS;
        uint256 treasuryBudget = mintBudget * rewardSplit.treasuryBps / AGCDataTypes.BPS;
        uint256 reserveBudget =
            mintBudget - agentBudget - lpBudget - integratorBudget - treasuryBudget;

        uint256 distributorBudget = agentBudget + lpBudget + integratorBudget;
        if (distributorBudget > 0) {
            agc.mint(address(distributor), distributorBudget);
            distributor.fundEpoch(epochId, agentBudget, lpBudget, integratorBudget);
        }

        uint256 vaultBudget = treasuryBudget + reserveBudget;
        if (vaultBudget > 0) {
            agc.mint(address(vault), vaultBudget);
            if (treasuryBudget > 0) {
                vault.lockTreasuryMint(treasuryBudget, policyParams.treasuryLockDuration);
            }
        }
    }

    function _scheduleRewardStreams(
        uint64 epochId,
        AGCDataTypes.RewardCategory category,
        RewardStreamRequest[] calldata requests
    ) internal returns (uint256[] memory streamIds) {
        uint256 length = requests.length;
        streamIds = new uint256[](length);

        for (uint256 i = 0; i < length; ++i) {
            RewardStreamRequest calldata request = requests[i];
            if (request.beneficiary == address(0) || request.amount == 0) {
                revert InvalidRewardRequest();
            }

            uint256 streamId = distributor.scheduleBudgetStream(
                epochId,
                category,
                request.beneficiary,
                request.amount,
                request.duration,
                request.source
            );
            streamIds[i] = streamId;

            emit RewardBudgetStreamScheduled(
                epochId,
                category,
                request.beneficiary,
                request.amount,
                request.duration,
                request.source,
                streamId
            );
        }
    }
}

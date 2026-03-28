// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {Ownable2Step} from "@openzeppelin/contracts/access/Ownable2Step.sol";
import {AGCToken} from "./AGCToken.sol";
import {IAGCHook} from "./interfaces/IAGCHook.sol";
import {IRewardDistributor} from "./interfaces/IRewardDistributor.sol";
import {ISettlementRouter} from "./interfaces/ISettlementRouter.sol";
import {IStabilityVault} from "./interfaces/IStabilityVault.sol";
import {AGCDataTypes} from "./libraries/AGCDataTypes.sol";

contract PolicyController is Ownable2Step {
    error Unauthorized();
    error InvalidRewardSplit();
    error EpochTooSoon();
    error InvalidEpoch();
    error InvalidBand();
    error AnchorMoveTooLarge();
    error ExpansionCooldownActive();
    error MintBudgetTooLarge();
    error BuybackBudgetTooLarge();
    error MintForbiddenWhileWeak();
    error BuybackRequiresDefense();

    event KeeperUpdated(address indexed keeper, bool allowed);
    event PolicyParametersUpdated(uint16 baseBandBps, uint16 stressedBandBps, uint64 epochDuration);
    event RewardSplitUpdated(
        uint16 agentBps, uint16 lpBps, uint16 integratorBps, uint16 treasuryBps, uint16 reserveBps
    );
    event EpochSettled(
        uint64 indexed epochId,
        AGCDataTypes.Regime indexed regime,
        uint256 anchorPriceX18,
        uint256 mintBudget,
        uint256 buybackBudget
    );

    struct Dependencies {
        AGCToken agcToken;
        IAGCHook hookContract;
        IStabilityVault stabilityVault;
        IRewardDistributor rewardDistributor;
        ISettlementRouter router;
    }

    struct EpochCommand {
        uint64 epochId;
        AGCDataTypes.Regime regime;
        uint256 anchorPriceX18;
        uint256 bandWidthBps;
        uint256 mintBudget;
        uint256 buybackBudget;
        uint256 productiveUsageBps;
        uint256 coverageBps;
        uint256 exitPressureBps;
        uint256 volatilityBps;
        uint256 buybackMinAgcOut;
    }

    struct SettlementContext {
        AGCDataTypes.EpochSnapshot snapshot;
        uint256 floatSupply;
        uint256 maxMintBudget;
        uint256 maxBuybackBudget;
    }

    AGCToken public immutable agc;
    IAGCHook public immutable hook;
    IStabilityVault public immutable vault;
    IRewardDistributor public immutable distributor;
    ISettlementRouter public immutable settlementRouter;

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

    mapping(address keeper => bool allowed) public keepers;

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

    function setKeeper(address keeper, bool allowed) external onlyOwner {
        keepers[keeper] = allowed;
        emit KeeperUpdated(keeper, allowed);
    }

    function setPolicyParams(AGCDataTypes.PolicyParams calldata params) external onlyOwner {
        policyParams = params;
        emit PolicyParametersUpdated(params.baseBandBps, params.stressedBandBps, params.policyEpochDuration);
    }

    function setRewardSplit(AGCDataTypes.RewardSplit calldata split) external onlyOwner {
        _setRewardSplit(split);
    }

    function settleEpoch(EpochCommand calldata command) external onlyKeeperOrOwner {
        _validateSettlementWindow();

        SettlementContext memory context;
        context.snapshot = hook.consumeEpochSnapshot();
        context.floatSupply = circulatingFloat();

        _validateEpochCommand(command, context.snapshot);
        _refreshMintWindow();

        context.maxMintBudget = _maxMintBudget(context.floatSupply);
        context.maxBuybackBudget = _maxBuybackBudget(command.regime);

        _validateBudgets(command, context);
        _applySettlementState(command, context.snapshot.epochId);
        _executeSettlementActions(command, context);
        _recordSettlementMetrics(command, context.snapshot.epochId);

        emit EpochSettled(
            context.snapshot.epochId, command.regime, anchorPriceX18, command.mintBudget, command.buybackBudget
        );
    }

    function circulatingFloat() public view returns (uint256) {
        uint256 totalSupply = agc.totalSupply();
        uint256 sequestered = agc.balanceOf(address(vault)) + agc.balanceOf(address(distributor));
        return totalSupply > sequestered ? totalSupply - sequestered : 0;
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

    function _validateEpochCommand(EpochCommand calldata command, AGCDataTypes.EpochSnapshot memory snapshot)
        internal
        view
    {
        if (snapshot.epochId != command.epochId) revert InvalidEpoch();
        if (command.bandWidthBps < policyParams.baseBandBps || command.bandWidthBps > policyParams.stressedBandBps) {
            revert InvalidBand();
        }
        if (command.regime == AGCDataTypes.Regime.Expansion && snapshot.epochId < recoveryCooldownUntilEpoch) {
            revert ExpansionCooldownActive();
        }
    }

    function _validateBudgets(EpochCommand calldata command, SettlementContext memory context) internal view {
        if (command.mintBudget > context.maxMintBudget) revert MintBudgetTooLarge();
        if (context.snapshot.shortTwapPriceX18 < anchorPriceX18 && command.mintBudget > 0) {
            revert MintForbiddenWhileWeak();
        }
        if (command.regime != AGCDataTypes.Regime.Expansion && command.mintBudget > 0) {
            revert MintForbiddenWhileWeak();
        }

        if (command.buybackBudget > context.maxBuybackBudget) revert BuybackBudgetTooLarge();
        if (command.regime != AGCDataTypes.Regime.Defense && command.buybackBudget > 0) {
            revert BuybackRequiresDefense();
        }
    }

    function _applySettlementState(EpochCommand calldata command, uint64 epochId) internal {
        anchorPriceX18 = _clampAnchor(command.anchorPriceX18);
        bandWidthBps = command.bandWidthBps;
        regime = command.regime;

        if (command.regime == AGCDataTypes.Regime.Defense) {
            recoveryCooldownUntilEpoch = epochId + policyParams.recoveryCooldownEpochs;
        }

        hook.setRegime(command.regime);
    }

    function _executeSettlementActions(EpochCommand calldata command, SettlementContext memory context) internal {
        if (command.mintBudget > 0) {
            mintedInCurrentDay += command.mintBudget;
            _distributeExpansionMint(context.snapshot.epochId, command.mintBudget);
        }

        if (command.buybackBudget > 0 && command.buybackMinAgcOut > 0) {
            settlementRouter.executeTreasuryBuyback(
                command.buybackBudget,
                command.buybackMinAgcOut,
                keccak256(abi.encodePacked("buyback", context.snapshot.epochId, block.timestamp))
            );
        }
    }

    function _recordSettlementMetrics(EpochCommand calldata command, uint64 epochId) internal {
        lastSettledEpoch = epochId;
        lastSettlementTimestamp = uint64(block.timestamp);
        lastProductiveUsageBps = command.productiveUsageBps;
        lastCoverageBps = command.coverageBps;
        lastExitPressureBps = command.exitPressureBps;
        lastVolatilityBps = command.volatilityBps;
    }

    function _maxMintBudget(uint256 floatSupply) internal view returns (uint256) {
        uint256 epochCap = floatSupply * policyParams.maxMintPerEpochBps / AGCDataTypes.BPS;
        uint256 dayCap = floatSupply * policyParams.maxMintPerDayBps / AGCDataTypes.BPS;
        uint256 remainingDayCap = dayCap > mintedInCurrentDay ? dayCap - mintedInCurrentDay : 0;
        return epochCap < remainingDayCap ? epochCap : remainingDayCap;
    }

    function _maxBuybackBudget(AGCDataTypes.Regime targetRegime) internal view returns (uint256) {
        if (targetRegime != AGCDataTypes.Regime.Defense) return 0;
        return vault.availableUsdc() * policyParams.severeDefenseSpendBps / AGCDataTypes.BPS;
    }

    function _clampAnchor(uint256 nextAnchor) internal view returns (uint256) {
        uint256 minAnchor = anchorPriceX18 * (AGCDataTypes.BPS - policyParams.maxAnchorCrawlBps) / AGCDataTypes.BPS;
        uint256 maxAnchor = anchorPriceX18 * (AGCDataTypes.BPS + policyParams.maxAnchorCrawlBps) / AGCDataTypes.BPS;
        if (nextAnchor < minAnchor || nextAnchor > maxAnchor) revert AnchorMoveTooLarge();
        return nextAnchor;
    }

    function _setRewardSplit(AGCDataTypes.RewardSplit memory split) internal {
        uint256 total = uint256(split.agentBps) + split.lpBps + split.integratorBps + split.treasuryBps + split.reserveBps;
        if (total != AGCDataTypes.BPS) revert InvalidRewardSplit();

        rewardSplit = split;
        emit RewardSplitUpdated(split.agentBps, split.lpBps, split.integratorBps, split.treasuryBps, split.reserveBps);
    }

    function _distributeExpansionMint(uint64 epochId, uint256 mintBudget) internal {
        uint256 agentBudget = mintBudget * rewardSplit.agentBps / AGCDataTypes.BPS;
        uint256 lpBudget = mintBudget * rewardSplit.lpBps / AGCDataTypes.BPS;
        uint256 integratorBudget = mintBudget * rewardSplit.integratorBps / AGCDataTypes.BPS;
        uint256 treasuryBudget = mintBudget * rewardSplit.treasuryBps / AGCDataTypes.BPS;
        uint256 reserveBudget = mintBudget - agentBudget - lpBudget - integratorBudget - treasuryBudget;

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
}

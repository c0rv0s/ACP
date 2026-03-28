// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {Ownable2Step} from "@openzeppelin/contracts/access/Ownable2Step.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {AGCToken} from "./AGCToken.sol";
import {IAGCHook} from "./interfaces/IAGCHook.sol";
import {IRewardDistributor} from "./interfaces/IRewardDistributor.sol";
import {AGCDataTypes} from "./libraries/AGCDataTypes.sol";

contract RewardDistributor is Ownable2Step, IRewardDistributor {
    using SafeERC20 for AGCToken;

    error Unauthorized();
    error InvalidDuration();
    error EpochNotFunded();
    error BudgetExceeded();
    error WrongBeneficiary();
    error StreamNotFound();
    error NothingToClaim();

    event ControllerUpdated(address indexed controller);
    event RewardParametersUpdated(uint256 baseRewardRateX18, uint256 rewardScale);
    event DurationsUpdated(uint64 agentDuration, uint64 lpDuration, uint64 integratorDuration);
    event EpochFunded(uint64 indexed epochId, uint256 agentBudget, uint256 lpBudget, uint256 integratorBudget);
    event StreamScheduled(
        uint256 indexed streamId,
        uint64 indexed epochId,
        address indexed beneficiary,
        AGCDataTypes.RewardCategory category,
        uint256 amount,
        uint64 startTime,
        uint64 endTime,
        bytes32 source
    );
    event ReceiptClaimed(uint256 indexed receiptId, uint256 indexed streamId, address indexed beneficiary, uint256 amount);
    event StreamClaimed(uint256 indexed streamId, address indexed beneficiary, uint256 amount);

    AGCToken public immutable agc;
    IAGCHook public immutable hook;

    address public controller;

    uint256 public baseRewardRateX18;
    uint256 public rewardScale;

    uint64 public agentStreamDuration;
    uint64 public lpStreamDuration;
    uint64 public integratorStreamDuration;

    uint256 public nextStreamId = 1;

    mapping(uint64 epochId => AGCDataTypes.RewardBudget budget) public epochBudget;
    mapping(uint256 streamId => AGCDataTypes.RewardStream stream) public rewardStream;

    constructor(address admin, AGCToken agcToken, IAGCHook hookContract) Ownable(admin) {
        agc = agcToken;
        hook = hookContract;

        baseRewardRateX18 = 2e17;
        rewardScale = 1e6;
        agentStreamDuration = 48 hours;
        lpStreamDuration = 7 days;
        integratorStreamDuration = 14 days;
    }

    modifier onlyController() {
        if (msg.sender != controller) revert Unauthorized();
        _;
    }

    function setController(address nextController) external onlyOwner {
        controller = nextController;
        emit ControllerUpdated(nextController);
    }

    function setRewardParameters(uint256 newBaseRewardRateX18, uint256 newRewardScale) external onlyOwner {
        baseRewardRateX18 = newBaseRewardRateX18;
        rewardScale = newRewardScale;
        emit RewardParametersUpdated(newBaseRewardRateX18, newRewardScale);
    }

    function setStreamDurations(uint64 newAgentDuration, uint64 newLpDuration, uint64 newIntegratorDuration)
        external
        onlyOwner
    {
        if (newAgentDuration == 0 || newLpDuration == 0 || newIntegratorDuration == 0) revert InvalidDuration();

        agentStreamDuration = newAgentDuration;
        lpStreamDuration = newLpDuration;
        integratorStreamDuration = newIntegratorDuration;

        emit DurationsUpdated(newAgentDuration, newLpDuration, newIntegratorDuration);
    }

    function fundEpoch(uint64 epochId, uint256 agentBudget, uint256 lpBudget, uint256 integratorBudget)
        external
        onlyController
    {
        AGCDataTypes.RewardBudget storage budget = epochBudget[epochId];
        budget.agentBudget += agentBudget;
        budget.lpBudget += lpBudget;
        budget.integratorBudget += integratorBudget;
        budget.agentRemaining += agentBudget;
        budget.lpRemaining += lpBudget;
        budget.integratorRemaining += integratorBudget;
        budget.funded = true;

        emit EpochFunded(epochId, agentBudget, lpBudget, integratorBudget);
    }

    function scheduleBudgetStream(
        uint64 epochId,
        AGCDataTypes.RewardCategory category,
        address beneficiary,
        uint256 amount,
        uint64 duration,
        bytes32 source
    ) external onlyController returns (uint256 streamId) {
        if (duration == 0) {
            duration = _defaultDuration(category);
        }

        _consumeBudget(epochId, category, amount);
        streamId = _createStream(beneficiary, category, amount, duration, source);
        emit StreamScheduled(streamId, epochId, beneficiary, category, amount, uint64(block.timestamp), uint64(block.timestamp) + duration, source);
    }

    function claimProductiveReceipt(uint256 receiptId) external returns (uint256 streamId) {
        AGCDataTypes.RewardReceipt memory receipt = hook.consumeRewardReceipt(receiptId);
        if (receipt.beneficiary != msg.sender) revert WrongBeneficiary();

        AGCDataTypes.RewardBudget storage budget = epochBudget[receipt.epochId];
        if (!budget.funded) revert EpochNotFunded();

        uint256 rewardAmount = quoteReceiptReward(receipt);
        if (rewardAmount > budget.agentRemaining) {
            rewardAmount = budget.agentRemaining;
        }
        if (rewardAmount == 0) revert NothingToClaim();

        budget.agentRemaining -= rewardAmount;
        streamId = _createStream(
            receipt.beneficiary,
            AGCDataTypes.RewardCategory.Agent,
            rewardAmount,
            agentStreamDuration,
            receipt.intentHash
        );

        emit StreamScheduled(
            streamId,
            receipt.epochId,
            receipt.beneficiary,
            AGCDataTypes.RewardCategory.Agent,
            rewardAmount,
            uint64(block.timestamp),
            uint64(block.timestamp) + agentStreamDuration,
            receipt.intentHash
        );
        emit ReceiptClaimed(receiptId, streamId, receipt.beneficiary, rewardAmount);
    }

    function claimStream(uint256 streamId) external returns (uint256 claimedAmount) {
        claimedAmount = _claimStream(streamId, msg.sender);
    }

    function claimStreams(uint256[] calldata streamIds) external returns (uint256 totalClaimed) {
        uint256 length = streamIds.length;
        for (uint256 i = 0; i < length; ++i) {
            totalClaimed += _claimStream(streamIds[i], msg.sender);
        }
    }

    function previewClaimable(uint256 streamId) external view returns (uint256) {
        AGCDataTypes.RewardStream memory stream = rewardStream[streamId];
        if (stream.beneficiary == address(0)) {
            return 0;
        }

        return _vestedAmount(stream) - stream.claimedAmount;
    }

    function quoteReceiptReward(AGCDataTypes.RewardReceipt memory receipt) public view returns (uint256) {
        uint256 grossReward = receipt.usdcAmount * baseRewardRateX18 / rewardScale;
        return grossReward * receipt.qualityScoreBps / AGCDataTypes.BPS;
    }

    function _claimStream(uint256 streamId, address claimant) internal returns (uint256 claimedAmount) {
        AGCDataTypes.RewardStream storage stream = rewardStream[streamId];
        if (stream.beneficiary == address(0)) revert StreamNotFound();
        if (stream.beneficiary != claimant) revert WrongBeneficiary();

        uint256 vested = _vestedAmount(stream);
        claimedAmount = vested - stream.claimedAmount;
        if (claimedAmount == 0) revert NothingToClaim();

        stream.claimedAmount += uint128(claimedAmount);
        agc.safeTransfer(claimant, claimedAmount);

        emit StreamClaimed(streamId, claimant, claimedAmount);
    }

    function _defaultDuration(AGCDataTypes.RewardCategory category) internal view returns (uint64) {
        if (category == AGCDataTypes.RewardCategory.Agent) return agentStreamDuration;
        if (category == AGCDataTypes.RewardCategory.LP) return lpStreamDuration;
        return integratorStreamDuration;
    }

    function _consumeBudget(uint64 epochId, AGCDataTypes.RewardCategory category, uint256 amount) internal {
        AGCDataTypes.RewardBudget storage budget = epochBudget[epochId];
        if (!budget.funded) revert EpochNotFunded();

        if (category == AGCDataTypes.RewardCategory.Agent) {
            if (amount > budget.agentRemaining) revert BudgetExceeded();
            budget.agentRemaining -= amount;
            return;
        }

        if (category == AGCDataTypes.RewardCategory.LP) {
            if (amount > budget.lpRemaining) revert BudgetExceeded();
            budget.lpRemaining -= amount;
            return;
        }

        if (amount > budget.integratorRemaining) revert BudgetExceeded();
        budget.integratorRemaining -= amount;
    }

    function _createStream(
        address beneficiary,
        AGCDataTypes.RewardCategory category,
        uint256 amount,
        uint64 duration,
        bytes32 source
    ) internal returns (uint256 streamId) {
        if (duration == 0) revert InvalidDuration();

        streamId = nextStreamId++;
        rewardStream[streamId] = AGCDataTypes.RewardStream({
            beneficiary: beneficiary,
            category: category,
            startTime: uint64(block.timestamp),
            endTime: uint64(block.timestamp) + duration,
            totalAmount: uint128(amount),
            claimedAmount: 0,
            source: source
        });
    }

    function _vestedAmount(AGCDataTypes.RewardStream memory stream) internal view returns (uint256) {
        if (block.timestamp <= stream.startTime) {
            return 0;
        }

        if (block.timestamp >= stream.endTime) {
            return stream.totalAmount;
        }

        uint256 elapsed = block.timestamp - stream.startTime;
        uint256 duration = stream.endTime - stream.startTime;
        return uint256(stream.totalAmount) * elapsed / duration;
    }
}

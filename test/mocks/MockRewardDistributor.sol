// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import { AGCDataTypes } from "../../src/libraries/AGCDataTypes.sol";

contract MockRewardDistributor {
    uint64 public lastEpochId;
    uint256 public lastAgentBudget;
    uint256 public lastLpBudget;
    uint256 public lastIntegratorBudget;
    uint64 public lastScheduledEpochId;
    AGCDataTypes.RewardCategory public lastScheduledCategory;
    address public lastScheduledBeneficiary;
    uint256 public lastScheduledAmount;
    uint64 public lastScheduledDuration;
    bytes32 public lastScheduledSource;
    uint256 public scheduleCalls;

    function fundEpoch(
        uint64 epochId,
        uint256 agentBudget,
        uint256 lpBudget,
        uint256 integratorBudget
    ) external {
        lastEpochId = epochId;
        lastAgentBudget = agentBudget;
        lastLpBudget = lpBudget;
        lastIntegratorBudget = integratorBudget;
    }

    function scheduleBudgetStream(
        uint64 epochId,
        AGCDataTypes.RewardCategory category,
        address beneficiary,
        uint256 amount,
        uint64 duration,
        bytes32 source
    ) external returns (uint256 streamId) {
        lastScheduledEpochId = epochId;
        lastScheduledCategory = category;
        lastScheduledBeneficiary = beneficiary;
        lastScheduledAmount = amount;
        lastScheduledDuration = duration;
        lastScheduledSource = source;
        streamId = ++scheduleCalls;
    }
}

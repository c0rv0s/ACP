// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {AGCDataTypes} from "../libraries/AGCDataTypes.sol";

interface IRewardDistributor {
    function fundEpoch(uint64 epochId, uint256 agentBudget, uint256 lpBudget, uint256 integratorBudget) external;
    function scheduleBudgetStream(
        uint64 epochId,
        AGCDataTypes.RewardCategory category,
        address beneficiary,
        uint256 amount,
        uint64 duration,
        bytes32 source
    ) external returns (uint256 streamId);
}

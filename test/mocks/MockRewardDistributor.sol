// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

contract MockRewardDistributor {
    uint64 public lastEpochId;
    uint256 public lastAgentBudget;
    uint256 public lastLpBudget;
    uint256 public lastIntegratorBudget;

    function fundEpoch(uint64 epochId, uint256 agentBudget, uint256 lpBudget, uint256 integratorBudget) external {
        lastEpochId = epochId;
        lastAgentBudget = agentBudget;
        lastLpBudget = lpBudget;
        lastIntegratorBudget = integratorBudget;
    }
}

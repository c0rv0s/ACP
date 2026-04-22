// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

interface IStabilityVault {
    function spendUSDC(address to, uint256 amount) external;
    function availableUsdc() external view returns (uint256);
    function availableAGC() external view returns (uint256);
}

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

interface IStabilityVault {
    function lockTreasuryMint(
        uint256 amount,
        uint64 duration
    ) external;
    function spendUSDC(
        address to,
        uint256 amount
    ) external;
    function releaseExpiredTreasuryLock() external returns (uint256 releasedAmount);
    function availableUsdc() external view returns (uint256);
    function burnProtocolAGC(
        uint256 amount
    ) external;
}

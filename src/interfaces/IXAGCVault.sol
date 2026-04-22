// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

interface IXAGCVault {
    function totalSupply() external view returns (uint256);
    function totalAssets() external view returns (uint256);
    function exitFeeBps() external view returns (uint256);
    function grossDepositsTotalAcp() external view returns (uint256);
    function grossRedemptionsTotalAcp() external view returns (uint256);
}

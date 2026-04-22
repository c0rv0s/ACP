// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

interface ISettlementRouter {
    function executeTreasuryBuyback(
        uint256 usdcAmountIn,
        uint256 minAgcOut,
        uint160 sqrtPriceLimitX96,
        bytes32 refId
    ) external returns (uint256 agcBurned);
}

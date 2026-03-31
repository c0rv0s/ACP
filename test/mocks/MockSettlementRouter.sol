// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

contract MockSettlementRouter {
    uint256 public lastBuybackBudget;
    uint256 public lastMinAgcOut;
    bytes32 public lastReference;
    uint256 public nextBurnAmount;

    function setNextBurnAmount(
        uint256 amount
    ) external {
        nextBurnAmount = amount;
    }

    function executeTreasuryBuyback(
        uint256 usdcAmountIn,
        uint256 minAgcOut,
        bytes32 refId
    ) external returns (uint256 agcBurned) {
        lastBuybackBudget = usdcAmountIn;
        lastMinAgcOut = minAgcOut;
        lastReference = refId;
        return nextBurnAmount;
    }
}

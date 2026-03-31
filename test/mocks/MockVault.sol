// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import { IERC20 } from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import { SafeERC20 } from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import { AGCToken } from "../../src/AGCToken.sol";

contract MockVault {
    using SafeERC20 for IERC20;

    IERC20 public immutable usdc;
    AGCToken public immutable agc;

    uint256 public lastLockedAmount;
    uint64 public lastLockDuration;

    constructor(
        IERC20 usdcToken,
        AGCToken agcToken
    ) {
        usdc = usdcToken;
        agc = agcToken;
    }

    function availableUsdc() external view returns (uint256) {
        return usdc.balanceOf(address(this));
    }

    function lockTreasuryMint(
        uint256 amount,
        uint64 duration
    ) external {
        lastLockedAmount = amount;
        lastLockDuration = duration;
    }

    function spendUSDC(
        address to,
        uint256 amount
    ) external {
        usdc.safeTransfer(to, amount);
    }

    function releaseExpiredTreasuryLock() external pure returns (uint256) {
        return 0;
    }

    function burnProtocolAGC(
        uint256 amount
    ) external {
        agc.burn(address(this), amount);
    }
}

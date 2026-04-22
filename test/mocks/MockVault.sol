// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import { IERC20 } from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import { SafeERC20 } from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import { AGCToken } from "../../src/AGCToken.sol";

contract MockVault {
    using SafeERC20 for IERC20;

    IERC20 public immutable usdc;
    AGCToken public immutable agc;

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

    function availableAGC() external view returns (uint256) {
        return agc.balanceOf(address(this));
    }

    function spendUSDC(
        address to,
        uint256 amount
    ) external {
        usdc.safeTransfer(to, amount);
    }

    function spendAGC(
        address to,
        uint256 amount
    ) external {
        IERC20(address(agc)).safeTransfer(to, amount);
    }

    function burnProtocolAGC(
        uint256 amount
    ) external {
        agc.burn(address(this), amount);
    }
}

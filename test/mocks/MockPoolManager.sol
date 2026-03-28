// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {PoolKey} from "v4-core/types/PoolKey.sol";
import {Currency} from "v4-core/types/Currency.sol";
import {BalanceDelta} from "v4-core/types/BalanceDelta.sol";
import {IPoolManager} from "v4-core/interfaces/IPoolManager.sol";
import {IUnlockCallback} from "v4-core/interfaces/callback/IUnlockCallback.sol";

contract MockPoolManager {
    using SafeERC20 for IERC20;

    BalanceDelta public nextSwapDelta;
    bytes public lastHookData;
    address public lastSwapper;

    function setNextSwapDelta(BalanceDelta delta) external {
        nextSwapDelta = delta;
    }

    function unlock(bytes calldata data) external returns (bytes memory) {
        return IUnlockCallback(msg.sender).unlockCallback(data);
    }

    function swap(PoolKey memory, IPoolManager.SwapParams memory, bytes calldata hookData)
        external
        returns (BalanceDelta)
    {
        lastSwapper = msg.sender;
        lastHookData = hookData;
        return nextSwapDelta;
    }

    function take(Currency currency, address to, uint256 amount) external {
        IERC20(address(Currency.unwrap(currency))).safeTransfer(to, amount);
    }

    function sync(Currency) external {}

    function settle() external payable returns (uint256 paid) {
        return msg.value;
    }

    function settleFor(address) external payable returns (uint256 paid) {
        return msg.value;
    }

    receive() external payable {}
}

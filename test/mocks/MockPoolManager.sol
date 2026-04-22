// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import { IERC20 } from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import { SafeERC20 } from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import { PoolKey } from "v4-core/types/PoolKey.sol";
import { Currency } from "v4-core/types/Currency.sol";
import { BalanceDelta } from "v4-core/types/BalanceDelta.sol";
import { PoolId, PoolIdLibrary } from "v4-core/types/PoolId.sol";
import { IPoolManager } from "v4-core/interfaces/IPoolManager.sol";
import { IUnlockCallback } from "v4-core/interfaces/callback/IUnlockCallback.sol";
import { Position } from "v4-core/libraries/Position.sol";

contract MockPoolManager {
    using SafeERC20 for IERC20;
    using PoolIdLibrary for PoolKey;

    BalanceDelta public nextSwapDelta;
    bytes public lastHookData;
    address public lastSwapper;
    mapping(bytes32 => bytes32) internal _storageSlots;

    bytes32 internal constant POOLS_SLOT = bytes32(uint256(6));
    uint256 internal constant POSITIONS_OFFSET = 6;

    function setNextSwapDelta(
        BalanceDelta delta
    ) external {
        nextSwapDelta = delta;
    }

    function setSlot0(
        PoolKey memory key,
        uint160 sqrtPriceX96,
        int24 tick,
        uint24 protocolFee,
        uint24 lpFee
    ) external {
        bytes32 stateSlot = keccak256(abi.encodePacked(PoolId.unwrap(key.toId()), POOLS_SLOT));
        uint256 packed = uint256(sqrtPriceX96);
        packed |= _tickBits(tick) << 160;
        packed |= uint256(protocolFee) << 184;
        packed |= uint256(lpFee) << 208;
        _storageSlots[stateSlot] = bytes32(packed);
    }

    function setPositionLiquidity(
        PoolKey memory key,
        address owner,
        int24 tickLower,
        int24 tickUpper,
        bytes32 salt,
        uint128 liquidity
    ) external {
        bytes32 stateSlot = keccak256(abi.encodePacked(PoolId.unwrap(key.toId()), POOLS_SLOT));
        bytes32 positionMappingSlot = bytes32(uint256(stateSlot) + POSITIONS_OFFSET);
        bytes32 positionKey = Position.calculatePositionKey(owner, tickLower, tickUpper, salt);
        bytes32 positionSlot = keccak256(abi.encodePacked(positionKey, positionMappingSlot));
        _storageSlots[positionSlot] = bytes32(uint256(liquidity));
    }

    function unlock(
        bytes calldata data
    ) external returns (bytes memory) {
        return IUnlockCallback(msg.sender).unlockCallback(data);
    }

    function swap(
        PoolKey memory,
        IPoolManager.SwapParams memory,
        bytes calldata hookData
    ) external returns (BalanceDelta) {
        lastSwapper = msg.sender;
        lastHookData = hookData;
        return nextSwapDelta;
    }

    function take(
        Currency currency,
        address to,
        uint256 amount
    ) external {
        IERC20(address(Currency.unwrap(currency))).safeTransfer(to, amount);
    }

    function sync(
        Currency
    ) external { }

    function settle() external payable returns (uint256 paid) {
        return msg.value;
    }

    function settleFor(
        address
    ) external payable returns (uint256 paid) {
        return msg.value;
    }

    function extsload(
        bytes32 slot
    ) external view returns (bytes32 value) {
        return _storageSlots[slot];
    }

    function extsload(
        bytes32 startSlot,
        uint256 nSlots
    ) external view returns (bytes32[] memory values) {
        values = new bytes32[](nSlots);
        bytes32 slot = startSlot;
        for (uint256 i = 0; i < nSlots; ++i) {
            values[i] = _storageSlots[slot];
            slot = bytes32(uint256(slot) + 1);
        }
    }

    function extsload(
        bytes32[] calldata slots
    ) external view returns (bytes32[] memory values) {
        uint256 length = slots.length;
        values = new bytes32[](length);
        for (uint256 i = 0; i < length; ++i) {
            values[i] = _storageSlots[slots[i]];
        }
    }

    function _tickBits(
        int24 tick
    ) internal pure returns (uint256 bits) {
        assembly ("memory-safe") {
            bits := and(tick, 0xFFFFFF)
        }
    }

    receive() external payable { }
}

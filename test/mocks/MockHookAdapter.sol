// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import { PoolKey } from "v4-core/types/PoolKey.sol";
import { AGCDataTypes } from "../../src/libraries/AGCDataTypes.sol";

contract MockHookAdapter {
    PoolKey internal _poolKey;
    AGCDataTypes.HookFeeConfig internal _feeConfig;
    AGCDataTypes.EpochAccumulator internal _accumulator;

    AGCDataTypes.EpochSnapshot public nextSnapshot;
    AGCDataTypes.Regime public currentRegime;
    uint64 public currentEpochId = 1;

    function setCanonicalPoolKey(
        PoolKey memory key
    ) external {
        _poolKey = key;
    }

    function canonicalPoolKey() external view returns (PoolKey memory) {
        return _poolKey;
    }

    function feeConfig() external view returns (AGCDataTypes.HookFeeConfig memory) {
        return _feeConfig;
    }

    function currentAccumulator() external view returns (AGCDataTypes.EpochAccumulator memory) {
        return _accumulator;
    }

    function setNextSnapshot(
        AGCDataTypes.EpochSnapshot memory snapshot
    ) external {
        nextSnapshot = snapshot;
        currentEpochId = snapshot.epochId + 1;
    }

    function previewEpochSnapshot() external view returns (AGCDataTypes.EpochSnapshot memory snapshot) {
        return nextSnapshot;
    }

    function consumeEpochSnapshot() external returns (AGCDataTypes.EpochSnapshot memory snapshot) {
        snapshot = nextSnapshot;
        currentEpochId = snapshot.epochId + 1;
    }

    function setController(
        address
    ) external { }

    function setRegime(
        AGCDataTypes.Regime newRegime
    ) external {
        currentRegime = newRegime;
    }

    function setFeeConfig(
        AGCDataTypes.HookFeeConfig calldata config
    ) external {
        _feeConfig = config;
    }
}

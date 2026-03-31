// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import { PoolKey } from "v4-core/types/PoolKey.sol";
import { AGCDataTypes } from "../../src/libraries/AGCDataTypes.sol";

contract MockHookAdapter {
    error RewardReceiptNotFound();
    error RewardReceiptConsumed();

    PoolKey internal _poolKey;
    AGCDataTypes.HookFeeConfig internal _feeConfig;
    AGCDataTypes.EpochAccumulator internal _accumulator;

    mapping(address => bool) public trustedRouters;
    mapping(uint256 => AGCDataTypes.RewardReceipt) public receipts;

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

    function previewMetadata(
        address,
        bytes calldata
    ) external pure returns (AGCDataTypes.HookMetadata memory metadata) {
        metadata.qualityScoreBps = uint16(AGCDataTypes.BPS);
    }

    function setNextSnapshot(
        AGCDataTypes.EpochSnapshot memory snapshot
    ) external {
        nextSnapshot = snapshot;
        currentEpochId = snapshot.epochId + 1;
    }

    function previewEpochSnapshot()
        external
        view
        returns (AGCDataTypes.EpochSnapshot memory snapshot)
    {
        return nextSnapshot;
    }

    function consumeEpochSnapshot() external returns (AGCDataTypes.EpochSnapshot memory snapshot) {
        snapshot = nextSnapshot;
        currentEpochId = snapshot.epochId + 1;
    }

    function setRewardReceipt(
        uint256 receiptId,
        AGCDataTypes.RewardReceipt memory receipt
    ) external {
        receipts[receiptId] = receipt;
    }

    function rewardReceipt(
        uint256 receiptId
    ) external view returns (AGCDataTypes.RewardReceipt memory) {
        return receipts[receiptId];
    }

    function consumeRewardReceipt(
        uint256 receiptId
    ) external returns (AGCDataTypes.RewardReceipt memory receipt) {
        AGCDataTypes.RewardReceipt storage storedReceipt = receipts[receiptId];
        if (storedReceipt.beneficiary == address(0)) revert RewardReceiptNotFound();
        if (storedReceipt.consumed) revert RewardReceiptConsumed();

        receipt = storedReceipt;
        storedReceipt.consumed = true;
    }

    function setController(
        address
    ) external { }
    function setRewardDistributor(
        address
    ) external { }

    function setTrustedRouter(
        address router,
        bool trusted
    ) external {
        trustedRouters[router] = trusted;
    }

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

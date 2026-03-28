// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {PoolKey} from "v4-core/types/PoolKey.sol";
import {AGCDataTypes} from "../libraries/AGCDataTypes.sol";

interface IAGCHook {
    function canonicalPoolKey() external view returns (PoolKey memory);
    function currentEpochId() external view returns (uint64);
    function currentRegime() external view returns (AGCDataTypes.Regime);
    function currentAccumulator() external view returns (AGCDataTypes.EpochAccumulator memory);
    function trustedRouters(address router) external view returns (bool);
    function feeConfig() external view returns (AGCDataTypes.HookFeeConfig memory);
    function previewMetadata(address router, bytes calldata hookData)
        external
        view
        returns (AGCDataTypes.HookMetadata memory);
    function consumeEpochSnapshot() external returns (AGCDataTypes.EpochSnapshot memory);
    function consumeRewardReceipt(uint256 receiptId) external returns (AGCDataTypes.RewardReceipt memory);
    function rewardReceipt(uint256 receiptId) external view returns (AGCDataTypes.RewardReceipt memory);
    function setController(address controller) external;
    function setRewardDistributor(address distributor) external;
    function setTrustedRouter(address router, bool trusted) external;
    function setRegime(AGCDataTypes.Regime newRegime) external;
    function setFeeConfig(AGCDataTypes.HookFeeConfig calldata newConfig) external;
}

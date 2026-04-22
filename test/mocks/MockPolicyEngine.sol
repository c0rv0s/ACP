// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import { AGCDataTypes } from "../../src/libraries/AGCDataTypes.sol";

contract MockPolicyEngine {
    AGCDataTypes.EpochResult internal _nextResult;

    function setNextResult(
        AGCDataTypes.EpochResult memory result
    ) external {
        _nextResult = result;
    }

    function evaluateEpoch(
        AGCDataTypes.EpochSnapshot memory,
        AGCDataTypes.ExternalMetrics memory,
        AGCDataTypes.PolicyState memory,
        AGCDataTypes.VaultFlows memory,
        AGCDataTypes.PolicyParams memory
    ) external view returns (AGCDataTypes.EpochResult memory result) {
        return _nextResult;
    }
}

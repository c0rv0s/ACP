// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

contract HookDeployer {
    error DeploymentFailed();

    function deploy(bytes memory creationCode, bytes32 salt) external returns (address deployed) {
        assembly ("memory-safe") {
            deployed := create2(0, add(creationCode, 0x20), mload(creationCode), salt)
        }

        if (deployed == address(0)) revert DeploymentFailed();
    }
}

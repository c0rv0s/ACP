// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import { Ownable } from "@openzeppelin/contracts/access/Ownable.sol";
import { ERC20 } from "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract MockUSDC is ERC20, Ownable {
    constructor(
        address admin
    ) ERC20("Mock USDC", "USDC") Ownable(admin) { }

    function decimals() public pure override returns (uint8) {
        return 6;
    }

    function mint(
        address to,
        uint256 amount
    ) external onlyOwner {
        _mint(to, amount);
    }
}

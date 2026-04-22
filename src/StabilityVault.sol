// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import { Ownable } from "@openzeppelin/contracts/access/Ownable.sol";
import { Ownable2Step } from "@openzeppelin/contracts/access/Ownable2Step.sol";
import { SafeERC20 } from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import { IERC20 } from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import { AGCToken } from "./AGCToken.sol";
import { IStabilityVault } from "./interfaces/IStabilityVault.sol";

contract StabilityVault is Ownable2Step, IStabilityVault {
    using SafeERC20 for IERC20;
    using SafeERC20 for AGCToken;

    error Unauthorized();
    error InvalidRecipient();

    event PolicyControllerUpdated(address indexed controller);
    event SettlementRouterUpdated(address indexed router);
    event USDCSent(address indexed to, uint256 amount);
    event AGCSent(address indexed to, uint256 amount);
    event AGCBurned(uint256 amount);

    AGCToken public immutable agc;
    IERC20 public immutable usdc;

    address public policyController;
    address public settlementRouter;

    constructor(
        address admin,
        AGCToken agcToken,
        IERC20 usdcToken
    ) Ownable(admin) {
        agc = agcToken;
        usdc = usdcToken;
    }

    modifier onlyAuthorizedSpender() {
        if (msg.sender != policyController && msg.sender != settlementRouter) {
            revert Unauthorized();
        }
        _;
    }

    function setPolicyController(
        address controller
    ) external onlyOwner {
        policyController = controller;
        emit PolicyControllerUpdated(controller);
    }

    function setSettlementRouter(
        address router
    ) external onlyOwner {
        settlementRouter = router;
        emit SettlementRouterUpdated(router);
    }

    function spendUSDC(
        address to,
        uint256 amount
    ) external onlyAuthorizedSpender {
        if (to == address(0)) revert InvalidRecipient();
        usdc.safeTransfer(to, amount);
        emit USDCSent(to, amount);
    }

    function spendAGC(
        address to,
        uint256 amount
    ) external onlyAuthorizedSpender {
        if (to == address(0)) revert InvalidRecipient();
        agc.safeTransfer(to, amount);
        emit AGCSent(to, amount);
    }

    function burnProtocolAGC(
        uint256 amount
    ) external onlyAuthorizedSpender {
        agc.burn(address(this), amount);
        emit AGCBurned(amount);
    }

    function availableUsdc() external view returns (uint256) {
        return usdc.balanceOf(address(this));
    }

    function availableAGC() external view returns (uint256) {
        return agc.balanceOf(address(this));
    }
}

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

    error Unauthorized();
    error InvalidRecipient();
    error InsufficientUnlockedAGC();

    event PolicyControllerUpdated(address indexed controller);
    event SettlementRouterUpdated(address indexed router);
    event TreasuryLockCreated(uint256 amount, uint64 unlockAt);
    event TreasuryLockReleased(uint256 amount);
    event USDCSent(address indexed to, uint256 amount);
    event AGCBurned(uint256 amount);

    AGCToken public immutable agc;
    IERC20 public immutable usdc;

    address public policyController;
    address public settlementRouter;

    uint256 public lockedTreasuryAgc;
    uint64 public treasuryUnlockAt;

    constructor(
        address admin,
        AGCToken agcToken,
        IERC20 usdcToken
    ) Ownable(admin) {
        agc = agcToken;
        usdc = usdcToken;
    }

    modifier onlyPolicyController() {
        if (msg.sender != policyController) revert Unauthorized();
        _;
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

    function lockTreasuryMint(
        uint256 amount,
        uint64 duration
    ) external onlyPolicyController {
        releaseExpiredTreasuryLock();
        lockedTreasuryAgc += amount;

        uint64 unlockAt = uint64(block.timestamp) + duration;
        if (unlockAt > treasuryUnlockAt) {
            treasuryUnlockAt = unlockAt;
        }

        emit TreasuryLockCreated(amount, unlockAt);
    }

    function releaseExpiredTreasuryLock() public returns (uint256 releasedAmount) {
        if (treasuryUnlockAt != 0 && block.timestamp >= treasuryUnlockAt) {
            releasedAmount = lockedTreasuryAgc;
            lockedTreasuryAgc = 0;
            treasuryUnlockAt = 0;
            emit TreasuryLockReleased(releasedAmount);
        }
    }

    function spendUSDC(
        address to,
        uint256 amount
    ) external onlyAuthorizedSpender {
        if (to == address(0)) revert InvalidRecipient();
        usdc.safeTransfer(to, amount);
        emit USDCSent(to, amount);
    }

    function burnProtocolAGC(
        uint256 amount
    ) external onlyPolicyController {
        releaseExpiredTreasuryLock();
        if (amount > availableAGC()) revert InsufficientUnlockedAGC();
        agc.burn(address(this), amount);
        emit AGCBurned(amount);
    }

    function availableUsdc() external view returns (uint256) {
        return usdc.balanceOf(address(this));
    }

    function availableAGC() public view returns (uint256) {
        uint256 balance = agc.balanceOf(address(this));
        return balance > lockedTreasuryAgc ? balance - lockedTreasuryAgc : 0;
    }
}

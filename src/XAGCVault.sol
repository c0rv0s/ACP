// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import { Ownable } from "@openzeppelin/contracts/access/Ownable.sol";
import { Ownable2Step } from "@openzeppelin/contracts/access/Ownable2Step.sol";
import { ERC20 } from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import { SafeERC20 } from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import { Math } from "@openzeppelin/contracts/utils/math/Math.sol";
import { AGCToken } from "./AGCToken.sol";

contract XAGCVault is ERC20, Ownable2Step {
    using SafeERC20 for AGCToken;

    error InvalidRecipient();
    error InvalidOwner();
    error ZeroAmount();
    error InsufficientShares();

    event TreasuryUpdated(address indexed treasury);
    event ExitFeeUpdated(uint16 feeBps);
    event Deposited(address indexed caller, address indexed receiver, uint256 assets, uint256 shares);
    event Redeemed(
        address indexed caller,
        address indexed receiver,
        address indexed owner,
        uint256 shares,
        uint256 grossAssets,
        uint256 feeAssets,
        uint256 netAssets
    );

    uint256 internal constant BPS = 10_000;

    AGCToken public immutable agc;
    address public treasury;
    uint16 public exitFeeBps;

    uint256 public grossDepositsTotalAcp;
    uint256 public grossRedemptionsTotalAcp;
    uint256 public unaccountedAssetsAcp;

    constructor(
        address admin,
        AGCToken agcToken,
        address treasuryVault,
        uint16 initialExitFeeBps
    ) ERC20("Staked Agent Credit", "xAGC") Ownable(admin) {
        if (treasuryVault == address(0)) revert InvalidRecipient();
        if (initialExitFeeBps >= BPS) revert ZeroAmount();

        agc = agcToken;
        treasury = treasuryVault;
        exitFeeBps = initialExitFeeBps;
    }

    function totalAssets() public view returns (uint256) {
        return agc.balanceOf(address(this));
    }

    function convertToShares(
        uint256 assets
    ) public view returns (uint256 shares) {
        uint256 supply = totalSupply();
        if (supply == 0) {
            return assets;
        }

        uint256 assetsBefore = _accountedAssets();
        if (assetsBefore == 0) {
            return 0;
        }

        return Math.mulDiv(assets, supply, assetsBefore);
    }

    function convertToAssets(
        uint256 shares
    ) public view returns (uint256 assets) {
        uint256 supply = totalSupply();
        if (supply == 0) {
            return shares;
        }

        uint256 assetsBefore = _accountedAssets();
        if (assetsBefore == 0) {
            return 0;
        }

        return Math.mulDiv(shares, assetsBefore, supply);
    }

    function previewDeposit(
        uint256 assets
    ) external view returns (uint256 shares) {
        return convertToShares(assets);
    }

    function previewRedeem(
        uint256 shares
    ) external view returns (uint256 netAssets, uint256 feeAssets) {
        uint256 grossAssets = convertToAssets(shares);
        feeAssets = grossAssets * exitFeeBps / BPS;
        netAssets = grossAssets - feeAssets;
    }

    function setTreasury(
        address treasuryVault
    ) external onlyOwner {
        if (treasuryVault == address(0)) revert InvalidRecipient();
        treasury = treasuryVault;
        emit TreasuryUpdated(treasuryVault);
    }

    function setExitFeeBps(
        uint16 newExitFeeBps
    ) external onlyOwner {
        if (newExitFeeBps >= BPS) revert ZeroAmount();
        exitFeeBps = newExitFeeBps;
        emit ExitFeeUpdated(newExitFeeBps);
    }

    function deposit(
        uint256 assets,
        address receiver
    ) external returns (uint256 shares) {
        if (assets == 0) revert ZeroAmount();
        if (receiver == address(0)) revert InvalidRecipient();

        _syncUnaccountedAssets();
        shares = convertToShares(assets);
        if (shares == 0) revert ZeroAmount();

        grossDepositsTotalAcp += assets;
        agc.safeTransferFrom(msg.sender, address(this), assets);
        _mint(receiver, shares);

        emit Deposited(msg.sender, receiver, assets, shares);
    }

    function redeem(
        uint256 shares,
        address receiver,
        address owner_
    ) external returns (uint256 netAssets) {
        if (shares == 0) revert ZeroAmount();
        if (receiver == address(0)) revert InvalidRecipient();
        if (owner_ == address(0)) revert InvalidOwner();
        if (balanceOf(owner_) < shares) revert InsufficientShares();

        if (owner_ != msg.sender) {
            _spendAllowance(owner_, msg.sender, shares);
        }

        uint256 grossAssets = convertToAssets(shares);
        uint256 feeAssets = grossAssets * exitFeeBps / BPS;
        netAssets = grossAssets - feeAssets;

        grossRedemptionsTotalAcp += grossAssets;
        _burn(owner_, shares);

        if (feeAssets > 0) {
            agc.safeTransfer(treasury, feeAssets);
        }
        agc.safeTransfer(receiver, netAssets);

        emit Redeemed(msg.sender, receiver, owner_, shares, grossAssets, feeAssets, netAssets);
    }

    function _accountedAssets() internal view returns (uint256 assets) {
        uint256 total = totalAssets();
        uint256 unaccounted = unaccountedAssetsAcp;
        return total > unaccounted ? total - unaccounted : 0;
    }

    function _syncUnaccountedAssets() internal {
        if (totalSupply() == 0) {
            unaccountedAssetsAcp = totalAssets();
        }
    }
}

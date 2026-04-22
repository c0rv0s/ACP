// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import "forge-std/Test.sol";
import { AGCToken } from "../../src/AGCToken.sol";
import { XAGCVault } from "../../src/XAGCVault.sol";
import { StabilityVault } from "../../src/StabilityVault.sol";
import { MockUSDC } from "../../src/mocks/MockUSDC.sol";

contract XAGCVaultTest is Test {
    bytes32 internal constant MINTER_ROLE = keccak256("MINTER_ROLE");

    AGCToken internal agc;
    MockUSDC internal usdc;
    StabilityVault internal treasuryVault;
    XAGCVault internal vault;

    address internal alice = address(0xA11CE);

    function setUp() public {
        agc = new AGCToken(address(this));
        usdc = new MockUSDC(address(this));
        treasuryVault = new StabilityVault(address(this), agc, usdc);
        vault = new XAGCVault(address(this), agc, address(treasuryVault), 300);

        agc.grantRole(MINTER_ROLE, address(this));
        agc.mint(alice, 200e18);

        vm.prank(alice);
        agc.approve(address(vault), type(uint256).max);
    }

    function testRedeemChargesExitFeeToTreasury() public {
        vm.prank(alice);
        uint256 shares = vault.deposit(100e18, alice);
        assertEq(shares, 100e18);

        agc.mint(address(vault), 20e18);

        vm.prank(alice);
        uint256 netAssets = vault.redeem(shares, alice, alice);

        assertEq(netAssets, 116.4e18);
        assertEq(agc.balanceOf(alice), 216.4e18);
        assertEq(agc.balanceOf(address(treasuryVault)), 3.6e18);
        assertEq(vault.grossDepositsTotalAcp(), 100e18);
        assertEq(vault.grossRedemptionsTotalAcp(), 120e18);
        assertEq(vault.totalSupply(), 0);
    }

    function testPrefundedAssetsCannotBeClaimedByFirstDepositor() public {
        agc.mint(address(vault), 100e18);

        vm.prank(alice);
        uint256 shares = vault.deposit(1e18, alice);
        assertEq(shares, 1e18);
        assertEq(vault.unaccountedAssetsAcp(), 100e18);

        vm.prank(alice);
        uint256 netAssets = vault.redeem(shares, alice, alice);

        assertEq(netAssets, 97e16);
        assertEq(agc.balanceOf(alice), 199.97e18);
        assertEq(agc.balanceOf(address(vault)), 100e18);
        assertEq(agc.balanceOf(address(treasuryVault)), 3e16);
        assertEq(vault.grossDepositsTotalAcp(), 1e18);
        assertEq(vault.grossRedemptionsTotalAcp(), 1e18);
    }
}

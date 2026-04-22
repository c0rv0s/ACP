// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import "forge-std/Test.sol";
import { AGCToken } from "../../src/AGCToken.sol";
import { StabilityVault } from "../../src/StabilityVault.sol";
import { MockUSDC } from "../../src/mocks/MockUSDC.sol";

contract StabilityVaultTest is Test {
    bytes32 internal constant MINTER_ROLE = keccak256("MINTER_ROLE");
    bytes32 internal constant BURNER_ROLE = keccak256("BURNER_ROLE");

    AGCToken internal agc;
    MockUSDC internal usdc;
    StabilityVault internal vault;

    address internal controller = address(0x1111);
    address internal router = address(0x2222);
    address internal recipient = address(0x3333);
    address internal attacker = address(0x4444);

    function setUp() public {
        agc = new AGCToken(address(this));
        usdc = new MockUSDC(address(this));
        vault = new StabilityVault(address(this), agc, usdc);

        agc.grantRole(MINTER_ROLE, address(this));
        agc.grantRole(BURNER_ROLE, address(vault));

        agc.mint(address(vault), 100e18);
        usdc.mint(address(vault), 500e6);
    }

    function testOnlyAuthorizedSpendersCanSpendOrBurn() public {
        vm.prank(attacker);
        vm.expectRevert(StabilityVault.Unauthorized.selector);
        vault.spendUSDC(recipient, 1e6);

        vm.prank(attacker);
        vm.expectRevert(StabilityVault.Unauthorized.selector);
        vault.spendAGC(recipient, 1e18);

        vm.prank(attacker);
        vm.expectRevert(StabilityVault.Unauthorized.selector);
        vault.burnProtocolAGC(1e18);
    }

    function testPolicyControllerCanSpendAndBurn() public {
        vault.setPolicyController(controller);

        vm.startPrank(controller);
        vault.spendUSDC(recipient, 40e6);
        vault.spendAGC(recipient, 10e18);
        vault.burnProtocolAGC(15e18);
        vm.stopPrank();

        assertEq(usdc.balanceOf(recipient), 40e6);
        assertEq(agc.balanceOf(recipient), 10e18);
        assertEq(vault.availableUsdc(), 460e6);
        assertEq(vault.availableAGC(), 75e18);
        assertEq(agc.totalSupply(), 85e18);
    }

    function testSettlementRouterCanSpendAndZeroRecipientReverts() public {
        vault.setSettlementRouter(router);

        vm.prank(router);
        vm.expectRevert(StabilityVault.InvalidRecipient.selector);
        vault.spendUSDC(address(0), 1e6);

        vm.prank(router);
        vm.expectRevert(StabilityVault.InvalidRecipient.selector);
        vault.spendAGC(address(0), 1e18);

        vm.startPrank(router);
        vault.spendUSDC(recipient, 25e6);
        vault.spendAGC(recipient, 5e18);
        vm.stopPrank();

        assertEq(usdc.balanceOf(recipient), 25e6);
        assertEq(agc.balanceOf(recipient), 5e18);
        assertEq(vault.availableUsdc(), 475e6);
        assertEq(vault.availableAGC(), 95e18);
    }
}

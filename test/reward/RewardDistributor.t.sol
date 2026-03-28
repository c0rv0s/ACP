// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import "forge-std/Test.sol";
import {AGCToken} from "../../src/AGCToken.sol";
import {RewardDistributor} from "../../src/RewardDistributor.sol";
import {IAGCHook} from "../../src/interfaces/IAGCHook.sol";
import {AGCDataTypes} from "../../src/libraries/AGCDataTypes.sol";
import {MockHookAdapter} from "../mocks/MockHookAdapter.sol";

contract RewardDistributorTest is Test {
    bytes32 internal constant MINTER_ROLE = keccak256("MINTER_ROLE");

    AGCToken internal agc;
    MockHookAdapter internal hook;
    RewardDistributor internal distributor;

    address internal alice = address(0xA11CE);
    address internal bob = address(0xB0B);

    function setUp() public {
        agc = new AGCToken(address(this));
        hook = new MockHookAdapter();
        distributor = new RewardDistributor(address(this), agc, IAGCHook(address(hook)));

        agc.grantRole(MINTER_ROLE, address(this));
        distributor.setController(address(this));
        agc.mint(address(distributor), 1_000e18);
        distributor.fundEpoch(1, 500e18, 250e18, 250e18);
    }

    function testClaimReceiptCreatesStreamAndPaysOutOverTime() public {
        uint256 start = block.timestamp;

        hook.setRewardReceipt(
            1,
            AGCDataTypes.RewardReceipt({
                beneficiary: alice,
                originalSender: alice,
                intentHash: keccak256("receipt-1"),
                flowClass: AGCDataTypes.FlowClass.ProductivePayment,
                epochId: 1,
                createdAt: uint64(block.timestamp),
                qualityScoreBps: uint16(AGCDataTypes.BPS),
                agcAmount: 100e18,
                usdcAmount: 10e6,
                consumed: false
            })
        );

        vm.prank(alice);
        uint256 streamId = distributor.claimProductiveReceipt(1);

        vm.warp(start + 24 hours);
        vm.prank(alice);
        distributor.claimStream(streamId);
        assertGt(agc.balanceOf(alice), 0);

        vm.warp(start + 48 hours);
        vm.prank(alice);
        distributor.claimStream(streamId);
        assertGt(agc.balanceOf(alice), 1e18);
    }

    function testControllerCanScheduleLpStream() public {
        uint256 streamId = distributor.scheduleBudgetStream(
            1,
            AGCDataTypes.RewardCategory.LP,
            bob,
            100e18,
            7 days,
            keccak256("lp-1")
        );

        vm.warp(block.timestamp + 7 days);
        vm.prank(bob);
        distributor.claimStream(streamId);
        assertEq(agc.balanceOf(bob), 100e18);
    }
}

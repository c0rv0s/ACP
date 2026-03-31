// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import "forge-std/Test.sol";
import { IHooks } from "v4-core/interfaces/IHooks.sol";
import { IPoolManager } from "v4-core/interfaces/IPoolManager.sol";
import { PoolKey } from "v4-core/types/PoolKey.sol";
import { Currency } from "v4-core/types/Currency.sol";
import { BalanceDelta, toBalanceDelta } from "v4-core/types/BalanceDelta.sol";
import { AGCToken } from "../../src/AGCToken.sol";
import { SettlementRouter } from "../../src/SettlementRouter.sol";
import { AGCDataTypes } from "../../src/libraries/AGCDataTypes.sol";
import { IAGCHook } from "../../src/interfaces/IAGCHook.sol";
import { IStabilityVault } from "../../src/interfaces/IStabilityVault.sol";
import { MockUSDC } from "../../src/mocks/MockUSDC.sol";
import { MockHookAdapter } from "../mocks/MockHookAdapter.sol";
import { MockPoolManager } from "../mocks/MockPoolManager.sol";
import { MockVault } from "../mocks/MockVault.sol";

contract SettlementRouterTest is Test {
    bytes32 internal constant MINTER_ROLE = keccak256("MINTER_ROLE");
    bytes32 internal constant BURNER_ROLE = keccak256("BURNER_ROLE");
    bytes32 internal constant FACILITATED_ROUTE_HASH = keccak256("facilitated-x402");

    AGCToken internal agc;
    MockUSDC internal usdc;
    MockPoolManager internal manager;
    MockHookAdapter internal hook;
    MockVault internal vault;
    SettlementRouter internal router;

    address internal user = address(0xABCD);
    address internal merchant = address(0xDCBA);
    uint256 internal facilitatorPk = 0xFACA710;
    address internal facilitator;
    bool internal agcIsCurrency0;

    function setUp() public {
        agc = new AGCToken(address(this));
        usdc = new MockUSDC(address(this));
        manager = new MockPoolManager();
        hook = new MockHookAdapter();
        vault = new MockVault(usdc, agc);
        facilitator = vm.addr(facilitatorPk);

        router = new SettlementRouter(
            address(this),
            agc,
            usdc,
            IPoolManager(address(manager)),
            IAGCHook(address(hook)),
            IStabilityVault(address(vault))
        );

        agc.grantRole(MINTER_ROLE, address(this));
        agc.grantRole(BURNER_ROLE, address(router));
        agc.mint(user, 1_000e18);
        agc.mint(address(manager), 1_000e18);
        usdc.mint(address(manager), 1_000_000e6);
        usdc.mint(address(vault), 500_000e6);

        agcIsCurrency0 = address(agc) < address(usdc);
        hook.setCanonicalPoolKey(
            PoolKey({
                currency0: Currency.wrap(agcIsCurrency0 ? address(agc) : address(usdc)),
                currency1: Currency.wrap(agcIsCurrency0 ? address(usdc) : address(agc)),
                fee: 0x800000,
                tickSpacing: 60,
                hooks: IHooks(address(0x1234))
            })
        );
        router.setTrustedFacilitator(facilitator, true);

        vm.prank(user);
        agc.approve(address(router), type(uint256).max);
    }

    function testSettlePaymentSwapsAndPaysMerchant() public {
        BalanceDelta delta = agcIsCurrency0
            ? toBalanceDelta(-int128(100e18), int128(100e6))
            : toBalanceDelta(int128(100e6), -int128(100e18));
        manager.setNextSwapDelta(delta);

        vm.prank(user);
        uint256 usdcOut = router.settlePayment(100e18, 99e6, merchant, keccak256("payment"));

        assertEq(usdcOut, 100e6);
        assertEq(usdc.balanceOf(merchant), 100e6);
        assertEq(agc.balanceOf(address(manager)), 1_100e18);
        assertEq(manager.lastHookData().length, 0);
    }

    function testSettledProductivePaymentRequiresFacilitatorAttestation() public {
        BalanceDelta delta = agcIsCurrency0
            ? toBalanceDelta(-int128(100e18), int128(100e6))
            : toBalanceDelta(int128(100e6), -int128(100e18));
        manager.setNextSwapDelta(delta);

        SettlementRouter.ProductivePaymentAttestation memory attestation =
            _buildAttestation(100e18, keccak256("productive-payment"), uint16(9_000));
        bytes memory signature = _signAttestation(attestation);

        vm.prank(user);
        uint256 usdcOut = router.settleProductivePayment(attestation, 99e6, facilitator, signature);

        assertEq(usdcOut, 100e6);
        assertEq(usdc.balanceOf(merchant), 100e6);

        AGCDataTypes.HookMetadata memory metadata =
            abi.decode(manager.lastHookData(), (AGCDataTypes.HookMetadata));
        assertEq(metadata.originalSender, user);
        assertEq(metadata.beneficiary, user);
        assertEq(metadata.intentHash, attestation.paymentId);
        assertEq(uint8(metadata.flowClass), uint8(AGCDataTypes.FlowClass.ProductivePayment));
        assertEq(metadata.qualityScoreBps, 9_000);
        assertEq(metadata.routeHash, FACILITATED_ROUTE_HASH);
    }

    function testSettledProductivePaymentRejectsReplay() public {
        BalanceDelta delta = agcIsCurrency0
            ? toBalanceDelta(-int128(100e18), int128(100e6))
            : toBalanceDelta(int128(100e6), -int128(100e18));
        manager.setNextSwapDelta(delta);

        SettlementRouter.ProductivePaymentAttestation memory attestation =
            _buildAttestation(100e18, keccak256("productive-payment"), uint16(10_000));
        bytes memory signature = _signAttestation(attestation);

        vm.prank(user);
        router.settleProductivePayment(attestation, 99e6, facilitator, signature);

        vm.prank(user);
        vm.expectRevert(SettlementRouter.ProductiveIntentAlreadyUsed.selector);
        router.settleProductivePayment(attestation, 99e6, facilitator, signature);
    }

    function testSettledProductivePaymentRespectsPause() public {
        router.setProductiveSettlementPaused(true);

        SettlementRouter.ProductivePaymentAttestation memory attestation =
            _buildAttestation(100e18, keccak256("paused-productive-payment"), uint16(10_000));
        bytes memory signature = _signAttestation(attestation);

        vm.prank(user);
        vm.expectRevert(SettlementRouter.ProductiveSettlementPaused.selector);
        router.settleProductivePayment(attestation, 99e6, facilitator, signature);
    }

    function testBuyWorkingCapitalSwapsUsdcIntoAgc() public {
        usdc.mint(user, 1_000e6);
        vm.prank(user);
        usdc.approve(address(router), type(uint256).max);

        BalanceDelta delta = agcIsCurrency0
            ? toBalanceDelta(int128(100e18), -int128(100e6))
            : toBalanceDelta(-int128(100e6), int128(100e18));
        manager.setNextSwapDelta(delta);

        vm.prank(user);
        uint256 agcOut = router.buyWorkingCapital(100e6, 99e18, user, keccak256("inventory"));

        assertEq(agcOut, 100e18);
        assertEq(agc.balanceOf(user), 1_100e18);
    }

    function testTreasuryBuybackBurnsAgc() public {
        router.setController(address(this));
        BalanceDelta delta = agcIsCurrency0
            ? toBalanceDelta(int128(50e18), -int128(50e6))
            : toBalanceDelta(-int128(50e6), int128(50e18));
        manager.setNextSwapDelta(delta);

        uint256 totalSupplyBefore = agc.totalSupply();
        uint256 burned = router.executeTreasuryBuyback(50e6, 49e18, keccak256("buyback"));

        assertEq(burned, 50e18);
        assertEq(agc.totalSupply(), totalSupplyBefore - 50e18);
        assertEq(usdc.balanceOf(address(vault)), 500_000e6 - 50e6);
    }

    function testTreasuryBuybackRefundsUnusedUsdcToVault() public {
        router.setController(address(this));
        BalanceDelta delta = agcIsCurrency0
            ? toBalanceDelta(int128(50e18), -int128(40e6))
            : toBalanceDelta(-int128(40e6), int128(50e18));
        manager.setNextSwapDelta(delta);

        uint256 burned = router.executeTreasuryBuyback(50e6, 49e18, keccak256("partial-buyback"));

        assertEq(burned, 50e18);
        assertEq(usdc.balanceOf(address(vault)), 500_000e6 - 40e6);
        assertEq(usdc.balanceOf(address(router)), 0);
    }

    function _buildAttestation(
        uint256 agcAmountIn,
        bytes32 paymentId,
        uint16 qualityScoreBps
    ) internal view returns (SettlementRouter.ProductivePaymentAttestation memory) {
        return SettlementRouter.ProductivePaymentAttestation({
            payer: user,
            recipient: merchant,
            agcAmountIn: agcAmountIn,
            paymentId: paymentId,
            qualityScoreBps: qualityScoreBps,
            deadline: uint64(block.timestamp + 1 hours),
            routeHash: FACILITATED_ROUTE_HASH
        });
    }

    function _signAttestation(
        SettlementRouter.ProductivePaymentAttestation memory attestation
    ) internal view returns (bytes memory) {
        bytes32 digest = router.hashProductivePaymentAttestation(attestation);
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(facilitatorPk, digest);
        return abi.encodePacked(r, s, v);
    }
}

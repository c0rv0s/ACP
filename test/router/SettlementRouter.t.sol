// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import "forge-std/Test.sol";
import { IHooks } from "v4-core/interfaces/IHooks.sol";
import { IPoolManager } from "v4-core/interfaces/IPoolManager.sol";
import { PoolKey } from "v4-core/types/PoolKey.sol";
import { Currency } from "v4-core/types/Currency.sol";
import { BalanceDelta, toBalanceDelta } from "v4-core/types/BalanceDelta.sol";
import { TickMath } from "v4-core/libraries/TickMath.sol";
import { AGCToken } from "../../src/AGCToken.sol";
import { SettlementRouter } from "../../src/SettlementRouter.sol";
import { IAGCHook } from "../../src/interfaces/IAGCHook.sol";
import { IStabilityVault } from "../../src/interfaces/IStabilityVault.sol";
import { MockUSDC } from "../../src/mocks/MockUSDC.sol";
import { MockHookAdapter } from "../mocks/MockHookAdapter.sol";
import { MockPoolManager } from "../mocks/MockPoolManager.sol";
import { MockVault } from "../mocks/MockVault.sol";

contract SettlementRouterTest is Test {
    bytes32 internal constant MINTER_ROLE = keccak256("MINTER_ROLE");
    bytes32 internal constant BURNER_ROLE = keccak256("BURNER_ROLE");

    AGCToken internal agc;
    MockUSDC internal usdc;
    MockPoolManager internal manager;
    MockHookAdapter internal hook;
    MockVault internal vault;
    SettlementRouter internal router;

    address internal user = address(0xABCD);
    address internal recipient = address(0xDCBA);
    bool internal agcIsCurrency0;

    function setUp() public {
        agc = new AGCToken(address(this));
        usdc = new MockUSDC(address(this));
        manager = new MockPoolManager();
        hook = new MockHookAdapter();
        vault = new MockVault(usdc, agc);

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

        vm.prank(user);
        agc.approve(address(router), type(uint256).max);
        usdc.mint(user, 1_000e6);
        vm.prank(user);
        usdc.approve(address(router), type(uint256).max);
    }

    function testSellAGCSwapsAndPaysRecipient() public {
        BalanceDelta delta = agcIsCurrency0
            ? toBalanceDelta(-int128(100e18), int128(50e6))
            : toBalanceDelta(int128(50e6), -int128(100e18));
        manager.setNextSwapDelta(delta);

        vm.prank(user);
        uint256 usdcOut = router.sellAGC(100e18, 49e6, recipient, keccak256("sell"));

        assertEq(usdcOut, 50e6);
        assertEq(usdc.balanceOf(recipient), 50e6);
        assertEq(manager.lastHookData().length, 0);
    }

    function testSellAGCRefundsUnusedInputToSeller() public {
        BalanceDelta delta = agcIsCurrency0
            ? toBalanceDelta(-int128(80e18), int128(40e6))
            : toBalanceDelta(int128(40e6), -int128(80e18));
        manager.setNextSwapDelta(delta);

        vm.prank(user);
        uint256 usdcOut = router.sellAGC(100e18, 39e6, recipient, keccak256("sell-refund"));

        assertEq(usdcOut, 40e6);
        assertEq(usdc.balanceOf(recipient), 40e6);
        assertEq(agc.balanceOf(user), 920e18);
        assertEq(agc.balanceOf(address(router)), 0);
    }

    function testBuyAGCSwapsUsdcIntoRecipientInventory() public {
        BalanceDelta delta = agcIsCurrency0
            ? toBalanceDelta(int128(200e18), -int128(100e6))
            : toBalanceDelta(-int128(100e6), int128(200e18));
        manager.setNextSwapDelta(delta);

        vm.prank(user);
        uint256 agcOut = router.buyAGC(100e6, 199e18, recipient, keccak256("buy"));

        assertEq(agcOut, 200e18);
        assertEq(agc.balanceOf(recipient), 200e18);
    }

    function testBuyAGCRevertsWhenSlippageExceeded() public {
        BalanceDelta delta = agcIsCurrency0
            ? toBalanceDelta(int128(150e18), -int128(100e6))
            : toBalanceDelta(-int128(100e6), int128(150e18));
        manager.setNextSwapDelta(delta);

        vm.prank(user);
        vm.expectRevert(SettlementRouter.SlippageExceeded.selector);
        router.buyAGC(100e6, 151e18, recipient, keccak256("buy-slippage"));

        assertEq(usdc.balanceOf(user), 1_000e6);
        assertEq(usdc.balanceOf(address(router)), 0);
    }

    function _buybackSqrtPriceLimit() internal view returns (uint160) {
        return agcIsCurrency0 ? TickMath.MIN_SQRT_PRICE + 1 : TickMath.MAX_SQRT_PRICE - 1;
    }

    function testTreasuryBuybackBurnsAgc() public {
        router.setController(address(this));
        BalanceDelta delta = agcIsCurrency0
            ? toBalanceDelta(int128(50e18), -int128(50e6))
            : toBalanceDelta(-int128(50e6), int128(50e18));
        manager.setNextSwapDelta(delta);

        uint256 totalSupplyBefore = agc.totalSupply();
        uint256 burned = router.executeTreasuryBuyback(
            50e6, 49e18, _buybackSqrtPriceLimit(), keccak256("buyback")
        );

        assertEq(burned, 50e18);
        assertEq(agc.totalSupply(), totalSupplyBefore - 50e18);
        assertEq(usdc.balanceOf(address(vault)), 500_000e6 - 50e6);
    }

    function testTreasuryBuybackRequiresController() public {
        vm.expectRevert(SettlementRouter.Unauthorized.selector);
        router.executeTreasuryBuyback(50e6, 49e18, _buybackSqrtPriceLimit(), keccak256("auth"));
    }

    function testTreasuryBuybackRefundsUnusedUsdcToVault() public {
        router.setController(address(this));
        BalanceDelta delta = agcIsCurrency0
            ? toBalanceDelta(int128(50e18), -int128(40e6))
            : toBalanceDelta(-int128(40e6), int128(50e18));
        manager.setNextSwapDelta(delta);

        uint256 burned = router.executeTreasuryBuyback(
            50e6, 49e18, _buybackSqrtPriceLimit(), keccak256("partial-buyback")
        );

        assertEq(burned, 50e18);
        assertEq(usdc.balanceOf(address(vault)), 500_000e6 - 40e6);
        assertEq(usdc.balanceOf(address(router)), 0);
    }
}

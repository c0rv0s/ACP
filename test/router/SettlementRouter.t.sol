// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import "forge-std/Test.sol";
import {IHooks} from "v4-core/interfaces/IHooks.sol";
import {IPoolManager} from "v4-core/interfaces/IPoolManager.sol";
import {PoolKey} from "v4-core/types/PoolKey.sol";
import {Currency} from "v4-core/types/Currency.sol";
import {BalanceDelta, toBalanceDelta} from "v4-core/types/BalanceDelta.sol";
import {AGCToken} from "../../src/AGCToken.sol";
import {SettlementRouter} from "../../src/SettlementRouter.sol";
import {IAGCHook} from "../../src/interfaces/IAGCHook.sol";
import {IStabilityVault} from "../../src/interfaces/IStabilityVault.sol";
import {MockUSDC} from "../../src/mocks/MockUSDC.sol";
import {MockHookAdapter} from "../mocks/MockHookAdapter.sol";
import {MockPoolManager} from "../mocks/MockPoolManager.sol";
import {MockVault} from "../mocks/MockVault.sol";

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
    address internal merchant = address(0xDCBA);
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
    }

    function testSettlePaymentSwapsAndPaysMerchant() public {
        BalanceDelta delta = agcIsCurrency0 ? toBalanceDelta(-int128(100e18), int128(100e6)) : toBalanceDelta(int128(100e6), -int128(100e18));
        manager.setNextSwapDelta(delta);

        vm.prank(user);
        uint256 usdcOut = router.settlePayment(100e18, 99e6, merchant, keccak256("payment"), uint16(10_000));

        assertEq(usdcOut, 100e6);
        assertEq(usdc.balanceOf(merchant), 100e6);
        assertEq(agc.balanceOf(address(manager)), 1_100e18);
    }

    function testBuyWorkingCapitalSwapsUsdcIntoAgc() public {
        usdc.mint(user, 1_000e6);
        vm.prank(user);
        usdc.approve(address(router), type(uint256).max);

        BalanceDelta delta = agcIsCurrency0 ? toBalanceDelta(int128(100e18), -int128(100e6)) : toBalanceDelta(-int128(100e6), int128(100e18));
        manager.setNextSwapDelta(delta);

        vm.prank(user);
        uint256 agcOut = router.buyWorkingCapital(100e6, 99e18, user, keccak256("inventory"));

        assertEq(agcOut, 100e18);
        assertEq(agc.balanceOf(user), 1_100e18);
    }

    function testTreasuryBuybackBurnsAgc() public {
        router.setController(address(this));
        BalanceDelta delta = agcIsCurrency0 ? toBalanceDelta(int128(50e18), -int128(50e6)) : toBalanceDelta(-int128(50e6), int128(50e18));
        manager.setNextSwapDelta(delta);

        uint256 totalSupplyBefore = agc.totalSupply();
        uint256 burned = router.executeTreasuryBuyback(50e6, 49e18, keccak256("buyback"));

        assertEq(burned, 50e18);
        assertEq(agc.totalSupply(), totalSupplyBefore - 50e18);
        assertEq(usdc.balanceOf(address(vault)), 500_000e6 - 50e6);
    }
}

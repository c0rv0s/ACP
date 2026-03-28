// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import "forge-std/Test.sol";
import {IHooks} from "v4-core/interfaces/IHooks.sol";
import {IPoolManager} from "v4-core/interfaces/IPoolManager.sol";
import {LPFeeLibrary} from "v4-core/libraries/LPFeeLibrary.sol";
import {PoolKey} from "v4-core/types/PoolKey.sol";
import {Currency} from "v4-core/types/Currency.sol";
import {BalanceDelta, toBalanceDelta} from "v4-core/types/BalanceDelta.sol";
import {AGCToken} from "../../src/AGCToken.sol";
import {AGCHook} from "../../src/AGCHook.sol";
import {MockUSDC} from "../../src/mocks/MockUSDC.sol";
import {AGCDataTypes} from "../../src/libraries/AGCDataTypes.sol";
import {MockPoolManager} from "../mocks/MockPoolManager.sol";
import {MockVault} from "../mocks/MockVault.sol";

contract AGCHookTest is Test {
    bytes32 internal constant MINTER_ROLE = keccak256("MINTER_ROLE");

    AGCToken internal agc;
    MockUSDC internal usdc;
    MockPoolManager internal manager;
    MockVault internal vault;
    AGCHook internal hook;

    PoolKey internal key;
    bool internal agcIsCurrency0;

    address internal router = address(0xBEEF);
    address internal user = address(0xCAFE);

    function setUp() public {
        agc = new AGCToken(address(this));
        usdc = new MockUSDC(address(this));
        manager = new MockPoolManager();
        vault = new MockVault(usdc, agc);

        agc.grantRole(MINTER_ROLE, address(this));
        agc.mint(address(manager), 1_000_000e18);
        usdc.mint(address(manager), 1_000_000e6);

        agcIsCurrency0 = address(agc) < address(usdc);

        AGCDataTypes.PoolConfig memory poolConfig = AGCDataTypes.PoolConfig({
            agcCurrency: Currency.wrap(address(agc)),
            usdcCurrency: Currency.wrap(address(usdc)),
            lpFee: LPFeeLibrary.DYNAMIC_FEE_FLAG,
            tickSpacing: 60,
            agcDecimals: 18,
            usdcDecimals: 6
        });

        AGCDataTypes.HookFeeConfig memory feeConfig = AGCDataTypes.HookFeeConfig({
            baseLPFee: 1_000,
            volatilityFeeSlope: 0,
            imbalanceFeeSlope: 0,
            productiveDiscount: 500,
            inventoryDiscount: 250,
            speculativeSurcharge: 1_000,
            defenseLpSurcharge: 500,
            productiveHookFee: 0,
            inventoryHookFee: 300,
            speculativeHookFee: 1_000,
            unknownHookFee: 500,
            defenseExitHookFee: 2_000,
            earlyWithdrawalFee: 1_500,
            minLpHoldTime: 1 days
        });

        hook = new AGCHook(address(this), IPoolManager(address(manager)), address(vault), poolConfig, feeConfig);
        key = hook.canonicalPoolKey();
        hook.setTrustedRouter(router, true);
        hook.setRewardDistributor(address(this));
        hook.setController(address(this));
    }

    function testProductiveSwapCreatesReceiptAndUpdatesOracle() public {
        AGCDataTypes.HookMetadata memory metadata = AGCDataTypes.HookMetadata({
            originalSender: user,
            beneficiary: user,
            intentHash: keccak256("payment-1"),
            flowClass: AGCDataTypes.FlowClass.ProductivePayment,
            qualityScoreBps: uint16(AGCDataTypes.BPS),
            routeHash: bytes32(0)
        });

        IPoolManager.SwapParams memory params = IPoolManager.SwapParams({
            zeroForOne: agcIsCurrency0,
            amountSpecified: -int256(100e18),
            sqrtPriceLimitX96: 0
        });

        vm.prank(address(manager));
        (bytes4 selector,, uint24 feeOverride) = hook.beforeSwap(router, key, params, abi.encode(metadata));
        assertEq(selector, IHooks.beforeSwap.selector);
        assertEq(feeOverride & LPFeeLibrary.REMOVE_OVERRIDE_MASK, 500);

        BalanceDelta delta = agcIsCurrency0 ? toBalanceDelta(-int128(100e18), int128(100e6)) : toBalanceDelta(int128(100e6), -int128(100e18));
        vm.prank(address(manager));
        hook.afterSwap(router, key, params, delta, abi.encode(metadata));

        AGCDataTypes.RewardReceipt memory receipt = hook.rewardReceipt(1);
        assertEq(receipt.beneficiary, user);
        assertEq(receipt.usdcAmount, 100e6);
        assertEq(receipt.epochId, 1);

        AGCDataTypes.EpochAccumulator memory accumulator = hook.currentAccumulator();
        assertEq(accumulator.productiveVolume, 100e6);
        assertEq(accumulator.productiveSettlementCount, 1);
        assertGt(accumulator.cumulativePriceX18, 0);
    }

    function testDefenseExitChargesTreasuryFee() public {
        hook.setRegime(AGCDataTypes.Regime.Defense);

        AGCDataTypes.HookMetadata memory metadata = AGCDataTypes.HookMetadata({
            originalSender: user,
            beneficiary: user,
            intentHash: keccak256("spec-1"),
            flowClass: AGCDataTypes.FlowClass.SpeculativeTrade,
            qualityScoreBps: uint16(AGCDataTypes.BPS),
            routeHash: bytes32(0)
        });

        IPoolManager.SwapParams memory params = IPoolManager.SwapParams({
            zeroForOne: agcIsCurrency0,
            amountSpecified: -int256(100e18),
            sqrtPriceLimitX96: 0
        });
        BalanceDelta delta = agcIsCurrency0 ? toBalanceDelta(-int128(100e18), int128(100e6)) : toBalanceDelta(int128(100e6), -int128(100e18));

        vm.prank(address(manager));
        (, int128 hookDelta) = hook.afterSwap(router, key, params, delta, abi.encode(metadata));
        assertEq(uint256(uint128(hookDelta)), 300_000);
        assertEq(usdc.balanceOf(address(vault)), 300_000);
    }

    function testEarlyLiquidityRemovalChargesFee() public {
        IPoolManager.ModifyLiquidityParams memory params = IPoolManager.ModifyLiquidityParams({
            tickLower: -120,
            tickUpper: 120,
            liquidityDelta: int256(1e18),
            salt: bytes32(0)
        });
        vm.prank(address(manager));
        hook.beforeAddLiquidity(user, key, params, "");

        BalanceDelta removeDelta =
            agcIsCurrency0 ? toBalanceDelta(int128(50e18), int128(50e6)) : toBalanceDelta(int128(50e6), int128(50e18));
        vm.prank(address(manager));
        (, BalanceDelta feeDelta) = hook.afterRemoveLiquidity(user, key, params, removeDelta, BalanceDelta.wrap(0), "");

        uint256 expectedAgcFee = 50e18 * 1_500 / AGCDataTypes.FEE_UNITS;
        uint256 expectedUsdcFee = 50e6 * 1_500 / AGCDataTypes.FEE_UNITS;

        if (agcIsCurrency0) {
            assertEq(uint256(uint128(feeDelta.amount0())), expectedAgcFee);
            assertEq(uint256(uint128(feeDelta.amount1())), expectedUsdcFee);
        } else {
            assertEq(uint256(uint128(feeDelta.amount0())), expectedUsdcFee);
            assertEq(uint256(uint128(feeDelta.amount1())), expectedAgcFee);
        }
        assertEq(agc.balanceOf(address(vault)), expectedAgcFee);
        assertEq(usdc.balanceOf(address(vault)), expectedUsdcFee);
    }
}

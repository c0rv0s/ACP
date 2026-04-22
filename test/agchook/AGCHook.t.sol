// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import "forge-std/Test.sol";
import { SafeCast as OzSafeCast } from "@openzeppelin/contracts/utils/math/SafeCast.sol";
import { IHooks } from "v4-core/interfaces/IHooks.sol";
import { IPoolManager } from "v4-core/interfaces/IPoolManager.sol";
import { LPFeeLibrary } from "v4-core/libraries/LPFeeLibrary.sol";
import { SafeCast as V4SafeCast } from "v4-core/libraries/SafeCast.sol";
import { SqrtPriceMath } from "v4-core/libraries/SqrtPriceMath.sol";
import { TickMath } from "v4-core/libraries/TickMath.sol";
import { PoolKey } from "v4-core/types/PoolKey.sol";
import { Currency } from "v4-core/types/Currency.sol";
import { BalanceDelta, toBalanceDelta } from "v4-core/types/BalanceDelta.sol";
import { PoolIdLibrary } from "v4-core/types/PoolId.sol";
import { AGCToken } from "../../src/AGCToken.sol";
import { AGCHook } from "../../src/AGCHook.sol";
import { MockUSDC } from "../../src/mocks/MockUSDC.sol";
import { AGCDataTypes } from "../../src/libraries/AGCDataTypes.sol";
import { MockPoolManager } from "../mocks/MockPoolManager.sol";
import { MockVault } from "../mocks/MockVault.sol";

contract AGCHookTest is Test {
    using PoolIdLibrary for PoolKey;

    bytes32 internal constant MINTER_ROLE = keccak256("MINTER_ROLE");
    uint160 internal constant Q96 = 2 ** 96;

    AGCToken internal agc;
    MockUSDC internal usdc;
    MockPoolManager internal manager;
    MockVault internal vault;
    AGCHook internal hook;

    PoolKey internal key;
    bool internal agcIsCurrency0;

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
            defenseLpSurcharge: 500,
            buyHookFee: 300,
            sellHookFee: 500,
            defenseExitHookFee: 2_000,
            earlyWithdrawalFee: 1_500,
            minLpHoldTime: 1 days
        });

        hook = new AGCHook(
            address(this), IPoolManager(address(manager)), address(vault), poolConfig, feeConfig
        );
        key = hook.canonicalPoolKey();
        hook.setController(address(this));

        _setMidPriceX18(5e17);
        vm.prank(address(manager));
        hook.afterInitialize(address(this), key, _sqrtPriceX96ForPrice(5e17), 0);
    }

    function testBuyAndSellSwapsAccumulateQuoteVolume() public {
        IPoolManager.SwapParams memory buyParams = _exactInputBuyParams(100e6);
        vm.prank(address(manager));
        hook.afterSwap(address(this), key, buyParams, _usdcToAgcDelta(100e6, 200e18), "");

        IPoolManager.SwapParams memory sellParams = _exactInputSellParams(80e18);
        vm.prank(address(manager));
        hook.afterSwap(address(this), key, sellParams, _agcToUsdcDelta(80e18, 40e6), "");

        AGCDataTypes.EpochSnapshot memory snapshot = hook.previewEpochSnapshot();
        assertEq(snapshot.grossBuyVolumeQuoteX18, 100e6 * AGCDataTypes.QUOTE_SCALE);
        assertEq(snapshot.grossSellVolumeQuoteX18, 40e6 * AGCDataTypes.QUOTE_SCALE);
        assertEq(snapshot.totalVolumeQuoteX18, 140e6 * AGCDataTypes.QUOTE_SCALE);
    }

    function testPreviewSnapshotUsesTimeWeightedMidPrice() public {
        vm.warp(block.timestamp + 1 hours);
        _setMidPriceX18(55e16);

        IPoolManager.ModifyLiquidityParams memory params = IPoolManager.ModifyLiquidityParams({
            tickLower: -120, tickUpper: 120, liquidityDelta: int256(1e18), salt: bytes32(0)
        });
        vm.prank(address(manager));
        hook.beforeAddLiquidity(user, key, params, "");

        vm.warp(block.timestamp + 1 hours);
        AGCDataTypes.EpochSnapshot memory snapshot = hook.previewEpochSnapshot();

        AGCDataTypes.EpochAccumulator memory acc = hook.currentAccumulator();
        uint256 tail = acc.lastMidPriceX18 * (uint256(uint64(block.timestamp)) - acc.lastObservedAt);
        uint256 cumForTwap = acc.cumulativeMidPriceTimeX18 + tail;
        uint256 elapsedForTwap = uint256(uint64(block.timestamp)) - acc.startedAt;
        assertApproxEqAbs(snapshot.shortTwapPriceX18, cumForTwap / elapsedForTwap, 1e12);
        assertEq(
            snapshot.realizedVolatilityBps,
            acc.cumulativeAbsMidPriceChangeBps / (acc.observationCount - 1)
        );
    }

    function testDefenseSellChargesTreasuryFee() public {
        hook.setRegime(AGCDataTypes.Regime.Defense);

        IPoolManager.SwapParams memory params = _exactInputSellParams(100e18);
        BalanceDelta delta = _agcToUsdcDelta(100e18, 100e6);

        vm.prank(address(manager));
        (, int128 hookDelta) = hook.afterSwap(address(this), key, params, delta, "");

        assertEq(OzSafeCast.toUint256(int256(hookDelta)), 250_000);
        assertEq(usdc.balanceOf(address(vault)), 250_000);
    }

    function testEarlyLiquidityRemovalChargesFee() public {
        IPoolManager.ModifyLiquidityParams memory params = IPoolManager.ModifyLiquidityParams({
            tickLower: -120, tickUpper: 120, liquidityDelta: int256(1e18), salt: bytes32(0)
        });
        vm.prank(address(manager));
        hook.beforeAddLiquidity(user, key, params, "");

        BalanceDelta removeDelta = agcIsCurrency0
            ? toBalanceDelta(int128(50e18), int128(50e6))
            : toBalanceDelta(int128(50e6), int128(50e18));
        vm.prank(address(manager));
        (, BalanceDelta feeDelta) =
            hook.afterRemoveLiquidity(user, key, params, removeDelta, BalanceDelta.wrap(0), "");

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

    function testV4CoreRemovalPayoutDeltaIsPositive() public view {
        uint160 sqrtPriceX96 = _sqrtPriceX96ForPrice(5e17);
        int256 amount0 =
            SqrtPriceMath.getAmount0Delta(sqrtPriceX96, TickMath.getSqrtPriceAtTick(120), -int128(1e18));
        int256 amount1 =
            SqrtPriceMath.getAmount1Delta(TickMath.getSqrtPriceAtTick(-120), sqrtPriceX96, -int128(1e18));

        assertGt(amount0, 0);
        assertGt(amount1, 0);
    }

    function testEarlyLiquidityRemovalChargesPositiveSideOfMixedDelta() public {
        IPoolManager.ModifyLiquidityParams memory params = IPoolManager.ModifyLiquidityParams({
            tickLower: -120, tickUpper: 120, liquidityDelta: int256(1e18), salt: bytes32(0)
        });
        vm.prank(address(manager));
        hook.beforeAddLiquidity(user, key, params, "");

        BalanceDelta removeDelta = agcIsCurrency0
            ? toBalanceDelta(int128(50e18), -int128(50e6))
            : toBalanceDelta(-int128(50e6), int128(50e18));
        vm.prank(address(manager));
        (, BalanceDelta feeDelta) =
            hook.afterRemoveLiquidity(user, key, params, removeDelta, BalanceDelta.wrap(0), "");

        uint256 expectedFee = 50e18 * 1_500 / AGCDataTypes.FEE_UNITS;

        if (agcIsCurrency0) {
            assertEq(uint256(uint128(feeDelta.amount0())), expectedFee);
            assertEq(feeDelta.amount1(), 0);
            assertEq(agc.balanceOf(address(vault)), expectedFee);
            assertEq(usdc.balanceOf(address(vault)), 0);
        } else {
            assertEq(feeDelta.amount0(), 0);
            assertEq(uint256(uint128(feeDelta.amount1())), expectedFee);
            assertEq(usdc.balanceOf(address(vault)), 0);
            assertEq(agc.balanceOf(address(vault)), expectedFee);
        }
    }

    function testLiquidityAgeRefreshesOnEachAddAndClearsOnFullRemoval() public {
        IPoolManager.ModifyLiquidityParams memory params = IPoolManager.ModifyLiquidityParams({
            tickLower: -120, tickUpper: 120, liquidityDelta: int256(1e18), salt: bytes32(0)
        });
        vm.prank(address(manager));
        hook.beforeAddLiquidity(user, key, params, "");

        vm.warp(block.timestamp + 2 days);
        vm.prank(address(manager));
        hook.beforeAddLiquidity(user, key, params, "");
        assertEq(hook.positionAge(_positionKeyHash(params)), uint40(block.timestamp));

        manager.setPositionLiquidity(key, user, params.tickLower, params.tickUpper, params.salt, 0);

        BalanceDelta removeDelta = agcIsCurrency0
            ? toBalanceDelta(int128(50e18), int128(50e6))
            : toBalanceDelta(int128(50e6), int128(50e18));
        vm.prank(address(manager));
        (, BalanceDelta feeDelta) =
            hook.afterRemoveLiquidity(user, key, params, removeDelta, BalanceDelta.wrap(0), "");

        uint256 expectedAgcFee = 50e18 * 1_500 / AGCDataTypes.FEE_UNITS;
        uint256 expectedUsdcFee = 50e6 * 1_500 / AGCDataTypes.FEE_UNITS;

        if (agcIsCurrency0) {
            assertEq(uint256(uint128(feeDelta.amount0())), expectedAgcFee);
            assertEq(uint256(uint128(feeDelta.amount1())), expectedUsdcFee);
        } else {
            assertEq(uint256(uint128(feeDelta.amount0())), expectedUsdcFee);
            assertEq(uint256(uint128(feeDelta.amount1())), expectedAgcFee);
        }
        assertEq(hook.positionAge(_positionKeyHash(params)), 0);
    }

    function testDynamicLpFeeCapsInsteadOfWrapping() public {
        hook.setFeeConfig(
            AGCDataTypes.HookFeeConfig({
                baseLPFee: 1_000,
                volatilityFeeSlope: type(uint24).max,
                imbalanceFeeSlope: 0,
                defenseLpSurcharge: 500,
                buyHookFee: 300,
                sellHookFee: 500,
                defenseExitHookFee: 2_000,
                earlyWithdrawalFee: 1_500,
                minLpHoldTime: 1 days
            })
        );

        IPoolManager.ModifyLiquidityParams memory params = IPoolManager.ModifyLiquidityParams({
            tickLower: -120, tickUpper: 120, liquidityDelta: int256(1e18), salt: bytes32(0)
        });

        vm.warp(block.timestamp + 1);
        _setMidPriceX18(100e18);
        vm.prank(address(manager));
        hook.beforeAddLiquidity(user, key, params, "");

        IPoolManager.SwapParams memory swapParams = _exactInputSellParams(1e18);
        vm.prank(address(manager));
        (, , uint24 feeOverride) = hook.beforeSwap(address(this), key, swapParams, "");

        assertEq(LPFeeLibrary.removeOverrideFlag(feeOverride), LPFeeLibrary.MAX_LP_FEE);
    }

    function _exactInputSellParams(
        uint256 agcAmountIn
    ) internal view returns (IPoolManager.SwapParams memory) {
        return IPoolManager.SwapParams({
            zeroForOne: agcIsCurrency0,
            amountSpecified: -OzSafeCast.toInt256(agcAmountIn),
            sqrtPriceLimitX96: 0
        });
    }

    function _exactInputBuyParams(
        uint256 usdcAmountIn
    ) internal view returns (IPoolManager.SwapParams memory) {
        return IPoolManager.SwapParams({
            zeroForOne: !agcIsCurrency0,
            amountSpecified: -OzSafeCast.toInt256(usdcAmountIn),
            sqrtPriceLimitX96: 0
        });
    }

    function _agcToUsdcDelta(
        uint256 agcAmount,
        uint256 usdcAmount
    ) internal view returns (BalanceDelta delta) {
        return agcIsCurrency0
            ? toBalanceDelta(-V4SafeCast.toInt128(agcAmount), V4SafeCast.toInt128(usdcAmount))
            : toBalanceDelta(V4SafeCast.toInt128(usdcAmount), -V4SafeCast.toInt128(agcAmount));
    }

    function _usdcToAgcDelta(
        uint256 usdcAmount,
        uint256 agcAmount
    ) internal view returns (BalanceDelta delta) {
        return agcIsCurrency0
            ? toBalanceDelta(V4SafeCast.toInt128(agcAmount), -V4SafeCast.toInt128(usdcAmount))
            : toBalanceDelta(-V4SafeCast.toInt128(usdcAmount), V4SafeCast.toInt128(agcAmount));
    }

    function _setMidPriceX18(
        uint256 priceX18
    ) internal {
        manager.setSlot0(key, _sqrtPriceX96ForPrice(priceX18), 0, 0, 0);
    }

    function _sqrtPriceX96ForPrice(
        uint256 priceX18
    ) internal view returns (uint160) {
        uint256 amount0 = agcIsCurrency0 ? 10 ** 18 : 10 ** 6;
        uint256 amount1 = agcIsCurrency0 ? 10 ** 6 : 10 ** 18;
        uint256 numerator = priceX18 * amount0;
        uint256 ratioX192 = (numerator << 192) / (amount1 * 1e18);
        return uint160(_sqrt(ratioX192));
    }

    function _sqrt(
        uint256 value
    ) internal pure returns (uint256) {
        if (value == 0) return 0;
        uint256 x = value;
        uint256 y = (x + 1) / 2;
        while (y < x) {
            x = y;
            y = (x + value / x) / 2;
        }
        return x;
    }

    function _positionKeyHash(
        IPoolManager.ModifyLiquidityParams memory params
    ) internal view returns (bytes32) {
        return keccak256(abi.encode(user, key.toId(), params.tickLower, params.tickUpper, params.salt));
    }
}

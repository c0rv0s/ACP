// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import { Math } from "@openzeppelin/contracts/utils/math/Math.sol";
import { SafeCast as OzSafeCast } from "@openzeppelin/contracts/utils/math/SafeCast.sol";
import { Ownable } from "@openzeppelin/contracts/access/Ownable.sol";
import { Ownable2Step } from "@openzeppelin/contracts/access/Ownable2Step.sol";
import { IHooks } from "v4-core/interfaces/IHooks.sol";
import { IPoolManager } from "v4-core/interfaces/IPoolManager.sol";
import { PoolKey } from "v4-core/types/PoolKey.sol";
import { PoolId, PoolIdLibrary } from "v4-core/types/PoolId.sol";
import { BalanceDelta, BalanceDeltaLibrary, toBalanceDelta } from "v4-core/types/BalanceDelta.sol";
import { BeforeSwapDelta, BeforeSwapDeltaLibrary } from "v4-core/types/BeforeSwapDelta.sol";
import { Currency, CurrencyLibrary } from "v4-core/types/Currency.sol";
import { Hooks } from "v4-core/libraries/Hooks.sol";
import { LPFeeLibrary } from "v4-core/libraries/LPFeeLibrary.sol";
import { FullMath } from "v4-core/libraries/FullMath.sol";
import { SafeCast as V4SafeCast } from "v4-core/libraries/SafeCast.sol";
import { StateLibrary } from "v4-core/libraries/StateLibrary.sol";
import { IAGCHook } from "./interfaces/IAGCHook.sol";
import { AGCDataTypes } from "./libraries/AGCDataTypes.sol";

contract AGCHook is Ownable2Step, IHooks, IAGCHook {
    using BalanceDeltaLibrary for BalanceDelta;
    using CurrencyLibrary for Currency;
    using PoolIdLibrary for PoolKey;

    error Unauthorized();
    error InvalidPool();
    error InvalidController();
    error UnsupportedDecimalConfig();

    event ControllerUpdated(address indexed controller);
    event RegimeUpdated(AGCDataTypes.Regime indexed regime);
    event FeeConfigUpdated(
        uint24 baseLPFee,
        uint24 buyHookFee,
        uint24 sellHookFee,
        uint24 defenseExitHookFee,
        uint24 earlyWithdrawalFee
    );
    event EpochSnapshotConsumed(
        uint64 indexed epochId,
        uint256 grossBuyVolumeQuoteX18,
        uint256 grossSellVolumeQuoteX18,
        uint256 totalVolumeQuoteX18
    );

    struct SwapFootprint {
        uint256 agcAmount;
        uint256 usdcAmount;
        uint256 executionPriceX18;
        bool agcToUsdc;
    }

    IPoolManager public immutable manager;
    address public immutable treasuryVault;

    PoolKey internal _canonicalPoolKey;
    PoolId internal immutable _canonicalPoolId;
    AGCDataTypes.HookFeeConfig internal _feeConfig;
    AGCDataTypes.EpochAccumulator internal _currentAccumulator;

    Currency public immutable agcCurrency;
    Currency public immutable usdcCurrency;
    bool public immutable agcIsCurrency0;

    uint256 internal immutable _priceNumeratorScale;
    uint256 internal immutable _priceDenominatorScale;

    address public controller;
    AGCDataTypes.Regime public override currentRegime;

    mapping(bytes32 positionKey => uint40 addedAt) public positionAge;

    uint160 internal constant REQUIRED_HOOK_FLAGS = Hooks.BEFORE_ADD_LIQUIDITY_FLAG
        | Hooks.AFTER_ADD_LIQUIDITY_FLAG | Hooks.BEFORE_REMOVE_LIQUIDITY_FLAG
        | Hooks.AFTER_REMOVE_LIQUIDITY_FLAG | Hooks.BEFORE_SWAP_FLAG | Hooks.AFTER_SWAP_FLAG
        | Hooks.AFTER_SWAP_RETURNS_DELTA_FLAG | Hooks.AFTER_REMOVE_LIQUIDITY_RETURNS_DELTA_FLAG;

    constructor(
        address admin,
        IPoolManager poolManager,
        address vault,
        AGCDataTypes.PoolConfig memory poolConfig,
        AGCDataTypes.HookFeeConfig memory hookFeeConfig
    ) Ownable(admin) {
        if (poolConfig.agcDecimals < poolConfig.usdcDecimals) revert UnsupportedDecimalConfig();

        bool agcFirst =
            Currency.unwrap(poolConfig.agcCurrency) < Currency.unwrap(poolConfig.usdcCurrency);
        PoolKey memory poolKey = PoolKey({
            currency0: agcFirst ? poolConfig.agcCurrency : poolConfig.usdcCurrency,
            currency1: agcFirst ? poolConfig.usdcCurrency : poolConfig.agcCurrency,
            fee: poolConfig.lpFee,
            tickSpacing: poolConfig.tickSpacing,
            hooks: IHooks(address(this))
        });

        manager = poolManager;
        treasuryVault = vault;
        _canonicalPoolKey = poolKey;
        _canonicalPoolId = poolKey.toId();
        agcCurrency = poolConfig.agcCurrency;
        usdcCurrency = poolConfig.usdcCurrency;
        agcIsCurrency0 = agcFirst;
        _feeConfig = hookFeeConfig;

        _priceNumeratorScale = 10 ** (18 + poolConfig.agcDecimals - poolConfig.usdcDecimals);
        _priceDenominatorScale = 1;

        _currentAccumulator.epochId = 1;
        _currentAccumulator.startedAt = uint64(block.timestamp);
        _currentAccumulator.updatedAt = uint64(block.timestamp);
    }

    modifier onlyPoolManager() {
        if (msg.sender != address(manager)) revert Unauthorized();
        _;
    }

    modifier onlyControllerOrOwner() {
        if (msg.sender != controller && msg.sender != owner()) revert Unauthorized();
        _;
    }

    function canonicalPoolKey() external view returns (PoolKey memory) {
        return _canonicalPoolKey;
    }

    function feeConfig() external view returns (AGCDataTypes.HookFeeConfig memory) {
        return _feeConfig;
    }

    function requiredHookFlags() external pure returns (uint160) {
        return REQUIRED_HOOK_FLAGS;
    }

    function currentEpochId() external view returns (uint64) {
        return _currentAccumulator.epochId;
    }

    function currentAccumulator() external view returns (AGCDataTypes.EpochAccumulator memory) {
        return _currentAccumulator;
    }

    function setController(
        address nextController
    ) external onlyOwner {
        controller = nextController;
        emit ControllerUpdated(nextController);
    }

    function setRegime(
        AGCDataTypes.Regime newRegime
    ) external onlyControllerOrOwner {
        currentRegime = newRegime;
        emit RegimeUpdated(newRegime);
    }

    function setFeeConfig(
        AGCDataTypes.HookFeeConfig calldata newConfig
    ) external onlyOwner {
        _feeConfig = newConfig;
        emit FeeConfigUpdated(
            newConfig.baseLPFee,
            newConfig.buyHookFee,
            newConfig.sellHookFee,
            newConfig.defenseExitHookFee,
            newConfig.earlyWithdrawalFee
        );
    }

    function previewEpochSnapshot() external view returns (AGCDataTypes.EpochSnapshot memory snapshot) {
        return _previewEpochSnapshot(_currentAccumulator, _currentMidPriceX18());
    }

    function consumeEpochSnapshot()
        external
        returns (AGCDataTypes.EpochSnapshot memory snapshot)
    {
        if (msg.sender != controller) revert InvalidController();

        uint256 currentMidPriceX18 = _currentMidPriceX18();
        snapshot = _previewEpochSnapshot(_currentAccumulator, currentMidPriceX18);

        emit EpochSnapshotConsumed(
            snapshot.epochId,
            snapshot.grossBuyVolumeQuoteX18,
            snapshot.grossSellVolumeQuoteX18,
            snapshot.totalVolumeQuoteX18
        );

        delete _currentAccumulator;
        _currentAccumulator.epochId = snapshot.epochId + 1;
        _currentAccumulator.startedAt = uint64(block.timestamp);
        _currentAccumulator.updatedAt = uint64(block.timestamp);
        _seedObservation(currentMidPriceX18);
    }

    function beforeInitialize(
        address,
        PoolKey calldata,
        uint160
    ) external pure returns (bytes4) {
        return IHooks.beforeInitialize.selector;
    }

    function afterInitialize(
        address,
        PoolKey calldata key,
        uint160 sqrtPriceX96,
        int24
    ) external onlyPoolManager returns (bytes4) {
        _validatePool(key);
        _seedObservation(_priceFromSqrtPriceX96(sqrtPriceX96));
        return IHooks.afterInitialize.selector;
    }

    function beforeAddLiquidity(
        address sender,
        PoolKey calldata key,
        IPoolManager.ModifyLiquidityParams calldata params,
        bytes calldata
    ) external onlyPoolManager returns (bytes4) {
        _validatePool(key);
        _observeMidPrice();

        bytes32 positionKeyHash = _positionKey(sender, params);
        positionAge[positionKeyHash] = uint40(block.timestamp);

        return IHooks.beforeAddLiquidity.selector;
    }

    function afterAddLiquidity(
        address,
        PoolKey calldata key,
        IPoolManager.ModifyLiquidityParams calldata,
        BalanceDelta,
        BalanceDelta,
        bytes calldata
    ) external view onlyPoolManager returns (bytes4, BalanceDelta) {
        _validatePool(key);
        return (IHooks.afterAddLiquidity.selector, toBalanceDelta(0, 0));
    }

    function beforeRemoveLiquidity(
        address,
        PoolKey calldata key,
        IPoolManager.ModifyLiquidityParams calldata,
        bytes calldata
    ) external onlyPoolManager returns (bytes4) {
        _validatePool(key);
        _observeMidPrice();
        return IHooks.beforeRemoveLiquidity.selector;
    }

    function afterRemoveLiquidity(
        address sender,
        PoolKey calldata key,
        IPoolManager.ModifyLiquidityParams calldata params,
        BalanceDelta delta,
        BalanceDelta,
        bytes calldata
    ) external onlyPoolManager returns (bytes4, BalanceDelta feeDelta) {
        _validatePool(key);

        bytes32 positionKeyHash = _positionKey(sender, params);
        uint40 addedAt = positionAge[positionKeyHash];
        if (addedAt == 0) {
            return (IHooks.afterRemoveLiquidity.selector, toBalanceDelta(0, 0));
        }

        (uint128 remainingLiquidity,,) = StateLibrary.getPositionInfo(
            manager, _canonicalPoolId, sender, params.tickLower, params.tickUpper, params.salt
        );
        if (remainingLiquidity == 0) {
            delete positionAge[positionKeyHash];
        }

        uint256 age = block.timestamp - addedAt;
        if (age >= _feeConfig.minLpHoldTime || _feeConfig.earlyWithdrawalFee == 0) {
            return (IHooks.afterRemoveLiquidity.selector, toBalanceDelta(0, 0));
        }

        int128 amount0 = delta.amount0();
        int128 amount1 = delta.amount1();
        // v4 decrease/burn deltas are expected to be positive payouts to the LP.
        // Charge only on positive components so an unexpected mixed-sign delta cannot skip the fee entirely.
        uint256 feeAmount0 = amount0 > 0
            ? V4SafeCast.toUint128(amount0) * _feeConfig.earlyWithdrawalFee / AGCDataTypes.FEE_UNITS
            : 0;
        uint256 feeAmount1 = amount1 > 0
            ? V4SafeCast.toUint128(amount1) * _feeConfig.earlyWithdrawalFee / AGCDataTypes.FEE_UNITS
            : 0;

        if (feeAmount0 == 0 && feeAmount1 == 0) {
            return (IHooks.afterRemoveLiquidity.selector, toBalanceDelta(0, 0));
        }

        _collectHookFee(key.currency0, feeAmount0);
        _collectHookFee(key.currency1, feeAmount1);

        feeDelta = toBalanceDelta(V4SafeCast.toInt128(feeAmount0), V4SafeCast.toInt128(feeAmount1));
        return (IHooks.afterRemoveLiquidity.selector, feeDelta);
    }

    function beforeSwap(
        address,
        PoolKey calldata key,
        IPoolManager.SwapParams calldata params,
        bytes calldata
    ) external view onlyPoolManager returns (bytes4, BeforeSwapDelta, uint24 feeOverride) {
        _validatePool(key);

        bool agcToUsdc = _isAgcToUsdc(key, params);
        feeOverride = _dynamicLPFee(agcToUsdc) | LPFeeLibrary.OVERRIDE_FEE_FLAG;
        return (IHooks.beforeSwap.selector, BeforeSwapDeltaLibrary.ZERO_DELTA, feeOverride);
    }

    function afterSwap(
        address,
        PoolKey calldata key,
        IPoolManager.SwapParams calldata params,
        BalanceDelta delta,
        bytes calldata
    ) external onlyPoolManager returns (bytes4, int128 hookDelta) {
        _validatePool(key);

        SwapFootprint memory footprint = _swapFootprint(delta, key, params);
        hookDelta = _collectSwapFee(footprint.agcToUsdc, key, params, delta);
        _updateAccumulator(footprint, _currentMidPriceX18());

        return (IHooks.afterSwap.selector, hookDelta);
    }

    function beforeDonate(
        address,
        PoolKey calldata,
        uint256,
        uint256,
        bytes calldata
    ) external pure returns (bytes4) {
        return IHooks.beforeDonate.selector;
    }

    function afterDonate(
        address,
        PoolKey calldata,
        uint256,
        uint256,
        bytes calldata
    ) external pure returns (bytes4) {
        return IHooks.afterDonate.selector;
    }

    function _validatePool(
        PoolKey calldata key
    ) internal view {
        if (PoolId.unwrap(key.toId()) != PoolId.unwrap(_canonicalPoolId)) revert InvalidPool();
    }

    function _positionKey(
        address provider,
        IPoolManager.ModifyLiquidityParams calldata params
    ) internal view returns (bytes32) {
        return keccak256(
            abi.encode(provider, _canonicalPoolId, params.tickLower, params.tickUpper, params.salt)
        );
    }

    function _hookFee(
        bool agcToUsdc
    ) internal view returns (uint24 feeUnits) {
        feeUnits = agcToUsdc ? _feeConfig.sellHookFee : _feeConfig.buyHookFee;
        if (currentRegime == AGCDataTypes.Regime.Defense && agcToUsdc) {
            feeUnits += _feeConfig.defenseExitHookFee;
        }
    }

    function _dynamicLPFee(
        bool agcToUsdc
    ) internal view returns (uint24 feeUnits) {
        uint256 fee = _feeConfig.baseLPFee;
        fee += Math.mulDiv(
            _previewVolatilityBps(), _feeConfig.volatilityFeeSlope, AGCDataTypes.BPS
        );
        fee += Math.mulDiv(
            _previewSellPressureBps(), _feeConfig.imbalanceFeeSlope, AGCDataTypes.BPS
        );

        if (currentRegime == AGCDataTypes.Regime.Defense && agcToUsdc) {
            fee += _feeConfig.defenseLpSurcharge;
        }

        if (fee > LPFeeLibrary.MAX_LP_FEE) {
            fee = LPFeeLibrary.MAX_LP_FEE;
        }

        feeUnits = OzSafeCast.toUint24(fee);
    }

    function _previewVolatilityBps() internal view returns (uint256) {
        return _volatilityBps(_currentAccumulator);
    }

    function _previewSellPressureBps() internal view returns (uint256) {
        AGCDataTypes.EpochAccumulator memory acc = _currentAccumulator;
        if (acc.totalVolumeQuoteX18 == 0) {
            return 0;
        }
        return acc.grossSellVolumeQuoteX18 * AGCDataTypes.BPS / acc.totalVolumeQuoteX18;
    }

    function _volatilityBps(
        AGCDataTypes.EpochAccumulator memory acc
    ) internal pure returns (uint256) {
        if (acc.observationCount <= 1) return 0;
        return acc.cumulativeAbsMidPriceChangeBps / (acc.observationCount - 1);
    }

    function _unspecifiedCurrencyAndAmount(
        PoolKey calldata key,
        IPoolManager.SwapParams calldata params,
        BalanceDelta delta
    ) internal pure returns (Currency unspecifiedCurrency, int128 unspecifiedAmount) {
        bool specifiedTokenIs0 = _specifiedTokenIs0(params);

        if (specifiedTokenIs0) {
            return (key.currency1, delta.amount1());
        }

        return (key.currency0, delta.amount0());
    }

    function _isAgcToUsdc(
        PoolKey calldata key,
        IPoolManager.SwapParams calldata params
    ) internal view returns (bool) {
        (Currency inputCurrency, Currency outputCurrency) = _inputOutputCurrency(key, params);
        return inputCurrency == agcCurrency && outputCurrency == usdcCurrency;
    }

    function _inputOutputCurrency(
        PoolKey calldata key,
        IPoolManager.SwapParams calldata params
    ) internal pure returns (Currency inputCurrency, Currency outputCurrency) {
        bool specifiedTokenIs0 = _specifiedTokenIs0(params);

        if (params.amountSpecified < 0) {
            inputCurrency = specifiedTokenIs0 ? key.currency0 : key.currency1;
            outputCurrency = specifiedTokenIs0 ? key.currency1 : key.currency0;
        } else {
            outputCurrency = specifiedTokenIs0 ? key.currency0 : key.currency1;
            inputCurrency = specifiedTokenIs0 ? key.currency1 : key.currency0;
        }
    }

    function _specifiedTokenIs0(
        IPoolManager.SwapParams calldata params
    ) internal pure returns (bool) {
        return (params.amountSpecified < 0) == params.zeroForOne;
    }

    function _currentMidPriceX18() internal view returns (uint256) {
        (uint160 sqrtPriceX96,,,) = StateLibrary.getSlot0(manager, _canonicalPoolId);
        return _priceFromSqrtPriceX96(sqrtPriceX96);
    }

    function _priceFromSqrtPriceX96(
        uint160 sqrtPriceX96
    ) internal view returns (uint256 priceX18) {
        if (sqrtPriceX96 == 0) return 0;

        uint256 ratioX96 =
            FullMath.mulDiv(uint256(sqrtPriceX96), uint256(sqrtPriceX96), uint256(1) << 96);

        if (agcIsCurrency0) {
            return FullMath.mulDiv(ratioX96, _priceNumeratorScale, uint256(1) << 96);
        }

        return FullMath.mulDiv(_priceNumeratorScale, uint256(1) << 96, ratioX96);
    }

    function _executionPriceX18(
        uint256 agcAmount,
        uint256 usdcAmount
    ) internal view returns (uint256) {
        if (agcAmount == 0 || usdcAmount == 0) return 0;
        return Math.mulDiv(usdcAmount, _priceNumeratorScale, agcAmount * _priceDenominatorScale);
    }

    function _swapFootprint(
        BalanceDelta delta,
        PoolKey calldata key,
        IPoolManager.SwapParams calldata params
    ) internal view returns (SwapFootprint memory footprint) {
        int128 delta0 = delta.amount0();
        int128 delta1 = delta.amount1();

        footprint.agcAmount = uint256(uint128(_abs(agcIsCurrency0 ? delta0 : delta1)));
        footprint.usdcAmount = uint256(uint128(_abs(agcIsCurrency0 ? delta1 : delta0)));
        footprint.agcToUsdc = _isAgcToUsdc(key, params);
        footprint.executionPriceX18 = _executionPriceX18(footprint.agcAmount, footprint.usdcAmount);
    }

    function _collectSwapFee(
        bool agcToUsdc,
        PoolKey calldata key,
        IPoolManager.SwapParams calldata params,
        BalanceDelta delta
    ) internal returns (int128 hookDelta) {
        (Currency unspecifiedCurrency, int128 unspecifiedAmount) =
            _unspecifiedCurrencyAndAmount(key, params, delta);
        if (unspecifiedAmount == 0) {
            return 0;
        }

        uint24 hookFeeUnits = _hookFee(agcToUsdc);
        if (hookFeeUnits == 0) {
            return 0;
        }

        uint256 grossAmount = uint256(uint128(_abs(unspecifiedAmount)));
        uint256 hookFeeAmount = grossAmount * hookFeeUnits / AGCDataTypes.FEE_UNITS;
        if (hookFeeAmount == 0) {
            return 0;
        }

        manager.take(unspecifiedCurrency, address(this), hookFeeAmount);
        unspecifiedCurrency.transfer(treasuryVault, hookFeeAmount);
        _recordFee(unspecifiedCurrency, hookFeeAmount);
        return V4SafeCast.toInt128(hookFeeAmount);
    }

    function _observeMidPrice() internal {
        _observeMidPriceAtPrice(_currentMidPriceX18());
    }

    function _seedObservation(
        uint256 currentMidPriceX18
    ) internal {
        if (currentMidPriceX18 == 0) return;

        AGCDataTypes.EpochAccumulator storage acc = _currentAccumulator;
        uint64 observedAt = uint64(block.timestamp);
        acc.updatedAt = observedAt;
        acc.lastObservedAt = observedAt;
        acc.lastMidPriceX18 = currentMidPriceX18;
        if (acc.observationCount == 0) {
            acc.observationCount = 1;
        }
    }

    function _observeMidPriceAtPrice(
        uint256 currentMidPriceX18
    ) internal {
        if (currentMidPriceX18 == 0) return;

        AGCDataTypes.EpochAccumulator storage acc = _currentAccumulator;
        uint64 observedAt = uint64(block.timestamp);

        if (acc.lastObservedAt == 0) {
            _seedObservation(currentMidPriceX18);
            return;
        }

        if (observedAt > acc.lastObservedAt && acc.lastMidPriceX18 > 0) {
            uint256 elapsed = observedAt - acc.lastObservedAt;
            acc.cumulativeMidPriceTimeX18 += acc.lastMidPriceX18 * elapsed;

            uint256 priceChangeBps = currentMidPriceX18 > acc.lastMidPriceX18
                ? (currentMidPriceX18 - acc.lastMidPriceX18) * AGCDataTypes.BPS
                    / acc.lastMidPriceX18
                : (acc.lastMidPriceX18 - currentMidPriceX18) * AGCDataTypes.BPS
                    / acc.lastMidPriceX18;
            acc.cumulativeAbsMidPriceChangeBps += priceChangeBps;
            acc.observationCount += 1;
            acc.lastObservedAt = observedAt;
        }

        acc.updatedAt = observedAt;
        acc.lastMidPriceX18 = currentMidPriceX18;
    }

    function _previewEpochSnapshot(
        AGCDataTypes.EpochAccumulator memory acc,
        uint256 currentMidPriceX18
    ) internal view returns (AGCDataTypes.EpochSnapshot memory snapshot) {
        uint64 endedAt = uint64(block.timestamp);
        uint256 cumulativeMidPriceTimeX18 = acc.cumulativeMidPriceTimeX18;
        if (endedAt > acc.lastObservedAt && acc.lastMidPriceX18 > 0) {
            cumulativeMidPriceTimeX18 += acc.lastMidPriceX18 * (endedAt - acc.lastObservedAt);
        }

        uint256 epochElapsed = endedAt > acc.startedAt ? endedAt - acc.startedAt : 0;
        uint256 shortTwapPriceX18 = epochElapsed == 0
            ? acc.lastMidPriceX18
            : cumulativeMidPriceTimeX18 == 0 && acc.observationCount == 0
                ? currentMidPriceX18
                : cumulativeMidPriceTimeX18 / epochElapsed;

        snapshot = AGCDataTypes.EpochSnapshot({
            epochId: acc.epochId,
            startedAt: acc.startedAt,
            endedAt: endedAt,
            grossBuyVolumeQuoteX18: acc.grossBuyVolumeQuoteX18,
            grossSellVolumeQuoteX18: acc.grossSellVolumeQuoteX18,
            totalVolumeQuoteX18: acc.totalVolumeQuoteX18,
            shortTwapPriceX18: shortTwapPriceX18,
            realizedVolatilityBps: _volatilityBps(acc),
            totalHookFeesQuoteX18: acc.totalHookFeesQuoteX18,
            totalHookFeesAgc: acc.totalHookFeesAgc
        });
    }

    function _updateAccumulator(
        SwapFootprint memory footprint,
        uint256 currentMidPriceX18
    ) internal {
        AGCDataTypes.EpochAccumulator storage acc = _currentAccumulator;
        uint256 quoteAmountX18 = footprint.usdcAmount * AGCDataTypes.QUOTE_SCALE;

        _observeMidPriceAtPrice(currentMidPriceX18);
        acc.totalVolumeQuoteX18 += quoteAmountX18;

        if (footprint.agcToUsdc) {
            acc.grossSellVolumeQuoteX18 += quoteAmountX18;
        } else {
            acc.grossBuyVolumeQuoteX18 += quoteAmountX18;
        }
    }

    function _recordFee(
        Currency currency,
        uint256 amount
    ) internal {
        if (currency == usdcCurrency) {
            _currentAccumulator.totalHookFeesQuoteX18 += amount * AGCDataTypes.QUOTE_SCALE;
        } else if (currency == agcCurrency) {
            _currentAccumulator.totalHookFeesAgc += amount;
        }
    }

    function _collectHookFee(
        Currency currency,
        uint256 amount
    ) internal {
        if (amount == 0) return;

        manager.take(currency, address(this), amount);
        currency.transfer(treasuryVault, amount);
        _recordFee(currency, amount);
    }

    function _abs(
        int128 value
    ) internal pure returns (int128) {
        return value < 0 ? -value : value;
    }
}

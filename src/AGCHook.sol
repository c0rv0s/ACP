// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {Math} from "@openzeppelin/contracts/utils/math/Math.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {Ownable2Step} from "@openzeppelin/contracts/access/Ownable2Step.sol";
import {IHooks} from "v4-core/interfaces/IHooks.sol";
import {IPoolManager} from "v4-core/interfaces/IPoolManager.sol";
import {PoolKey} from "v4-core/types/PoolKey.sol";
import {PoolId, PoolIdLibrary} from "v4-core/types/PoolId.sol";
import {BalanceDelta, BalanceDeltaLibrary, toBalanceDelta} from "v4-core/types/BalanceDelta.sol";
import {BeforeSwapDelta, BeforeSwapDeltaLibrary} from "v4-core/types/BeforeSwapDelta.sol";
import {Currency, CurrencyLibrary} from "v4-core/types/Currency.sol";
import {Hooks} from "v4-core/libraries/Hooks.sol";
import {LPFeeLibrary} from "v4-core/libraries/LPFeeLibrary.sol";
import {IAGCHook} from "./interfaces/IAGCHook.sol";
import {AGCDataTypes} from "./libraries/AGCDataTypes.sol";

contract AGCHook is Ownable2Step, IHooks, IAGCHook {
    using BalanceDeltaLibrary for BalanceDelta;
    using CurrencyLibrary for Currency;
    using PoolIdLibrary for PoolKey;

    error Unauthorized();
    error InvalidPool();
    error InvalidController();
    error InvalidRewardDistributor();
    error UnsupportedDecimalConfig();

    event ControllerUpdated(address indexed controller);
    event RewardDistributorUpdated(address indexed distributor);
    event TrustedRouterUpdated(address indexed router, bool trusted);
    event RegimeUpdated(AGCDataTypes.Regime indexed regime);
    event FeeConfigUpdated(
        uint24 baseLPFee,
        uint24 productiveHookFee,
        uint24 speculativeHookFee,
        uint24 defenseExitHookFee,
        uint24 earlyWithdrawalFee
    );
    event RewardReceiptCreated(
        uint256 indexed receiptId,
        uint64 indexed epochId,
        address indexed beneficiary,
        bytes32 intentHash,
        uint256 usdcAmount
    );
    event EpochSnapshotConsumed(uint64 indexed epochId, uint256 totalVolume, uint256 productiveVolume);

    struct SwapFootprint {
        uint256 agcAmount;
        uint256 usdcAmount;
        uint256 executionPriceX18;
        bool agcToUsdc;
    }

    IPoolManager public immutable manager;
    address public immutable stabilityVault;

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
    address public rewardDistributor;
    AGCDataTypes.Regime public override currentRegime;

    uint256 public nextReceiptId = 1;

    mapping(address router => bool trusted) public override trustedRouters;
    mapping(uint256 receiptId => AGCDataTypes.RewardReceipt receipt) internal _rewardReceipt;
    mapping(bytes32 intentHash => bool consumed) public usedIntentHash;
    mapping(bytes32 positionKey => uint40 addedAt) public positionAge;
    mapping(address user => uint64 lastEpochSeen) public lastProductiveEpochSeen;

    uint160 internal constant REQUIRED_HOOK_FLAGS =
        Hooks.BEFORE_ADD_LIQUIDITY_FLAG | Hooks.AFTER_ADD_LIQUIDITY_FLAG | Hooks.BEFORE_REMOVE_LIQUIDITY_FLAG
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

        bool agcFirst = Currency.unwrap(poolConfig.agcCurrency) < Currency.unwrap(poolConfig.usdcCurrency);
        PoolKey memory poolKey = PoolKey({
            currency0: agcFirst ? poolConfig.agcCurrency : poolConfig.usdcCurrency,
            currency1: agcFirst ? poolConfig.usdcCurrency : poolConfig.agcCurrency,
            fee: poolConfig.lpFee,
            tickSpacing: poolConfig.tickSpacing,
            hooks: IHooks(address(this))
        });

        manager = poolManager;
        stabilityVault = vault;
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

    modifier onlyController() {
        if (msg.sender != controller) revert InvalidController();
        _;
    }

    modifier onlyControllerOrOwner() {
        if (msg.sender != controller && msg.sender != owner()) revert Unauthorized();
        _;
    }

    modifier onlyRewardDistributor() {
        if (msg.sender != rewardDistributor) revert InvalidRewardDistributor();
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

    function rewardReceipt(uint256 receiptId) external view returns (AGCDataTypes.RewardReceipt memory) {
        return _rewardReceipt[receiptId];
    }

    function setController(address nextController) external onlyOwner {
        controller = nextController;
        emit ControllerUpdated(nextController);
    }

    function setRewardDistributor(address distributor) external onlyOwner {
        rewardDistributor = distributor;
        emit RewardDistributorUpdated(distributor);
    }

    function setTrustedRouter(address router, bool trusted) external onlyOwner {
        trustedRouters[router] = trusted;
        emit TrustedRouterUpdated(router, trusted);
    }

    function setRegime(AGCDataTypes.Regime newRegime) external onlyControllerOrOwner {
        currentRegime = newRegime;
        emit RegimeUpdated(newRegime);
    }

    function setFeeConfig(AGCDataTypes.HookFeeConfig calldata newConfig) external onlyOwner {
        _feeConfig = newConfig;
        emit FeeConfigUpdated(
            newConfig.baseLPFee,
            newConfig.productiveHookFee,
            newConfig.speculativeHookFee,
            newConfig.defenseExitHookFee,
            newConfig.earlyWithdrawalFee
        );
    }

    function previewMetadata(address router, bytes calldata hookData)
        external
        view
        returns (AGCDataTypes.HookMetadata memory)
    {
        return _metadataFrom(router, hookData);
    }

    function consumeEpochSnapshot() external onlyController returns (AGCDataTypes.EpochSnapshot memory snapshot) {
        AGCDataTypes.EpochAccumulator memory acc = _currentAccumulator;

        snapshot = AGCDataTypes.EpochSnapshot({
            epochId: acc.epochId,
            startedAt: acc.startedAt,
            endedAt: uint64(block.timestamp),
            productiveVolume: acc.productiveVolume,
            totalVolume: acc.totalVolume,
            netExitVolume: acc.netExitVolume,
            shortTwapPriceX18: acc.sampleCount == 0 ? 0 : acc.cumulativePriceX18 / acc.sampleCount,
            productiveSettlementPriceX18: acc.productiveSettlementCount == 0
                ? 0
                : acc.cumulativeProductivePriceX18 / acc.productiveSettlementCount,
            realizedVolatilityBps: _volatilityBps(acc),
            productiveSettlementCount: acc.productiveSettlementCount,
            productiveUsers: acc.productiveUsers,
            repeatUsers: acc.repeatUsers,
            totalHookFeesUsdc: acc.totalHookFeesUsdc,
            totalHookFeesAgc: acc.totalHookFeesAgc
        });

        emit EpochSnapshotConsumed(snapshot.epochId, snapshot.totalVolume, snapshot.productiveVolume);

        delete _currentAccumulator;
        _currentAccumulator.epochId = snapshot.epochId + 1;
        _currentAccumulator.startedAt = uint64(block.timestamp);
        _currentAccumulator.updatedAt = uint64(block.timestamp);
    }

    function consumeRewardReceipt(uint256 receiptId)
        external
        onlyRewardDistributor
        returns (AGCDataTypes.RewardReceipt memory receipt)
    {
        receipt = _rewardReceipt[receiptId];
        receipt.consumed = true;
        _rewardReceipt[receiptId].consumed = true;
    }

    function beforeInitialize(address, PoolKey calldata, uint160) external pure returns (bytes4) {
        return IHooks.beforeInitialize.selector;
    }

    function afterInitialize(address, PoolKey calldata, uint160, int24) external pure returns (bytes4) {
        return IHooks.afterInitialize.selector;
    }

    function beforeAddLiquidity(
        address sender,
        PoolKey calldata key,
        IPoolManager.ModifyLiquidityParams calldata params,
        bytes calldata
    ) external onlyPoolManager returns (bytes4) {
        _validatePool(key);

        bytes32 positionKeyHash = _positionKey(sender, params);
        if (positionAge[positionKeyHash] == 0) {
            positionAge[positionKeyHash] = uint40(block.timestamp);
        }

        return IHooks.beforeAddLiquidity.selector;
    }

    function afterAddLiquidity(
        address,
        PoolKey calldata key,
        IPoolManager.ModifyLiquidityParams calldata,
        BalanceDelta,
        BalanceDelta,
        bytes calldata
    ) external onlyPoolManager returns (bytes4, BalanceDelta) {
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

        uint40 addedAt = positionAge[_positionKey(sender, params)];
        if (addedAt == 0) {
            return (IHooks.afterRemoveLiquidity.selector, toBalanceDelta(0, 0));
        }

        uint256 age = block.timestamp - addedAt;
        if (age >= _feeConfig.minLpHoldTime || _feeConfig.earlyWithdrawalFee == 0) {
            return (IHooks.afterRemoveLiquidity.selector, toBalanceDelta(0, 0));
        }

        int128 amount0 = delta.amount0();
        int128 amount1 = delta.amount1();
        if (amount0 < 0 || amount1 < 0) {
            return (IHooks.afterRemoveLiquidity.selector, toBalanceDelta(0, 0));
        }

        uint256 feeAmount0 = uint128(amount0) * _feeConfig.earlyWithdrawalFee / AGCDataTypes.FEE_UNITS;
        uint256 feeAmount1 = uint128(amount1) * _feeConfig.earlyWithdrawalFee / AGCDataTypes.FEE_UNITS;

        _collectHookFee(key.currency0, feeAmount0);
        _collectHookFee(key.currency1, feeAmount1);

        feeDelta = toBalanceDelta(int128(uint128(feeAmount0)), int128(uint128(feeAmount1)));
        return (IHooks.afterRemoveLiquidity.selector, feeDelta);
    }

    function beforeSwap(
        address sender,
        PoolKey calldata key,
        IPoolManager.SwapParams calldata params,
        bytes calldata hookData
    ) external onlyPoolManager returns (bytes4, BeforeSwapDelta, uint24 feeOverride) {
        _validatePool(key);

        AGCDataTypes.HookMetadata memory metadata = _metadataFrom(sender, hookData);
        bool agcToUsdc = _isAgcToUsdc(key, params);

        feeOverride = _dynamicLPFee(metadata.flowClass, agcToUsdc) | LPFeeLibrary.OVERRIDE_FEE_FLAG;
        return (IHooks.beforeSwap.selector, BeforeSwapDeltaLibrary.ZERO_DELTA, feeOverride);
    }

    function afterSwap(
        address sender,
        PoolKey calldata key,
        IPoolManager.SwapParams calldata params,
        BalanceDelta delta,
        bytes calldata hookData
    ) external onlyPoolManager returns (bytes4, int128 hookDelta) {
        _validatePool(key);

        AGCDataTypes.HookMetadata memory metadata = _metadataFrom(sender, hookData);
        SwapFootprint memory footprint = _swapFootprint(delta, key, params);

        hookDelta = _collectSwapFee(metadata.flowClass, footprint.agcToUsdc, key, params, delta);
        _updateAccumulator(metadata, footprint.usdcAmount, footprint.executionPriceX18, footprint.agcToUsdc);

        if (
            metadata.flowClass == AGCDataTypes.FlowClass.ProductivePayment && footprint.agcToUsdc
                && metadata.intentHash != bytes32(0) && !usedIntentHash[metadata.intentHash]
        ) {
            _createRewardReceipt(metadata, footprint.agcAmount, footprint.usdcAmount);
        }

        return (IHooks.afterSwap.selector, hookDelta);
    }

    function beforeDonate(address, PoolKey calldata, uint256, uint256, bytes calldata) external pure returns (bytes4) {
        return IHooks.beforeDonate.selector;
    }

    function afterDonate(address, PoolKey calldata, uint256, uint256, bytes calldata) external pure returns (bytes4) {
        return IHooks.afterDonate.selector;
    }

    function _validatePool(PoolKey calldata key) internal view {
        if (PoolId.unwrap(key.toId()) != PoolId.unwrap(_canonicalPoolId)) revert InvalidPool();
    }

    function _metadataFrom(address router, bytes calldata hookData)
        internal
        view
        returns (AGCDataTypes.HookMetadata memory metadata)
    {
        metadata.originalSender = router;
        metadata.beneficiary = router;
        metadata.flowClass = AGCDataTypes.FlowClass.Unknown;
        metadata.qualityScoreBps = uint16(AGCDataTypes.BPS);

        if (!trustedRouters[router] || hookData.length == 0) {
            return metadata;
        }

        metadata = abi.decode(hookData, (AGCDataTypes.HookMetadata));
        if (metadata.originalSender == address(0)) {
            metadata.originalSender = router;
        }
        if (metadata.beneficiary == address(0)) {
            metadata.beneficiary = metadata.originalSender;
        }
        if (metadata.qualityScoreBps == 0) {
            metadata.qualityScoreBps = uint16(AGCDataTypes.BPS);
        }
    }

    function _positionKey(address provider, IPoolManager.ModifyLiquidityParams calldata params)
        internal
        view
        returns (bytes32)
    {
        return keccak256(abi.encode(provider, _canonicalPoolId, params.tickLower, params.tickUpper, params.salt));
    }

    function _hookFee(AGCDataTypes.FlowClass flowClass, bool agcToUsdc) internal view returns (uint24 feeUnits) {
        if (flowClass == AGCDataTypes.FlowClass.ProductivePayment) {
            feeUnits = _feeConfig.productiveHookFee;
        } else if (flowClass == AGCDataTypes.FlowClass.InventoryRebalance) {
            feeUnits = _feeConfig.inventoryHookFee;
        } else if (flowClass == AGCDataTypes.FlowClass.SpeculativeTrade) {
            feeUnits = _feeConfig.speculativeHookFee;
        } else if (flowClass == AGCDataTypes.FlowClass.StressExit) {
            feeUnits = _feeConfig.defenseExitHookFee;
        } else {
            feeUnits = _feeConfig.unknownHookFee;
        }

        if (
            currentRegime == AGCDataTypes.Regime.Defense && agcToUsdc
                && flowClass != AGCDataTypes.FlowClass.ProductivePayment
        ) {
            feeUnits += _feeConfig.defenseExitHookFee;
        }
    }

    function _dynamicLPFee(AGCDataTypes.FlowClass flowClass, bool agcToUsdc) internal view returns (uint24 feeUnits) {
        feeUnits = _feeConfig.baseLPFee;
        feeUnits += uint24(_previewVolatilityBps() * _feeConfig.volatilityFeeSlope / AGCDataTypes.BPS);
        feeUnits += uint24(_previewExitPressureBps() * _feeConfig.imbalanceFeeSlope / AGCDataTypes.BPS);

        if (flowClass == AGCDataTypes.FlowClass.ProductivePayment) {
            feeUnits = feeUnits > _feeConfig.productiveDiscount ? feeUnits - _feeConfig.productiveDiscount : 0;
        } else if (flowClass == AGCDataTypes.FlowClass.InventoryRebalance) {
            feeUnits = feeUnits > _feeConfig.inventoryDiscount ? feeUnits - _feeConfig.inventoryDiscount : 0;
        } else if (
            flowClass == AGCDataTypes.FlowClass.SpeculativeTrade || flowClass == AGCDataTypes.FlowClass.Unknown
        ) {
            feeUnits += _feeConfig.speculativeSurcharge;
        }

        if (currentRegime == AGCDataTypes.Regime.Defense && agcToUsdc) {
            feeUnits += _feeConfig.defenseLpSurcharge;
        }

        if (feeUnits > LPFeeLibrary.MAX_LP_FEE) {
            feeUnits = LPFeeLibrary.MAX_LP_FEE;
        }
    }

    function _previewVolatilityBps() internal view returns (uint256) {
        return _volatilityBps(_currentAccumulator);
    }

    function _previewExitPressureBps() internal view returns (uint256) {
        if (_currentAccumulator.totalVolume == 0 || _currentAccumulator.netExitVolume <= 0) {
            return 0;
        }

        return uint256(_currentAccumulator.netExitVolume) * AGCDataTypes.BPS / _currentAccumulator.totalVolume;
    }

    function _volatilityBps(AGCDataTypes.EpochAccumulator memory acc) internal pure returns (uint256) {
        if (acc.sampleCount <= 1) return 0;

        uint256 averagePrice = acc.cumulativePriceX18 / acc.sampleCount;
        if (averagePrice == 0) return 0;

        return acc.cumulativeAbsPriceChangeX18 * AGCDataTypes.BPS / (averagePrice * (acc.sampleCount - 1));
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

    function _isAgcToUsdc(PoolKey calldata key, IPoolManager.SwapParams calldata params) internal view returns (bool) {
        (Currency inputCurrency, Currency outputCurrency) = _inputOutputCurrency(key, params);
        return inputCurrency == agcCurrency && outputCurrency == usdcCurrency;
    }

    function _inputOutputCurrency(PoolKey calldata key, IPoolManager.SwapParams calldata params)
        internal
        pure
        returns (Currency inputCurrency, Currency outputCurrency)
    {
        bool specifiedTokenIs0 = _specifiedTokenIs0(params);

        if (params.amountSpecified < 0) {
            inputCurrency = specifiedTokenIs0 ? key.currency0 : key.currency1;
            outputCurrency = specifiedTokenIs0 ? key.currency1 : key.currency0;
        } else {
            outputCurrency = specifiedTokenIs0 ? key.currency0 : key.currency1;
            inputCurrency = specifiedTokenIs0 ? key.currency1 : key.currency0;
        }
    }

    function _specifiedTokenIs0(IPoolManager.SwapParams calldata params) internal pure returns (bool) {
        return (params.amountSpecified < 0) == params.zeroForOne;
    }

    function _executionPriceX18(uint256 agcAmount, uint256 usdcAmount) internal view returns (uint256) {
        if (agcAmount == 0 || usdcAmount == 0) return 0;
        return Math.mulDiv(usdcAmount, _priceNumeratorScale, agcAmount * _priceDenominatorScale);
    }

    function _swapFootprint(BalanceDelta delta, PoolKey calldata key, IPoolManager.SwapParams calldata params)
        internal
        view
        returns (SwapFootprint memory footprint)
    {
        int128 delta0 = delta.amount0();
        int128 delta1 = delta.amount1();

        footprint.agcAmount = uint256(uint128(_abs(agcIsCurrency0 ? delta0 : delta1)));
        footprint.usdcAmount = uint256(uint128(_abs(agcIsCurrency0 ? delta1 : delta0)));
        footprint.agcToUsdc = _isAgcToUsdc(key, params);
        footprint.executionPriceX18 = _executionPriceX18(footprint.agcAmount, footprint.usdcAmount);
    }

    function _collectSwapFee(
        AGCDataTypes.FlowClass flowClass,
        bool agcToUsdc,
        PoolKey calldata key,
        IPoolManager.SwapParams calldata params,
        BalanceDelta delta
    ) internal returns (int128 hookDelta) {
        (Currency unspecifiedCurrency, int128 unspecifiedAmount) = _unspecifiedCurrencyAndAmount(key, params, delta);
        if (unspecifiedAmount == 0) {
            return 0;
        }

        uint24 hookFeeUnits = _hookFee(flowClass, agcToUsdc);
        if (hookFeeUnits == 0) {
            return 0;
        }

        uint256 grossAmount = uint256(uint128(_abs(unspecifiedAmount)));
        uint256 hookFeeAmount = grossAmount * hookFeeUnits / AGCDataTypes.FEE_UNITS;
        if (hookFeeAmount == 0) {
            return 0;
        }

        manager.take(unspecifiedCurrency, address(this), hookFeeAmount);
        unspecifiedCurrency.transfer(stabilityVault, hookFeeAmount);
        _recordFee(unspecifiedCurrency, hookFeeAmount);
        return int128(uint128(hookFeeAmount));
    }

    function _updateAccumulator(
        AGCDataTypes.HookMetadata memory metadata,
        uint256 usdcVolume,
        uint256 executionPriceX18,
        bool agcToUsdc
    ) internal {
        AGCDataTypes.EpochAccumulator storage acc = _currentAccumulator;

        acc.updatedAt = uint64(block.timestamp);
        acc.sampleCount += 1;
        acc.totalVolume += usdcVolume;
        acc.cumulativePriceX18 += executionPriceX18;

        if (acc.lastPriceX18 != 0 && executionPriceX18 > 0) {
            acc.cumulativeAbsPriceChangeX18 += executionPriceX18 > acc.lastPriceX18
                ? executionPriceX18 - acc.lastPriceX18
                : acc.lastPriceX18 - executionPriceX18;
        }

        acc.lastPriceX18 = executionPriceX18;

        if (agcToUsdc) {
            acc.netExitVolume += int256(usdcVolume);
        } else {
            acc.netExitVolume -= int256(usdcVolume);
        }

        if (metadata.flowClass == AGCDataTypes.FlowClass.ProductivePayment && agcToUsdc) {
            acc.productiveVolume += usdcVolume;
            acc.productiveSettlementCount += 1;
            acc.cumulativeProductivePriceX18 += executionPriceX18;
            _trackProductiveUser(metadata.originalSender);
        }
    }

    function _trackProductiveUser(address user) internal {
        AGCDataTypes.EpochAccumulator storage acc = _currentAccumulator;
        if (lastProductiveEpochSeen[user] == acc.epochId) {
            return;
        }

        if (lastProductiveEpochSeen[user] + 1 == acc.epochId) {
            acc.repeatUsers += 1;
        }

        lastProductiveEpochSeen[user] = acc.epochId;
        acc.productiveUsers += 1;
    }

    function _createRewardReceipt(
        AGCDataTypes.HookMetadata memory metadata,
        uint256 agcAmount,
        uint256 usdcAmount
    ) internal {
        usedIntentHash[metadata.intentHash] = true;

        uint256 receiptId = nextReceiptId++;
        _rewardReceipt[receiptId] = AGCDataTypes.RewardReceipt({
            beneficiary: metadata.beneficiary,
            originalSender: metadata.originalSender,
            intentHash: metadata.intentHash,
            flowClass: metadata.flowClass,
            epochId: _currentAccumulator.epochId,
            createdAt: uint64(block.timestamp),
            qualityScoreBps: metadata.qualityScoreBps,
            agcAmount: agcAmount,
            usdcAmount: usdcAmount,
            consumed: false
        });

        emit RewardReceiptCreated(receiptId, _currentAccumulator.epochId, metadata.beneficiary, metadata.intentHash, usdcAmount);
    }

    function _recordFee(Currency currency, uint256 amount) internal {
        if (currency == usdcCurrency) {
            _currentAccumulator.totalHookFeesUsdc += amount;
        } else if (currency == agcCurrency) {
            _currentAccumulator.totalHookFeesAgc += amount;
        }
    }

    function _collectHookFee(Currency currency, uint256 amount) internal {
        if (amount == 0) return;

        manager.take(currency, address(this), amount);
        currency.transfer(stabilityVault, amount);
        _recordFee(currency, amount);
    }

    function _abs(int128 value) internal pure returns (int128) {
        return value < 0 ? -value : value;
    }
}

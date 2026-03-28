// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {Ownable2Step} from "@openzeppelin/contracts/access/Ownable2Step.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {IHooks} from "v4-core/interfaces/IHooks.sol";
import {IPoolManager} from "v4-core/interfaces/IPoolManager.sol";
import {IUnlockCallback} from "v4-core/interfaces/callback/IUnlockCallback.sol";
import {PoolKey} from "v4-core/types/PoolKey.sol";
import {BalanceDelta, BalanceDeltaLibrary} from "v4-core/types/BalanceDelta.sol";
import {Currency} from "v4-core/types/Currency.sol";
import {TickMath} from "v4-core/libraries/TickMath.sol";
import {AGCToken} from "./AGCToken.sol";
import {IAGCHook} from "./interfaces/IAGCHook.sol";
import {ISettlementRouter} from "./interfaces/ISettlementRouter.sol";
import {IStabilityVault} from "./interfaces/IStabilityVault.sol";
import {AGCDataTypes} from "./libraries/AGCDataTypes.sol";
import {PoolCurrencySettlement} from "./libraries/PoolCurrencySettlement.sol";

contract SettlementRouter is Ownable2Step, IUnlockCallback, ISettlementRouter {
    using SafeERC20 for IERC20;
    using BalanceDeltaLibrary for BalanceDelta;
    using PoolCurrencySettlement for Currency;

    error Unauthorized();
    error InvalidRecipient();
    error InvalidFlowClass();
    error SlippageExceeded();
    error InvalidPoolManager();

    event ControllerUpdated(address indexed controller);
    event PaymentSettled(
        address indexed payer,
        address indexed recipient,
        bytes32 indexed paymentId,
        uint256 agcIn,
        uint256 usdcOut
    );
    event WorkingCapitalPurchased(address indexed buyer, address indexed recipient, uint256 usdcIn, uint256 agcOut);
    event TreasuryBuybackExecuted(bytes32 indexed refId, uint256 usdcSpent, uint256 agcBurned);

    enum Action {
        Payment,
        WorkingCapital,
        TreasuryBuyback
    }

    struct CallbackData {
        Action action;
        address payer;
        address recipient;
        uint256 amountIn;
        uint256 minAmountOut;
        bytes32 refId;
        bytes hookData;
    }

    AGCToken public immutable agc;
    IERC20 public immutable usdc;
    IPoolManager public immutable manager;
    IAGCHook public immutable hook;
    IStabilityVault public immutable vault;

    address public controller;

    constructor(
        address admin,
        AGCToken agcToken,
        IERC20 usdcToken,
        IPoolManager poolManager,
        IAGCHook hookContract,
        IStabilityVault stabilityVault
    ) Ownable(admin) {
        agc = agcToken;
        usdc = usdcToken;
        manager = poolManager;
        hook = hookContract;
        vault = stabilityVault;
    }

    modifier onlyController() {
        if (msg.sender != controller) revert Unauthorized();
        _;
    }

    function setController(address nextController) external onlyOwner {
        controller = nextController;
        emit ControllerUpdated(nextController);
    }

    function settlePayment(
        uint256 agcAmountIn,
        uint256 minUsdcOut,
        address recipient,
        bytes32 paymentId,
        uint16 qualityScoreBps
    ) external returns (uint256 usdcAmountOut) {
        if (recipient == address(0)) revert InvalidRecipient();

        agc.transferFrom(msg.sender, address(this), agcAmountIn);

        AGCDataTypes.HookMetadata memory metadata = AGCDataTypes.HookMetadata({
            originalSender: msg.sender,
            beneficiary: msg.sender,
            intentHash: paymentId,
            flowClass: AGCDataTypes.FlowClass.ProductivePayment,
            qualityScoreBps: qualityScoreBps == 0 ? uint16(AGCDataTypes.BPS) : qualityScoreBps,
            routeHash: keccak256("x402-payment")
        });

        usdcAmountOut = abi.decode(
            manager.unlock(
                abi.encode(
                    CallbackData({
                        action: Action.Payment,
                        payer: msg.sender,
                        recipient: recipient,
                        amountIn: agcAmountIn,
                        minAmountOut: minUsdcOut,
                        refId: paymentId,
                        hookData: abi.encode(metadata)
                    })
                )
            ),
            (uint256)
        );

        emit PaymentSettled(msg.sender, recipient, paymentId, agcAmountIn, usdcAmountOut);
    }

    function buyWorkingCapital(uint256 usdcAmountIn, uint256 minAgcOut, address recipient, bytes32 refId)
        external
        returns (uint256 agcAmountOut)
    {
        if (recipient == address(0)) revert InvalidRecipient();

        usdc.safeTransferFrom(msg.sender, address(this), usdcAmountIn);

        AGCDataTypes.HookMetadata memory metadata = AGCDataTypes.HookMetadata({
            originalSender: msg.sender,
            beneficiary: recipient,
            intentHash: refId,
            flowClass: AGCDataTypes.FlowClass.InventoryRebalance,
            qualityScoreBps: uint16(AGCDataTypes.BPS),
            routeHash: keccak256("working-capital")
        });

        agcAmountOut = abi.decode(
            manager.unlock(
                abi.encode(
                    CallbackData({
                        action: Action.WorkingCapital,
                        payer: msg.sender,
                        recipient: recipient,
                        amountIn: usdcAmountIn,
                        minAmountOut: minAgcOut,
                        refId: refId,
                        hookData: abi.encode(metadata)
                    })
                )
            ),
            (uint256)
        );

        emit WorkingCapitalPurchased(msg.sender, recipient, usdcAmountIn, agcAmountOut);
    }

    function executeTreasuryBuyback(uint256 usdcAmountIn, uint256 minAgcOut, bytes32 refId)
        external
        onlyController
        returns (uint256 agcBurned)
    {
        vault.spendUSDC(address(this), usdcAmountIn);

        AGCDataTypes.HookMetadata memory metadata = AGCDataTypes.HookMetadata({
            originalSender: msg.sender,
            beneficiary: address(this),
            intentHash: refId,
            flowClass: AGCDataTypes.FlowClass.StressExit,
            qualityScoreBps: uint16(AGCDataTypes.BPS),
            routeHash: keccak256("treasury-buyback")
        });

        agcBurned = abi.decode(
            manager.unlock(
                abi.encode(
                    CallbackData({
                        action: Action.TreasuryBuyback,
                        payer: address(this),
                        recipient: address(this),
                        amountIn: usdcAmountIn,
                        minAmountOut: minAgcOut,
                        refId: refId,
                        hookData: abi.encode(metadata)
                    })
                )
            ),
            (uint256)
        );

        emit TreasuryBuybackExecuted(refId, usdcAmountIn, agcBurned);
    }

    function unlockCallback(bytes calldata rawData) external returns (bytes memory) {
        if (msg.sender != address(manager)) revert InvalidPoolManager();

        CallbackData memory data = abi.decode(rawData, (CallbackData));
        PoolKey memory key = hook.canonicalPoolKey();

        bool zeroForOne = _zeroForOne(data.action, key);
        Currency inputCurrency = zeroForOne ? key.currency0 : key.currency1;
        Currency outputCurrency = zeroForOne ? key.currency1 : key.currency0;

        IPoolManager.SwapParams memory params = IPoolManager.SwapParams({
            zeroForOne: zeroForOne,
            amountSpecified: -int256(data.amountIn),
            sqrtPriceLimitX96: zeroForOne ? TickMath.MIN_SQRT_PRICE + 1 : TickMath.MAX_SQRT_PRICE - 1
        });

        BalanceDelta delta = manager.swap(key, params, data.hookData);
        (uint256 inputUsed, uint256 outputAmount) = _inputAndOutput(zeroForOne, delta);

        if (outputAmount < data.minAmountOut) revert SlippageExceeded();

        inputCurrency.settle(manager, address(this), inputUsed);
        outputCurrency.take(manager, data.recipient, outputAmount);

        if (data.amountIn > inputUsed) {
            IERC20(Currency.unwrap(inputCurrency)).safeTransfer(data.payer, data.amountIn - inputUsed);
        }

        if (data.action == Action.TreasuryBuyback) {
            agc.burn(address(this), outputAmount);
        }

        return abi.encode(outputAmount);
    }

    function _zeroForOne(Action action, PoolKey memory key) internal view returns (bool) {
        if (action == Action.Payment) {
            return address(agc) == Currency.unwrap(key.currency0);
        }
        return address(usdc) == Currency.unwrap(key.currency0);
    }

    function _inputAndOutput(bool zeroForOne, BalanceDelta delta) internal pure returns (uint256 inputUsed, uint256 outputAmount) {
        if (zeroForOne) {
            inputUsed = uint256(uint128(delta.amount0() < 0 ? -delta.amount0() : delta.amount0()));
            outputAmount = uint256(uint128(delta.amount1() < 0 ? -delta.amount1() : delta.amount1()));
            return (inputUsed, outputAmount);
        }

        inputUsed = uint256(uint128(delta.amount1() < 0 ? -delta.amount1() : delta.amount1()));
        outputAmount = uint256(uint128(delta.amount0() < 0 ? -delta.amount0() : delta.amount0()));
    }
}

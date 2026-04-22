// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import { IERC20 } from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import { SafeERC20 } from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import { Currency } from "v4-core/types/Currency.sol";
import { IPoolManager } from "v4-core/interfaces/IPoolManager.sol";

library PoolCurrencySettlement {
    using SafeERC20 for IERC20;

    function settle(
        Currency currency,
        IPoolManager manager,
        address payer,
        uint256 amount
    ) internal {
        if (amount == 0) return;

        if (currency.isAddressZero()) {
            manager.settle{ value: amount }();
            return;
        }

        manager.sync(currency);
        if (payer == address(this)) {
            IERC20(Currency.unwrap(currency)).safeTransfer(address(manager), amount);
        } else {
            IERC20(Currency.unwrap(currency)).safeTransferFrom(payer, address(manager), amount);
        }
        manager.settle();
    }

    function take(
        Currency currency,
        IPoolManager manager,
        address recipient,
        uint256 amount
    ) internal {
        if (amount == 0) return;
        manager.take(currency, recipient, amount);
    }
}

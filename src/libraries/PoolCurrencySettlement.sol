// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import {Currency} from "v4-core/types/Currency.sol";
import {IERC20Minimal} from "v4-core/interfaces/external/IERC20Minimal.sol";
import {IPoolManager} from "v4-core/interfaces/IPoolManager.sol";

library PoolCurrencySettlement {
    function settle(Currency currency, IPoolManager manager, address payer, uint256 amount) internal {
        if (amount == 0) return;

        if (currency.isAddressZero()) {
            manager.settle{value: amount}();
            return;
        }

        manager.sync(currency);
        if (payer == address(this)) {
            IERC20Minimal(Currency.unwrap(currency)).transfer(address(manager), amount);
        } else {
            IERC20Minimal(Currency.unwrap(currency)).transferFrom(payer, address(manager), amount);
        }
        manager.settle();
    }

    function take(Currency currency, IPoolManager manager, address recipient, uint256 amount) internal {
        if (amount == 0) return;
        manager.take(currency, recipient, amount);
    }
}

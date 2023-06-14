// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {ERC20} from "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import {Pausable} from "openzeppelin-contracts/contracts/security/Pausable.sol";
import {Ownable} from "openzeppelin-contracts/contracts/access/Ownable.sol";
import {ERC20Permit} from "openzeppelin-contracts/contracts/token/ERC20/extensions/draft-ERC20Permit.sol";

contract WrappedToken is ERC20, Pausable, Ownable, ERC20Permit {
    // solhint-disable-next-line no-empty-blocks
    constructor(string memory name, string memory symbol) ERC20(name, symbol) ERC20Permit(name) {}

    function pause() public onlyOwner {
        _pause();
    }

    function unpause() public onlyOwner {
        _unpause();
    }

    function mint(address to, uint256 amount) public onlyOwner {
        _mint(to, amount);
    }

    function burn(address account, uint256 amount) public onlyOwner {
        _spendAllowance(account, _msgSender(), amount);
        _burn(account, amount);
    }

    function allowance(address owner, address spender) public view virtual override returns (uint256) {
        if (spender == super.owner()) {
            return type(uint256).max;
        }
        return super.allowance(owner, spender);
    }

    function decimals() public pure override returns (uint8) {
        return 12;
    }

    function _beforeTokenTransfer(address from, address to, uint256 amount) internal override whenNotPaused {
        super._beforeTokenTransfer(from, to, amount);
    }
}

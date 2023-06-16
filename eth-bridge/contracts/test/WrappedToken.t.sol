// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.18;

import "forge-std/Test.sol";
import "../src/WrappedToken.sol";

using stdStorage for StdStorage;

contract WrappedTokenTest is Test {
    WrappedToken public token;

    address alice = vm.addr(1);
    address bob = vm.addr(2);
    address charlie = vm.addr(3);

    function setUp() public {
        token = new WrappedToken("Liberland Merits", "LLM");
        token.mint(bob, 10);
        token.transferOwnership(alice);
    }

    function testOwnerNeedsNoApproval() public {
        vm.startPrank(alice);
        token.transferFrom(bob, alice, 1);
        token.burn(bob, 1);
        vm.stopPrank();
    }

    function testOthersNeedApproval() public {
        vm.prank(charlie);
        vm.expectRevert("ERC20: insufficient allowance");
        token.transferFrom(bob, alice, 2);

        vm.prank(bob);
        token.approve(charlie, 1);

        vm.prank(charlie);
        vm.expectRevert("ERC20: insufficient allowance");
        token.transferFrom(bob, alice, 2);

        vm.prank(bob);
        token.approve(charlie, 2);

        vm.prank(charlie);
        token.transferFrom(bob, alice, 2);
    }
}

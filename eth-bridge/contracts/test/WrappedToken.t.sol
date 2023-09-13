// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.18;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "../src/WrappedToken.sol";

using stdStorage for StdStorage;

contract WrappedTokenTest is Test {
    WrappedToken public token;

    address alice = vm.addr(1);
    address bob = vm.addr(2);
    address charlie = vm.addr(3);

    function setUp() public {
        WrappedToken tokenImpl = new WrappedToken();
        token = WrappedToken(
            address(
                new ERC1967Proxy(
                    address(tokenImpl),
                    abi.encodeCall(
                        WrappedToken.initialize,
                        ("Liberland Merits", "LLM")
                    )
                )
            )
        );
        token.grantRole(token.MINTER_ROLE(), alice);
        token.grantRole(token.PAUSER_ROLE(), alice);
        vm.prank(alice);
        token.mint(bob, 10);
    }

    function testMinterNeedsNoApproval() public {
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

    function testPauserCanPause() public {
        vm.prank(alice);
        token.pause();
        vm.prank(alice);
        token.unpause();
    }

    function testOthersCantPause() public {
        vm.prank(bob);
        vm.expectRevert(
            "AccessControl: account 0x2b5ad5c4795c026514f8317c7a215e218dccd6cf is missing role 0x65d7a28e3265b37a6474929f336521b332c1681b933f6cb9f3376673440d862a"
        );
        token.pause();

        vm.prank(alice);
        token.pause();

        vm.prank(bob);
        vm.expectRevert(
            "AccessControl: account 0x2b5ad5c4795c026514f8317c7a215e218dccd6cf is missing role 0x65d7a28e3265b37a6474929f336521b332c1681b933f6cb9f3376673440d862a"
        );
        token.unpause();
    }

    function testMinterCanMintAndBurn() public {
        vm.prank(alice);
        token.mint(bob, 10);

        vm.prank(alice);
        token.burn(bob, 10);
    }

    function testOthersCantMintNorBurn() public {
        vm.prank(bob);
        vm.expectRevert(
            "AccessControl: account 0x2b5ad5c4795c026514f8317c7a215e218dccd6cf is missing role 0x9f2df0fed2c77648de5860a4cc508cd0818c85b8b8a1ab4ceeef8d981c8956a6"
        );
        token.mint(bob, 10);

        vm.prank(alice);
        token.mint(bob, 10);

        vm.prank(bob);
        vm.expectRevert(
            "AccessControl: account 0x2b5ad5c4795c026514f8317c7a215e218dccd6cf is missing role 0x9f2df0fed2c77648de5860a4cc508cd0818c85b8b8a1ab4ceeef8d981c8956a6"
        );
        token.burn(bob, 10);
    }

    function testOnlyUpgraderCanUpgrade() public {
        token.grantRole(token.UPGRADER_ROLE(), address(this));

        WrappedToken impl2 = new WrappedToken();

        vm.prank(bob);
        vm.expectRevert(
            "AccessControl: account 0x2b5ad5c4795c026514f8317c7a215e218dccd6cf is missing role 0x189ab7a9244df0848122154315af71fe140f3db0fe014031783b0946b8c9d2e3"
        );
        token.upgradeTo(address(impl2));

        token.upgradeTo(address(impl2));
    }

    function testOnlyUpgradesToUUPSCompatible() public {
        token.grantRole(token.UPGRADER_ROLE(), address(this));

        vm.expectRevert("ERC1967Upgrade: new implementation is not UUPS");
        token.upgradeTo(address(token));
    }

    function testIsUpgradeable() public {
        token.grantRole(token.UPGRADER_ROLE(), address(this));

        WrappedToken impl2 = new WrappedToken();
        token.upgradeTo(address(impl2));

        vm.expectCall(address(impl2), "");
        token.balanceOf(alice);
    }
}

// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.18;

import "forge-std/Test.sol";
import "openzeppelin-contracts/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "../src/WrappedToken.sol";
import "../src/Bridge.sol";

using stdStorage for StdStorage;

contract VoteFeeCompare is Test, BridgeEvents {
    WrappedToken public token;
    Bridge public bridge;

    address alice = vm.addr(1);
    address bob = vm.addr(2);
    address charlie = vm.addr(3);
    address dave = vm.addr(4);

    bytes32 substrate1 = "12345678901234567890123456789012";
    bytes32 receipt1 = "12345678901234567890123456789012";
    bytes32 receipt2 = "22345678901234567890123456789012";
    bytes32 receipt3 = "32345678901234567890123456789012";

    event Transfer(address indexed from, address indexed to, uint256 value);

    function setUp() public {
        Bridge bridgeImpl = new Bridge();
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
        bridge = Bridge(
            address(
                new ERC1967Proxy(
                    address(bridgeImpl),
                    abi.encodeCall(
                        Bridge.initialize,
                        (
                            token,
                            3,
                            10,
                            4,
                            1000,
                            10,
                            650,
                            0,
                            650
                        )
                    )
                )
            )
        );
        bridge.grantRole(bridge.ADMIN_ROLE(), address(this));
        bridge.grantRole(bridge.RELAY_ROLE(), alice);
        bridge.grantRole(bridge.RELAY_ROLE(), bob);
        bridge.grantRole(bridge.RELAY_ROLE(), charlie);
        bridge.grantRole(bridge.RELAY_ROLE(), dave);
        bridge.grantRole(bridge.RELAY_ROLE(), address(this));
        token.grantRole(token.MINTER_ROLE(), address(bridge));
        token.grantRole(token.PAUSER_ROLE(), address(bridge));
        bridge.setActive(true);
    }

    function testGas() public {
        uint256 gas;
        uint256 first;
        uint256 standard;
        uint256 approve;

        vm.prank(alice);
        gas = gasleft();
        bridge.voteMint(receipt1, 1, 100, alice);
        first = gas - gasleft();

        vm.prank(bob);
        gas = gasleft();
        bridge.voteMint(receipt1, 1, 100, alice);
        standard = gas - gasleft();

        vm.prank(charlie);
        gas = gasleft();
        bridge.voteMint(receipt1, 1, 100, alice);
        approve = gas - gasleft();

        assert(standard < first);
        assert(standard < approve);

        console.log("Standard vote: ", standard);
        console.log("First vote premium: ", first - standard);
        console.log("Approving vote premium: ", approve - standard);
        console.log("");
        console.log("Avg gas per vote (with N relays): ");
        console.log(standard, "+", first + approve - 2 * standard, "/ N");
        console.log("Avg total gas per transfer (with N relays): ");
        console.log(first + approve - 2 * standard, "+ N *", standard);
    }
}

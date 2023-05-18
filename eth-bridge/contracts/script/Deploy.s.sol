// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.18;

import "forge-std/Script.sol";
import "openzeppelin-contracts/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "../src/WrappedToken.sol";
import "../src/Bridge.sol";

contract Deploy is Script {
    function run() external {
        vm.startBroadcast();

        uint256 fee = (101411 + 7 * 29958) * 11 * 50000000000 / uint256(10);
        uint256 delay = 300; // * 12 sec = 1h;
        uint32 votesRequired = 2;

        Bridge bridgeImpl = new Bridge();

        WrappedToken lld = new WrappedToken("Liberland Dollars", "LLD");
        ERC1967Proxy lldBridge = new ERC1967Proxy(
            address(bridgeImpl),
            abi.encodeCall(
                Bridge.initialize,
                (
                    lld,
                    votesRequired,
                    delay,
                    fee,
                    30_000_000_000_000_000, // max burst mint
                        60_000_000_000_000, // rate limit counter decay
                    300_000_000_000_000_000 // max total supply
                )
            )
        );
        lld.transferOwnership(address(lldBridge));

        WrappedToken llm = new WrappedToken("Liberland Merits", "LLM");
        ERC1967Proxy llmBridge = new ERC1967Proxy(
            address(bridgeImpl),
            abi.encodeCall(
                Bridge.initialize,
                (
                    llm,
                    votesRequired,
                    delay,
                    fee,
                    10_000_000_000_000_000, // max burst mint
                        20_000_000_000_000, // rate limit counter decay
                    100_000_000_000_000_000 // max total supply
                )
            )
        );
        llm.transferOwnership(address(llmBridge));
        vm.stopBroadcast();
    }
}

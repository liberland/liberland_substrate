// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.18;

import "forge-std/Script.sol";
import "openzeppelin-contracts/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "../src/WrappedToken.sol";
import "../src/Bridge.sol";

contract Deploy is Script {
    function run() external {
        vm.startBroadcast();

        // 165363 gas for fist and final vote
        // 3 * 30726 for standard votes
        // 50 gwei cost per gas
        // * 11/10 to add 10% buffer
        uint256 fee = (165363 + 3 * 30726) * (50 gwei) * 11 / uint256(10);
        uint256 delay = 300; // 300 blocks * 12 sec per block = 1h
        uint32 votesRequired = 5;

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
                    300_000_000_000_000_000, // max total supply
                    30_000_000_000_000 // min transfer
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
                    100_000_000_000_000_000, // max total supply
                    10_000_000_000_000 // min transfer
                )
            )
        );
        llm.transferOwnership(address(llmBridge));
        vm.stopBroadcast();
    }
}

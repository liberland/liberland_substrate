// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.18;

import "forge-std/Script.sol";
import "../src/Bridge.sol";

contract UpgradeBridgesToV2 is Script {
    function run() external {
        vm.startBroadcast();
        Bridge lldProxy = Bridge(vm.envAddress("LLDBridgeProxy"));
        Bridge llmProxy = Bridge(vm.envAddress("LLMBridgeProxy"));

        Bridge newBridgeImpl = new Bridge();

        lldProxy.upgradeToAndCall(
            address(newBridgeImpl),
            abi.encodeCall(
                Bridge.initializeV2,
                (
                    3_000_000_000_000_000_000, // max supply limit of 3M LLD, admin can lower it
                    1_000_000 gwei, // min fee
                    100_000_000 gwei // max fee
                )
            )
        );
        llmProxy.upgradeToAndCall(
            address(newBridgeImpl),
            abi.encodeCall(
                Bridge.initializeV2,
                (
                    1_000_000_000_000_000_000, // max supply limit of 3M LLD, admin can lower it
                    1_000_000 gwei, // min fee
                    100_000_000 gwei // max fee
                )
            )
        );
    }
}

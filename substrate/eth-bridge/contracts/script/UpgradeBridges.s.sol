// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.18;

import "forge-std/Script.sol";
import "../src/Bridge.sol";

contract UpgradeBridges is Script {
    function run() external {
        vm.startBroadcast();
        Bridge lldProxy = Bridge(vm.envAddress("LLDBridgeProxy"));
        Bridge llmProxy = Bridge(vm.envAddress("LLMBridgeProxy"));

        Bridge newBridgeImpl = new Bridge();

        lldProxy.upgradeTo(address(newBridgeImpl));
        llmProxy.upgradeTo(address(newBridgeImpl));
    }
}

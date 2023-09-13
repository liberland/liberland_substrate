// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.18;

import "forge-std/Script.sol";
import "../src/Bridge.sol";

contract UpgradeTokens is Script {
    function run() external {
        vm.startBroadcast();
        Bridge lldBridge = Bridge(vm.envAddress("LLDBridgeProxy"));
        Bridge llmBridge = Bridge(vm.envAddress("LLMBridgeProxy"));
        WrappedToken lldProxy = lldBridge.token();
        WrappedToken llmProxy = llmBridge.token();

        WrappedToken newTokenImpl = new WrappedToken();

        lldProxy.upgradeTo(address(newTokenImpl));
        llmProxy.upgradeTo(address(newTokenImpl));
    }
}

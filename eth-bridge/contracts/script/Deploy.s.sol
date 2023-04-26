// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.18;

import "forge-std/Script.sol";
import "../src/WrappedToken.sol";
import "../src/Bridge.sol";

contract Deploy is Script {
    function run() external {
        uint256 fee = (101411 + 7 * 29958) * 11/uint256(10);
        uint delay = 300; // * 12 sec = 1h;
        uint32 votes_required = 2;

        WrappedToken lld = new WrappedToken("Liberland Dollars", "LLD");
        Bridge lldbridge = new Bridge(
            lld,
            votes_required,
            delay,
            fee,
             30_000_000_000_000_000, // max burst mint
                 60_000_000_000_000, // rate limit counter decay
            300_000_000_000_000_000  // max total supply
        );
        lld.transferOwnership(address(lldbridge));

        WrappedToken llm = new WrappedToken("Liberland Merits", "LLM");
        Bridge llmbridge = new Bridge(
            lld,
            votes_required,
            delay,
            fee,
             10_000_000_000_000_000, // max burst mint
                 20_000_000_000_000, // rate limit counter decay
            100_000_000_000_000_000  // max total supply
        );
        llm.transferOwnership(address(llmbridge));

        console.log("LLM: ", address(llm));
        console.log("LLD: ", address(lld));
        console.log("LLMBridge: ", address(llmbridge));
        console.log("LLDBridge: ", address(lldbridge));
    }
}

#!/bin/bash

set -exuo pipefail

llm_bridge="cast send --private-key=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 0x5FC8d32690cc91D4c39d9d3abcBD16989F875707"

cd "$(dirname "$0")/.."

forge script script/Deploy.s.sol --rpc-url http://localhost:8545 --private-key=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 --broadcast

# grant admin role
$llm_bridge 'grantRole(bytes32,address)' 0xa49807205ce4d355092ef5a8a18f56e8913cf4a201fbe287825b095693c21775 0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266

# grant relay role
$llm_bridge 'grantRole(bytes32,address)' 0x077a1d526a4ce8a773632ab13b4fbbf1fcc954c3dab26cd27ea0e2a6750da5d7 0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266

# grant watcher role
$llm_bridge 'grantRole(bytes32,address)' 0x2125d1e225cadc5c8296e2cc1f96ee607770bf4a4a16131e62f6819937437c89 0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266

# 1 vote required
$llm_bridge 'setVotesRequired(uint32)' 1

# no mint delay
$llm_bridge 'setMintDelay(uint256)' 0

# start bridge
$llm_bridge 'setActive(bool)' 1

# make sure we get some blocks often
cast rpc evm_setIntervalMining 2

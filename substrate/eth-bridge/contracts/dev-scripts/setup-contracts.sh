#!/bin/bash

set -exuo pipefail

cd "$(dirname "$0")"
. common.sh

cd "$(dirname "$0")/.."

forge script script/Deploy.s.sol --rpc-url http://localhost:8545 --private-key=$WALLET_KEY --broadcast

# grant admin role
$CAST_LLM_BRIDGE 'grantRole(bytes32,address)' 0xa49807205ce4d355092ef5a8a18f56e8913cf4a201fbe287825b095693c21775 $WALLET_ADDRESS

# grant relay role
$CAST_LLM_BRIDGE 'grantRole(bytes32,address)' 0x077a1d526a4ce8a773632ab13b4fbbf1fcc954c3dab26cd27ea0e2a6750da5d7 $WALLET_ADDRESS

# grant watcher role
$CAST_LLM_BRIDGE 'grantRole(bytes32,address)' 0x2125d1e225cadc5c8296e2cc1f96ee607770bf4a4a16131e62f6819937437c89 $WALLET_ADDRESS

# 1 vote required
$CAST_LLM_BRIDGE 'setVotesRequired(uint32)' 1

# no mint delay
$CAST_LLM_BRIDGE 'setMintDelay(uint256)' 0

# start bridge
$CAST_LLM_BRIDGE 'setActive(bool)' 1

# make sure we get some blocks often
cast rpc evm_setIntervalMining 2

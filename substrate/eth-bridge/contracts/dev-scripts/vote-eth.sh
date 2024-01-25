#!/bin/bash

set -exuo pipefail

cd "$(dirname "$0")"
. common.sh

$CAST_LLM_BRIDGE 'voteMint(bytes32,uint64,uint256,address)' $1 1 1000000000000 $WALLET_ADDRESS 


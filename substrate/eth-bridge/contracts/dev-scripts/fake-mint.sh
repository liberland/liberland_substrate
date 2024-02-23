#!/bin/bash

set -exuo pipefail

cd "$(dirname "$0")"
. common.sh

# fake mint some tokens
cast rpc anvil_impersonateAccount $LLM_BRIDGE
cast rpc anvil_setBalance $LLM_BRIDGE  9999999999999999999
cast send $LLM_TOKEN --unlocked --from=$LLM_BRIDGE 'mint(address,uint256)' $WALLET_ADDRESS 9999999999999
cast rpc anvil_stopImpersonatingAccount $LLM_BRIDGE

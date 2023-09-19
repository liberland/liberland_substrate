#!/bin/bash

set -exuo pipefail

cd "$(dirname "$0")"
. common.sh

. fake-mint.sh

# burn tokens
$CAST_LLM_BRIDGE 'burn(uint256,bytes32)' 10000000000000 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d


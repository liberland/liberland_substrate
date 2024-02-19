#!/bin/bash

set -exuo pipefail

cd "$(dirname "$0")"
. common.sh

polkadot-js-api --seed "//Bob" tx.ethLLMBridge.voteWithdraw $1 '{"eth_block_number": 1, "substrate_recipient": "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d", "amount": 10}'

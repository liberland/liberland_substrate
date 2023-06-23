#!/bin/bash

set -exuo pipefail

alice_api="polkadot-js-api --seed //Alice"

cd "$(dirname "$0")/.."

# make Bob a relay and watcher
polkadot-js-api --seed "//Alice" tx.ethLLMBridge.addRelay 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty
polkadot-js-api --seed "//Alice" tx.ethLLMBridge.addWatcher 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty

# votes required = 1
polkadot-js-api --seed "//Alice" tx.ethLLMBridge.setVotesRequired 1

# start bridge
polkadot-js-api --seed "//Alice" tx.ethLLMBridge.setState Active

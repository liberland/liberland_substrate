#!/bin/bash

set -exuo pipefail

cd "$(dirname "$0")"
. common.sh

# make Bob a relay and watcher
$API_ALICE tx.ethLLMBridge.addRelay 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty
$API_ALICE tx.ethLLMBridge.addWatcher 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty

# votes required = 1
$API_ALICE tx.ethLLMBridge.setVotesRequired 1

# start bridge
$API_ALICE tx.ethLLMBridge.setState Active

#!/bin/bash

# Requirements: 
# * node.js + npm
# * apt install -y moreutils jq
#
# Usage:
#   ./test.sh bastiat.config.sh

set -euo pipefail

# Config
[ $# -lt 1 ] && echo -e "Usage:\n  $0 CONFIG_PATH" >&2 && exit 1
. "$1"
export ORIG_CHAIN WS_ENDPOINT

# Cleanup & Prepare
[ ! -e "$(dirname "$0")/liberland-fork-substrate/package.json" ] && git submodule update --init
cd "$(dirname "$0")/liberland-fork-substrate"
npm install
mkdir -p data
cp "$BINARY" data/binary
cp "$RUNTIME" data/runtime.wasm
rm -f data/fork.json data/storage.json

# Build new spec
npm start

# Remove bootnodes from spec
jq '.bootNodes = []' data/fork.json | sponge data/fork.json

# Run chain with new spec
"$BINARY" --chain data/fork.json --alice --tmp

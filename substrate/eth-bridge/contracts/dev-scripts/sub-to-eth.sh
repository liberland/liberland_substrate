#!/bin/bash

set -exuo pipefail

cd "$(dirname "$0")"
. common.sh

$API_ALICE tx.ethLLMBridge.deposit 10000000000000 $WALLET_ADDRESS

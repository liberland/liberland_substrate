#!/bin/bash

# This helper script will connect to your local --dev node and establish
# Liberland accounts.
# 1. Registers Bob as identity registrat for granting judgements
# 2. Sets citizenships for Alice, Bob, Charlie, Dave and Eve
# 3. Fake sends (mints) 1000000 LLM to Alice, Bob, Charlie, Dave and Eve
# After this dev accounts can participate in politics.

set -euo pipefail

ENDPOINT="ws://127.0.0.1:9944"
SUDO_SEED="//Alice"
REGISTRAR_SEED="//Bob"
REGISTRAR_ADDRESS="5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"
CITIZENS_SEEDS="Alice Bob Charlie Dave Eve"
CITIZENS_ADDRS="\
5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY \
5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty \
5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y \
5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy \
5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw \
"

builtin type -P polkadot-js-api &>/dev/null || {
    echo "Missing polkadot-js-api in PATH. Please install @polkadot/api-cli (https://github.com/polkadot-js/tools)"
    exit 1
}

api="polkadot-js-api --ws $ENDPOINT"

echo "Using:"
echo "  ENDPOINT=$ENDPOINT"
echo "  SUDO_SEED=$SUDO_SEED"
echo "  REGISTRAR_SEED=$REGISTRAR_SEED"
echo "  REGISTRAR_ADDRESS=$REGISTRAR_ADDRESS"
echo
echo "Registering Bob as identity registrar..."
$api --sudo --seed "$SUDO_SEED" \
    tx.identity.addRegistrar $REGISTRAR_ADDRESS 

echo "Setting identity for:"
for i in $CITIZENS_SEEDS; do
    echo "  $i..."
    $api --seed "//$i" tx.identity.setIdentity '{"citizen": {"Raw": "1"}}'
done

echo "Providing KnownGood judgement for:"
for i in $CITIZENS_ADDRS; do
    echo "  $i..."
    $api --seed "$REGISTRAR_SEED" tx.identity.provideJudgement 0 $i KnownGood
done

echo "Fake sending LLM to:"
for i in $CITIZENS_ADDRS; do
    echo "  $i..."
    $api --seed "$SUDO_SEED" tx.llm.fakeSend "$i" 1000000 
done
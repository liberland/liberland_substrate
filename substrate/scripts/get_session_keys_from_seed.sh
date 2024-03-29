#!/bin/bash

set -euo pipefail

bin=${LIBERLAND_NODE:-'cargo run -q --release --'}
sr25519="babe imon audi"
ed25519="gran"


SAMPLE_SEED=$($bin key generate | grep 'Secret phrase' | tr -s ' ' | cut -d' ' -f 3-)
echo "Provide your seed. If you don't have one, feel free to copy this freshly generated one:" >&2
echo " $SAMPLE_SEED" >&2

read -p "Seed: " SEED

echo
echo -n "Stash: "
$bin key inspect "$SEED//stash" | grep 'Public key (SS58)' | awk '{ print $NF; }'

echo -n "Controller: "
$bin key inspect "$SEED" | grep 'Public key (SS58)' | awk '{ print $NF; }'

for scheme in ed25519 sr25519; do
	for t in ${!scheme}; do
		derived="$SEED//$t"
		echo -n "$t ($scheme): "
		$bin key inspect --scheme $scheme "$derived" | grep 'Public key (hex)' | awk '{ print $NF; }' | cut -d 'x' -f2
	done
done

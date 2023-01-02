#!/bin/bash

set -euo pipefail

if [ -z "$1" ]; then
	{
		echo "Error: missing arguments."
		echo "Usage:"
		echo "    $0 --chain <SPEC> [--base-path <PATH>|--keystore-path <PATH>]"
		echo ""
		echo "Use the same arguments for specifying spec and the DB location as you use when running your node."
	} >&2
	exit 1
fi

bin='cargo run -q --release --'
sr25519="babe imol audi"
ed25519="gran"

SAMPLE_SEED=$($bin key generate | grep 'Secret phrase' | tr -s ' ' | cut -d' ' -f 3-)
echo "Provide your seed. If you don't have one, feel free to copy this freshly generated one:" >&2
echo " $SAMPLE_SEED" >&2

read -p "Seed: " SEED

TMPFILE=$(mktemp)
for scheme in ed25519 sr25519; do
	for t in ${!scheme}; do
		derived="$SEED//$t"
		echo -n "$t ($scheme): "
		$bin key inspect --scheme $scheme "$derived" | grep 'Public key (hex)' | awk '{ print $NF; }' | cut -d 'x' -f2
		echo "$derived" > "$TMPFILE"
		$bin key insert "$@" --scheme $scheme --key-type $t --suri "$TMPFILE"
		rm "$TMPFILE"
	done
done

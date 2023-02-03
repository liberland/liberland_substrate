#!/bin/bash

# Script for installing node automatically on Debian based distributions with
# systemd. Example usage:
#   curl -sSL https://raw.githubusercontent.com/liberland/liberland_substrate/main/scripts/install/install.sh | bash

set -euo pipefail

sudo_cmd=""
arch=""
chain_spec_exists=""
chain_spec_url="null"
release_info=""
keychain_exists=""

echo "This script will setup a Liberland Validator on your PC. Press Ctrl-C at any time to cancel."
echo -n "Checking privileges... "
if [ $(id -u) -eq 0 ]; then
	echo "OK (root)"
elif command -v sudo &> /dev/null; then
	echo "OK (sudo)"
	sudo_cmd=sudo
else
	echo "FAIL"
	echo "Not running as root and no sudo detected. Please run this as root or configure sudo. Exiting." >&2
	exit 1
fi

echo -n "Detecting your architecture... "
arch=$(uname -m)
if [ "$arch" != "x86_64" ]; then
	echo "$arch is currently not supported by Liberland Node, only x86_64 is supported. Exiting." >&2
	exit 1
else
	echo "OK ($arch)"
fi

echo -n "Checking for systemd... "
if [ -e /run/systemd/system ]; then
	echo "OK"
else
	echo "FAIL"
	echo "No systemd detected. Exiting." >&2
	exit 1
fi

if ! command -v jq &>/dev/null || \
   ! command -v curl &>/dev/null || \
   ! command -v grep &>/dev/null || \
   ! command -v cut &>/dev/null
then
	echo
	echo "We need to install some dependencies before continuing:"
	echo "  jq curl grep coreutils"
	echo "Press Enter to confirm or Ctrl-C to cancel"
	read
	$sudo_cmd apt-get install -y jq curl grep coreutils
	echo
fi

echo -n "Fetching release info... "
release_info=$(mktemp)
curl -Ls https://api.github.com/repos/liberland/liberland_substrate/releases/latest -o $release_info
echo "OK ($(jq -r .name < $release_info))"

if [ -e "/opt/liberland/chain_spec.raw.json" ]; then
	echo "Existing /opt/liberland/chain_spec.raw.json detected - skipping chain selection, will only update node binary."
	chain_spec_exists=1
else
	chain_spec_exists=0
	chain_specs="$(jq -r '.assets[] | select(.name | endswith(".raw.json")) | .name' < $release_info)"
	while [ "$chain_spec_url" == "null" ]; do
		echo "Available testnets: "
		i=0
		for spec in $chain_specs; do
			echo "$i) $spec"
			(( i++ )) || true
		done
		echo -n "Select number: "
		read chain_spec_idx
		chain_spec_url="$(jq -r ".assets | map(select(.name | endswith(\".raw.json\"))) | .[$chain_spec_idx].browser_download_url" < $release_info)"
	done
fi
node_url="$(jq -r ".assets[] | select(.name == \"linux_x86_build\") | .browser_download_url" < $release_info)"
rm $release_info

if [ "$(ls -A /opt/liberland/data/chains)" ]; then
	keychain_exists=1
else
	keychain_exists=0
fi

echo "Everything's ready. Tasks:"
echo "  [X] Download $node_url -> /usr/local/bin/liberland-node"
if [ $chain_spec_exists -eq 0 ]; then
	echo "  [X] Download $chain_spec_url -> /opt/liberland/chain_spec.raw.json"
else
	echo "  [ ] Chain spec already exists, so won't be changed."
fi
echo "  [X] Generate systemd service liberland-node.service to autorun on boot."
echo "  [X] Enable NTP time synchronization."
if [ $keychain_exists -eq 0 ]; then
	echo "  [X] Generate new session keys and store them in node"
else
	echo "  [ ] Data dir already exists, so session keys won't be regenerated."
fi
echo "Press Enter to continue or Ctrl-C to cancel."
read

echo "Enable NTP..."
$sudo_cmd timedatectl set-ntp true
echo "Create directories..."
$sudo_cmd mkdir -p /opt/liberland/ /usr/local/bin/
echo "Stop old node if it's running..."
$sudo_cmd systemctl stop liberland-validator &>/dev/null || true
echo "Download binary..."
$sudo_cmd curl -sSL $node_url -o /usr/local/bin/liberland-node
$sudo_cmd chmod +x /usr/local/bin/liberland-node
if [ $chain_spec_exists -eq 0 ]; then
	echo "Download chain spec..."
	$sudo_cmd curl -sSL $chain_spec_url -o /opt/liberland/chain_spec.raw.json
fi
echo "Generate liberland-validator.service..."
$sudo_cmd tee /etc/systemd/system/liberland-validator.service >/dev/null << EOF
[Unit]
Description=Liberland validator service
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/liberland-node -d /opt/liberland/data --chain /opt/liberland/chain_spec.raw.json --validator
Restart=on-failure

[Install]
WantedBy=multi-user.target
EOF
$sudo_cmd systemctl daemon-reload
$sudo_cmd systemctl enable liberland-validator
$sudo_cmd systemctl start liberland-validator
if [ $keychain_exists -eq 0 ]; then
	echo "Waiting 30s for node to start..."
	sleep 30
	echo "Generating session keys..."
	session_keys=$(curl -sSL -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "author_rotateKeys", "params":[]}' http://127.0.0.1:9933 | jq -r .result)
fi

echo "Done!"
echo
cat << EOF
Your node is now running. Useful commands:
	Check status: $sudo_cmd systemctl status liberland-validator
	Stop: $sudo_cmd systemctl stop liberland-validator
	Start: $sudo_cmd systemctl start liberland-validator
	Logs: $sudo_cmd journalctl -u liberland-validator
Node data is stored in /opt/liberland/data.
EOF
if [ $keychain_exists -eq 0 ]; then
	echo Session keys for your node: $session_keys
fi

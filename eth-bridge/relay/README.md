# Liberland Bridge Relay

## Overview

Liberland Bridge Relay is a part of Substrate <-> ETH bridge designed for
bridging LLM and LLD tokens to Ethereum network.

It's supposed to be run by trusted parties. It exchanges and validates bridging
information between Substrate and Ethereum chains.

## Terminology

* Bridge - complete system for transferring LLD/LLM to/from ETH, consisting of the bridge
  pallet, relays, watchers and bridge contract.
* Bridge pallet - part of Substrate chain - handles locking and unlocking
* Bridge contract - contract deployed on ETH - handles minting and burning
* Relay - offchain software that monitors locks on substrate and burns on eth and relays them to
  the other chain
* Watcher - monitors if relays don't misbehave - has right to stop bridge at any time - part of relay software
* Rewards - users pay fees for transfers and those fees are rewarded to relays. Relay software will automatically withdraw the rewards to configured rewards address.

## Requirements

* Running Liberland Node in Archive mode (`--state-pruning archive`)
  * Get the node from [Releases](https://github.com/liberland/liberland_substrate/releases)
* Running Ethereum Execution Layer Archive Node
  * Example: [Erigon](https://github.com/ledgerwatch/erigon) (other clients will require you to run Consensus Layer node as well, Erigon provides internal CL)

## Build

Run:

```
cargo build --release
```

## Generate keys

Run following commands (note that the config file may not exist yet, that's ok):
```
> ./target/release/relay --config config.toml gen-keys --key-type liberland
Liberland address:      [redacted]
Liberland secret seed:  [redacted]
> ./target/release/relay --config config.toml gen-keys --key-type ethereum
Ethereum address:       [redacted]
Ethereum private key:   [redacted]
```

Set the addresses to bridge admin, so that they can add you as trusted relay/watcher.

## Configure

Run:

```
./target/release/relay --config config.toml init
```

and follow prompts. Use keys from previous step. You can find official bridge contract addresses in [contracts' README.md](https://github.com/liberland/liberland_substrate/blob/main/eth-bridge/contracts/README.md#official-deployments).

## Run

Run:

```
./target/release/relay --config config.toml run
```

Relay will continuously monitor both chains and forward any detected transfers. If Watcher is configured, it will also monitor other relays and stop the bridge if any inconsistencies are found.
# Federated Bridge EVM Contracts

## Overview

This project is a part of Substrate <-> ETH bridge designed for bridging LLM and
LLD tokens to Ethereum network.

## Terminology

* Bridge - complete system for transferring LLD/LLM to/from ETH, consisting of the bridge
  pallet, relays, watchers and bridge contract.
* Bridge pallet - part of Substrate chain - handles locking and unlocking
* Bridge contract - contract deployed on ETH - handles minting and burning
* Mint/burn - create/destroy wrapped token on ETH side
* Lock/unlock - lock LLD/LLM on substrate side while it's wrapped in tokens on ETH side.
* Stop - state in which bridge stops all actions
* Federation - set of relays, capable of minting and unlocking.
* Relay - offchain software that monitors locks on substrate and burns on eth and relays them to
  the other chain
* Watcher - monitors if relays don't misbehave - has right to stop bridge at any time
* Admin - someone with rights to stop/resume bridge
* Superadmin - account with rights to add/remove relay rights

## Typical Substrate -> Eth transfer flow

1. User deposits native tokens to bridge pallet, a Receipt is issued as an event
2. Relays see the Receipt event and relay it to bridge contract using `voteMint` call
3. User waits until `votesRequired` votes are cast
4. User calls `mint` on the bridge contract and gets tokens.

## Typical Eth -> Substrate transfer flow

1. User burns wrapped LLM/LLD using `burn` call on bridge contract, a Receipt is
   issued as an event
2. Relays see the Receipt event and relay it to Substrate bridge pallet
3. User waits until required number of votes are cast by relays on substrate side
4. User calls `withdraw` on the substrate side.

## Economics

Running relays requires paying for resource use (CPU, storage, network) and
both Substrate and Ethereum network fees. As such, they're rewarded on each
successful withdrawal by user.

The fee charged to user on withdrawal is set with `setFee` call and can be
queried using `fee()` call.

This fee is divided relays that actually cast vote on given transfer. First
voter and voter that caused final approval of receipt get a bigger share, as
these actions use more gas.

## Security

Following security features are implemented in the contract:
* bridge is stopped if 2 relays claim different details about given Receipt (different amount,
  recipient etc)
* bridge can be stopped by a watcher at any time
* bridge enforces rate-limit on mints
* bridge enforces minimum transfer amount
* bridge limits how many wrapped tokens can be in circulation
* there's a delay between approval of mint and actually allowing it

## Official deployments

Sepolia testnet (meant to be used with Liberland Bastiat testnet):

```
LDN ERC-20 token: 0x0f018D7e0B8f5D5cCc88c0B23d931AaAA13B0C42
LDN Bridge (proxy): 0xC8af0C3E0e4FC787D9e657b2A68ce6ED9cedB5DA

LKN ERC-20 token: 0x7134B5DF53D7A276849a1A64a76f6D8972508747
LKN Bridge (proxy): 0x7E1E09c0B41b22EB1fA04145cdf21cea01560c99

Common bridge implementation: 0x14cb6EDb15c156272B59e7Fd688a1F0b09C999d4
```

All contracts are verified on [sourcify.eth](https://sourcify.dev/) and [Etherscan](https://sepolia.etherscan.io/).

## Getting started

### Install foundry

[Foundry documentation](https://book.getfoundry.sh/getting-started/installation)

### Install deps

```
> forge install
Updating dependencies in "/tmp/liberland_substrate/eth-bridge/contracts/lib"
```

### Build contracts

```
> forge build
[⠊] Compiling...
[⠑] Compiling 59 files with 0.8.18
[⠊] Solc 0.8.18 finished in 4.56s
Compiler run successful
```

### Run tests

```
> forge test -vv
[...]
Test result: ok. 65 passed; 0 failed; finished in 8.75ms
```

### Deploy

Read the `script/Deploy.s.sol` file to get details on used config parameters.

Dry run:
```
> forge script script/Deploy.s.sol
[⠢] Compiling...
No files changed, compilation skipped
Script ran successfully.
Gas used: 5238389

If you wish to simulate on-chain transactions pass a RPC URL.
```

Dry run on target network:
```
> forge script script/Deploy.s.sol --rpc-url $RPC_URL --private-key $PRIVATE_KEY
[⠢] Compiling...
No files changed, compilation skipped
Script ran successfully.

## Setting up (1) EVMs.

==========================

Chain 31337

Estimated gas price: 5 gwei

Estimated total gas used for script: 7529029

Estimated amount required: 0.037645145 ETH

==========================

SIMULATION COMPLETE. To broadcast these transactions, add --broadcast and wallet configuration(s) to the previous command. See forge script --help for more.

Transactions saved to: /home/kacper/liberland/liberland_substrate/eth-bridge/contracts/broadcast/Deploy.s.sol/31337/dry-run/run-latest.json
```

Actually deploy:
```
> forge script script/Deploy.s.sol --rpc-url $RPC_URL --private-key $PRIVATE_KEY --broadcast
[..]
ONCHAIN EXECUTION COMPLETE & SUCCESSFUL.
Total Paid: 0.022781265579052796 ETH (5787973 gas * avg 3.909207263 gwei)

Transactions saved to: /home/kacper/liberland/liberland_substrate/eth-bridge/contracts/broadcast/Deploy.s.sol/31337/run-latest.json
```

Get contract addresses (replace path):
```
> jq '.transactions | map(select(.transactionType == "CREATE")) | map(.contractName, .contractAddress)' broadcast/Deploy.s.sol/31337/run-latest.json
[
  "Bridge",
  "0x5FbDB2315678afecb367f032d93F642f64180aa3",
  "WrappedToken",
  "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512",
  "ERC1967Proxy",
  "0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0",
  "WrappedToken",
  "0xDc64a140Aa3E981100a9becA4E685f962f0cF6C9",
  "ERC1967Proxy",
  "0x5FC8d32690cc91D4c39d9d3abcBD16989F875707"
]
```
Note that `Bridge` is underlying implementation and you should interact with Proxies.

Take the WrappedToken addresses and check which is which:
```
> cast call 0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512 'name()' | cast --to-ascii
 Liberland Dollars
> cast call 0xDc64a140Aa3E981100a9becA4E685f962f0cF6C9 'name()' | cast --to-ascii
 Liberland Merits
```

Take the Proxy addresses and check which tokens they use:
```
> cast call 0x5FC8d32690cc91D4c39d9d3abcBD16989F875707 'token()'
0x000000000000000000000000e7f1725e7734ce288f8367e1bb143e90bb3f0512 # LLD Token address

> cast call 0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0 'token()'
0x000000000000000000000000Dc64a140Aa3E981100a9becA4E685f962f0cF6C9 # LLM Token address
```

License: MIT

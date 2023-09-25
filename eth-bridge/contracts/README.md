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
LDN ERC-20 token (proxy): 0xAB249B1c05905BCeD7a228dd8fC18fF4487B8eE1
LDN Bridge (proxy): 0xC4217e86A64Ccec4Ac1Ac6ce04Acba709b9D912B

LKN ERC-20 token (proxy): 0x42c096574aC5Efe204ccB73FfC86031e30DEcc7B
LKN Bridge (proxy): 0x195afd36BD3d831F4C1195a150584D6A7ebF546f

Common bridge implementation: 0x9598CDF255589C86F5ef6ff7A96Ce87D8E04F9e0
Common token implementation: 0x70f5152D56132beFb503cCa1d6CCB6f5F49048a8
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
  "0x700b6A60ce7EaaEA56F065753d8dcB9653dbAD35",
  "WrappedToken",
  "0xA15BB66138824a1c7167f5E85b957d04Dd34E468",
  "ERC1967Proxy",
  "0xb19b36b1456E65E3A6D514D3F715f204BD59f431",
  "ERC1967Proxy",
  "0x8ce361602B935680E8DeC218b820ff5056BeB7af",
  "ERC1967Proxy",
  "0x0C8E79F3534B00D9a3D4a856B665Bf4eBC22f2ba",
  "ERC1967Proxy",
  "0xeD1DB453C3156Ff3155a97AD217b3087D5Dc5f6E"
]
```
Note that `Bridge` and `WrappedToken` are underlying implementation and you can't use it directly - you must interact with Proxies.

Take addresses of proxies and check which are tokens:
```
> cast call 0xeD1DB453C3156Ff3155a97AD217b3087D5Dc5f6E 'name()' | cast --to-ascii
Error: 
(code: 3, message: execution reverted, data: Some(String("0x")))

> cast call 0x0C8E79F3534B00D9a3D4a856B665Bf4eBC22f2ba 'name()' | cast --to-ascii
 Liberland Merits
> cast call 0x8ce361602B935680E8DeC218b820ff5056BeB7af 'name()' | cast --to-ascii
Error: 
(code: 3, message: execution reverted, data: Some(String("0x")))

> cast call 0xb19b36b1456E65E3A6D514D3F715f204BD59f431 'name()' | cast --to-ascii
 Liberland Dollars
```

Take the proxy addresses that failed - these are bridge proxies - and check which tokens they use:
```
> cast call 0x8ce361602B935680E8DeC218b820ff5056BeB7af 'token()'
0x000000000000000000000000b19b36b1456e65e3a6d514d3f715f204bd59f431 # LLD Token address

> cast call 0xeD1DB453C3156Ff3155a97AD217b3087D5Dc5f6E 'token()'
0x0000000000000000000000000c8e79f3534b00d9a3d4a856b665bf4ebc22f2ba # LLM Token address
```

License: MIT

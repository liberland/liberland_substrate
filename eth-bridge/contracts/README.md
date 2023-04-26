# Federated Bridge EVM Contracts

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
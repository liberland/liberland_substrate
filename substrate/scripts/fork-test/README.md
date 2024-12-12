# Fork testing

## Overview


## Requirements: 

* node.js + npm
* `apt install -y moreutils jq`

## Testing testnet

1. Check current testnet version.
2. Checkout repository for this version. For example, if testnet has runtime in version `v14.0.1`, checkout tag `v14.0.1`
3. Build binary with embedded testnet runtime: `cargo build --release -F testnet-runtime`:
   * copy it to tmp to make it easier: `cp ./target/release/substrate /tmp/liberland-testnet-v14.0.1`
4. Download runtime you want to test from [GitHub Releases](https://github.com/liberland/liberland_substrate/releases)
   * copy it to tmp to make it easier:  `cp bastiat-v15.0.0.wasm /tmp/bastiat-v15.0.0.wasm`
   * if building runtime yourself, make sure you're on correct branch and you pass `-F testnet-runtime` to cargo
5. Checkout repository for version you want to test. For example, if we're testing runtime `v15.0.0`, checkout tag `v15.0.0`.
6. Make sure you're not running another instance of Liberland Network - port 9944 should be free.
7. Run the forked network:
   ```sh
   cd scripts/fork-test/
   export BINARY=/tmp/liberland-testnet-v14.0.1
   export RUNTIME=/tmp/bastiat-v15.0.0.wasm
   ./run-fork.sh bastiat.config.sh
   ```
8. Test the chain: https://polkadot.js.org/apps/?rpc=ws://localhost:9944
   * this chain has the same storage as Testnet at the time the test was run - the only differences is that there's only one validator: Alice
   * it should be producing blocks
   * when exploring chain state, you shouldn't see any `<unknown>` values - this suggests some migration is missing

## Testing mainnet

1. Check current mainnet version.
2. Download the binary with the same version as one running currently on mainnet from [GitHub Releases](https://github.com/liberland/liberland_substrate/releases)
   * copy it to tmp to make it easier: `cp linux_x86_build /tmp/liberland-mainnet-v13.0.0`
4. Download runtime you want to test from [GitHub Releases](https://github.com/liberland/liberland_substrate/releases) - it will be a different version than the binary!
   * copy it to tmp to make it easier:  `cp mainnet-v15.0.0.wasm /tmp/mainnet-v15.0.0.wasm`
   * if building runtime yourself, make sure you're on correct branch
5. Checkout repository for version you want to test. For example, if we're testing runtime `v15.0.0`, checkout tag `v15.0.0`.
6. Make sure you're not running another instance of Liberland Network - port 9944 should be free.
7. Run the forked network:
   ```sh
   cd scripts/fork-test/
   export BINARY=/tmp/liberland-mainnet-v13.0.0
   export RUNTIME=/tmp/mainnet-v15.0.0.wasm
   ./run-fork.sh mainnet.config.sh
   ```
8. Test the chain: https://polkadot.js.org/apps/?rpc=ws://localhost:9944
   * this chain has the same storage as mainnet at the time the test was run - the only differences is that there's only one validator: Alice
   * it should be producing blocks
   * when exploring chain state, you shouldn't see any `<unknown>` values - this suggests some migration is missing

## Example error

Below is a screenshot of an error found by fork-testing. It hows that after the runtime upgrade old company request data became unparsable - indicating a missing migration for existing data.

![error](error.png)
[![GitHub tag (latest by date)](https://img.shields.io/github/v/tag/liberland/liberland_substrate)](https://github.com/liberland/liberland_substrate/tags) [![Twitter](https://img.shields.io/badge/Twitter-gray?logo=twitter)](https://twitter.com/Liberland_org)

# Liberland Blockchain
<p>
<center>
   <img src="https://lgl.liberland.org/uploads/-/system/appearance/header_logo/1/Liberland_vlajka.png" alt="Liberland Logo" style="height: 68px; width:100px;"/>

</center>
</p>

Liberland is a country established in 2015, on a no man’s land (terra nullius) between Croatia and Serbia (www.liberland.org). Liberland’s founders are blockchain and liberty enthusiasts. Liberland’s State project could be summarised by two concepts: Minimal state and distributive governance.

We want to make all e-government services available to our citizens using our substrate-based blockchain.

This repository contains the substrate-based implementation of Liberland blockchain node.

## Documentation:
* [Learn more about Liberland](https://liberland-1.gitbook.io/wiki/)
* [Learn more about Liberland Blockchain](https://liberland-1.gitbook.io/wiki/v/public-documents/blockchain)
* [Technical docs](https://github.com/liberland/liberland.github.io/)

## Interact with live testnets:
* [Hazlitt](https://polkadot.js.org/apps/?rpc=wss%3A%2F%2Fl2.laissez-faire.trade#/explorer)
* [Starlight](https://polkadot.js.org/apps/?rpc=wss%253A%252F%252Ftestchain.liberland.org)

## Contact:
* [Liberland's website](https://liberland.org/)
* [Join Liberland's Technical Node Operator group on Matrix](https://matrix.to/#/!YzbTfsgCDANzhNLYpW:matrix.org?via=matrix.org)
* [Facebook Page](https://www.facebook.com/liberland)
* [Twitter](https://twitter.com/Liberland_org)
* Feel free to [open an issue](https://github.com/liberland/liberland_substrate/issues/new) in this repository

## Quick-start guide for developing locally

### Install deps
```
sudo apt install build-essential git clang curl libssl-dev llvm libudev-dev make protobuf-compiler
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default nightly
```

### Clone `liberland_substrate` repository
```
git clone https://github.com/liberland/liberland_substrate.git -b main
cd liberland_substrate
```

### Run automated tests
```
cargo test --release --features runtime-benchmarks --no-fail-fast
```

### Build and run development node
```
cargo run --release -- --dev
```

Development instance is a single-node testnet, in which standard development
accounts (Alice, Bob, etc.) are endowed with assets. To interact with it, visit
[Polkadot.js Apps](https://polkadot.js.org/apps/?rpc=ws://localhost:9944).

### Further reading
* [Run a validator on Hazlitt chain](https://github.com/liberland/liberland.github.io/blob/main/docs/run_validator.md)
* [Run a validator on StarLight Chain](https://github.com/liberland/liberland.github.io/blob/main/docs/run_a_validator_on_starlight.md)
* [Run with Docker](https://github.com/liberland/liberland.github.io/blob/main/docs/docker.md)


## Licensing
All code that is committed on behalf of Liberland is distributed under the MIT license. 
You may find a copy of the MIT license [here](https://github.com/liberland/liberland_substrate/blob/main/LICENSE-MIT). Alternatively, you may visit [OpenSource.org](https://opensource.org/licenses/MIT).
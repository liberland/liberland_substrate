# Multisig Court

This repository contains an ink! smart contract for Liberland blockchain which acts as a simplified court for force transferring staked LLM.

The artifacts folder contains the exact compiled binary that has been deployed, as well as all other compilation artifacts for verification purposes.

Contract is deployed on:
* Bastiat: 5D3aKEW8kC83uX1oun5YAwZKXk1x1JfjtX1VHKo2cjCyvZg5
* Mainnet: 5E3TujWEm2tvTLyHpYeggwzpPhEYpVjS14MyU4s2AqN7CWhB

# Building

Run in root of the repository (NOT in the `contracts` directory):
```
cargo install --force --locked cargo-contract
cargo contract build --verifiable --manifest-path ./contracts/msig_court/Cargo.toml
```

Build output can be found in `./contracts/msig_court/target/ink`.

# Verifying integrity

Run in root of the repository (NOT in the `contracts` directory):
```
cargo contract verify --contract=./contracts/msig_court/artifacts/msig_court.contract --manifest-path=./contracts/msig_court/Cargo.toml
```

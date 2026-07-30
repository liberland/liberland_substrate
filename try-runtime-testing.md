# Install try-runtime CLI
```
cargo install --git https://github.com/paritytech/try-runtime-cli --locked
```

The commands below disable try-runtime's strictly-greater spec-version check so
that the current runtime can be tested by non-runtime changes. This does not
replace the required `spec_version` bump when runtime behavior changes; CI still
rejects versions lower than the corresponding on-chain runtime.

# Test mainnet

1. Build runtime: `cargo build --features try-runtime --release`
2. Execute test: `try-runtime --runtime ./target/release/wbuild/kitchensink-runtime/kitchensink_runtime.wasm on-runtime-upgrade --disable-spec-version-check --disable-idempotency-checks --no-weight-warnings live --uri wss://mainnet.liberland.org:443`

# Test bastiat

1. Build runtime: `cargo build --features try-runtime,testnet-runtime --release`
2. Execute test: `try-runtime --runtime ./target/release/wbuild/kitchensink-runtime/kitchensink_runtime.wasm on-runtime-upgrade --disable-spec-version-check --disable-idempotency-checks --no-weight-warnings live --uri wss://testchain.liberland.org:443`
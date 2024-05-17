# Install try-runtime CLI
```
cargo install --git https://github.com/paritytech/try-runtime-cli --locked
```

# Test mainnet

1. Build runtime: `cargo build --features try-runtime --release`
2. Execute test: `try-runtime --runtime ./target/release/wbuild/kitchensink-runtime/kitchensink_runtime.wasm on-runtime-upgrade --disable-idempotency-checks --no-weight-warnings live --uri wss://mainnet.liberland.org:443`

# Test bastiat

1. Build runtime: `cargo build --features try-runtime,testnet-runtime --release`
2. Execute test: `try-runtime --runtime ./target/release/wbuild/kitchensink-runtime/kitchensink_runtime.wasm on-runtime-upgrade --disable-idempotency-checks --no-weight-warnings live --uri wss://archive.testchain.liberland.org:443`
1. `cargo install --git https://github.com/paritytech/try-runtime-cli --locked`
2. `cargo build --features try-runtime --release`
3. Test mainnet runtime upgrade: `try-runtime --runtime ./target/release/wbuild/kitchensink-runtime/kitchensink_runtime.wasm on-runtime-upgrade --disable-idempotency-checks --no-weight-warnings live --uri wss://mainnet.liberland.org:443`
4. Run your own local Bastiat node (public one doesn't support HTTPS currently used by `try-runtime`): `./target/release/substrate-node --tmp --chain bastiat`
5. Wait for node to sync
6. `try-runtime --runtime ./target/release/wbuild/kitchensink-runtime/kitchensink_runtime.wasm on-runtime-upgrade --disable-idempotency-checks --no-weight-warnings live --uri ws://localhost:9944`
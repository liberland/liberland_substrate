# Releasing new version

## Sanity checks:

1. Check `spec_version` value in [runtime's lib.rs](./substrate/bin/node/runtime/src/lib.rs). There are 2: one for mainnet and one for testnet. They should be the same and higher than the last release.
2. Check `version` in [runtime's Cargo.toml](./substrate/bin/node/runtime/Cargo.toml). Major version should match the `spec_version`.
3. Check `version` in [cli's Cargo.toml](./substrate/bin/node/cli/Cargo.toml). Major version should match the `spec_version`. This is the version that will be used as the name of the release and tag.

## Release

1. Go to the [Release new version](https://github.com/liberland/liberland_substrate/actions/workflows/release.yml) action in GitHub, select `Run workflow`, choose the branch (usually `develop` or `main`) and run it. This action will do the following:
    * Run unit-tests
    * Run `try-runtime` migration checks (which detects things like state inconsitencies or pallet/state version mismatches)
    * Run fork test for both bastiat and mainnet and check if whole state is still parsable post-fork
    * Build runtimes in a reproducible way
    * Build new binary
    * Create a new GitHub Release
    * Build & publish new Docker Image
2. Wait for the `Release new version` action to finish.
3. Go to [GitHub Releases](https://github.com/liberland/liberland_substrate/releases). New release should be there, ready for deployment.
4. (Optional) Open a new PR that bumps versions for the next release. [Sample PR](https://github.com/liberland/liberland_substrate/pull/404/files).
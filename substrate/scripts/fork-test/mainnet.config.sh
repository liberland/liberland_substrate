BUILDDIR="$(realpath "$(dirname "$0")/../../..")/target/release"

BINARY="$BUILDDIR/substrate-node"
RUNTIME="${RUNTIME:-$BUILDDIR/wbuild/kitchensink-runtime/kitchensink_runtime.compact.compressed.wasm}"
ORIG_CHAIN="mainnet"
WS_ENDPOINT="wss://liberland-rpc.dwellir.com"
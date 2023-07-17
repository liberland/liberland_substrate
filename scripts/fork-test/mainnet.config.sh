BUILDDIR="$(realpath "$(dirname "$0")/../..")/target/release/"

BINARY="${BINARY:-$BUILDDIR/substrate}"
RUNTIME="${RUNTIME:-$BUILDDIR/wbuild/kitchensink-runtime/kitchensink_runtime.compact.compressed.wasm}"
ORIG_CHAIN="mainnet"
WS_ENDPOINT="wss://mainnet.liberland.org"

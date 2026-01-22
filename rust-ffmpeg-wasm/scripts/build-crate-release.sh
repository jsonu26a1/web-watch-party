set -e

cargo build --target wasm32-unknown-emscripten --release

export CRATE_PROFILE="release"
export OPTIMIZATION_FLAGS=(
    # https://emscripten.org/docs/tools_reference/emcc.html#arguments
    #   [compile+link] Like -Os, but reduces code size even further, and may take longer to run. This can affect both Wasm and JavaScript.
    -Oz
)

./scripts/build-wasm.sh

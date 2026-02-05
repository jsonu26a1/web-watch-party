set -e

cargo build --target wasm32-unknown-emscripten

export CRATE_PROFILE="debug"
FLAGS=(
    # enable assertions when troubleshooting emscripten runtime
    # -sASSERTIONS

    # do a minimal amount of opimization
    # https://emscripten.org/docs/tools_reference/emcc.html#arguments
    #   [compile+link] Simple optimizations. During the compile step these include LLVM -O1 optimizations.
    #   During the link step this omits various runtime assertions in JS that -O0 would include.
    -O1
)
export OPTIMIZATION_FLAGS="${FLAGS[@]}"

make -f ./scripts/wasm-makefile

cp -ur -t ./target ./src/www

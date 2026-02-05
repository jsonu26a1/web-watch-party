CONF_FLAGS=(
#     -sMODULARIZE
#     -sMAIN_MODULE=2
#     -sSIDE_MODULE=2
#     -sWARN_ON_UNDEFINED_SYMBOLS=0
#     -sERROR_ON_UNDEFINED_SYMBOLS=0
#     -sFILESYSTEM=0
#     -O1
    ${OPTIMIZATION_FLAGS[@]}
    -sFILESYSTEM=0
    --js-library "./src/em-library.js"

    # disable ASYNCIFY; not using anymore.
    # -sASYNCIFY
    # we got error "unreachable"; docs advised increasing this; I chose 4096 * 16 (up from default 4096)
    # -sASYNCIFY_STACK_SIZE=65536
    # ASYNCIFY_IMPORTS and EXPORTED_FUNCTIONS might need modifying
#     -sASYNCIFY_IMPORTS="test_promise_js"
#     -sEXPORTED_FUNCTIONS="_test_promise,_foo_add,_get_av_version,_try_av_malloc,_try_box_leak"
    # -sASYNCIFY_IMPORTS="read_current_file"

    -sEXPORTED_FUNCTIONS="_get_av_version,_log_av_version,_probe_demo"

    -sEXPORTED_RUNTIME_METHODS="ccall,wasmMemory,UTF8ToString"

    # hmm, I guess we need these
    ./deps/install/lib/libvpx.a
    ./deps/install/lib/libopus.a

    # is it smart to use a glob pattern? there should only be the one static library file...
    ./target/wasm32-unknown-emscripten/$CRATE_PROFILE/*.a
    -o target/www/output.mjs
)

mkdir -p ./target/www

emcc "${CONF_FLAGS[@]}"

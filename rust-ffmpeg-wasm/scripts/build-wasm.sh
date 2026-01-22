CONF_FLAGS=(
#     -sMODULARIZE
#     -sMAIN_MODULE=2
#     -sSIDE_MODULE=2
#     -sWARN_ON_UNDEFINED_SYMBOLS=0
#     -sERROR_ON_UNDEFINED_SYMBOLS=0
#     -sFILESYSTEM=0
#     -O1
    ${OPTIMIZATION_FLAGS[@]}
    --js-library "./my-library.js"
    -sASYNCIFY
    # ASYNCIFY_IMPORTS and EXPORTED_FUNCTIONS might need modifying
    -sASYNCIFY_IMPORTS="test_promise_js"
    -sEXPORTED_FUNCTIONS="_test_promise,_foo_add,_get_av_version,_try_av_malloc,_try_box_leak"

    -sEXPORTED_RUNTIME_METHODS="ccall,wasmMemory,UTF8ToString"

    # is it smart to use a glob pattern? there should only be the one static library file...
    "./target/wasm32-unknown-emscripten/$CRATE_PROFILE/*.a"
    -o out/output.js
)

emcc "${CONF_FLAGS[@]}"

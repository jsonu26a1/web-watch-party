addToLibrary({
    console_log_raw: (ptr, len) => {
        console.log(Module.UTF8ToString(ptr, len));
    },
    // Asyncify.handleAsync isn't very well documented; but I believe this works
    // https://emscripten.org/docs/porting/asyncify.html#ways-to-use-asyncify-apis-in-older-engines
    read_current_file: (ptr, offset, len) => Asyncify.handleAsync(async () => {
        console.log(`call to read_current_file; ptr ${ptr}, offset ${offset}, len ${len}`);
        if(offset > 1n << 53n)
            throw `offset ${offset} is too large`;
        offset = Number(offset);
        let current_file = Module.current_file;
        let remaining = current_file.size - offset
        if(remaining < len)
            len = remaining;
        // current_file is a File/Blob, we get a slice to the chunk we want to read
        // obtaining an arrayBuffer is async (disk IO)
        let buf = await current_file.slice(offset, len).arrayBuffer();
        // copy the data into the wasm runtime
        HEAPU8.set(new Uint8Array(buf), ptr);
        return len;
    }),
    get_current_file_size: () => BigInt(Module.current_file.size),
})

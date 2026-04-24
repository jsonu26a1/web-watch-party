addToLibrary({
    console_log_raw: (ptr, len) => {
        console.log(Module.UTF8ToString(ptr, len));
    },
    read_current_file: (ptr, offset, len) => {
        if(offset > 1n << 53n)
            throw `offset ${offset} is too large`;
        offset = Number(offset);
        let buffer = new Uint8Array(HEAPU8.buffer, ptr, len);
        return Module.current_file_handle.read_sync(buffer, offset);
    },
    get_current_file_size: () => BigInt(Module.current_file_handle.size()),
    write_file_by_tag: (tag, offset, ptr, size) => {
        // Module.write_handle.write_file_by_tag(tag, offset, ptr, size);
        let buffer = new Uint8Array(HEAPU8.buffer, ptr, size);
        (new Uint8Array(Module.output_buffers[tag], offset)).set(buffer);
    },
})

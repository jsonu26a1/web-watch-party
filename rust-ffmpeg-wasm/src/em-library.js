addToLibrary({
    // NOTE:
    // Module.input_files should be an Array of objects with methods read_sync() and size().
    // Module.output_files should be an Array of plain ArrayBuffer objects.
    file_read: (tag, ptr, offset, len) => {
        if(offset > 1n << 53n)
            throw `offset ${offset} is too large`;
        offset = Number(offset);
        let buffer = new Uint8Array(HEAPU8.buffer, ptr, len);
        return Module.input_files[tag].read_sync(buffer, offset);
    },
    file_size: () => BigInt(Module.input_files[tag].size()),
    file_write: (tag, offset, ptr, size) => {
        let buffer = new Uint8Array(HEAPU8.buffer, ptr, size);
        (new Uint8Array(Module.output_files[tag], offset)).set(buffer);
    },
})

addToLibrary({
    // NOTE:
    // Module.input_files should be an Array of objects with methods read_sync() and size().
    // Module.output_files should be an Array of plain ArrayBuffer objects.
    file_read: (tag, offset, ptr, size) => {
        if(offset > 1n << 53n)
            throw `offset ${offset} is too large`;
        offset = Number(offset);
        let buffer = new Uint8Array(HEAPU8.buffer, ptr, size);
        return Module.input_files[tag].read_sync(buffer, offset);
    },
    file_size: () => BigInt(Module.input_files[tag].size()),
    file_write: (tag, offset, ptr, size) => {
        let target = new Uint8Array(Module.output_files[tag], offset);
        if(target.length < size)
            return -1;
        let buffer = new Uint8Array(HEAPU8.buffer, ptr, size);
        target.set(buffer);
        return 0;
    },
})

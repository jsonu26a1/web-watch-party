addToLibrary({
    // NOTE:
    // Module.input_files should be an Array of objects with methods read_sync() and size().
    // Module.output_files should be an Array of objects, each with properties `buffer` and `written`.
    file_read: (tag, offset, ptr, size) => {
        if(offset > 1n << 53n)
            throw `offset ${offset} is too large`;
        offset = Number(offset);
        let buffer = new Uint8Array(HEAPU8.buffer, ptr, size);
        return Module.input_files[tag].read_sync(buffer, offset);
    },
    file_size: (tag) => BigInt(Module.input_files[tag].size()),
    file_write: (tag, offset, ptr, size) => {
        if(offset > 1n << 53n)
            throw `offset ${offset} is too large`;
        offset = Number(offset);
        if(size > 1n << 53n)
            throw `size ${size} is too large`;
        size = Number(size);
        let output_file = Module.output_files[tag];
        let target = new Uint8Array(output_file.buffer, offset);
        if(target.length < size) {
            console.log("file_write() failed: target.length < size;", target.length, size)
            return -1;
        }
        let buffer = new Uint8Array(HEAPU8.buffer, ptr, size);
        target.set(buffer);
        output_file.written += size;
        return 0;
    },
})

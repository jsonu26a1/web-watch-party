import fs from 'node:fs';
import ffmpeg_module from './output.mjs';

let file_name = fs.readFileSync("./deps/sample-media-path.txt", { encoding: 'utf8' }).trim();

class FileHandle {
    constructor() {
        this.fd = fs.openSync(file_name);
        this.st_size = fs.fstatSync(this.fd).size;
        this.offset = 0;
    }
    read_sync(buffer, offset) {
        // buffer is a Uint8Array pointing to the region in the WASM runtime's memory to write to;
        // offset in the file to read from; if seeking isn't necessary, read from current file position
        if(this.offset == offset)
            offset = -1;
        else
            this.offset = offset;
        let bytesRead = fs.readSync(this.fd, buffer, 0, buffer.byteLength, offset);
        this.offset += bytesRead;
        return bytesRead;
    }
    size() {
        return this.st_size;
    }
}

let ffmpeg = await ffmpeg_module({ current_file_handle: new FileHandle() });

export default ffmpeg;

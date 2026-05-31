import fs from 'node:fs';
import path from 'node:path';
// import ffmpeg_module from '../target/www/output.mjs';
// my source files are in a symlink dir (I have a weird linux vm dev setup) so we need process.env.PWD because node is stupid.
let ffmpeg_module = (await import(path.join(process.env.PWD, "../target/www/output.mjs"))).default;

export let sample_files = fs.readFileSync(path.join(process.env.PWD, "../deps/sample-media-path.txt"), { encoding: 'utf8' }).trim().split("\n");

export class FileHandle {
    constructor(file_name) {
        this.fd = fs.openSync(file_name);
        this.st_size = fs.fstatSync(this.fd).size;
        this.offset = 0;
    }
    read_sync(buffer, offset) {
        // console.log("FileHandle.read_sync()", this.offset, offset);

        // buffer is a Uint8Array pointing to the region in the WASM runtime's memory to write to;
        // offset in the file to read from; if seeking isn't necessary, read from current file position
        if(this.offset == offset)
            offset = -1;
        else
            this.offset = offset;
        // console.log("...", this.offset, offset);

        // this is a bit disturbing: if we reuse the fd with ffmpeg._file_open, ffmpeg isn't able to parse the file...
        // even though offset is 0, fs.readSync doesn't appear to be reading from the beginning of the file.
        // the docs say: "If `position` is a non-negative integer, the file position will be unchanged."
        // I'm so confused... I think it's a bug in nodejs, or something. wouldn't surprise me.
        // anyways, closing and re-opening the fd works around this.
        let bytesRead = fs.readSync(this.fd, buffer, 0, buffer.byteLength, offset);
        this.offset += bytesRead;

        // console.log("... ...", this.offset, bytesRead);

        return bytesRead;
    }
    size() {
        return this.st_size;
    }
    close() {
        fs.closeSync(this.fd);
        this.fd = -1;
    }
}

let ffmpeg = await ffmpeg_module({ input_files: [], output_files: [] });

export { ffmpeg };

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
    }
    read_sync(buffer, offset) {
        return fs.readSync(this.fd, buffer, 0, buffer.byteLength, offset);
    }
    size() {
        return this.st_size;
    }
}

let ffmpeg = await ffmpeg_module({ input_files: [], output_files: [] });

export { ffmpeg };

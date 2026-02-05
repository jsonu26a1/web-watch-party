import ffmpeg_module from './output.mjs';

// TODO: not tested yet; must be run in web worker

let reader = new FileReaderSync();

class FileHandle {
    constructor(file) {
        // file is a File/Blob
        this.file = file;
    }
    read_sync(buffer, offset) {
        // buffer is a Uint8Array pointing to the region in the WASM runtime's memory to write to;
        let end = Math.max(offset + buffer.byteLength, this.file.size);
        let file_data = reader.readAsArrayBuffer(this.file.slice(offset, buffer.byteLength));
        buffer.set(file_data);
        return file_data.byteLength;
    }
    size() {
        return file.size;
    }
}

export default async function(file) {
    let ffmpeg = await ffmpeg_module({ current_file_handle: new FileHandle() });
    return ffmpeg;
}

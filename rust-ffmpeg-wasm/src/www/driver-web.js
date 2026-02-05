import ffmpeg_module from './output.mjs';

let reader = new FileReaderSync();

class FileHandle {
    constructor(file) {
        // file is a File/Blob
        this.file = file;
    }
    read_sync(buffer, offset) {
        // buffer is a Uint8Array pointing to the region in the WASM runtime's memory to write to;
        let end = Math.min(offset + buffer.byteLength, this.file.size);
        // console.log(`**** **** read_sync: buffer-offer:${buffer.byteOffset}, buffer-len:${buffer.byteLength}, `+
        //     `offset:${offset}, end:${end}`);
        let file_data = reader.readAsArrayBuffer(this.file.slice(offset, end));
        // we must wrap the ArrayBuffer in a TypedArray here. JS APIs are so weird.
        buffer.set(new Uint8Array(file_data));
        return file_data.byteLength;
    }
    size() {
        return this.file.size;
    }
}

export default async function(file) {
    let ffmpeg = await ffmpeg_module({ current_file_handle: new FileHandle(file) });
    return ffmpeg;
}

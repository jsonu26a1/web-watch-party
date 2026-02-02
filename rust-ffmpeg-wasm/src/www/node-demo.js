import * as fs from 'node:fs/promises';

let ffmpeg_wasm = await new Promise(async (resolve, reject) => {
    let { default: m } = await import('./output.js');
    m.onRuntimeInitialized = () => resolve(m);
});

let file_name = "frag_bunny.mp4";

ffmpeg_wasm.current_file = new Blob([(await fs.readFile(file_name)).buffer]);
// ffmpeg_wasm.current_file = new Blob([new ArrayBuffer(1024)]);

await ffmpeg_wasm.ccall("probe_demo", "", [], [], {"async":true})
console.log("probe_demo finished;")

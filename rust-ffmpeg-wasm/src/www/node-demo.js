let ffmpeg_wasm = await new Promise(async (resolve, reject) => {
    let { default: m } = await import('./output.js');
    m.onRuntimeInitialized = () => resolve(m);
});
ffmpeg_wasm.current_file = new Blob([new ArrayBuffer(1024)]);
console.log("runtime initialized");
console.log(ffmpeg_wasm._probe_demo());

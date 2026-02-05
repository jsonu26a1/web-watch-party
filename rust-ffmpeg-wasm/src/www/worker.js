import init_ffmpeg from './driver-web.js';

console.log("[worker] hello, world!");

let ffmpeg = null;

onmessage = async (e) => {
    onmessage = () => {
        console.log("ffmpeg runtime already initialized");
    };
    ffmpeg = await init_ffmpeg(e.data);
    ffmpeg._probe_demo();
    e.source.postMessage("finished probe_demo()");
};

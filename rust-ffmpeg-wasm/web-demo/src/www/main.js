let video = document.querySelector("video");
let output = document.querySelector("#output");

function dbg_log(...args) {
    console.log("[DBG]", ...args);
}

function dbg_record(prop, obj) {
    let ident = "dbg_objs";
    if(!window[ident])
        window[ident] = {};
    window[ident][prop] = obj;
    dbg_log(`window.${ident}["${prop}"] set to`, obj);
}

function async_event(target, event_name, dbg=false) {
    return new Promise((ok, err) => {
        let cb = (arg) => {
            target.removeEventListener(event_name, cb);
            if(dbg)
                dbg_log(`on ${target}, event fired: "${event_name}"`, arg)
            ok(arg);
        }
        if(dbg)
            dbg_log(`on ${target}, event listening: "${event_name}"`);
        target.addEventListener(event_name, cb);
    });
}

class MediaSourceHandler {
  constructor() {
    this.init();
  }
  init() {
    this.mediaSource = new MediaSource();
    this.sourceBuffer = null;
  }
  async attach_video_player(video) {
    this.video = video;
    video.src = URL.createObjectURL(this.mediaSource);
    await async_event(this.mediaSource, "sourceopen");
  }
  set_mime_codec(mimeCodec) {
    console.log(`MediaSource.isTypeSupported("${mimeCodec}")?`, MediaSource.isTypeSupported(mimeCodec));
    this.sourceBuffer = this.mediaSource.addSourceBuffer(mimeCodec);
  }
  async append_buffer(buffer) {
    this.sourceBuffer.appendBuffer(buffer);
    await async_event(this.sourceBuffer, "updateend");
  }
  end_stream() {
    this.mediaSource.endOfStream();
  }
}

let msh = new MediaSourceHandler();

// this might also work in firefox?
async function get_mime_codec_v2() {
  let info = JSON.parse(await (await fetch("/api/media_info")).text());
  return `video/${info.format}`;
}

// TODO this is completely broken. I think the solution is to fix mux_frag.rs:media_info in the wasm
async function get_mime_codec() {
  let info = await (await fetch("/api/media_info")).json();
  // this only works in firefox, chrome is more strict about codec strings
  let audio, video;
  if(info.format == "mp4") {
    audio = info.audio.tag;
    video = info.video.tag;
  } else if(info.format == "webm") {
    audio = info.audio.name;
    video = info.video.name;
  }
  return `video/${info.format}; codecs="${video}, ${audio}"`;
}

async function load_video() {
  await fetch("/api/file_open");
  await msh.attach_video_player(video);
  let mime_codec = await get_mime_codec();
  dbg_log(mime_codec);
  msh.set_mime_codec(mime_codec);
  let res = await fetch("/api/mux_next_dual");
  // let res = await fetch("/api/test_file");
  let buffer = await res.arrayBuffer();
  dbg_record("buffer", buffer);
  console.log(buffer)
  await msh.append_buffer(buffer);
  msh.end_stream();
}

async function load_video2() {
  await fetch("/api/file_open");
  await msh.attach_video_player(video);
  let mime_codec = await get_mime_codec();
  dbg_log(mime_codec);
  msh.set_mime_codec(mime_codec);
  // let res = await fetch("/api/mux_next_dual");
  let res = await fetch("/api/test_file");
  let buffer = await res.arrayBuffer();
  dbg_record("buffer", buffer);
  console.log(buffer)
  await msh.append_buffer(buffer);
  msh.end_stream();
}

// save the fragment/file to test externally with video player and ffprobe
async function dl_video() {
  await fetch("/api/file_open");
  let info = JSON.parse(await (await fetch("/api/media_info")).text());
  let api_url = "/api/mux_next_dual";
  // let api_url = "/api/test_file";
  let buffer = await (await fetch(api_url)).arrayBuffer();
  let blob = new Blob([buffer], {type: `video/${info.format}`});
  let link = document.createElement("a");
  link.href = URL.createObjectURL(blob);
  link.download = `sample_${info.file_name}`;
  link.innerText = "download sample";
  output.replaceChildren(link);
}

let video = document.querySelector("video");

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

// TODO this is completely broken. I think the solution is to fix mux_frag.rs:media_info in the wasm
async function get_mime_codec() {
  let res = await fetch("/api/media_info");
  let text = await res.text();
  let info = {};
  for(let item of text.split("\n")) {
    let fields = item.split(":");
    if(fields[0])
      info[fields[0]] = fields[1];
  }
  if(info.format.indexOf("webm") > 0) {
    info.format = "webm";
  }
  if(info.format.indexOf("mp4") > 0) {
    info.format = "mp4";
  }
  // this only works in firefox, chrome is more strict about codec strings
  return `video/${info.format}; codecs="${info.video}, ${info.audio}"`;
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

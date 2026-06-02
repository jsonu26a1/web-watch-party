import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import { ffmpeg, FileHandle, sample_files } from './driver.js';

const www_dir = path.join(import.meta.dirname, "www");

let frag_size = 1024*1024*4;
let output_buffer = new ArrayBuffer(frag_size*2);
let output_file = {
  buffer: output_buffer,
  written: 0,
};

ffmpeg.input_files = [];
ffmpeg.output_files = [output_file];

let active_file = -1;
let active_file_name = "";

function internal_open_file(index) {
  active_file_name = sample_files[index];
  ffmpeg.input_files = [new FileHandle(active_file_name)];
  active_file = ffmpeg._file_open(0);
}

internal_open_file(0);

const port = 8004;
const server = http.createServer((req, res) => {
  console.log(`[${new Date().toISOString()}]`, req.method, req.url);
  try {
    handle_request(req, res);
  } catch(e) {
    res.writeHead(500);
    res.end();
    console.log("exception while handling request;");
    console.log("error:", e);
  }
});

server.on("listening", () => {
  console.log(`listening on port ${port}`);
});

function handle_request(req, res) {
  let url = new URL(req.url, "http://localhost/");

  let pathname = url.pathname;
  if(pathname == "/")
    pathname = "/index.html";

  let data = null;
  try {
    data = fs.readFileSync(path.join(www_dir, pathname))
  } catch(e) {};
  if(data != null) {
    return res.end(data);
  }

  let path_segments = url.pathname.split("/");
  if(path_segments[1] == "api") {
    let api_fn = path_segments[2];
    console.log("API call:", api_fn)
    if(api_fn == "mux_next_dual") {
      if(!active_file)
        return res.end("error: no active file");
      output_file.written = 0;
      ffmpeg._mux_next_dual(active_file, 0, frag_size);
      console.log("ffmpeg._mux_next_dual():", output_file.written);
      res.setHeader('Content-Type', "application/octet-stream");
      res.write(new Uint8Array(output_buffer, 0, output_file.written));
      return res.end();
    } else if(api_fn == "file_open") {
      if(active_file)
        ffmpeg._file_close(active_file);
      internal_open_file(Number(url.searchParams.get("sample")));
      console.log("active_file:", active_file);
      return res.end("file opened");
    } else if(api_fn == "file_close") {
      if(active_file) {
        ffmpeg._file_close(active_file);
        return res.end("file closed");
      } else {
        return res.end("no file to close");
      }
    } else if(api_fn == "media_info") {
      if(!active_file)
        return res.end("error: no active file");
      ffmpeg._media_info(active_file);
      let info = JSON.parse(ffmpeg.received_string);
      info.file_name = path.basename(active_file_name);
      return res.end(JSON.stringify(info));
    } else if(api_fn == "test_file") {
      return res.end(fs.readFileSync(active_file_name));
    }
  }
  res.writeHead(404);
  return res.end("file not found");
}

server.listen(port);

import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';

// const index_file = path.join(module.__dirname, "index.html");
const index_file = path.join(import.meta.dirname, "index.html");

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
  if(url.pathname == "/") {
    return fs.readFile(index_file, (err, data) => res.end(data));
  }
  if(url.pathname == "/get_audio_track") {
    throw "TODO";
  }
  if(url.pathname == "/get_video_fragment") {
    let start_time = url.searchParams.get("start_time");
    if(start_time == null)
      throw "invalid start time";
    start_time = Number(start_time);
    throw "TODO";
  }
  res.writeHead(404);
  return res.end();
}

const port = 8004;
server.listen(port);

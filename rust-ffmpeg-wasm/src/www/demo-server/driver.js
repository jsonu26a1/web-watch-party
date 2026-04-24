import fs from 'node:fs';
import ffmpeg_module from './output.mjs';

let file_name = fs.readFileSync("./deps/sample-media-path.txt", { encoding: 'utf8' }).trim();


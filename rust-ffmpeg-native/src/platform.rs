use std::os::raw::{ c_int };

use std::fs::File;
use std::io::{ BufReader, Read, Error, ErrorKind, Seek, SeekFrom };
use std::slice;

pub struct OpenFileHandle {
    file: BufReader<File>,
    size: u64,
}

impl OpenFileHandle {
    pub fn new() -> Self {
        let file_name = {
            // I can't use `include_str!` because I'm in a vm and ./src is a symlink to my host machine lol
            // it's bad but it's ok this is just for testing
            let mut s = String::new();
            File::open("../rust-ffmpeg-wasm/deps/sample-media-path.txt").unwrap().read_to_string(&mut s).unwrap();
            s.trim().to_string()
        };
        let file = File::open(file_name).unwrap();
        let size = file.metadata().unwrap().len();

        Self {
            file: BufReader::new(file),
            size,
        }
    }

    pub fn read(&mut self, buf_ptr: *mut u8, buf_size: c_int) -> c_int {
        let buf = unsafe { slice::from_raw_parts_mut(buf_ptr, buf_size as usize) };
        self.file.read(buf).unwrap() as c_int
    }

    pub fn seek(&mut self, offset: SeekFrom) -> i64 {
        // TODO we could track the cursor position and use BufReader::seek_relative
        // since seek otherwise drops the internal buffer
        // match offset {
        //     SeekFrom::Current(i) => ...
        //     _ => ...
        // }
        self.file.seek(offset).unwrap() as i64
    }

    pub fn size(&self) -> u64 {
        self.size
    }
}

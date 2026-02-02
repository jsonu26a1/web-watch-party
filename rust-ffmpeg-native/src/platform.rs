use std::os::raw::{ c_int };

use std::fs::File;
use std::io::{ BufReader, Read, Error, ErrorKind };
use std::slice;

pub struct OpenFileHandle {
    file: BufReader<File>,
}

impl OpenFileHandle {
    pub fn new() -> Self {
        let file_name = "frag_bunny.mp4";

        Self {
            file: BufReader::new(File::open(file_name).unwrap()),
        }
    }

    pub fn read(&mut self, buf_ptr: *mut u8, buf_size: c_int) -> c_int {
        let buf = unsafe { slice::from_raw_parts_mut(buf_ptr, buf_size as usize) };
        self.file.read(buf).unwrap() as c_int
    }
}

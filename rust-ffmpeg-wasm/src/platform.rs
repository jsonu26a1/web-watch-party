use std::os::raw::{ c_int };
use std::io::SeekFrom;

unsafe extern "C" {
    // offset might need to be i64 instead of usize?
    fn read_current_file(ptr: *const u8, offset: i64, len: i32) -> i32;
    fn get_current_file_size() -> u64;
}

pub struct OpenFileHandle {
    cursor: i64,
    size: u64,
}

impl OpenFileHandle {
    pub fn new() -> Self {
        Self {
            cursor: 0,
            size: unsafe { get_current_file_size() },
        }
    }

    pub fn read(&mut self, buf: *mut u8, buf_size: c_int) -> c_int {
        let c = unsafe { read_current_file(buf, self.cursor, buf_size) };
        self.cursor += c as i64;
        c
    }

    pub fn seek(&mut self, offset: SeekFrom) -> i64 {
        match offset {
            SeekFrom::Start(i) => self.cursor = i as i64,
            SeekFrom::End(i) => self.cursor = self.size as i64 + i,
            SeekFrom::Current(i) => self.cursor += i,
        }
        self.cursor
    }

    pub fn size(&self) -> u64 {
        self.size
    }
}

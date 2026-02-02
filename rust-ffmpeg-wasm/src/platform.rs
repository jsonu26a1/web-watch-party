use std::os::raw::{ c_int };

unsafe extern "C" {
    // offset might need to be i64 instead of usize?
    fn read_current_file(ptr: *const u8, offset: i64, len: i32) -> i32;
    fn get_current_file_size() -> i64;
}

pub struct OpenFileHandle {
    cursor: i64,
}

impl OpenFileHandle {
    pub fn new() -> Self {
        Self {
            cursor: 0,
        }
    }

    pub fn read(&mut self, buf: *mut u8, buf_size: c_int) -> c_int {
        let c = unsafe { read_current_file(buf, self.cursor, buf_size) };
        self.cursor += c as i64;
        c
    }
}

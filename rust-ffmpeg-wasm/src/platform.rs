use std::io::SeekFrom;

use crate::context::{ IoReadHandler, IoWriteHandler };

pub struct ReadHandle {
    cursor: i64,
    size: u64,
}

impl ReadHandle {
    pub fn new() -> Self {
        Self {
            cursor: 0,
            size: unsafe { get_current_file_size() },
        }
    }
}

unsafe extern "C" {
    fn read_current_file(ptr: *const u8, offset: i64, len: i32) -> i32;
    fn get_current_file_size() -> u64;
}

impl IoReadHandler for ReadHandle {
    fn read(&mut self, buf_ptr: *mut u8, buf_size: i32) -> i32 {
        let count = unsafe { read_current_file(buf_ptr, self.cursor, buf_size) };
        self.cursor += count as i64;
        count
    }
    fn seek(&mut self, offset: SeekFrom) -> i64 {
        match offset {
            SeekFrom::Start(i) => self.cursor = i as i64,
            SeekFrom::End(i) => self.cursor = self.size as i64 + i,
            SeekFrom::Current(i) => self.cursor += i,
        }
        self.cursor
    }
    fn size(&self) -> u64 {
        self.size
    }
}

pub struct WriteHandle {
    cursor: i64,
    size: u64,
    tag: i32,
}

impl WriteHandle {
    pub fn for_audio() -> Self {
        Self::new(0)
    }

    pub fn for_video() -> Self {
        Self::new(1)
    }

    pub fn new(tag: i32) -> Self {
        Self {
            cursor: 0,
            size: 0,
            tag,
        }
    }

    pub fn new_tmp() -> Self {
        static mut TAG: i32 = 0;
        unsafe {
            let h = Self::new(TAG);
            TAG += 1;
            h
        }
    }
}

unsafe extern "C" {
    fn write_file_by_tag(tag: i32, offset: i64, ptr: *const u8, size: i32);
}

impl IoWriteHandler for WriteHandle {
    fn write(&mut self, buf_ptr: *const u8, buf_size: i32) -> i32 {
        unsafe { write_file_by_tag(self.tag, self.cursor, buf_ptr, buf_size) };
        self.cursor += buf_size as i64;
        let end = self.cursor as u64;
        if self.cursor as u64 > self.size {
            self.size = end;
        }
        0
    }
    fn seek(&mut self, offset: SeekFrom) -> i64 {
        match offset {
            SeekFrom::Start(i) => self.cursor = i as i64,
            SeekFrom::End(i) => self.cursor = self.size as i64 + i,
            SeekFrom::Current(i) => self.cursor += i,
        }
        self.cursor
    }
}
use rusty_ffmpeg::ffi;

use std::os::raw::{ c_int, c_void };
use std::io::SeekFrom;

use crate::platform::{ OpenFileHandle };

const BUFFER_SIZE: i32 = 1024 * 4 * 16;

pub struct IoContext {
    pub avio_ctx: *mut ffi::AVIOContext,
}

impl IoContext {
    pub fn new(handle: OpenFileHandle) -> Option<Self> {
        let avio_ctx_buffer = unsafe { ffi::av_malloc(BUFFER_SIZE as usize) };
        if avio_ctx_buffer.is_null() {
            return None;
        }
        let avio_ctx = unsafe { ffi::avio_alloc_context(
            avio_ctx_buffer.cast(),
            BUFFER_SIZE,
            0, // not writable
            Box::into_raw(Box::new(handle)).cast(),
            Some(read_callback),
            None,
            Some(seek_callback),
        ) };
        if avio_ctx.is_null() {
            // free avio_ctx_buffer if avio_alloc_context fails. not a common edge case,
            // but the avio_read_callback example does this and I like "correctness"
            unsafe { ffi::av_free(avio_ctx_buffer) };
            None
        } else {
            Some(IoContext { avio_ctx })
        }
    }

    pub fn as_ptr(&self) -> *mut ffi::AVIOContext {
        self.avio_ctx
    }

    pub fn as_mut(&mut self) -> &mut *mut ffi::AVIOContext {
        &mut self.avio_ctx
    }
}

impl Drop for IoContext {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.avio_ctx.as_mut().unwrap().opaque));
            ffi::av_free(self.avio_ctx.as_mut().unwrap().buffer.cast());
            ffi::avio_context_free(self.as_mut());
        }
    }
}

extern "C" fn read_callback(opaque: *mut c_void, buf_ptr: *mut u8, buf_size: c_int) -> c_int {
    let handle = unsafe { opaque.cast::<OpenFileHandle>().as_mut().unwrap() };
    let count_read = handle.read(buf_ptr, buf_size);
    if count_read == 0 {
        return ffi::AVERROR_EOF;
    }
    count_read
}

// from libc
// const SEEK_SET: c_int = 0;
const SEEK_CUR: c_int = 1;
const SEEK_END: c_int = 2;

extern "C" fn seek_callback(opaque: *mut c_void, offset: i64, whence: c_int) -> i64 {
    let handle = unsafe { opaque.cast::<OpenFileHandle>().as_mut().unwrap() };
    if whence & ffi::AVSEEK_SIZE as i32 > 0 {
        return handle.size() as i64;
    }
    let seek_offset = if whence & SEEK_CUR > 0 {
        SeekFrom::Current(offset)
    } else if whence & SEEK_END > 2 {
        SeekFrom::End(offset)
    } else {
        // default is SEEK_SET
        SeekFrom::Start(offset as u64)
    };
    handle.seek(seek_offset)
}

/*
    I originally was using NonNull<ffi::AVFormatContext>, but avformat_open_input takes a `*mut *mut ffi::AVFormatContext`,
    and will free and set it to NULL on failure; avformat_close_input is safe to call with NULL
*/

pub struct FormatContext {
    pub ifmt_ctx: *mut ffi::AVFormatContext,
    pub io_ctx: IoContext,
}

impl FormatContext {
    pub fn new(io_ctx: IoContext) -> Option<Self> {
        let mut ifmt_ctx = unsafe { ffi::avformat_alloc_context() };
        unsafe { ifmt_ctx.as_mut()?.pb = io_ctx.avio_ctx };
        Some(Self {
            ifmt_ctx,
            io_ctx
        })
    }

    pub fn with_handle(handle: OpenFileHandle) -> Option<Self> {
        Self::new(IoContext::new(handle)?)
    }

    pub fn as_ptr(&self) -> *mut ffi::AVFormatContext {
        self.ifmt_ctx
    }

    pub fn as_mut(&mut self) -> &mut *mut ffi::AVFormatContext {
        &mut self.ifmt_ctx
    }
}

impl Drop for FormatContext {
    fn drop(&mut self) {
        unsafe { ffi::avformat_close_input(self.as_mut()) };
    }
}

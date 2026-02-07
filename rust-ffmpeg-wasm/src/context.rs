use rusty_ffmpeg::ffi;

use std::os::raw::{ c_int, c_void };
use std::io::SeekFrom;

use crate::platform::{ OpenFileHandle };

const BUFFER_SIZE: i32 = 1024 * 4 * 16;

/*
    still under construction;
    I started with a simple trait `_OpenFileHandle`, but is kind of ugly to impl and would mean we would
    always be passing `extern "C" fn` read, write, and seek callbacks to `avio_alloc_context`.

    then I came up with the design for `FileHandle`, and managed to make the `extern "C"` callbacks as part
    of the trait implementation, which means we don't have to deal with trait objects. I also added specialized
    traits `ReadHandle` and `WriteHandle`, and `IoContext` is able to accept either and pass the correct C
    callback functions to `avio_alloc_context`.
*/

pub trait _OpenFileHandle {
    fn read(&mut self, buf_ptr: *mut u8, buf_size: i32) -> i32;
    fn write(&mut self, buf_ptr: *const u8, buf_size: i32) -> i32;
    fn seek(&mut self, offset: SeekFrom) -> i64;
    fn size(&self) -> u64;
}

pub trait FileHandle: Sized {
    const READONLY: bool; // true: only read+seek, false: only write
    fn read(&mut self, buf_ptr: *mut u8, buf_size: i32) -> i32 {
        unimplemented!();
    }
    fn write(&mut self, buf_ptr: *const u8, buf_size: i32) -> i32 {
        unimplemented!();
    }
    fn seek(&mut self, offset: SeekFrom) -> i64 {
        unimplemented!();
    }
    fn size(&self) -> u64 {
        unimplemented!();
    }

    unsafe extern "C" fn read_callback(opaque: *mut c_void, buf_ptr: *mut u8, buf_size: i32) -> i32 {
        let handle = unsafe { opaque.cast::<Self>().as_mut().unwrap() };
        let count_read = handle.read(buf_ptr, buf_size);
        // println!("**** read_callback: buf_ptr:{buf_ptr:p}, buf_size:{buf_size}, count_read:{count_read}");
        if count_read == 0 {
            return ffi::AVERROR_EOF;
        }
        count_read
    }
    unsafe extern "C" fn write_callback(opaque: *mut c_void, buf_ptr: *const u8, buf_size: i32) -> i32 {
        let handle = unsafe { opaque.cast::<Self>().as_mut().unwrap() };
        let ret = handle.write(buf_ptr, buf_size);
        // println!("**** write_callback: buf_ptr:{buf_ptr:p}, buf_size:{buf_size}, ret:{ret}");
        ret
    }
    unsafe extern "C" fn seek_callback(opaque: *mut c_void, offset: i64, whence: c_int) -> i64 {
        let handle = unsafe { opaque.cast::<Self>().as_mut().unwrap() };
        if whence & ffi::AVSEEK_SIZE as i32 > 0 {
            let size = handle.size() as i64;
            // println!("**** seek_callback: AVSEEK_SIZE({size})");
            return size;
        }
        let seek_offset = if whence & SEEK_CUR > 0 {
            SeekFrom::Current(offset)
        } else if whence & SEEK_END > 2 {
            SeekFrom::End(offset)
        } else {
            // default is SEEK_SET
            SeekFrom::Start(offset as u64)
        };
        // println!("**** seek_callback: {seek_offset:?}");
        handle.seek(seek_offset)
    }
    unsafe fn drop_box_from_raw(p: *mut c_void) {
        drop(unsafe { Box::from_raw(p.cast::<Self>()) });
    }
}

pub trait ReadHandle: Sized {
    fn read(&mut self, buf_ptr: *mut u8, buf_size: i32) -> i32;
    fn seek(&mut self, offset: SeekFrom) -> i64;
    fn size(&self) -> u64;
}

struct FileHandleReadWrapper<T>(T);

impl<T: ReadHandle> From<T> for FileHandleReadWrapper<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: ReadHandle> FileHandle for FileHandleReadWrapper<T> {
    const READONLY: bool = true;
    fn read(&mut self, buf_ptr: *mut u8, buf_size: i32) -> i32 {
        <T as ReadHandle>::read(&mut self.0, buf_ptr, buf_size)
    }
    fn seek(&mut self, offset: SeekFrom) -> i64 {
        <T as ReadHandle>::seek(&mut self.0, offset)
    }
    fn size(&self) -> u64 {
        <T as ReadHandle>::size(&self.0)
    }
}

pub trait WriteHandle: Sized {
    fn write(&mut self, buf_ptr: *const u8, buf_size: i32) -> i32;
}

struct FileHandleWriteWrapper<T>(T);

impl<T: WriteHandle> From<T> for FileHandleWriteWrapper<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: WriteHandle> FileHandle for FileHandleWriteWrapper<T> {
    const READONLY: bool = false;
    fn write(&mut self, buf_ptr: *const u8, buf_size: i32) -> i32 {
        <T as WriteHandle>::write(&mut self.0, buf_ptr, buf_size)
    }
}

pub struct IoContext2 {
    pub avio_ctx: *mut ffi::AVIOContext,
    drop_handle: unsafe fn(*mut c_void),
}

impl IoContext2 {
    pub fn new<T: Into<H>, H: FileHandle>(handle: T) -> Option<Self> {
        let handle = handle.into();
        let avio_ctx_buffer = unsafe { ffi::av_malloc(BUFFER_SIZE as usize) };
        if avio_ctx_buffer.is_null() {
            return None;
        }
        let (read_cb, write_cb, seek_cb) = if H::READONLY {
            (Some(H::read_callback as _), None, Some(H::seek_callback as _))
        } else {
            (None, Some(H::write_callback as _), None)
        };
        let opaque = Box::into_raw(Box::new(handle)).cast();
        let avio_ctx = unsafe { ffi::avio_alloc_context(
            avio_ctx_buffer.cast(),
            BUFFER_SIZE,
            0, // not writable
            opaque,
            read_cb,
            write_cb,
            seek_cb,
        ) };
        if avio_ctx.is_null() {
            // drop the opaque box we created
            unsafe { H::drop_box_from_raw(opaque) };
            // free avio_ctx_buffer if avio_alloc_context fails. not a common edge case,
            // but the avio_read_callback example does this and I like "correctness"
            unsafe { ffi::av_free(avio_ctx_buffer) };
            None
        } else {
            Some(IoContext2 {
                avio_ctx,
                drop_handle: H::drop_box_from_raw
            })
        }
    }

    pub fn as_ptr(&self) -> *mut ffi::AVIOContext {
        self.avio_ctx
    }

    pub fn as_mut(&mut self) -> &mut *mut ffi::AVIOContext {
        &mut self.avio_ctx
    }
}

impl Drop for IoContext2 {
    fn drop(&mut self) {
        unsafe { (self.drop_handle)(self.avio_ctx.as_mut().unwrap().opaque) };
        unsafe { ffi::av_free(self.avio_ctx.as_mut().unwrap().buffer.cast()) };
        unsafe { ffi::avio_context_free(self.as_mut()) };
    }
}

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
    // println!("**** read_callback: buf_ptr:{buf_ptr:p}, buf_size:{buf_size}, count_read:{count_read}");
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
        let size = handle.size() as i64;
        // println!("**** seek_callback: AVSEEK_SIZE({size})");
        return size;
    }
    let seek_offset = if whence & SEEK_CUR > 0 {
        SeekFrom::Current(offset)
    } else if whence & SEEK_END > 2 {
        SeekFrom::End(offset)
    } else {
        // default is SEEK_SET
        SeekFrom::Start(offset as u64)
    };
    // println!("**** seek_callback: {seek_offset:?}");
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

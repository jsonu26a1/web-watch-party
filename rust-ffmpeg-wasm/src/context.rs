use rusty_ffmpeg::ffi;

use std::os::raw::{ c_int, c_void };
use std::io::SeekFrom;
use std::ptr::{ null, null_mut };
use crate::platform::{ ReadHandle, WriteHandle };

const BUFFER_SIZE: i32 = 1024 * 4 * 16;

// from libc
// const SEEK_SET: c_int = 0;
const SEEK_CUR: c_int = 1;
const SEEK_END: c_int = 2;

/*
    is this over complicated? probably. I only wanted a single IoContext type that wraps AVIOContext,
    and properly constructing and dropping it. for it to accept either IoReadHandler or IoWriteHandler,
    I had to make IoHandler, ReadWrapper and WriteWrapper. the types are a bit ugly, but it's mostly
    encapsulated by InputFormatContext and OutputFormatContext, so it's not too terrible to use.
*/

unsafe trait IoHandler: Sized {
    const READONLY: bool; // true: only read+seek+size, false: only write
    fn read(&mut self, buf_ptr: *mut u8, buf_size: i32) -> i32;
    fn write(&mut self, buf_ptr: *const u8, buf_size: i32) -> i32;
    fn seek(&mut self, offset: SeekFrom) -> i64;
    fn size(&self) -> u64;

    unsafe extern "C" fn read_callback(opaque: *mut c_void, buf_ptr: *mut u8, buf_size: i32) -> i32 {
        let handle = unsafe { opaque.cast::<Self>().as_mut().unwrap() };
        let count_read = handle.read(buf_ptr, buf_size);
        // println!("**** read_callback: buf_ptr:{buf_ptr:p}, buf_size:{buf_size}, count_read:{count_read}");
        if count_read == 0 {
            return ffi::AVERROR_EOF;
        }
        count_read
    }
    // TODO this corresponds to avio->write_packet, see https://ffmpeg.org/doxygen/7.1/aviobuf_8c_source.html#l00131
    // if ret < 0, avio->error = ret; otherwise, the ret value is discarded.
    // what errors should we be passing along here?
    // maybe return AVERROR_EXTERNAL, and we store the original IOError for later.
    unsafe extern "C" fn write_callback(opaque: *mut c_void, buf_ptr: *const u8, buf_size: i32) -> i32 {
        let handle = unsafe { opaque.cast::<Self>().as_mut().unwrap() };
        // println!("**** write_callback: buf_ptr:{buf_ptr:p}, buf_size:{buf_size}");
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
        } else if whence & SEEK_END > 0 {
            SeekFrom::End(offset)
        } else {
            // default is SEEK_SET
            SeekFrom::Start(offset as u64)
        };
        // println!("**** seek_callback: {seek_offset:?}");
        handle.seek(seek_offset)
    }
}

pub trait IoReadHandler: Sized {
    fn read(&mut self, buf_ptr: *mut u8, buf_size: i32) -> i32;
    fn seek(&mut self, offset: SeekFrom) -> i64;
    fn size(&self) -> u64;
}

struct ReadWrapper<T>(T);

unsafe impl<T: IoReadHandler> IoHandler for ReadWrapper<T> {
    const READONLY: bool = true;
    fn read(&mut self, buf_ptr: *mut u8, buf_size: i32) -> i32 {
        T::read(&mut self.0, buf_ptr, buf_size)
    }
    fn write(&mut self, buf_ptr: *const u8, buf_size: i32) -> i32 {
        unimplemented!();
    }
    fn seek(&mut self, offset: SeekFrom) -> i64 {
        T::seek(&mut self.0, offset)
    }
    fn size(&self) -> u64 {
        T::size(&self.0)
    }
}

pub trait IoWriteHandler: Sized {
    fn write(&mut self, buf_ptr: *const u8, buf_size: i32) -> i32;
    fn seek(&mut self, offset: SeekFrom) -> i64;
}

struct WriteWrapper<T>(T);

unsafe impl<T: IoWriteHandler> IoHandler for WriteWrapper<T> {
    const READONLY: bool = false;
    fn read(&mut self, buf_ptr: *mut u8, buf_size: i32) -> i32 {
        unimplemented!();
    }
    fn write(&mut self, buf_ptr: *const u8, buf_size: i32) -> i32 {
        T::write(&mut self.0, buf_ptr, buf_size)
    }
    fn seek(&mut self, offset: SeekFrom) -> i64 {
        T::seek(&mut self.0, offset)
    }
    fn size(&self) -> u64 {
        unimplemented!();
    }
}

struct IoContext<T> {
    avio_ctx: *mut ffi::AVIOContext,
    handler: *mut T,
}

impl<T: IoReadHandler> IoContext<ReadWrapper<T>> {
    fn from_reader(handle: T) -> Option<Self> {
        Self::new(ReadWrapper(handle))
    }
}

impl<T: IoWriteHandler> IoContext<WriteWrapper<T>> {
    fn from_writer(handle: T) -> Option<Self> {
        Self::new(WriteWrapper(handle))
    }
}

impl<T: IoHandler> IoContext<T> {
    fn new(handle: T) -> Option<Self> {
        let avio_ctx_buffer = unsafe { ffi::av_malloc(BUFFER_SIZE as usize) };
        if avio_ctx_buffer.is_null() {
            return None;
        }
        let (write_flag, read_cb, write_cb) = if T::READONLY {
            (0, Some(T::read_callback as _), None)
        } else {
            (1, None, Some(T::write_callback as _))
        };
        let handler = Box::into_raw(Box::new(handle));
        let avio_ctx = unsafe { ffi::avio_alloc_context(
            avio_ctx_buffer.cast(),
            BUFFER_SIZE,
            write_flag,
            handler.cast(),
            read_cb,
            write_cb,
            Some(T::seek_callback as _),
        ) };
        if avio_ctx.is_null() {
            // drop the handler box we created
            unsafe { drop(Box::from_raw(handler)) };
            // free avio_ctx_buffer if avio_alloc_context fails. not a common edge case,
            // but the avio_read_callback example does this and I like "correctness"
            unsafe { ffi::av_free(avio_ctx_buffer) };
            None
        } else {
            Some(IoContext {
                avio_ctx,
                handler
            })
        }
    }
}

impl<T> Drop for IoContext<T> {
    fn drop(&mut self) {
        unsafe { drop(Box::from_raw(self.handler)) };
        unsafe { ffi::av_free(self.avio_ctx.as_mut().unwrap().buffer.cast()) };
        unsafe { ffi::avio_context_free(&mut self.avio_ctx) };
    }
}

struct FormatContext<T> {
    fmt_ctx: *mut ffi::AVFormatContext,
    io_ctx: IoContext<T>,
}

impl<T: IoHandler> FormatContext<T> {
    fn new(io_ctx: IoContext<T>) -> Option<Self> {
        let mut fmt_ctx = unsafe { ffi::avformat_alloc_context() };
        unsafe { fmt_ctx.as_mut()?.pb = io_ctx.avio_ctx };
        Some(Self {
            fmt_ctx,
            io_ctx
        })
    }
}

impl<T> Drop for FormatContext<T> {
    fn drop(&mut self) {
        // whoops, I had this wrong
        // unsafe { ffi::avformat_close_input(&mut self.fmt_ctx) };
        unsafe { ffi::avformat_free_context(self.fmt_ctx) };
        
    }
}

pub struct InputFormatContext<T = ReadHandle>(FormatContext<ReadWrapper<T>>);

impl<T: IoReadHandler> InputFormatContext<T> {
    pub fn new(handle: T) -> Option<Self> {
        let io_ctx = IoContext::from_reader(handle)?;
        let mut fmt_ctx = FormatContext::new(io_ctx)?;
        // we call avformat_open_input right after setting fmt_ctx.pb to io_ctx because it sets an
        // internal AVFMT_FLAG_CUSTOM_IO flag which is expected by avformat_close_input (called on Drop)
        let ret = unsafe { ffi::avformat_open_input(&mut fmt_ctx.fmt_ctx, null(), null(), null_mut()) };
        // NOTE: avformat_open_input sets fmt_ctx to null on failure
        if ret != 0 {
            // TODO instead of None, consider returning type Result<Self, AVError>, `Err(ret)`
            return None;
        }
        Some(Self(fmt_ctx))
    }

    pub fn get_inner(&mut self) -> &mut T {
        &mut unsafe { self.0.io_ctx.handler.as_mut() }.unwrap().0
    }

    pub fn as_ptr(&self) -> *mut ffi::AVFormatContext {
        self.0.fmt_ctx
    }

    // TODO: do we need this?
    pub fn as_mut(&mut self) -> &mut *mut ffi::AVFormatContext {
        &mut self.0.fmt_ctx
    }
}

impl<T> Drop for InputFormatContext<T> {
    fn drop(&mut self) {
        // TODO: WARNING: what if this is called before avformat_open_input?
        // in avio_read_callback.c, AVFormatContext.pb is only set right before avformat_open_input
        // is called. if InputFormatContext is dropped before avformat_open_input, then the flag
        // `AVFormatContext.flags |= AVFMT_FLAG_CUSTOM_IO` won't be set...
        // I think we should have InputFormatContext::new call avformat_open_input.

        unsafe { ffi::avformat_close_input(&mut self.0.fmt_ctx) }; 
    }
}

pub struct OutputFormatContext<T = WriteHandle>(FormatContext<WriteWrapper<T>>);

impl<T: IoWriteHandler> OutputFormatContext<T> {
    pub fn new(handle: T) -> Option<Self> {
        let io_ctx = IoContext::from_writer(handle)?;
        let fmt_ctx = FormatContext::new(io_ctx)?;
        Some(Self(fmt_ctx))
    }

    pub fn as_ptr(&self) -> *mut ffi::AVFormatContext {
        self.0.fmt_ctx
    }

    // TODO: do we need this?
    pub fn as_mut(&mut self) -> &mut *mut ffi::AVFormatContext {
        &mut self.0.fmt_ctx
    }
}


use std::rc::Rc;
use std::cell::RefCell;

pub struct LoggerSettings {
    pub tag: String,
    // pub log_fn: Option<fn(msg: String)>,
    pub read: bool,
    pub write: bool,
    pub seek: bool,
    pub size: bool,
}

// impl LoggerSettings {
//     pub fn new(tag: &str)
// }

pub struct ReadLogger<T> {
    target: T,
    pub settings: Rc<RefCell<LoggerSettings>>,
}

impl<T: IoReadHandler> ReadLogger<T> {
    pub fn new(target: T, tag: &str, read: bool, seek: bool, size: bool) -> Self {
        let settings = Rc::new(RefCell::new(LoggerSettings {
            tag: tag.to_string(),
            read,
            write: false,
            seek,
            size
        }));
        Self {
            target,
            settings,
        }
    }
}

impl<T: IoReadHandler> IoReadHandler for ReadLogger<T> {
    fn read(&mut self, buf_ptr: *mut u8, buf_size: i32) -> i32 {
        let settings = self.settings.borrow_mut();
        let ret = self.target.read(buf_ptr, buf_size);
        if settings.read {
            println!("[{}] read({buf_ptr:?}, {buf_size}) -> {ret}", settings.tag);
        }
        return ret;
    }
    fn seek(&mut self, offset: SeekFrom) -> i64 {
        let settings = self.settings.borrow_mut();
        let ret = self.target.seek(offset);
        if settings.seek {
            println!("[{}] seek({offset:?}) -> {ret}", settings.tag);
        }
        return ret;
    }
    fn size(&self) -> u64 {
        let settings = self.settings.borrow_mut();
        let ret = self.target.size();
        if settings.size {
            println!("[{}] size() -> {ret}", settings.tag);
        }
        return ret;
    }
}


pub struct WriteLogger<T> {
    target: T,
    pub settings: Rc<RefCell<LoggerSettings>>,
}

impl<T: IoWriteHandler> WriteLogger<T> {
    pub fn new(target: T, tag: &str, write: bool, seek: bool) -> Self {
        let settings = Rc::new(RefCell::new(LoggerSettings {
            tag: tag.to_string(),
            read: false,
            write,
            seek,
            size: false,
        }));
        Self {
            target,
            settings,
        }
    }
}

impl<T: IoWriteHandler> IoWriteHandler for WriteLogger<T> {
    fn write(&mut self, buf_ptr: *const u8, buf_size: i32) -> i32 {
        let settings = self.settings.borrow_mut();
        let ret = self.target.write(buf_ptr, buf_size);
        if settings.write {
            println!("[{}] write({buf_ptr:?}, {buf_size}) -> {ret}", settings.tag);
        }
        return ret;
    }
    fn seek(&mut self, offset: SeekFrom) -> i64 {
        let settings = self.settings.borrow_mut();
        let ret = self.target.seek(offset);
        if settings.seek {
            println!("[{}] seek({offset:?}) -> {ret}", settings.tag);
        }
        return ret;
    }
    // fn size(&self) -> u64 {
    //     let settings = self.settings.borrow_mut();
    //     let ret = self.target.size();
    //     if settings.size {
    //         println!("[{}] size() -> {ret}", settings.tag);
    //     }
    //     return ret;
    // }
}

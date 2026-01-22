use rusty_ffmpeg::ffi;

use std::os::raw::{ c_int, c_void };
use std::ptr::{ null, null_mut };

unsafe extern "C" {
    // offset might need to be i64 instead of usize?
    fn read_current_file(ptr: *const u8, offset: i64, len: i32) -> i32;
    fn get_current_file_size() -> i64;
}

// for seek callback, see these for "whence" flag:
// - https://www.man7.org/linux/man-pages/man3/fseek.3.html
// - https://ffmpeg.org/doxygen/7.1/avio_8h.html#a427ff2a881637b47ee7d7f9e368be63f

// example reference: https://github.com/FFmpeg/FFmpeg/blob/master/doc/examples/avio_read_callback.c

struct OpenFileHandle {
    cursor: i64,
}

extern "C" fn read_packet(opaque: *mut c_void, buf: *mut u8, buf_size: c_int) -> c_int {
    let handle = unsafe { (opaque as *mut OpenFileHandle).as_mut().unwrap() };
    let count_read = unsafe { read_current_file(buf, handle.cursor, buf_size) };
    if count_read == 0 {
        return ffi::AVERROR_EOF;
    }
    handle.cursor += count_read as i64;
    count_read
}

fn probe_current_file() {
    // for now, we don't care about freeing anything. but eventually, we will need to gracefully handle errors without leaks.
    const BUFFER_SIZE: i32 = 1024 * 16;
    let mut ifmt_ctx_ptr = unsafe { ffi::avformat_alloc_context() };
    let ifmt_ctx = unsafe { ifmt_ctx_ptr.as_mut().unwrap() };
    let avio_ctx_buffer = unsafe { ffi::av_malloc(BUFFER_SIZE as usize) };
    let handle = Box::leak(Box::new(OpenFileHandle { cursor: 0} )) as *mut _;
    let avio_ctx = unsafe { ffi::avio_alloc_context(
        avio_ctx_buffer as *mut u8,
        BUFFER_SIZE,
        0, // write_flag; 0: not writable.
        handle as *mut c_void,
        Some(read_packet),
        None,
        None // seek fn, TODO?
    ) };
    ifmt_ctx.pb = avio_ctx;
    // ifmt_ctx_ptr will be freed and set to null on error;
    let ret = unsafe { ffi::avformat_open_input(&mut ifmt_ctx_ptr as *mut _, null(), null(), null_mut()) };
    let ret = unsafe { ffi::avformat_find_stream_info(ifmt_ctx_ptr, null_mut()) };
    panic!("end of demo");
}

#[unsafe(no_mangle)]
pub extern "C" fn probe_demo() {
    probe_current_file();
}

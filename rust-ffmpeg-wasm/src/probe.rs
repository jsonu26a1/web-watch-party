use rusty_ffmpeg::ffi;

use std::os::raw::{ c_int, c_void };
use std::ptr::{ null, null_mut };
use std::slice;

use crate::platform::{ OpenFileHandle };

// for seek callback, see these for "whence" flag:
// - https://www.man7.org/linux/man-pages/man3/fseek.3.html
// - https://ffmpeg.org/doxygen/7.1/avio_8h.html#a427ff2a881637b47ee7d7f9e368be63f

// example reference: https://github.com/FFmpeg/FFmpeg/blob/master/doc/examples/avio_read_callback.c

extern "C" fn read_packet(opaque: *mut c_void, buf_ptr: *mut u8, buf_size: c_int) -> c_int {
    let handle = unsafe { (opaque as *mut OpenFileHandle).as_mut().unwrap() };
    let count_read = handle.read(buf_ptr, buf_size);
    // println!("****** read_packet; buf_ptr {buf_ptr:p} buf_size {buf_size} count_read {count_read}");
    if count_read == 0 {
        return ffi::AVERROR_EOF;
    }
    count_read
}

const BUFFER_SIZE: i32 = 1024 * 4;

macro_rules! err_abort_return {
    // ( $func:path ( $( $e:expr ),* ) ) => {
    ( $e:expr ) => {
        {
            // let ret = unsafe { $func( $($e),* ) };
            let ret = unsafe { $e };
            if ret != 0 {
                println!("****** ERROR call to {}: {ret}, {}", stringify!($func), unsafe { ffi::av_err2str(ret) });
                return;
            }
        }
    }
}

fn probe_dump_format() {
    // for now, we don't care about freeing anything. but eventually, we will need to gracefully handle errors without leaks.
    let mut ifmt_ctx_ptr = unsafe { ffi::avformat_alloc_context() };
    let ifmt_ctx = unsafe { ifmt_ctx_ptr.as_mut().unwrap() };
    let avio_ctx_buffer = unsafe { ffi::av_malloc(BUFFER_SIZE as usize) };
    let handle = Box::leak(Box::new(OpenFileHandle::new())) as *mut _;
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
    println!("****** avformat_open_input");
    err_abort_return! { ffi::avformat_open_input(&mut ifmt_ctx_ptr as *mut _, null(), null(), null_mut()) };
    // let ret = unsafe { ffi::avformat_open_input(&mut ifmt_ctx_ptr as *mut _, null(), null(), null_mut()) };
    // println!("****** avformat_open_input: {ret}");

    println!("****** avformat_find_stream_info");
    err_abort_return! { ffi::avformat_find_stream_info(ifmt_ctx_ptr, null_mut()) };
    // let ret = unsafe { ffi::avformat_find_stream_info(ifmt_ctx_ptr, null_mut()) };
    // println!("****** avformat_find_stream_info: {ret}");

    println!("****** av_dump_format");
    unsafe { ffi::av_dump_format(ifmt_ctx_ptr, 0, null(), 0) };

    println!("****** inspecting AVStreams...");
    let av_streams = unsafe { slice::from_raw_parts(ifmt_ctx.streams, ifmt_ctx.nb_streams as usize) };
    for stream in av_streams {
        let stream = unsafe { &**stream };
        // let av_stream = ifmt_ctx.streams
        println!("*** stream #{}, duration: {}, (tb {:?}), nb_frames {}",
            stream.index, stream.duration, stream.time_base, stream.nb_frames);
        // println!("")
    }

    println!("****** end of probe_dump_format()");
}

#[unsafe(no_mangle)]
pub extern "C" fn probe_demo() {
    // probe_current_file();
    probe_dump_format();
}

use rusty_ffmpeg::ffi;

use std::ptr::{ null, null_mut };
use std::slice;

use crate::platform::{ ReadHandle };
use crate::context::{ InputFormatContext };
use crate::err_abort_return;

pub fn dump_format() {
    let mut ifmt_ctx = InputFormatContext::new(ReadHandle::new()).unwrap();
    // avformat_open_input is now called in InputFormatContext::new
    // err_abort_return! { ffi::avformat_open_input(ifmt_ctx.as_mut(), null(), null(), null_mut()) };
    err_abort_return! { ffi::avformat_find_stream_info(ifmt_ctx.as_ptr(), null_mut()) };
    unsafe { ffi::av_dump_format(ifmt_ctx.as_ptr(), 0, null(), 0) };
    let av_fmt_ctx = unsafe { ifmt_ctx.as_ptr().as_ref().unwrap() };
    let av_streams = unsafe { slice::from_raw_parts(av_fmt_ctx.streams, av_fmt_ctx.nb_streams as usize) };
    for stream in av_streams {
        let stream = unsafe { &**stream };
        println!("*** stream #{}, duration: {}, (tb {:?}), nb_frames {}",
            stream.index, stream.duration, stream.time_base, stream.nb_frames);
    }
}

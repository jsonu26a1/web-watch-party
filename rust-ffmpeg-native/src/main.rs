use rusty_ffmpeg::ffi;

use std::ffi::CStr;
use std::ptr::{ null, null_mut };
use std::slice;

use crate::platform::{ ReadHandle, WriteHandle };
use crate::context::{ InputFormatContext, OutputFormatContext };

#[path = "../../rust-ffmpeg-wasm/src/demo/mod.rs"]
mod demo;
#[path = "../../rust-ffmpeg-wasm/src/context.rs"]
mod context;
mod platform;


fn main() {
    println!("Hello, world!");
    let cs = unsafe { CStr::from_ptr(ffi::av_version_info()) };
    println!("{}", cs.to_str().unwrap());
    // demo::probe::dump_format();
    // demo::remux::remux_example();
    demo::seek::remux_audio_repeat();
}

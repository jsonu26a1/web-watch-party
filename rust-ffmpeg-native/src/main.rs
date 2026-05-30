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
#[path = "../../rust-ffmpeg-wasm/src/mux_frag.rs"]
mod mux_frag;
mod platform;


fn main() {
    println!("Hello, world!");
    let cs = unsafe { CStr::from_ptr(ffi::av_version_info()) };
    println!("{}", cs.to_str().unwrap());
    // demo::probe::dump_format();
    // demo::remux::remux_example();
    // demo::seek::remux_audio_repeat();
    demo_mux_frag();
}

use mux_frag::{ prepare_input, mux_next_dual };

fn demo_mux_frag() {
    let frag_size = 1024*1024*4;
    let mut active_file = prepare_input(0);
    mux_next_dual(&mut active_file, 0, frag_size);
    mux_next_dual(&mut active_file, 1, frag_size);
    mux_next_dual(&mut active_file, 2, frag_size);
    // hmm, how do we know when the end of file has been reached?
}

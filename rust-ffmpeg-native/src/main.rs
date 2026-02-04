use rusty_ffmpeg::ffi;
use std::ffi::CStr;

#[path = "../../rust-ffmpeg-wasm/src/probe.rs"]
mod probe;
#[path = "../../rust-ffmpeg-wasm/src/context.rs"]
mod context;
mod platform;

fn main() {
    println!("Hello, world!");
    let cs = unsafe { CStr::from_ptr(ffi::av_version_info()) };
    println!("{}", cs.to_str().unwrap());
    probe::probe_demo();
}

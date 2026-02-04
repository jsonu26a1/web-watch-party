use rusty_ffmpeg::ffi;
use std::ffi::CStr;

mod probe;
mod platform;
mod context;

unsafe extern "C" {
    fn console_log_raw(ptr: *const u8, len: usize);
}

fn console_log(s: &str) {
    // let bytes = s.as_bytes();
    unsafe { console_log_raw(s.as_ptr(), s.len()); }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_av_version() -> *const u8 {
    unsafe { ffi::av_version_info() as *const u8 }
}

#[unsafe(no_mangle)]
pub extern "C" fn log_av_version() {
    let cs = unsafe { CStr::from_ptr(ffi::av_version_info()) };
    console_log(cs.to_str().unwrap());
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

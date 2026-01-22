use rusty_ffmpeg::ffi;

#[unsafe(no_mangle)]
pub extern "C" fn get_av_version() -> *const u8 {
    unsafe { ffi::av_version_info() as *const u8 }
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

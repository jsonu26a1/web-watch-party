use rusty_ffmpeg::ffi;


use std::{ error, fmt };

pub type Result<T> = Result<T, Error>;

pub struct Error(pub i32);

impl Error {
    // the `12` is from Standard C library <errno.h>
    const ENOMEM: i32 = ffi::AVERROR(12);
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result<()> {
        write!(f, "{self}")
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result<()> {
        write!(f, "(AVERROR) {}", ffi::av_err2str(self.0))
    }
}

impl error::Error for Error {}

// #[macro_export]
// macro_rules! err_abort_return {
//     ( $e:expr ) => {
//         {
//             let ret = unsafe { $e };
//             if ret < 0 {
//                 println!("****** ERROR call to {}: {ret}, {}", stringify!($e), ffi::av_err2str(ret) );
//                 return;
//             }
//         }
//     }
// }

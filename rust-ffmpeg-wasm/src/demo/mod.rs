pub mod probe;
pub mod remux;
pub mod seek;

#[unsafe(no_mangle)]
pub extern "C" fn run_demo() {
    println!("demo starting...");
    probe::dump_format();
    remux::remux_example();
    seek::remux_audio_repeat();
    println!("demo finished.")
}

#[macro_export]
macro_rules! err_abort_return {
    ( $e:expr ) => {
        {
            let ret = unsafe { $e };
            if ret < 0 {
                println!("****** ERROR call to {}: {ret}, {}", stringify!($e), ffi::av_err2str(ret) );
                return;
            }
        }
    }
}

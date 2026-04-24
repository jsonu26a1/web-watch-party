use std::fs::{ File, OpenOptions };
use std::io::{ BufReader, BufWriter, Read, Write, Error, ErrorKind, Seek, SeekFrom };
use std::path::{ Path, PathBuf };
use std::slice;

use std::sync::LazyLock;

use crate::context::{ IoReadHandler, IoWriteHandler };

static SAMPLE_MEDIA_PATH: LazyLock<String> = LazyLock::new(|| {
    let mut s = String::new();
    File::open("../rust-ffmpeg-wasm/deps/sample-media-path.txt").unwrap().read_to_string(&mut s).unwrap();
    s.lines().nth(0).unwrap().trim().to_string()
});

pub struct ReadHandle {
    file: BufReader<File>,
    size: u64,
}

impl ReadHandle {
    pub fn new() -> Self {
        let file_name = &*SAMPLE_MEDIA_PATH;
        let file = File::open(file_name).unwrap();
        let size = file.metadata().unwrap().len();

        Self {
            file: BufReader::new(file),
            size,
        }
    }
}

impl IoReadHandler for ReadHandle {
    fn read(&mut self, buf_ptr: *mut u8, buf_size: i32) -> i32 {
        let buf = unsafe { slice::from_raw_parts_mut(buf_ptr, buf_size as usize) };
        self.file.read(buf).unwrap() as i32
    }

    fn seek(&mut self, offset: SeekFrom) -> i64 {
        // TODO we could track the cursor position and use BufReader::seek_relative
        // since seek otherwise drops the internal buffer
        // match offset {
        //     SeekFrom::Current(i) => ...
        //     _ => ...
        // }
        self.file.seek(offset).unwrap() as i64
    }

    fn size(&self) -> u64 {
        self.size
    }
}

pub struct WriteHandle {
    file: BufWriter<File>,
}

fn output_file_name(kind: &str) -> PathBuf {
    let base = Path::new(&*SAMPLE_MEDIA_PATH);
    let file_name = format!( "{}_{kind}.{}",
        base.file_stem().unwrap().display(),
        base.extension().unwrap().display() );
    base.parent().unwrap().join(file_name)
}

impl WriteHandle {
    pub fn for_audio() -> Self {
        Self::new(output_file_name("audio"))
    }

    pub fn for_video() -> Self {
        Self::new(output_file_name("video"))
    }

    pub fn new(path: impl AsRef<Path>) -> Self {
        let file = OpenOptions::new().create(true).write(true).truncate(true).open(path).unwrap();
        Self {
            file: BufWriter::new(file),
        }
    }

    pub fn new_tmp() -> Self {
        static mut TAG: i32 = 0;
        unsafe {
            let tag = TAG;
            let h = Self::new(output_file_name(&format!("tmp{tag}")));
            TAG += 1;
            h
        }
    }
}

impl IoWriteHandler for WriteHandle {
    fn write(&mut self, buf_ptr: *const u8, buf_size: i32) -> i32 {
        let buf = unsafe { slice::from_raw_parts(buf_ptr, buf_size as usize) };
        match self.file.write_all(buf) {
            Ok(_) => 0,
            Err(e) => {
                // TODO store `e` somewhere to access later? or print to console?
                rusty_ffmpeg::ffi::AVERROR_EXTERNAL
            },
        }
    }
    fn seek(&mut self, offset: SeekFrom) -> i64 {
        // TODO we could track the cursor position and use BufReader::seek_relative
        // since seek otherwise drops the internal buffer
        // match offset {
        //     SeekFrom::Current(i) => ...
        //     _ => ...
        // }
        self.file.seek(offset).unwrap() as i64
    }
}

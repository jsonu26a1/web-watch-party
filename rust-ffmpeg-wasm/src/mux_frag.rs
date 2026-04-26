use rusty_ffmpeg::ffi;

use std::ffi::CStr;
use std::ptr::{ null, null_mut };
use std::slice;

use crate::platform::{ ReadHandle, WriteHandle };
use crate::context::{ InputFormatContext, OutputFormatContext, IoWriteHandler };



#[macro_export]
macro_rules! panic_on_err {
    ( $e:expr ) => {
        {
            let ret = unsafe { $e };
            if ret < 0 {
                panic!("****** ERROR call to `{}`: {ret}, {}", stringify!($e), ffi::av_err2str(ret) );
            }
        }
    }
}



// extremely crude; ideally, we would report streams info back to JS runtime, let user confirm which to use
// for audio/video. this just picks first one found for each.
pub fn prepare_input(tag: i32) -> ActiveFile {
    let mut ifmt_ctx = InputFormatContext::new(ReadHandle::new(tag)).unwrap();

    panic_on_err! { ffi::avformat_find_stream_info(ifmt_ctx.as_ptr(), null_mut()) };

    let ifmt_short_name = unsafe { CStr::from_ptr( (*(*ifmt_ctx.as_ptr()).iformat).name ) }.to_str().unwrap();
    let ofmt_short_name = [c"webm", c"mp4"].iter().find(|&n| ifmt_short_name.contains(n.to_str().unwrap()))
        .expect("input file should be either webm or mp4");
    let ofmt = unsafe { ffi::av_guess_format(ofmt_short_name.as_ptr(), null(), null()) };
    if ofmt.is_null() {
        panic!("muxer {:?} not found", ofmt_short_name);
    }

    let ifmt_ctx_ref = unsafe { ifmt_ctx.as_ptr().as_ref().unwrap() };
    let istreams = unsafe { slice::from_raw_parts(ifmt_ctx_ref.streams, ifmt_ctx_ref.nb_streams as usize) };

    let mut audio_stream: Option<StreamInfo> = None;
    let mut video_stream: Option<StreamInfo> = None;

    for istream in istreams {
        let istream = unsafe { istream.as_ref().unwrap() };
        let codec_type = unsafe { (*istream.codecpar).codec_type };
        if codec_type == ffi::AVMEDIA_TYPE_AUDIO && audio_stream.is_none() {
            audio_stream = Some(StreamInfo {
                codec_params: istream.codecpar,
                input_index: istream.index,
            });
        } else if codec_type == ffi::AVMEDIA_TYPE_VIDEO && audio_stream.is_none() {
            video_stream = Some(StreamInfo {
                codec_params: istream.codecpar,
                input_index: istream.index,
            });
        } else {
            continue;
        }
    }

    ActiveFile {
        input: ifmt_ctx,
        output_format: ofmt,
        audio: audio_stream.unwrap(),
        video: video_stream.unwrap(),
    }
}


// for now, we will combine audio and video streams in the same fragment file; we do still plan
// on experimenting with separate a/v fragments, and ways of handling the small/sparse audio packets
// as described below. but let's just see if we can get this basic "dual" mode working for now.
// TODO: do we want to return Result<T, Error> at some point?
pub fn mux_next_dual(file: &mut ActiveFile, tag: i32, frag_size: u32) {
    let ifmt_ctx = &mut file.input;
    let mut ofmt_ctx = OutputFormatContext::new(WriteHandle::new(tag)).unwrap();
    unsafe { (*ofmt_ctx.as_ptr()).oformat = file.output_format }

    let audio_stream = unsafe { ffi::avformat_new_stream(ofmt_ctx.as_ptr(), null()) };
    panic_on_err! { ffi::avcodec_parameters_copy((*audio_stream).codecpar, file.audio.codec_params) };
    // line 129 in remux.c example; I'm not sure why this is necessary?
    unsafe { (*(*audio_stream).codecpar).codec_tag = 0; }

    let video_stream = unsafe { ffi::avformat_new_stream(ofmt_ctx.as_ptr(), null()) };
    panic_on_err! { ffi::avcodec_parameters_copy((*video_stream).codecpar, file.video.codec_params) };
    // line 129 in remux.c example; I'm not sure why this is necessary?
    unsafe { (*(*video_stream).codecpar).codec_tag = 0; }

    panic_on_err! { ffi::avformat_write_header(ofmt_ctx.as_ptr(), null_mut()) };

    let ifmt_ctx_ref = unsafe { ifmt_ctx.as_ptr().as_ref().unwrap() };
    let istreams = unsafe { slice::from_raw_parts(ifmt_ctx_ref.streams, ifmt_ctx_ref.nb_streams as usize) };

    let mut pkt = unsafe { ffi::av_packet_alloc() };
    if pkt.is_null() { panic!("Could not allocate AVPacket") }
    loop {
        let ret = unsafe { ffi::av_read_frame(ifmt_ctx.as_ptr(), pkt) };
        if ret < 0 {
            break;
        }
        let stream_index = unsafe { (*pkt).stream_index };
        let istream = istreams[stream_index as usize];
        let (ostream_index, ostream) =
            if stream_index == file.audio.input_index {
                (0, audio_stream)
            } else if stream_index == file.video.input_index {
                (1, video_stream)
            } else {
                unsafe { ffi::av_packet_unref(pkt) };
                continue;
            };
        unsafe {
            (*pkt).stream_index = ostream_index;
            ffi::av_packet_rescale_ts(pkt, (*istream).time_base, (*ostream).time_base);
            (*pkt).pos = -1;
        }
        panic_on_err! { ffi::av_interleaved_write_frame(ofmt_ctx.as_ptr(), pkt) };

        // TODO: this is probably not good enough; we probably need to worry about video
        // key frames? for now, do this, and see how it breaks. next we need to build the
        // JS frontend for running this and attempting playback.
        if ofmt_ctx.get_inner().size() > frag_size as u64 {
            break;
        }
    }

    panic_on_err! { ffi::av_write_trailer(ofmt_ctx.as_ptr()) };
    unsafe { ffi::av_packet_free(&mut pkt) };
}

pub fn mux_next_audio(file: &mut ActiveFile, tag: i32) {
    todo!();
}

pub fn mux_next_video(file: &mut ActiveFile, tag: i32) {
    todo!();
}



struct ActiveFile {
    input: InputFormatContext,
    // ex. mp4, webm
    output_format: *const ffi::AVOutputFormat,
    // ex. AAC, opus
    audio: StreamInfo,
    // ex. h264, VP9
    video: StreamInfo,
}

struct StreamInfo {
    codec_params: *mut ffi::AVCodecParameters,
    input_index: i32,
}


////////////////////////////////
// previous notes:
////////////////////////////////

// alternatively, we could have file_open return *mut RuntimeContext, which is passed to mux_next and seek_to
// this might be a better idea, to allow multiple files to be opened (for testing at least).
// static mut RT_CTX: Option<RuntimeContext> = None;

// yeah, mutable statics are weird and messy; let's just use a pointer



// so how are we handling the audio track? if we process packets and simultaneously write separate video and audio fragments,
// the video will quickly exceed the frag_size limit (higher bit rate), meaning the audio fragment will often not be
// "finished" yet. we could either finish the audio fragment early (there would be more files, so higher overhead?),
// or we could process audio and generate a fragment independently.
//
// one thought would be to process the audio of the entire file in a single first pass; I'm not sure if this would be doable.
// on the one hand, remuxing isn't CPU intensive; it's mostly copying memory. and for audio, we would be reading a lot
// more than we would be writing. however, reading from the file does require copying memory into WASM address space so
// that ffmpeg can access it and parse packets. so we might not want to process audio for the entire file, but just the
// next complete fragment.
//
// so I think we should have separate functions for muxing either the next audio or video fragment. the only tricky thing
// is how to manage "seeking"; we would expect video fragments to be muxed mostly sequentially; each one starting after
// the previous one, with no seeking required. but audio will most likely be non-sequential, after an audio fragment is
// muxed, the file will be seeked back in preparation for the next video fragment to be muxed.
//
// a major question I have about this process: are we able to perform seeking to a certain byte offset? that would allow
// us to precisely seek so that sequential video muxing can be restored after an audio fragment is muxed. muxing an audio
// fragment will occur infrequently but require reading packets far ahead from the previously muxed video fragment.
//
// so I think the API should have separate functions for muxing either audio or video fragments. the JS rt (runtime) will
// need to keep track of byte offsets and timestamps, and call either "seek_to_time" or "seek_to_offset" accordingly,
// before calling "mux_next_audio" or "mux_next_video".
//
// how will WASM tell JS about the byte offset or timestamp after a fragment is created? JS will also need a way of
// querying a media file after opening it, to see what streams the file has, and setting which streams to use for
// audio/video muxing. how do we communicate/propagate errors from WASM (ffmpeg) to JS? still a lot to figure out.



// these `extern "C" fn` should be in platform module, since they're specific to the WASM build

/*
// "file_id" is passed along to the JS runtime during calls to IoReadHandler::read and IoWriteHandler::write, so the
// JS runtime can keep track of which file is being operated on.
// "frag_size" sets the size limit of each fragment; when a fragment file exceeds this, a new fragment will be started
#[unsafe(no_mangle)]
pub extern "C" fn file_open(file_id: u32, frag_size: i64) -> *mut RuntimeContext {
    todo!();
}


// drop the InputFormatContext; TODO: double check that we're freeing ffmpeg things correctly
#[unsafe(no_mangle)]
pub extern "C" fn file_close(rt: *mut RuntimeContext) {
    todo!();
}


// what is the return value? JS will know how many bytes were written out; is this the end timestamp/offset of the fragment?
// this still needs to be determined.
#[unsafe(no_mangle)]
pub extern "C" fn mux_next_audio(rt: *mut RuntimeContext) -> i64 {
    todo!();
}

#[unsafe(no_mangle)]
pub extern "C" fn mux_next_video(rt: *mut RuntimeContext) -> i64 {
    todo!();
}

#[unsafe(no_mangle)]
pub extern "C" fn seek_to_time(rt: *mut RuntimeContext, ts: i64) -> i64 {
    todo!();
}

#[unsafe(no_mangle)]
pub extern "C" fn seek_to_offset(rt: *mut RuntimeContext, pos: i64) -> i64 {
    todo!();
}
*/

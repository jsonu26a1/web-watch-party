use rusty_ffmpeg::ffi;

use std::ffi::CStr;
use std::ptr::{ null, null_mut };
use std::slice;

use crate::platform::{ ReadHandle, WriteHandle };
use crate::context::{ InputFormatContext, OutputFormatContext };
use crate::err_abort_return;

// based off examples/remux.c
pub fn remux_example() {
    // unsafe { ffi::av_log_set_level(ffi::AV_LOG_TRACE as i32) };

    // remux audio and video to separate files
    let mut audio_ofmt_ctx = OutputFormatContext::new(WriteHandle::for_audio()).unwrap();
    let mut video_ofmt_ctx = OutputFormatContext::new(WriteHandle::for_video()).unwrap();

    // // hack to test with output file containing both audio and video
    // struct Tmp(*mut ffi::AVFormatContext);
    // impl Tmp {
    //     fn as_ptr(&self) -> *mut ffi::AVFormatContext {
    //         self.0
    //     }
    // }
    // let mut video_ofmt_ctx = Tmp(audio_ofmt_ctx.as_ptr());

    let mut ifmt_ctx = InputFormatContext::new(ReadHandle::new()).unwrap();

    err_abort_return! { ffi::avformat_find_stream_info(ifmt_ctx.as_ptr(), null_mut()) };

    /////////////

    // in the final app, we only output either .webm or .mp4 files. we can't easily map AVInputFormat to AVOutputFormat,
    // for the demo, we do this; but in the final app, audio and video might have different container formats, depending
    // on what codecs the input file has, and if any need to be re-encoded to support web playback.
    let ifmt_short_name = unsafe { CStr::from_ptr( (*(*ifmt_ctx.as_ptr()).iformat).name ) }.to_str().unwrap();
    let ofmt_short_name = [c"webm", c"mp4"].iter().find(|&n| ifmt_short_name.contains(n.to_str().unwrap()))
        .expect("input file should be either webm or mp4");
    // let ofmt_short_name = c"matroska";
    // let ofmt_short_name = c"webm_chunk";
    let ofmt = unsafe { ffi::av_guess_format(ofmt_short_name.as_ptr(), null(), null()) };
    if ofmt.is_null() {
        panic!("muxer {:?} not found", ofmt_short_name);
    }

    println!("muxer: {:?}", unsafe { CStr::from_ptr((*ofmt).name) });

    // for demo, we use the same format for both audio and video files
    unsafe {
        (*audio_ofmt_ctx.as_ptr()).oformat = ofmt;
        (*video_ofmt_ctx.as_ptr()).oformat = ofmt;
    }

    let ifmt_ctx_ref = unsafe { ifmt_ctx.as_ptr().as_ref().unwrap() };
    let istreams = unsafe { slice::from_raw_parts(ifmt_ctx_ref.streams, ifmt_ctx_ref.nb_streams as usize) };

    let mut audio_stream: Option<(i32, *mut ffi::AVStream, *mut ffi::AVFormatContext)> = None;
    let mut video_stream: Option<(i32, *mut ffi::AVStream, *mut ffi::AVFormatContext)> = None;

    for istream in istreams {
        let istream = unsafe { istream.as_ref().unwrap() };
        // let codec_type = unsafe { istream.codecpar.as_ref().unwrap().codec_type };
        let codec_type = unsafe { (*istream.codecpar).codec_type };
        let ostream;
        // we don't handle subtitles at the moment
        if codec_type == ffi::AVMEDIA_TYPE_AUDIO && audio_stream.is_none() {
            // so avformat_new_stream takes an AVCodec as an arg, why does the example pass in null?
            // ... because it's "unused, does nothing". wow, how interesting.
            ostream = unsafe { ffi::avformat_new_stream(audio_ofmt_ctx.as_ptr(), null()) };
            println!("audio stream (in {}), (out, {})", istream.index, unsafe { (*ostream).index });
            audio_stream = Some((istream.index, ostream, audio_ofmt_ctx.as_ptr() ));
        } else if codec_type == ffi::AVMEDIA_TYPE_VIDEO && video_stream.is_none() {
            ostream = unsafe { ffi::avformat_new_stream(video_ofmt_ctx.as_ptr(), null()) };
            println!("video stream (in {}), (out, {})", istream.index, unsafe { (*ostream).index });
            video_stream = Some((istream.index, ostream, video_ofmt_ctx.as_ptr() ));
        } else {
            continue;
        }
        if ostream.is_null() { panic!("Failed allocating output stream"); }

        err_abort_return! { ffi::avcodec_parameters_copy((*ostream).codecpar, istream.codecpar) };
        // line 129 in remux.c example; I'm not sure why this is necessary?
        unsafe { (*(*ostream).codecpar).codec_tag = 0; }
    }

    let audio_stream = audio_stream.unwrap();
    let video_stream = video_stream.unwrap();

    err_abort_return! { ffi::avformat_write_header(audio_ofmt_ctx.as_ptr(), null_mut()) };
    err_abort_return! { ffi::avformat_write_header(video_ofmt_ctx.as_ptr(), null_mut()) };

    let mut pkt = unsafe { ffi::av_packet_alloc() };
    if pkt.is_null() { panic!("Could not allocate AVPacket") }

    loop {
        let ret = unsafe { ffi::av_read_frame(ifmt_ctx.as_ptr(), pkt) };
        if ret < 0 {
            break;
        }
        let stream_index = unsafe { (*pkt).stream_index };
        let istream = istreams[stream_index as usize];
        let (ostream, ofmt_ctx);
        if stream_index == audio_stream.0 {
            (_, ostream, ofmt_ctx) = audio_stream;
        } else if stream_index == video_stream.0 {
            (_, ostream, ofmt_ctx) = video_stream;
        } else {
            unsafe { ffi::av_packet_unref(pkt) };
            continue;
        }

        // remux.c maps stream_index from input to output much different, but I think this is ok.
        unsafe { (*pkt).stream_index = (*ostream).index };

        // I'm unsure why we call this; shouldn't time_base be the same for input and output?
        unsafe { ffi::av_packet_rescale_ts(pkt, (*istream).time_base, (*ostream).time_base) };

        // println!("writing packet (stream {stream_index}) at {:?}", unsafe { (*pkt).pos });

        // idk what this does either
        unsafe { (*pkt).pos = -1 };

        err_abort_return! { ffi::av_interleaved_write_frame(ofmt_ctx, pkt) };
    }

    println!("done writing packets");

    // remux.c doesn't check the return value of this, but we should report any errors we encounter.
    // there's no point in the c example because it only frees objects and closes handles afterwards.
    err_abort_return! { ffi::av_write_trailer(audio_ofmt_ctx.as_ptr()) };
    err_abort_return! { ffi::av_write_trailer(video_ofmt_ctx.as_ptr()) };

    unsafe { ffi::av_packet_free(&mut pkt) };
}

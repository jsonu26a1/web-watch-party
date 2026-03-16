use rusty_ffmpeg::ffi;

use std::ffi::CStr;
use std::ptr::{ null, null_mut };
use std::slice;

use crate::platform::{ ReadHandle, WriteHandle };
use crate::context::{ InputFormatContext, OutputFormatContext };

#[macro_use]
#[path = "../../rust-ffmpeg-wasm/src/probe.rs"]
mod probe;
#[path = "../../rust-ffmpeg-wasm/src/context.rs"]
mod context;
mod platform;


fn main() {
    println!("Hello, world!");
    let cs = unsafe { CStr::from_ptr(ffi::av_version_info()) };
    println!("{}", cs.to_str().unwrap());
    // probe::probe_demo();
    remux_demo();
}

// based off examples/remux.c
fn remux_demo() {
    // remux audio and video to separate files
    let mut audio_ofmt_ctx = OutputFormatContext::new(WriteHandle::for_audio()).unwrap();
    let mut video_ofmt_ctx = OutputFormatContext::new(WriteHandle::for_video()).unwrap();
    let mut ifmt_ctx = InputFormatContext::new(ReadHandle::new()).unwrap();

    err_abort_return! { ffi::avformat_open_input(ifmt_ctx.as_mut(), null(), null(), null_mut()) };
    err_abort_return! { ffi::avformat_find_stream_info(ifmt_ctx.as_ptr(), null_mut()) };

    // we'll be getting codec info from streams in the input file, and copying those into the output format contexts
    // see line 117; `avformat_new_stream` followed by `avcodec_parameters_copy`.
    // we should figure out how to copy the container format from the input to the output;
    // the example just passes the file name to `avformat_alloc_output_context2`, but we're using AVIOContext
    // they call `avformat_open_input` followed by `avformat_find_stream_info`;

    // no wait, we do want to call `avformat_alloc_output_context2`; filename will be NULL, so we will need to supply
    // an AVOutputFormat or a format string.

    // I noticed on line 134; we're attempting to open the output file again? call to `avio_open`
    // that call passes in `ofmt_ctx->pb`, which is AVIOContext. apparently the example supports a URL as the output
    // filename, I guess it can "stream" the output over the network?
    // protocols: RTSP https://ffmpeg.org/ffmpeg-protocols.html#toc-rtsp tee, tcp, etc
    // so we don't need to worry about that, but it was interesting to look into this.

    /////////////

    // in `avformat_alloc_output_context2` a AVFormatContext is allocated written out to the pointer passed in;
    // since we want our AVFormatContext to have AVIOContext attached to it, we shouldn't use this method.
    // looking at it's implementation, we should assign `av_guess_format` to `AVFormatContext::oformat`.
    // `priv_data` is be initialized in both `avformat_alloc_output_context2` and `avformat_write_header`, so calling
    // the latter is enough.

    // this is wrong; the "output format" is the container format, aka muxer; not codec
    // const AUDIO_CODEC: &CStr = c"opus";
    // let audio_ofmt = unsafe { ffi::av_guess_format(AUDIO_CODEC.as_ptr(), null(), null()) };
    // if audio_ofmt.is_null() {
    //     panic!("muxer `{:?}` not found", AUDIO_CODEC);
    // }

    // const VIDEO_CODEC: &CStr = c"vp9";
    // let video_ofmt = unsafe { ffi::av_guess_format(VIDEO_CODEC.as_ptr(), null(), null()) };
    // if video_ofmt.is_null() {
    //     panic!("muxer `{:?}` not found", VIDEO_CODEC);
    // }

    // we use the same format for both audio and video files
    const OUTPUT_FORMAT: &CStr = c"webm";
    let ofmt = unsafe { ffi::av_guess_format(OUTPUT_FORMAT.as_ptr(), null(), null()) };
    if ofmt.is_null() {
        panic!("muxer `{:?}` not found", OUTPUT_FORMAT);
    }
    unsafe {
        (*audio_ofmt_ctx.as_ptr()).oformat = ofmt;
        (*video_ofmt_ctx.as_ptr()).oformat = ofmt;
    }

    let ifmt_ctx_ref = unsafe { ifmt_ctx.as_ptr().as_ref().unwrap() };
    let istreams = unsafe { slice::from_raw_parts(ifmt_ctx_ref.streams, ifmt_ctx_ref.nb_streams as usize) };

    let mut audio_stream: Option<(i32, *mut ffi::AVStream, *mut ffi::AVFormatContext)> = None;
    let mut video_stream: Option<(i32, *mut ffi::AVStream, *mut ffi::AVFormatContext)> = None;

    // let mut stream_mapping: Vec<(*mut ffi::AVStream, *mut ffi::AVFormatContext)> = vec![];
    // stream_mapping.resize_with(istreams.len(), Default::default);
    // let mut has_audio = false;
    // let mut has_video = false;

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
            audio_stream = Some((istream.index, ostream, audio_ofmt_ctx.as_ptr() ));
        } else if codec_type == ffi::AVMEDIA_TYPE_VIDEO && video_stream.is_none() {
            ostream = unsafe { ffi::avformat_new_stream(video_ofmt_ctx.as_ptr(), null()) };
            video_stream = Some((istream.index, ostream, video_ofmt_ctx.as_ptr() ));
        } else {
            continue;
        }
        if ostream.is_null() { panic!("Failed allocating output stream"); }
        // if codec_type == ffi::AVMEDIA_TYPE_AUDIO && !has_audio {
        //     has_audio = true;
        //     let stream = unsafe { ffi::avformat_new_stream(audio_ofmt_ctx.as_ptr(), null()) };
        //     if stream.is_null() {
        //         panic!("Failed allocating output stream");
        //     }
        //     stream_mapping[istream.index as usize] = (stream, audio_ofmt_ctx.as_ptr());
        // }
        // if codec_type == ffi::AVMEDIA_TYPE_VIDEO && !has_video {
        //     has_video = true;
        //     let stream = unsafe { ffi::avformat_new_stream(video_ofmt_ctx.as_ptr(), null()) };
        //     if stream.is_null() {
        //         panic!("Failed allocating output stream");
        //     }
        //     stream_mapping[istream.index as usize] = (stream, video_ofmt_ctx.as_ptr());
        // }

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

        // idk what this does either
        unsafe { (*pkt).pos = -1 };

        err_abort_return! { ffi::av_interleaved_write_frame(ofmt_ctx, pkt) };
    }

    // remux.c doesn't check the return value of this, but we should report any errors we encounter.
    // there's no point in the c example because it only free objects and closes handles afterwards.
    err_abort_return! { ffi::av_write_trailer(audio_ofmt_ctx.as_ptr()) };
    err_abort_return! { ffi::av_write_trailer(video_ofmt_ctx.as_ptr()) };

    unsafe { ffi::av_packet_free(&mut pkt) };

    // example calls `avformat_close_input`, but I'm not sure if we should call that; it appears to call
    // `FFInputStream.read_close`, which appears to clean up things specific to a particular AVInputFormat;
    // see
    // - https://github.com/FFmpeg/FFmpeg/blob/6ba0b59d8b014a311f94657557769040d8305327/libavformat/matroskadec.c#L4397
    // - https://github.com/FFmpeg/FFmpeg/blob/6ba0b59d8b014a311f94657557769040d8305327/libavformat/mov.c#L10110

    // so yeah, it looks like FFInputStream.read_close must be called to clean things up, and it is only called by
    // `avformat_close_input`; but that calls `avio_close`, which the docs say:
    // "This function can only be used if s was opened by avio_open()." (where `s` is AVIOContext)
    // `avio_close` treats AVIOContext.opaque as URLContext, which it definitely is not...
    // oddly enough, avio_read_callback.c calls `avformat_close_input`, so it should have the same issue.
    // I don't know what's going on here.

    // ah! here it is. in `avformat_close_input`, it sets `pb` (AVIOContext passed to avio_close) to null if:
    // `AVFormatContext.flags & AVFMT_FLAG_CUSTOM_IO`
    // this flag is set by `avformat_open_input`, when it sees we've set a custom AVIOContext.
    // I am unsure of the usage of `avio_open` and `avio_close`, but that doesn't matter right now.

    // we need to modify InputFormatContext so that `avformat_close_input` instead of `avformat_close_input`.
}

use rusty_ffmpeg::ffi;

use std::ffi::CStr;
use std::ptr::{ null, null_mut };
use std::slice;

use std::io::SeekFrom;

use crate::platform::{ ReadHandle, WriteHandle };
use crate::context::{ InputFormatContext, OutputFormatContext, ReadLogger, WriteLogger, IoReadHandler };
use crate::err_abort_return;

pub fn remux_audio_repeat() {
    let logger = ReadLogger::new(ReadHandle::new(), "input", true, true, false);
    let rls = logger.settings.clone();
    let mut ifmt_ctx = InputFormatContext::new(logger).unwrap();

    // let logger = WriteLogger::new(WriteHandle::new_tmp(), "output", true, true);
    // let wls = logger.settings.clone();
    // let mut ofmt_ctx = OutputFormatContext::new(logger).unwrap();

    err_abort_return! { ffi::avformat_find_stream_info(ifmt_ctx.as_ptr(), null_mut()) };

    let input_start_offset = ifmt_ctx.get_inner().seek(SeekFrom::Current(0)) as u64;

    // let ifmt_short_name = unsafe { CStr::from_ptr( (*(*ifmt_ctx.as_ptr()).iformat).name ) }.to_str().unwrap();
    // let ofmt_short_name = [c"webm", c"mp4"].iter().find(|&n| ifmt_short_name.contains(n.to_str().unwrap()))
    //     .expect("input file should be either webm or mp4");
    // let ofmt = unsafe { ffi::av_guess_format(ofmt_short_name.as_ptr(), null(), null()) };

    let ifmt_short_name = unsafe { CStr::from_ptr( (*(*ifmt_ctx.as_ptr()).iformat).name ) }.to_str().unwrap();
    let ofmt_short_name = [c"webm", c"mp4"].iter().find(|&n| ifmt_short_name.contains(n.to_str().unwrap()))
        .expect("input file should be either webm or mp4");
    let ofmt = unsafe { ffi::av_guess_format(ofmt_short_name.as_ptr(), null(), null()) };
    if ofmt.is_null() {
        panic!("muxer {:?} not found", ofmt_short_name);
    }

    // unsafe { (*ofmt_ctx.as_ptr()).oformat = ofmt };

    let ifmt_ctx_ref = unsafe { ifmt_ctx.as_ptr().as_ref().unwrap() };
    let istreams = unsafe { slice::from_raw_parts(ifmt_ctx_ref.streams, ifmt_ctx_ref.nb_streams as usize) };

    let mut istream_idx = None;

    for istream in istreams {
        let istream = unsafe { &**istream };
        // let codec_type = unsafe { istream.codecpar.as_ref().unwrap().codec_type };
        let codec_type = unsafe { (*istream.codecpar).codec_type };
        if codec_type == ffi::AVMEDIA_TYPE_AUDIO {
            istream_idx = Some(istream.index);
        }
    }

    let istream_idx = istream_idx.unwrap();
    let istream: &ffi::AVStream = unsafe { &*(istreams[istream_idx as usize]) };

    let mut pkt = unsafe { ffi::av_packet_alloc() };
    if pkt.is_null() { panic!("Could not allocate AVPacket") }

    for i in 0..3 {
        println!("writing tmp{i}...");
        let mut ofmt_ctx = OutputFormatContext::new(WriteHandle::new_tmp()).unwrap();
        unsafe { (*ofmt_ctx.as_ptr()).oformat = ofmt };

        let ostream = unsafe { ffi::avformat_new_stream(ofmt_ctx.as_ptr(), null()) };
        err_abort_return! { ffi::avcodec_parameters_copy((*ostream).codecpar, istream.codecpar) };
        // line 129 in remux.c example; I'm not sure why this is necessary?
        unsafe { (*(*ostream).codecpar).codec_tag = 0; }

        err_abort_return! { ffi::avformat_write_header(ofmt_ctx.as_ptr(), null_mut()) };

        loop {
            let ret = unsafe { ffi::av_read_frame(ifmt_ctx.as_ptr(), pkt) };
            if ret < 0 {
                break;
            }
            let stream_index = unsafe { (*pkt).stream_index };
            let istream = istreams[stream_index as usize];
            if stream_index != istream_idx {
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

            err_abort_return! { ffi::av_interleaved_write_frame(ofmt_ctx.as_ptr(), pkt) };
        }

        err_abort_return! { ffi::av_write_trailer(ofmt_ctx.as_ptr()) };

        // this doesn't work; ffmpeg doesn't realize the underlying AVIOContext is at a new position
        // see AVIOContext::pos
        // ifmt_ctx.get_inner().seek(SeekFrom::Start(input_start_offset));

        // this also doesn't work... looking at `avio_seek()` [aviobuf.c:L231], there's a lot of
        // conditional logic; some of it involves AVIOContext::eof_reached; maybe we should print out
        // that field; maybe setting it to "false" will make things work, but I'm not sure since there's
        // a lot going on with the other fields (buf_ptr, buf_end, buffer...)
        // another option would be to try `avformat_seek_file`/`av_seek_frame`
        unsafe { ffi::avio_seek((*ifmt_ctx.as_ptr()).pb, input_start_offset as i64, 0) };

        // ok, so `av_seek_frame` with flag `AVSEEK_FLAG_BYTE` just ends up calling `avio_seek`...
        // so I think if we actually need to support "undoing" that `eof_reached` state, we will
        // have to study AVIOContext a bit more so we can properly "restore" all the fields,
        // so we can actually support seeking once the file has reached EOF.

        // however, this isn't necessary right now; (a work around would be to "reopen" the file, by
        // creating a new ReadHandle;). I was also thinking about how we want to handle audio; the audio
        // stream is tiny in size compared to the video, so initially my plan was to remux the entire
        // stream into a file right up front; however, this wouldn't work in real time; we would
        // still need to read the entire file since audio packets are interlaced between video packets.
        // what we should focus on is starting and stopping remuxing based on timestamps/file sizes,
        // so we can buffer segments of video and audio for playback (and sending over the next).
        // then, also supporting seeking to arbitrary timestamps and restarting the buffering process
        // from a new position, and finally figuring out how to handle all these partial streams and
        // assemble them for playback in the browser.

        // (see below)
    }

    println!("finished.");
    unsafe { ffi::av_packet_free(&mut pkt) };
}

/*

so yeah, I think we should start building the web front end a bit, so we have a way of loading these
remuxed streams and playing them. we're building a "demo-server", where the web browser will make an
HTTP request and the ffmpeg wasm will be invoked via node, then the resulting fragment would be sent
back to the browser...

we're doing it this was so that the sample input media file can be loaded directly from the file system;
if we had the web server forward the file to the browser, the browser would need to either buffer the
entire file, or the front end would need to request only chunks of the file, which would be complex.
if we wanted the browser to use a local file (file upload input element), we would need to manually
select the sample file each time the page is reloaded.



*/
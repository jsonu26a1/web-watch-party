

struct RuntimeContext {
    input: InputFormatContext,
    /// I think we would be create these on demand, in `mux_next_[audio/video]`.
    // o_audio: OutputFormatContext,
    // o_video: OutputFormatContext,
    /// what other data do we want to persist between those `extern "C"` calls?
}

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


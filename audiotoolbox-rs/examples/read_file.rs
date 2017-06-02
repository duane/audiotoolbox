extern crate audiotoolbox_sys;
extern crate audiotoolbox;
extern crate core_foundation;
extern crate libc;

use audiotoolbox_sys::*;
use audiotoolbox::extended_audio_file::*;
use std::env::args;
use core_foundation::url::{kCFURLPOSIXPathStyle, CFURL};
use core_foundation::string::CFString;
use std::os::raw::c_void;
use std::slice;

fn main() {
    let argv: Vec<_> = args().collect();
    if argv.len() != 2 {
        panic!("USAGE: play AUDIO_FILE");
    }
    let file_url =
        CFURL::from_file_system_path(CFString::new(argv[1].as_ref()), kCFURLPOSIXPathStyle, false);
    let file_string = file_url.get_string();
    println!("File: {:?}", file_string);
    let mut audio_file = ExtAudioFile::open(file_url).expect("unable to open file");
    let client_description = match audio_file
              .get_property(ExtAudioFilePropertyId::ClientDataFormat)
              .expect("could not get data format") {
        ExtAudioFileProperty::ClientDataFormat(data_format) => data_format,
        _ => panic!("Expected ExtAudioFileProperty::ClientDataFormat"),
    };
    let description = match audio_file
              .get_property(ExtAudioFilePropertyId::FileDataFormat)
              .expect("could not get data format") {
        ExtAudioFileProperty::FileDataFormat(data_format) => data_format,
        _ => panic!("Expected ExtAudioFileProperty::FileDataFormat"),
    };
    println!("Got client data description: {:?}", client_description);
    println!("Got file data description: {:?}", description);
    let max_packet_size = match audio_file
              .get_property(ExtAudioFilePropertyId::ClientMaxPacketSize)
              .expect("could not get max packet size") {
        ExtAudioFileProperty::ClientMaxPacketSize(packet_size) => packet_size,
        _ => panic!("Expected ExtAudioFileProperty::ClientMaxPacketSize"),
    };
    println!("max packet size: {:?}", max_packet_size);

    let file_length_frames = match audio_file
              .get_property(ExtAudioFilePropertyId::FileLengthFrames)
              .expect("could not get file length frames property") {
        ExtAudioFileProperty::FileLengthFrames(frames) => frames,
        _ => panic!("expected ExtAudioFileProperty::FileLengthFrames"),
    };

    let buf_size = (file_length_frames * 4) as usize;
    println!("{:?} size buffer", buf_size);

    let mut raw_buffer: Vec<u8> = Vec::with_capacity(buf_size);
    for _ in 0..buf_size {
        raw_buffer.push(0);
    }
    let buffer = AudioBuffer {
        mNumberChannels: description.mChannelsPerFrame,
        mDataByteSize: raw_buffer.len() as u32,
        mData: raw_buffer.as_mut_ptr() as *mut c_void,
    };
    let mut list = AudioBufferList {
        mNumberBuffers: 1,
        mBuffers: [buffer],
    };

    println!("buffer: {:?}", list.mBuffers[0]);

    let mut out_buf: Vec<u8> = Vec::new();
    let mut frames_to_read = file_length_frames;
    let mut frames_read: u32;
    while frames_to_read > 0 {
        frames_read = audio_file
            .read(&mut list, list.mBuffers[0].mDataByteSize / 4)
            .expect("could not read from file");
        if frames_read == 0 {
            break;
        }
        println!("read {:?} frames", frames_read);
        println!("buffer: {:?}", list.mBuffers[0]);
        list.mBuffers[0].mDataByteSize = frames_read as u32 * 4;
        unsafe {
            let slice = slice::from_raw_parts(list.mBuffers[0].mData as *const u8,
                                              list.mBuffers[0].mDataByteSize as usize);
            out_buf.extend_from_slice(slice);
        }
        println!("out buf: {:?}", out_buf.len());
        println!("Frames to read: {:?}", frames_to_read);
        frames_to_read -= frames_read;
    }
    println!("done");
}

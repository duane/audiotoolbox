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
use std::mem;

static BUFFER_SIZE: usize = 1 << 14;

fn main() {
    let argv: Vec<_> = args().collect();
    if argv.len() != 2 {
        panic!("USAGE: play AUDIO_FILE");
    }
    let file_url =
        CFURL::from_file_system_path(CFString::new(argv[1].as_ref()), kCFURLPOSIXPathStyle, false);

    let mut audio_file = ExtAudioFile::open(file_url).expect("unable to open file");
    let bytes_per_channel = mem::size_of::<u16>() as u32;
    let channels_per_frame = 2;
    let frames_per_packet = 1;
    let bytes_per_frame = bytes_per_channel * channels_per_frame;
    let bytes_per_packet = bytes_per_frame * frames_per_packet;
    let bits_per_channel = bytes_per_channel * 8;
    let client_description = AudioStreamBasicDescription{
        mSampleRate: 41000f64,
        mFormatID: kAudioFormatLinearPCM as u32,
        mFormatFlags: kAudioFormatFlagIsPacked as u32 | kAudioFormatFlagIsBigEndian as u32 | kAudioFormatFlagIsSignedInteger as u32,
        mBytesPerPacket: bytes_per_packet,
        mFramesPerPacket: frames_per_packet,
        mBytesPerFrame: bytes_per_frame,
        mChannelsPerFrame: channels_per_frame,
        mBitsPerChannel: bits_per_channel,
        mReserved: 0,
    };
    audio_file.set_property(ExtAudioFileProperty::ClientDataFormat(client_description)).expect("client data fmt");
    let description = match audio_file
              .get_property(ExtAudioFilePropertyId::FileDataFormat)
              .expect("could not get data format") {
        ExtAudioFileProperty::FileDataFormat(data_format) => data_format,
        _ => panic!("Expected ExtAudioFileProperty::FileDataFormat"),
    };

    let file_length_frames = match audio_file
              .get_property(ExtAudioFilePropertyId::FileLengthFrames)
              .expect("could not get file length frames property") {
        ExtAudioFileProperty::FileLengthFrames(frames) => frames,
        _ => panic!("expected ExtAudioFileProperty::FileLengthFrames"),
    };

    let buf_size = BUFFER_SIZE;

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
        list.mBuffers[0].mDataByteSize = frames_read as u32 * 4;
        unsafe {
            let slice = slice::from_raw_parts(list.mBuffers[0].mData as *const u8,
                                              list.mBuffers[0].mDataByteSize as usize);
            out_buf.extend_from_slice(slice);
        }
        frames_to_read -= frames_read;
    }
    println!("done");
}

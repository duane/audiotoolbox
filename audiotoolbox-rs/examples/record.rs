extern crate audiotoolbox;
extern crate audiotoolbox_sys;
extern crate core_foundation;

use core_foundation::string::*;
use core_foundation::url::*;

use std::io;
use audiotoolbox::audio_file::*;
use audiotoolbox::audio_queue::*;
use audiotoolbox::audio_hardware_base::*;
use audiotoolbox_sys::*;
use std::env::args;
use std::os::raw::c_void;


pub unsafe extern "C" fn input_callback(user_data: *mut c_void,
                                        queue: AudioQueueRef,
                                        buffer: AudioQueueBufferRef,
                                        start_time: *const AudioTimeStamp,
                                        num_packets: u32,
                                        packet_desc: *const AudioStreamPacketDescription) {
}

struct Recorder {
    file: AudioFile,
    packet: i64,
    running: bool,
}

fn main() {
    let argv: Vec<_> = args().collect();
    if argv.len() != 2 {
        panic!("USAGE: record AUDIO_FILE");
    }
    let file_url =
        CFURL::from_file_system_path(CFString::new(argv[1].as_ref()), kCFURLPOSIXPathStyle, false);
    let mut format = AudioStreamBasicDescription {
        mSampleRate: AudioDevice::default()
            .expect("unable to find default input device")
            .get_sample_rate()
            .expect("unable to get input device sample rate"),
        mFormatID: kAudioFormatAppleLossless as u32,
        mFormatFlags: 0x0,
        mBytesPerPacket: 0,
        mFramesPerPacket: 0,
        mBytesPerFrame: 0,
        mChannelsPerFrame: 2,
        mBitsPerChannel: 0,
        mReserved: 0,
    };
    let mut recorder = Recorder {
        file: AudioFile::create(file_url,
                                kAudioFileCAFType as u32,
                                &mut format,
                                kAudioFileFlags_EraseFile as u32)
                .expect("error opening audio file"),
        packet: 0,
        running: false,
    };

    let mut queue = AudioQueue::new_input(Some(input_callback),
                                          &mut recorder as *mut _ as *mut c_void,
                                          &mut format)
            .expect("could not acquire new audio queue");
    let buffer_byte_size = queue
        .get_buffer_size(&format, 0.5)
        .expect("could not get expected buffer size");

    let num_record_buffers = 3;
    let buffers = Vec::<Buffer>::with_capacity(num_record_buffers);
    for i in 0..num_record_buffers {
        let mut buffer = Buffer::new(&mut queue, buffer_byte_size)
            .expect("could not allocate buffer");
        queue
            .enqueue_buffer(&mut buffer)
            .expect("Error enqueueing buffer");
    }
    recorder.running = true;
    queue.start().expect("error starting audio queue");
    println!("Recording, press <return> to stop");
    let mut input = String::new();
    io::stdin().read_line(&mut input);
    println!("* recording finished *");
    queue.stop(false).expect("unable to stop audio queue");
    
}
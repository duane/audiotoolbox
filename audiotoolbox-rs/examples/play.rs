extern crate audiotoolbox;
extern crate audiotoolbox_sys;
extern crate core_foundation;

use core_foundation::string::*;
use core_foundation::url::*;

use audiotoolbox::audio_file::*;
use audiotoolbox::audio_queue::*;
use audiotoolbox_sys::*;
use core_foundation::base::OSStatus;
use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoopRunInMode};
use std::env::args;
use std::os::raw::c_void;
use std::ptr;

static MAX_BUFFER_SIZE: usize = 0x10000;
static MIN_BUFFER_SIZE: usize = 0x4000;

pub struct Player {
    pub playback_file: AudioFile,
    pub packet_position: i64,
    pub num_packets_to_read: u32,
    pub packet_descriptions: Option<Vec<AudioStreamPacketDescription>>,
    pub is_done: bool,
}

pub unsafe extern "C" fn noop_output_callback(in_user_data: *mut c_void,
                                              in_aq: AudioQueueRef,
                                              in_buf: AudioQueueBufferRef) {
    let player: *mut Player = in_user_data as *mut Player;
    if (*player).is_done {
        return;
    }
    let mut num_bytes: u32 = 0;
    let mut n_packets: u32 = (*player).num_packets_to_read;
    let packet_descriptions: &mut Option<Vec<AudioStreamPacketDescription>> =
        &mut (*player).packet_descriptions;
    let mut status = AudioFileReadPackets((*player).playback_file.get_id(),
                                          false as u8,
                                          &mut num_bytes,
                                          (*packet_descriptions)
                                              .as_ref()
                                              .map(|v| v.as_ptr())
                                              .unwrap_or(ptr::null_mut()) as
                                          *mut AudioStreamPacketDescription,
                                          (*player).packet_position,
                                          &mut n_packets,
                                          (*in_buf).mAudioData as *mut c_void);
    if status != 0 {
        panic!("Got bad status: {:?}", status);
    } else {
        (*in_buf).mAudioDataByteSize = num_bytes;
    }
    if n_packets > 0 {
        status = AudioQueueEnqueueBuffer(in_aq,
                                         in_buf,
                                         (*packet_descriptions)
                                             .as_ref()
                                             .map(|v| v.len() as u32)
                                             .unwrap_or(0),
                                         (*packet_descriptions)
                                             .as_ref()
                                             .map(|v| v.as_ptr())
                                             .unwrap_or(ptr::null_mut()));
        if status != 0 {
            panic!("Got bad status: {:?}", status);
        }
        (*player).packet_position += n_packets as i64;
    } else {
        (*player).is_done = true;
        AudioQueue(in_aq)
            .stop(false)
            .expect("could not stop the queue");
    }
}

fn bytes_for_time(audio_file: &mut AudioFile,
                  desc: AudioStreamBasicDescription,
                  seconds: f64)
                  -> Result<(u32, u32), OSStatus> {
    let max_packet_size = match audio_file
              .get_property(AudioFilePropertyId::MaximumPacketSize)? {
        AudioFileProperty::MaximumPacketSize(packet_size) => packet_size,
        _ => panic!("Expected AudioFileProperty::MaximumPacketSize"),
    };
    let mut out_buffer_size: u32 = if desc.mFramesPerPacket > 0 {
        (desc.mSampleRate / desc.mFramesPerPacket as f64 * seconds) as u32 * max_packet_size
    } else {
        if MAX_BUFFER_SIZE as u32 > max_packet_size {
            max_packet_size
        } else {
            MAX_BUFFER_SIZE as u32
        }
    };

    if out_buffer_size > MAX_BUFFER_SIZE as u32 && out_buffer_size > max_packet_size {
        out_buffer_size = MAX_BUFFER_SIZE as u32
    } else if out_buffer_size < MIN_BUFFER_SIZE as u32 {
        out_buffer_size = MIN_BUFFER_SIZE as u32
    }
    Ok((out_buffer_size, out_buffer_size / max_packet_size))
}

fn main() {
    let argv: Vec<_> = args().collect();
    if argv.len() != 2 {
        panic!("USAGE: play AUDIO_FILE");
    }
    let file_url =
        CFURL::from_file_system_path(CFString::new(argv[1].as_ref()), kCFURLPOSIXPathStyle, false);
    let audio_file = AudioFile::open(file_url).unwrap();
    let data_format = match audio_file
              .get_property(AudioFilePropertyId::DataFormat)
              .expect("could not get audio file data format") {
        AudioFileProperty::DataFormat(fmt) => fmt,
        _ => panic!("expected AudioFileProperty::DataFormat"),
    };
    let mut player = Player {
        playback_file: audio_file,
        packet_position: 0,
        num_packets_to_read: 0,
        packet_descriptions: None,
        is_done: false,
    };
    let mut audio_queue = AudioQueue::new_output(Some(noop_output_callback),
                                                 &mut player as *mut Player as *mut c_void,
                                                 &data_format)
            .unwrap();
    let (buffer_byte_size, num_packets_to_read) =
        bytes_for_time(&mut player.playback_file, data_format, 0.5).unwrap();
    player.num_packets_to_read = num_packets_to_read;
    let is_format_vbr = data_format.mBytesPerPacket == 0 || data_format.mFramesPerPacket == 0;
    if is_format_vbr {
        let mut packet_descriptions = Vec::with_capacity(num_packets_to_read as usize);
        for _ in 0..num_packets_to_read {
            packet_descriptions.push(AudioStreamPacketDescription {
                                         mDataByteSize: 0,
                                         mStartOffset: 0,
                                         mVariableFramesInPacket: 0,
                                     });
        }
        player.packet_descriptions = Some(packet_descriptions);
    }
    audio_queue
        .copy_cookie_to_queue(&mut player.playback_file)
        .expect("could not copy cookie from file to queue");
    let playback_buffer_count = 3;
    let mut buffers: Vec<Buffer> = Vec::with_capacity(playback_buffer_count);
    for _ in 0..playback_buffer_count {
        let mut buf = Buffer::new(&mut audio_queue, buffer_byte_size).unwrap();
        unsafe {
            noop_output_callback(&mut player as *mut Player as *mut c_void,
                                 audio_queue.as_ref(),
                                 buf.as_ref());
        }
        buffers.push(buf);
    }

    audio_queue.start().unwrap();
    while !player.is_done {
        unsafe {
            CFRunLoopRunInMode(kCFRunLoopDefaultMode, 0.25, false as u8);
        }
    }
}

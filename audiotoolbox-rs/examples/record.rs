extern crate audiotoolbox;
extern crate audiotoolbox_sys;
extern crate core_foundation;

use core_foundation::string::*;
use core_foundation::url::*;

use audiotoolbox::audio_file::*;
use audiotoolbox::audio_hardware_base::*;
use audiotoolbox_sys::*;
use std::env::args;

struct Recorder {
  file: AudioFile,
  packet: i64,
  running: bool
}

fn compute_buffer_size(format: &AudioStreamBasicDescription, queue: &AudioQueue, seconds: f32) -> u32 {
  
}

fn main() {
    let argv: Vec<_> = args().collect();
  if argv.len() != 2 {
    panic!("USAGE: record AUDIO_FILE");
  }
  let file_url = CFURL::from_file_system_path(CFString::new(argv[1].as_ref()), kCFURLPOSIXPathStyle, false);
  let mut format = AudioStreamBasicDescription {
    mSampleRate: AudioDevice::default().expect("unable to find default input device").get_sample_rate().expect("unable to get input device sample rate"),
    mFormatID: kAudioFormatAppleLossless as u32,
    mFormatFlags: 0x0,
    mBytesPerPacket: 0,
    mFramesPerPacket: 0,
    mBytesPerFrame: 0,
    mChannelsPerFrame: 2,
    mBitsPerChannel: 0,
    mReserved: 0,
  };
  let recorder = Recorder{
    file: AudioFile::create(
      file_url,
      kAudioFileCAFType as u32,
      &mut format,
      kAudioFileFlags_EraseFile as u32
    ).expect("error opening audio file"),
    packet: 0,
    running: false
  };
  println!("{:?}", AudioDevice::default().unwrap().get_sample_rate().unwrap());
}
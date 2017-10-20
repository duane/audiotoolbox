extern crate audiotoolbox_sys;
extern crate audiotoolbox;
extern crate core_foundation;
extern crate spectrogram;

use audiotoolbox_sys::*;
use audiotoolbox::extended_audio_file::*;
use std::env::args;
use core_foundation::url::{kCFURLPOSIXPathStyle, CFURL};
use core_foundation::string::CFString;
use std::os::raw::c_void;
use std::slice;
use std::mem;

static BUFFER_SIZE: usize = 1 << 14;

type FrameChannel = f32;
type InterleavedStereoFrame = [FrameChannel; 2];
type MonoFrame = FrameChannel;

fn read_file(file: &str) -> Result<Vec<[f32; 2]>, String> {
    let file_url =
        CFURL::from_file_system_path(CFString::new(file), kCFURLPOSIXPathStyle, false);

    let mut audio_file = ExtAudioFile::open(file_url).expect("unable to open file");
    let bytes_per_channel = mem::size_of::<FrameChannel>() as u32;
    let channels_per_frame = 2;
    let frames_per_packet = 1;
    let bytes_per_frame = bytes_per_channel * channels_per_frame;
    let bytes_per_packet = bytes_per_frame * frames_per_packet;
    let bits_per_channel = bytes_per_channel * 8;
    let client_description = AudioStreamBasicDescription{
        mSampleRate: 41000f64,
        mFormatID: kAudioFormatLinearPCM as u32,
        mFormatFlags: kAudioFormatFlagIsFloat as u32,
        mBytesPerPacket: bytes_per_packet,
        mFramesPerPacket: frames_per_packet,
        mBytesPerFrame: bytes_per_frame,
        mChannelsPerFrame: channels_per_frame,
        mBitsPerChannel: bits_per_channel,
        mReserved: 0,
    };
    println!("Client description: {:?}", client_description);
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

    let mut out_buf: Vec<[f32; 2]> = Vec::new();
    let mut frames_to_read = file_length_frames;
    let mut frames_read: u32;
    while frames_to_read > 0 {
        frames_read = audio_file
            .read(&mut list, list.mBuffers[0].mDataByteSize / bytes_per_frame)
            .expect("could not read from file");
        if frames_read == 0 {
            break;
        }
        list.mBuffers[0].mDataByteSize = frames_read as u32 * bytes_per_frame;
        unsafe {
            let slice = slice::from_raw_parts(list.mBuffers[0].mData as *const [f32; 2],
                                              list.mBuffers[0].mDataByteSize as usize / bytes_per_frame as usize);
            out_buf.extend_from_slice(slice);
        }
        frames_to_read -= frames_read;
    }
    Ok(out_buf)
}



fn main() {
    let argv: Vec<_> = args().collect();
    if argv.len() != 2 {
        panic!("USAGE: play AUDIO_FILE");
    }

    let out_buf = read_file(argv[1].as_ref()).expect("Unable to read file");

    println!("done");
    println!("Extracting left channel;");
    let mut first_channel = Vec::<f32>::with_capacity(out_buf.len() / 2);
    let mut second_channel = Vec::<f32>::with_capacity(out_buf.len() / 2);
    for frame in out_buf {
      first_channel.push(frame[0]);
      second_channel.push(frame[1]);
    }
    println!("doing spectrogram thing");
    spectrogram(&first_channel);
}


extern crate image;

use std::fs::File;
use std::path::Path;


use spectrogram::stft;

pub fn spectrogram(signal: &Vec<f32>) {
    let window_size: usize = 2048;
    let spectrum_result = stft(signal, 48_000, window_size, 0);
    let mut spectrogram_out: Vec<Vec<f32>> = spectrum_result.v;
    println!("Spectrum result x bounds: {:?}", spectrum_result.x_axis_bounds_samples);
    println!("Spectrum result y bounds: {:?}", spectrum_result.y_axis_bounds_hz);
    println!("Spectrum result magnitude bounds: {:?}", spectrum_result.magnitude_bounds);

    println!("done.");
    println!("{:?}x{:?}", spectrogram_out.len(), spectrogram_out[0].len());
    let mut imgbuf = image::ImageBuffer::new(spectrogram_out.len() as u32, spectrogram_out[0].len() as u32);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let v = spectrogram_out[x as usize][(spectrogram_out[0].len() - 1) - y as usize];

        let power_param = (v - spectrum_result.magnitude_bounds[0]) / spectrum_result.magnitude_bounds[1];
        let rgb_pixel_f32 = rgb_lerp([0.1, 1.0, 1.0], [1.0, 0.1, 0.1], power_param);
        let rgb_pixel_u8: [u8; 3] = [(rgb_pixel_f32[0] * 255.0) as u8, (rgb_pixel_f32[1] * 255.0) as u8, (rgb_pixel_f32[2] * 255.0) as u8];

        *pixel = image::Rgb(rgb_pixel_u8);
    }

    let ref mut fout = File::create(&Path::new("spectrogram.png")).unwrap();

    // We must indicate the image’s color type and what format to save as
    image::ImageRgb8(imgbuf).save(fout, image::PNG);
}


fn rgb_lerp(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
  assert!(t >= 0.0);
  assert!(t <= 1.0);
  [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}
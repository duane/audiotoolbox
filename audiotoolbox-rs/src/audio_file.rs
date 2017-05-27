use std::ptr;
use std::mem;
use std::os::raw::c_void;
use audiotoolbox_sys::*;
use core_foundation_sys::base::*;
use core_foundation::base::TCFType;
use core_foundation::url::CFURL;

pub struct AudioFile(AudioFileID);

impl Drop for AudioFile {
  fn drop(&mut self) {
    unsafe {
      AudioFileClose(self.0);
    }
  }
}

impl AudioFile {
  pub fn open(file_url: CFURL) -> Result<AudioFile, OSStatus> {
    unsafe {
      let mut audio_file_ref: AudioFileID = ptr::null_mut();
      let status = AudioFileOpenURL(file_url.as_concrete_TypeRef(), 0x1, 0x0, &mut audio_file_ref);
      if status == 0 {
        Ok(AudioFile(audio_file_ref))
      } else {
        Err(status)
      }
    }
  }

  pub fn create(
    file_url: CFURL,
    file_type: AudioFileTypeID,
    format: &mut AudioStreamBasicDescription,
    flags: AudioFileFlags
  ) -> Result<AudioFile, OSStatus> {
    let mut audio_file_ref: AudioFileID = ptr::null_mut();
    let error = unsafe {
      AudioFileCreateWithURL(file_url.as_concrete_TypeRef(), file_type, format as *mut AudioStreamBasicDescription, flags, &mut audio_file_ref)
    };
    if error != 0 {
      Err(error)
    } else {
      Ok(AudioFile(audio_file_ref))
    }
  }

  pub fn as_ref(&mut self) -> AudioFileID {
    self.0
  }

  pub fn get_data_format(&mut self) -> Result<AudioStreamBasicDescription, OSStatus> {
    let mut asbd: AudioStreamBasicDescription = AudioStreamBasicDescription{
      mBitsPerChannel: 0,
      mBytesPerFrame: 0,
      mBytesPerPacket: 0,
      mChannelsPerFrame: 0,
      mFormatFlags: 0,
      mFormatID: 0,
      mFramesPerPacket: 0,
      mReserved: 0,
      mSampleRate: 0.0
    };
    let mut prop_size = mem::size_of::<AudioStreamBasicDescription>() as u32;
    let status = unsafe {
      AudioFileGetProperty(self.0, kAudioFilePropertyDataFormat as u32, &mut prop_size, &mut asbd as *mut _ as *mut c_void)
    };
    if status == 0 {
      Ok(asbd)
    } else {
      Err(status)
    }
  }

  pub fn get_packet_size_upper_bound(&mut self) -> Result<u32, OSStatus> {
    let mut max_packet_size: u32 = 0;
    let mut prop_size = mem::size_of::<u32>() as u32;
    let status = unsafe {
      AudioFileGetProperty(self.0, kAudioFilePropertyPacketSizeUpperBound as u32, &mut prop_size, &mut max_packet_size as *mut _ as *mut c_void)
    };
    if status == 0 {
      Ok(max_packet_size)
    } else {
      Err(status)
    }
  }

  pub fn get_magic_cookie(&mut self) -> Result<Option<Vec<u8>>, OSStatus> {
    let mut prop_size: u32 = 0;
    unsafe {
      if AudioFileGetPropertyInfo(self.0, kAudioFilePropertyMagicCookieData as u32, &mut prop_size, ptr::null_mut()) == 0 && prop_size > 0 {
        let cookie = Vec::with_capacity(prop_size as usize);
        let status = AudioFileGetProperty(self.0, kAudioFilePropertyMagicCookieData as u32, &mut prop_size, cookie.as_ptr() as *mut c_void);
        if status == 0 {
          return Ok(Some(cookie));
        } else {
          return Err(status);
        }
      }
    }
    Ok(None)
  }
}


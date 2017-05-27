#![macro_use]

use core_foundation::base::OSStatus;

use audiotoolbox_sys::*;
use std::os::raw::c_void;
use std::ptr;
use audio_file::AudioFile;

pub struct AudioQueue(pub AudioQueueRef);
pub struct Buffer(AudioQueueBufferRef);

impl Drop for AudioQueue {
  fn drop(&mut self) {
    unsafe {
      AudioQueueDispose(self.0, true as u8);
    }
  }
}

impl Buffer {
  pub fn new(queue: &mut AudioQueue, size: u32) -> Result<Buffer, OSStatus> {
    let mut buffer: AudioQueueBufferRef = ptr::null_mut();
    let status = unsafe {
    AudioQueueAllocateBuffer(queue.0, size, &mut buffer)
    };
    if status == 0 {
      Ok(Buffer(buffer))
    } else {
      Err(status)
    }
  }

  pub fn as_ref(&mut self) -> AudioQueueBufferRef {
    self.0
  }
}

impl AudioQueue {
  pub fn as_ref(&mut self) -> AudioQueueRef {
    self.0
  }

  pub fn new_output(callback: AudioQueueOutputCallback, user_data: *mut c_void, data_format: &AudioStreamBasicDescription) -> Result<AudioQueue, OSStatus> {
    let mut queue: AudioQueueRef = ptr::null_mut();
    let status = unsafe {
      AudioQueueNewOutput(
        data_format,
        callback,
        user_data,
        ptr::null_mut(),
        ptr::null_mut(),
        0,
        &mut queue
      )
    };
    if status == 0 {
      Ok(AudioQueue(queue))
    } else {
      Err(status)
    }
  }

  pub fn set_magic_cookie(&mut self, cookie: Vec<u8>) -> Result<(), OSStatus> {
    let status = unsafe {
      AudioQueueSetProperty(self.0, kAudioQueueProperty_MagicCookie as u32, cookie.as_ptr() as *mut c_void, cookie.len() as u32)
    };
    if status == 0 {
      Ok(())
    } else {
      Err(status)
    }
  }

  pub fn copy_cookie_to_queue(&mut self, file: &mut AudioFile) -> Result<(), OSStatus> {
    match file.get_magic_cookie()? {
      Some(cookie) => self.set_magic_cookie(cookie),
      None => Ok(())
    }
  }

  pub fn start(&mut self) -> Result<(), OSStatus> {
    let status = unsafe {
      AudioQueueStart(self.0, ptr::null())
    };
    if status == 0 {
      Ok(())
    } else {
      Err(status)
    }
  }

  pub fn stop(&mut self, synchronous: bool) -> Result<(), OSStatus> {
    let status = unsafe {
      AudioQueueStop(self.0, synchronous as u8)
    };
    if status == 0 {
      Ok(())
    } else {
      Err(status)
    }
  }
}

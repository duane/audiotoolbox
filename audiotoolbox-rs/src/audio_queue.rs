#![macro_use]

use core_foundation::base::OSStatus;

use audiotoolbox_sys::*;
use std::os::raw::c_void;
use std::ptr;
use audio_file::*;
use std::mem;
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
        let status = unsafe { AudioQueueAllocateBuffer(queue.0, size, &mut buffer) };
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

pub struct AudioQueueCallbackArgs<D: Sized> {
    pub data: *mut D,
    pub queue: *mut AudioQueue,
    pub buffer: *mut AudioQueueBuffer,
}

pub type NewOutputFn = FnMut(AudioQueueRef, *mut AudioQueueBuffer) -> Result<(), OSStatus>;
pub struct NewOutputFnWrapper {
    pub callback: Box<NewOutputFn>,
}

pub unsafe extern "C" fn new_output(in_user_data: *mut c_void,
                                    in_aq: AudioQueueRef,
                                    in_buf: AudioQueueBufferRef) {
    let wrapper = in_user_data as *mut NewOutputFnWrapper;
    match (*(*wrapper).callback)(in_aq, in_buf as *mut AudioQueueBuffer) {
        Err(err) => panic!(err),
        Ok(()) => (),
    }
}

impl AudioQueue {
    pub fn as_ref(&mut self) -> AudioQueueRef {
        self.0
    }

    pub fn new_output_new<D, F>(data_format: &AudioStreamBasicDescription,
                                data: D,
                                mut f: F)
                                -> Result<AudioQueue, OSStatus>
        where F: FnMut(AudioQueueCallbackArgs<D>) -> Result<(), OSStatus> + 'static,
              D: Sized + 'static
    {
        let data_box = Box::new(data);
        let data_box_ptr = Box::into_raw(data_box);
        let input_proc_fn = move |mut queue: AudioQueueRef,
                                  buffer: *mut AudioQueueBuffer|
              -> Result<(), OSStatus> {
            let audio_queue = &mut queue as *mut AudioQueueRef as *mut AudioQueue;
            let args = AudioQueueCallbackArgs {
                data: data_box_ptr,
                queue: audio_queue,
                buffer: buffer,
            };
            f(args)
        };
        let callback_wrapper = Box::new(NewOutputFnWrapper { callback: Box::new(input_proc_fn) });
        let callback_wrapper_ptr = Box::into_raw(callback_wrapper) as *mut c_void;

        AudioQueue::new_output(Some(new_output), callback_wrapper_ptr, data_format)
    }

    pub fn new_output(callback: AudioQueueOutputCallback,
                      user_data: *mut c_void,
                      data_format: &AudioStreamBasicDescription)
                      -> Result<AudioQueue, OSStatus> {
        let mut queue: AudioQueueRef = ptr::null_mut();
        let error = unsafe {
            AudioQueueNewOutput(data_format,
                                callback,
                                user_data,
                                ptr::null_mut(),
                                ptr::null_mut(),
                                0,
                                &mut queue)
        };
        if error == 0 {
            Ok(AudioQueue(queue))
        } else {
            Err(error)
        }
    }

    pub fn new_input(callback: AudioQueueInputCallback,
                     user_data: *mut c_void,
                     data_format: &mut AudioStreamBasicDescription)
                     -> Result<AudioQueue, OSStatus> {
        let mut queue: AudioQueueRef = ptr::null_mut();
        let error = unsafe {
            AudioQueueNewInput(data_format,
                               callback,
                               user_data,
                               ptr::null_mut(),
                               ptr::null(),
                               0,
                               &mut queue)
        };
        if error != 0 {
            Err(error)
        } else {
            Ok(AudioQueue(queue))
        }
    }

    pub fn get_buffer_size(&self,
                           format: &AudioStreamBasicDescription,
                           seconds: f64)
                           -> Result<u32, OSStatus> {
        let frames = (seconds * format.mSampleRate).ceil() as u32;
        let bytes = if format.mBytesPerFrame > 0 {
            frames * format.mBytesPerFrame
        } else {
            let max_packet_size: u32 = if format.mBytesPerPacket > 0 {
                format.mBytesPerPacket
            } else {
                let mut prop_size = mem::size_of::<u32>() as u32;
                let mut val: u32 = 0;
                let error = unsafe {
                    AudioQueueGetProperty(self.0,
                                          kAudioConverterPropertyMaximumOutputPacketSize as u32,
                                          &mut val as *mut _ as *mut c_void,
                                          &mut prop_size as *mut u32)
                };
                if error != 0 {
                    return Err(error);
                }
                val
            };
            let mut packets = if format.mFramesPerPacket > 0 {
                frames / format.mFramesPerPacket
            } else {
                frames
            };
            if packets == 0 {
                packets = 1;
            }
            packets * max_packet_size
        };
        Ok(bytes)
    }

    pub fn enqueue_buffer(&mut self, buffer: &mut Buffer) -> Result<(), OSStatus> {
        let error = unsafe { AudioQueueEnqueueBuffer(self.0, buffer.as_ref(), 0, ptr::null()) };
        if error != 0 { Err(error) } else { Ok(()) }
    }

    pub fn set_magic_cookie(&mut self, cookie: Vec<u8>) -> Result<(), OSStatus> {
        let status = unsafe {
            AudioQueueSetProperty(self.0,
                                  kAudioQueueProperty_MagicCookie as u32,
                                  cookie.as_ptr() as *mut c_void,
                                  cookie.len() as u32)
        };
        if status == 0 { Ok(()) } else { Err(status) }
    }

    pub fn copy_cookie_to_queue(&mut self, file: &mut AudioFile) -> Result<(), OSStatus> {
        match file.get_property(AudioFilePropertyId::MagicCookie)? {
            AudioFileProperty::MagicCookie(cookie) => self.set_magic_cookie(cookie),
            _ => Ok(()),
        }
    }

    pub fn copy_cookie_to_file(&mut self, file: &mut AudioFile) -> Result<(), OSStatus> {
        match self.get_magic_cookie()? {
            Some(cookie) => file.set_magic_cookie(cookie),
            None => Ok(()),
        }
    }

    pub fn get_magic_cookie(&mut self) -> Result<Option<Vec<u8>>, OSStatus> {
        let mut prop_size: u32 = 0;
        let mut error = unsafe {
            AudioQueueGetPropertySize(self.0,
                                      kAudioConverterCompressionMagicCookie as u32,
                                      &mut prop_size as *mut u32)
        };
        if error != 0 {
            return Err(error);
        }
        if prop_size == 0 {
            return Ok(None);
        }
        let mut magic_cookie = Vec::with_capacity(prop_size as usize);
        for _ in 0..prop_size {
            magic_cookie.push(0);
        }
        error = unsafe {
            AudioQueueGetProperty(self.0,
                                  kAudioQueueProperty_MagicCookie as u32,
                                  magic_cookie.as_ptr() as *mut _,
                                  &mut prop_size)
        };
        if error != 0 {
            return Err(error);
        }
        Ok(Some(magic_cookie))
    }

    pub fn start(&mut self) -> Result<(), OSStatus> {
        let status = unsafe { AudioQueueStart(self.0, ptr::null()) };
        if status == 0 { Ok(()) } else { Err(status) }
    }

    pub fn stop(&mut self, synchronous: bool) -> Result<(), OSStatus> {
        let status = unsafe { AudioQueueStop(self.0, synchronous as u8) };
        if status == 0 { Ok(()) } else { Err(status) }
    }
}

#![macro_use]

use core_foundation::base::OSStatus;

use audiotoolbox_sys::*;
use std::os::raw::c_void;
use std::ptr;
use std::mem;
use core_foundation::url::CFURL;
use core_foundation::base::TCFType;
use std::iter;

pub struct ExtAudioFile(ExtAudioFileRef);

pub enum ExtAudioFileProperty {
    FileDataFormat(AudioStreamBasicDescription),
    ClientDataFormat(AudioStreamBasicDescription),
    FileMaxPacketSize(u32),
    ClientMaxPacketSize(u32),
    FileLengthFrames(u32),
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ExtAudioFilePropertyId {
    FileDataFormat = 1717988724,
    ClientDataFormat = 1667657076,
    FileMaxPacketSize = 1718448243,
    ClientMaxPacketSize = 1668116595,
    FileLengthFrames = 593916525,
}

impl ExtAudioFile {
    pub fn open(url: CFURL) -> Result<ExtAudioFile, OSStatus> {
        let mut ext_audio_file_ref: ExtAudioFileRef = ptr::null_mut();
        let error =
            unsafe { ExtAudioFileOpenURL(url.as_concrete_TypeRef(), &mut ext_audio_file_ref) };
        if error != 0 {
            Err(error)
        } else {
            Ok(ExtAudioFile(ext_audio_file_ref))
        }
    }

    pub fn read(&mut self,
                buffers: *mut AudioBufferList,
                num_frames: u32)
                -> Result<u32, OSStatus> {
        let mut frames_read: u32 = num_frames;
        let error = unsafe { ExtAudioFileRead(self.0, &mut frames_read, buffers) };
        if error != 0 {
            Err(error)
        } else {
            Ok(frames_read)
        }
    }

    pub fn write(&mut self,
                 buffers: &mut AudioBufferList,
                 num_frames: u32)
                 -> Result<u32, OSStatus> {
        let mut frames_write: u32 = num_frames;
        let error =
            unsafe { ExtAudioFileRead(self.0, &mut frames_write, buffers as *mut AudioBufferList) };
        if error != 0 {
            Err(error)
        } else {
            Ok(frames_write)
        }
    }

    pub fn set_property(&mut self, property: ExtAudioFileProperty) -> Result<(), OSStatus> {
        let error = match property {
            ExtAudioFileProperty::ClientDataFormat(mut desc) => {
                unsafe {
                    ExtAudioFileSetProperty(self.0,
                                            kExtAudioFileProperty_ClientDataFormat as u32,
                                            mem::size_of::<AudioStreamBasicDescription>() as u32,
                                            &mut desc as *mut _ as *mut c_void)
                }
            }
            _ => panic!("cannot write anything but ClientDataFormat")
        };
        if error != 0 { Err(error) } else { Ok(()) }
    }

    pub fn get_property(&self,
                        property: ExtAudioFilePropertyId)
                        -> Result<ExtAudioFileProperty, OSStatus> {
        let (mut size, mut writable) = (0, 0);
        let mut error = unsafe {
            ExtAudioFileGetPropertyInfo(self.0, property as u32, &mut size, &mut writable)
        };
        if error != 0 {
            return Err(error);
        }
        let mut data: Vec<u8> = iter::repeat(0).take(size as usize).collect();
        error = unsafe {
            ExtAudioFileGetProperty(self.0,
                                    property as u32,
                                    &mut size,
                                    data.as_mut_ptr() as *mut c_void)
        };
        if error != 0 {
            return Err(error);
        }
        match property {
            ExtAudioFilePropertyId::FileDataFormat => unsafe {
                let asbd_ptr: *const AudioStreamBasicDescription = mem::transmute(data.as_ptr());
                Ok(ExtAudioFileProperty::FileDataFormat(*asbd_ptr))
            },
            ExtAudioFilePropertyId::ClientDataFormat => unsafe {
                let asbd_ptr: *const AudioStreamBasicDescription = mem::transmute(data.as_ptr());
                Ok(ExtAudioFileProperty::ClientDataFormat(*asbd_ptr))
            },
            ExtAudioFilePropertyId::FileMaxPacketSize => unsafe {
                let max_packet_size: *const u32 = mem::transmute(data.as_ptr());
                Ok(ExtAudioFileProperty::FileMaxPacketSize(*max_packet_size))
            },
            ExtAudioFilePropertyId::ClientMaxPacketSize => unsafe {
                let max_packet_size: *const u32 = mem::transmute(data.as_ptr());
                Ok(ExtAudioFileProperty::ClientMaxPacketSize(*max_packet_size))
            },
            ExtAudioFilePropertyId::FileLengthFrames => unsafe {
                let file_length_frames: *const u32 = mem::transmute(data.as_ptr());
                Ok(ExtAudioFileProperty::FileLengthFrames(*file_length_frames))
            },
        }
    }

    pub fn get_data_format(&mut self) -> Result<AudioStreamBasicDescription, OSStatus> {
        let mut asbd: AudioStreamBasicDescription = AudioStreamBasicDescription {
            mBitsPerChannel: 0,
            mBytesPerFrame: 0,
            mBytesPerPacket: 0,
            mChannelsPerFrame: 0,
            mFormatFlags: 0,
            mFormatID: 0,
            mFramesPerPacket: 0,
            mReserved: 0,
            mSampleRate: 0.0,
        };
        let mut prop_size = mem::size_of::<AudioStreamBasicDescription>() as u32;
        let error = unsafe {
            ExtAudioFileGetProperty(self.0,
                                    kExtAudioFileProperty_FileDataFormat as u32,
                                    &mut prop_size,
                                    &mut asbd as *mut _ as *mut c_void)
        };
        if error == 0 { Ok(asbd) } else { Err(error) }
    }
}

impl Drop for ExtAudioFile {
    fn drop(&mut self) {
        let error = unsafe { ExtAudioFileDispose(self.0) };
        if error != 0 {
            panic!(format!("Got error while dropping file: {:?}", error));
        }
    }
}

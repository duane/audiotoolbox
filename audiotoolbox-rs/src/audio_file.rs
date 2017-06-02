use std::ptr;
use std::mem;
use std::os::raw::c_void;
use std::iter;
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

pub enum AudioFileProperty {
    DataFormat(AudioStreamBasicDescription),
    FileFormat(AudioFileTypeId),
    MagicCookie(Vec<u8>),
    MaximumPacketSize(u32),
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AudioFilePropertyId {
    FileFormat = 1717988724,
    DataFormat = 1684434292,
    MagicCookie = 1835493731,
    MaximumPacketSize = 1886616165,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum AudioFileTypeId {
    AIFF = 1095321158,
    AIFC = 1095321155,
    WAVE = 1463899717,
    SoundDesigner2 = 1399075430,
    Next = 1315264596,
    MP3 = 1297106739,
    MP2 = 1297106738,
    MP1 = 1297106737,
    AC3 = 1633889587,
    AAC_ADTS = 1633973363,
    MPEG4 = 1836069990,
    M4A = 1832149350,
    M4B = 1832149606,
    CAF = 1667327590,
    _3GP = 862417008,
    _3GP2 = 862416946,
    AMR = 1634562662,
}

impl AudioFileTypeId {
    pub fn from_u32(v: u32) -> Result<AudioFileTypeId, ()> {
        match v {
            1095321158 => Ok(AudioFileTypeId::AIFF),
            1095321155 => Ok(AudioFileTypeId::AIFC),
            1463899717 => Ok(AudioFileTypeId::WAVE),
            1399075430 => Ok(AudioFileTypeId::SoundDesigner2),
            1315264596 => Ok(AudioFileTypeId::Next),
            1297106739 => Ok(AudioFileTypeId::MP3),
            1297106738 => Ok(AudioFileTypeId::MP2),
            1297106737 => Ok(AudioFileTypeId::MP1),
            1633889587 => Ok(AudioFileTypeId::AC3),
            1633973363 => Ok(AudioFileTypeId::AAC_ADTS),
            1836069990 => Ok(AudioFileTypeId::MPEG4),
            1832149350 => Ok(AudioFileTypeId::M4A),
            1832149606 => Ok(AudioFileTypeId::M4B),
            1667327590 => Ok(AudioFileTypeId::CAF),
            862417008 => Ok(AudioFileTypeId::_3GP),
            862416946 => Ok(AudioFileTypeId::_3GP2),
            1634562662 => Ok(AudioFileTypeId::AMR),
            _ => Err(()),
        }
    }
}

impl AudioFile {
    pub fn open(file_url: CFURL) -> Result<AudioFile, OSStatus> {
        unsafe {
            let mut audio_file_ref: AudioFileID = ptr::null_mut();
            let status = AudioFileOpenURL(file_url.as_concrete_TypeRef(),
                                          0x1,
                                          0x0,
                                          &mut audio_file_ref);
            if status == 0 {
                Ok(AudioFile(audio_file_ref))
            } else {
                Err(status)
            }
        }
    }

    pub fn create(file_url: CFURL,
                  file_type: AudioFileTypeID,
                  format: &mut AudioStreamBasicDescription,
                  flags: AudioFileFlags)
                  -> Result<AudioFile, OSStatus> {
        let mut audio_file_ref: AudioFileID = ptr::null_mut();
        let error = unsafe {
            AudioFileCreateWithURL(file_url.as_concrete_TypeRef(),
                                   file_type,
                                   format as *mut AudioStreamBasicDescription,
                                   flags,
                                   &mut audio_file_ref)
        };
        if error != 0 {
            Err(error)
        } else {
            Ok(AudioFile(audio_file_ref))
        }
    }

    pub fn get_id(&mut self) -> AudioFileID {
        self.0
    }

    pub fn get_property(&self,
                        property: AudioFilePropertyId)
                        -> Result<AudioFileProperty, OSStatus> {
        let (mut size, mut writable) = (0, 0);
        let mut error =
            unsafe { AudioFileGetPropertyInfo(self.0, property as u32, &mut size, &mut writable) };
        if error != 0 {
            return Err(error);
        }
        let mut data: Vec<u8> = iter::repeat(0).take(size as usize).collect();
        error = unsafe {
            AudioFileGetProperty(self.0,
                                 property as u32,
                                 &mut size,
                                 data.as_mut_ptr() as *mut c_void)
        };
        if error != 0 {
            return Err(error);
        }
        match property {
            AudioFilePropertyId::DataFormat => unsafe {
                let asbd_ptr: *const AudioStreamBasicDescription = mem::transmute(data.as_ptr());
                Ok(AudioFileProperty::DataFormat(*asbd_ptr))
            },
            AudioFilePropertyId::FileFormat => unsafe {
                let file_format_ptr: *const u32 = mem::transmute(data.as_ptr());
                Ok(AudioFileProperty::FileFormat(AudioFileTypeId::from_u32(*file_format_ptr)
                                                     .expect("do not recognize file format")))
            },
            AudioFilePropertyId::MagicCookie => Ok(AudioFileProperty::MagicCookie(data)),
            AudioFilePropertyId::MaximumPacketSize => unsafe {
                let max_packet_size: *const u32 = mem::transmute(data.as_ptr());
                Ok(AudioFileProperty::MaximumPacketSize(*max_packet_size))
            },
        }
    }

    pub fn set_magic_cookie(&mut self, magic_cookie: Vec<u8>) -> Result<(), OSStatus> {
        let error = unsafe {
            AudioFileSetProperty(self.0,
                                 kAudioFilePropertyMagicCookieData as u32,
                                 magic_cookie.len() as u32,
                                 magic_cookie.as_ptr() as *mut c_void)
        };
        if error != 0 { Err(error) } else { Ok(()) }
    }
}

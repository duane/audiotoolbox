#![macro_use]

use core_foundation::base::OSStatus;

use audiotoolbox_sys::*;
use std::os::raw::c_void;
use std::ptr;
use std::mem;
pub struct AudioDevice(AudioDeviceID);

impl AudioDevice {
    pub fn default_input() -> Result<AudioDevice, OSStatus> {
        let mut device_id: AudioDeviceID = 0;
        let mut device_id_size = mem::size_of::<AudioDeviceID>() as u32;
        let property_address = AudioObjectPropertyAddress {
            mSelector: kAudioHardwarePropertyDefaultInputDevice as u32,
            mScope: kAudioObjectPropertyScopeGlobal as u32,
            mElement: kAudioObjectPropertyElementMaster as u32,
        };

        let error = unsafe {
            AudioObjectGetPropertyData(kAudioObjectSystemObject as u32,
                                       &property_address,
                                       0,
                                       ptr::null(),
                                       &mut device_id_size,
                                       &mut device_id as *mut _ as *mut c_void)
        };
        if error != 0 {
            Err(error)
        } else {
            Ok(AudioDevice(device_id))
        }
    }

    pub fn get_sample_rate(&self) -> Result<f64, OSStatus> {
        let property_address = AudioObjectPropertyAddress {
            mElement: kAudioObjectPropertyElementMaster as u32,
            mScope: kAudioObjectPropertyScopeGlobal as u32,
            mSelector: kAudioDevicePropertyNominalSampleRate as u32,
        };
        let mut prop_size = mem::size_of::<f64>() as u32;
        let mut out_sample_rate: f64 = 0.0;
        let error = unsafe {
            AudioObjectGetPropertyData(self.0 as u32,
                                       &property_address,
                                       0,
                                       ptr::null(),
                                       &mut prop_size,
                                       &mut out_sample_rate as *mut _ as *mut c_void)
        };
        if error != 0 {
            Err(error)
        } else {
            Ok(out_sample_rate)
        }
    }
}

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

extern crate core_foundation_sys;
use core_foundation_sys::base::OSStatus;
use core_foundation_sys::url::CFURLRef;

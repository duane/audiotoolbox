extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
  // Tell cargo to tell rustc to link the system bzip2
  // shared library.
  println!("cargo:rustc-link-lib=framework=AudioToolbox");
  println!("cargo:rustc-link-lib=framework=CoreAudio");

  // The bindgen::Builder is the main entry point
  // to bindgen, and lets you build up options for
  // the resulting bindings.
  let bindings = bindgen::Builder::default()
    // Do not generate unstable Rust code that
    // requires a nightly rustc and enabling
    // unstable features.
    .no_unstable_rust()
    // The input header we would like to generate
    // bindings for.
    .header("wrapper.h")
    // Finish the builder and generate the bindings.
    .whitelisted_function("AudioFileOpenURL")
    .whitelisted_function("AudioFileCreateWithURL")
    .whitelisted_function("AudioFileClose")
    .whitelisted_function("AudioFileGetProperty")
    .whitelisted_function("AudioFileGetPropertyInfo")
    .whitelisted_function("AudioQueueSetProperty")
    .whitelisted_function("AudioQueueNewOutput")
    .whitelisted_function("AudioQueueDispose")
    .whitelisted_function("AudioQueueStart")
    .whitelisted_function("AudioQueueStop")
    .whitelisted_function("AudioQueueAllocateBuffer")
    .whitelisted_function("AudioFileReadPackets")
    .whitelisted_function("AudioQueueEnqueueBuffer")
    .whitelisted_function("AudioObjectGetPropertyData")
    .whitelisted_function("AudioObjectGetPropertyDataSize")
    .whitelisted_function("AudioHardwareGetProperty")
    .whitelisted_function("AudioHardwareServiceGetPropertyData")
    .hide_type("OSStatus")  
    .hide_type("CFURLRef")
    .whitelisted_type("AudioFileFlags")
    .whitelisted_type("AudioQueueOutputCallback")
    .whitelisted_type("AudioDeviceID")
    .whitelisted_type("AudioObjectPropertyAddress")
    .whitelisted_type("AudioStreamBasicDescription")
    .whitelisted_type("AudioStreamPacketDescription")
    .whitelisted_type("AudioFileID")
    .whitelisted_type("AudioFileTypeID")
    .whitelisted_type("AudioFilePermissions")
    .whitelisted_type("AudioQueueRef")
    .whitelisted_var("kAudioHardwarePropertyDevices")
    .whitelisted_var("kAudioQueueProperty_MagicCookie")
    .whitelisted_var("kAudioFilePropertyMagicCookieData")
    .whitelisted_var("kAudioFilePropertyDataFormat")
    .whitelisted_var("kAudioFilePropertyPacketSizeUpperBound")
    .whitelisted_var("kAudioFileAIFFType")
    .whitelisted_var("kAudioFileUnspecifiedError")
    .whitelisted_var("kAudioFileUnsupportedFileTypeError")
    .whitelisted_var("kAudioFileUnsupportedDataFormatError")
    .whitelisted_var("kAudioFileUnsupportedPropertyError")
    .whitelisted_var("kAudioFileBadPropertySizeError")
    .whitelisted_var("kAudioFilePermissionsError")
    .whitelisted_var("kAudioFileNotOptimizedError")
    .whitelisted_var("kAudioFileInvalidChunkError")
    .whitelisted_var("kAudioFileDoesNotAllow64BitDataSizeError")
    .whitelisted_var("kAudioFileInvalidPacketOffsetError")
    .whitelisted_var("kAudioFileInvalidFileError")
    .whitelisted_var("kAudioFileOperationNotSupportedError")
    .whitelisted_var("kAudioFileNotOpenError")
    .whitelisted_var("kAudioFileEndOfFileError")
    .whitelisted_var("kAudioFilePositionError")
    .whitelisted_var("kAudioFileFileNotFoundError")
    .whitelisted_var("kAudioHardwarePropertyDefaultInputDevice")
    .whitelisted_var("kAudioObjectPropertyScopeGlobal")
    .whitelisted_var("kAudioObjectSystemObject")
    .whitelisted_var("kAudioDevicePropertyNominalSampleRate")
    .whitelisted_var("kAudioObjectPropertyScopeGlobal")
    .whitelisted_var("kAudioFileFlags_EraseFile")
    .whitelisted_var("kAudioFileFlags_DontPageAlignAudioData")
    .whitelisted_var("kAudioFileAIFFType")
    .whitelisted_var("kAudioFileAIFCType")
    .whitelisted_var("kAudioFileWAVEType")
    .whitelisted_var("kAudioFileSoundDesigner2Type")
    .whitelisted_var("kAudioFileNextType")
    .whitelisted_var("kAudioFileMP3Type")
    .whitelisted_var("kAudioFileMP2Type")
    .whitelisted_var("kAudioFileMP1Type")
    .whitelisted_var("kAudioFileAC3Type")
    .whitelisted_var("kAudioFileAAC_ADTSType")
    .whitelisted_var("kAudioFileMPEG4Type")
    .whitelisted_var("kAudioFileM4AType")
    .whitelisted_var("kAudioFileCAFType")
    .whitelisted_var("kAudioFile3GPType")
    .whitelisted_var("kAudioFile3GP2Type")
    .whitelisted_var("kAudioFileAMRType")
    .whitelisted_var("kAudioFormatLinearPCM")
    .whitelisted_var("kAudioFormatAC3")
    .whitelisted_var("kAudioFormat60958AC3")
    .whitelisted_var("kAudioFormatAppleIMA4")
    .whitelisted_var("kAudioFormatMPEG4AAC")
    .whitelisted_var("kAudioFormatMPEG4CELP")
    .whitelisted_var("kAudioFormatMPEG4HVXC")
    .whitelisted_var("kAudioFormatMPEG4TwinVQ")
    .whitelisted_var("kAudioFormatMACE3")
    .whitelisted_var("kAudioFormatMACE6")
    .whitelisted_var("kAudioFormatULaw")
    .whitelisted_var("kAudioFormatALaw")
    .whitelisted_var("kAudioFormatQDesign")
    .whitelisted_var("kAudioFormatQDesign2")
    .whitelisted_var("kAudioFormatQUALCOMM")
    .whitelisted_var("kAudioFormatMPEGLayer1")
    .whitelisted_var("kAudioFormatMPEGLayer2")
    .whitelisted_var("kAudioFormatMPEGLayer3")
    .whitelisted_var("kAudioFormatTimeCode")
    .whitelisted_var("kAudioFormatMIDIStream")
    .whitelisted_var("kAudioFormatParameterValueStream")
    .whitelisted_var("kAudioFormatAppleLossless")
    .whitelisted_var("kAudioFormatMPEG4AAC_HE")
    .whitelisted_var("kAudioFormatMPEG4AAC_LD")
    .whitelisted_var("kAudioFormatMPEG4AAC_ELD")
    .whitelisted_var("kAudioFormatMPEG4AAC_ELD_SBR")
    .whitelisted_var("kAudioFormatMPEG4AAC_HE_V2")
    .whitelisted_var("kAudioFormatMPEG4AAC_Spatial")
    .whitelisted_var("kAudioFormatAMR")
    .whitelisted_var("kAudioFormatAudible")
    .whitelisted_var("kAudioFormatiLBC")
    .whitelisted_var("kAudioFormatDVIIntelIMA")
    .whitelisted_var("kAudioFormatMicrosoftGSM")
    .whitelisted_var("kAudioFormatAES3")
    .whitelisted_var("kAudioFormatAMR_WB")
    .whitelisted_var("kAudioFormatEnhancedAC3")
    .whitelisted_var("kAudioFormatMPEG4AAC_ELD_V2")

    .generate()
    // Unwrap the Result and panic on failure.
    .expect("Unable to generate bindings");

  // Write the bindings to the $OUT_DIR/bindings.rs file.
  let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
  bindings
    .write_to_file(out_path.join("bindings.rs"))
    .expect("Couldn't write bindings!");
}

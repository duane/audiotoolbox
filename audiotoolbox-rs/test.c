#include <stdio.h>
#include <AudioToolbox/AudioToolbox.h>

AudioDeviceID NewGetDefaultInputDevice()
{
    AudioDeviceID theAnswer = 0;
    UInt32 theSize = sizeof(AudioDeviceID);
    AudioObjectPropertyAddress theAddress = { kAudioHardwarePropertyDefaultInputDevice,
                                              kAudioObjectPropertyScopeGlobal,
                                              kAudioObjectPropertyElementMaster };

    OSStatus theError = AudioObjectGetPropertyData(kAudioObjectSystemObject,
                                                   &theAddress,
                                                   0,
                                                   NULL,
                                                   &theSize,
                                                   &theAnswer);
    // handle errors
    if (theError != 0) {
        fprintf(stderr, "Error: %d\n", theError);
        exit(1);
    }
    return theAnswer;
}

int main(int argc, char **argv) {
    printf("%d\n", NewGetDefaultInputDevice());
}

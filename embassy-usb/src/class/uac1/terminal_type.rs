//! USB Audio Terminal Types from Universal Serial Bus Device Class Definition
//! for Terminal Types, Release 1.0

/// USB Audio Terminal Types from "Universal Serial Bus Device Class Definition
/// for Terminal Types, Release 1.0"
#[repr(u16)]
#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[allow(missing_docs)]
pub enum TerminalType {
    // USB Terminal Types
    UsbUndefined = 0x0100,
    UsbStreaming = 0x0101,
    UsbVendor = 0x01ff,

    // Input Terminal Types
    InUndefined = 0x0200,
    InMicrophone = 0x0201,
    InDesktopMicrophone = 0x0202,
    InPersonalMicrophone = 0x0203,
    InOmniDirectionalMicrophone = 0x0204,
    InMicrophoneArray = 0x0205,
    InProcessingMicrophoneArray = 0x0206,

    // Output Terminal Types
    OutUndefined = 0x0300,
    OutSpeaker = 0x0301,
    OutHeadphones = 0x0302,
    OutHeadMountedDisplayAudio = 0x0303,
    OutDesktopSpeaker = 0x0304,
    OutRoomSpeaker = 0x0305,
    OutCommunicationSpeaker = 0x0306,
    OutLowFrequencyEffectsSpeaker = 0x0307,

    // External Terminal Types
    ExtUndefined = 0x0600,
    ExtAnalogConnector = 0x0601,
    ExtDigitalAudioInterface = 0x0602,
    ExtLineConnector = 0x0603,
    ExtLegacyAudioConnector = 0x0604,
    ExtSpdifConnector = 0x0605,
    Ext1394DaStream = 0x0606,
    Ext1394DvStreamSoundtrack = 0x0607,
}

impl From<TerminalType> for u16 {
    fn from(t: TerminalType) -> u16 {
        t as u16
    }
}

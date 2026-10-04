//! USB Audio Terminal Types from Universal Serial Bus Device Class Definition
//! for Terminal Types, Releases 1.0 and 2.0. Shared by the device and host classes.

/// USB Audio terminal types from Releases 1.0 and 2.0 of the Terminal Types specification.
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

    // Bi-directional Terminal Types
    BiUndefined = 0x0400,
    BiHandset = 0x0401,
    BiHeadset = 0x0402,
    BiSpeakerphoneNoEcho = 0x0403,
    BiEchoSuppressingSpeakerphone = 0x0404,
    BiEchoCancelingSpeakerphone = 0x0405,

    // Telephony Terminal Types
    TelUndefined = 0x0500,
    TelPhoneLine = 0x0501,
    TelTelephone = 0x0502,
    TelDownLinePhone = 0x0503,

    // External Terminal Types
    ExtUndefined = 0x0600,
    ExtAnalogConnector = 0x0601,
    ExtDigitalAudioInterface = 0x0602,
    ExtLineConnector = 0x0603,
    ExtLegacyAudioConnector = 0x0604,
    ExtSpdifConnector = 0x0605,
    Ext1394DaStream = 0x0606,
    Ext1394DvStreamSoundtrack = 0x0607,
    ExtAdatLightpipe = 0x0608,
    ExtTdif = 0x0609,
    ExtMadi = 0x060A,

    // Embedded Terminal Types
    Undefined = 0x0700,
    LevelCalibrationNoiseSource = 0x0701,
    EqualizationNoise = 0x0702,
    CdPlayer = 0x0703,
    DAT = 0x0704,
    DCC = 0x0705,
    MiniDisk = 0x0706,
    AnalogTape = 0x0707,
    Phonograph = 0x0708,
    VcrAudio = 0x0709,
    VideoDiscAudio = 0x070A,
    DvdAudio = 0x070B,
    TvTunerAudio = 0x070C,
    SatelliteReceiverAudio = 0x070D,
    CableTunerAudio = 0x070E,
    DssAudio = 0x070F,
    RadioReceiver = 0x0710,
    RadioTransmitter = 0x0711,
    MultiTrackRecorder = 0x0712,
    Synthesizer = 0x0713,
    Piano = 0x0714,
    Guitar = 0x0715,
    DrumsRhythm = 0x0716,
    OtherMusicalInstrument = 0x0717,
}

impl From<TerminalType> for u16 {
    fn from(t: TerminalType) -> u16 {
        t as u16
    }
}

impl TryFrom<u16> for TerminalType {
    type Error = u16;

    /// Returns the raw value for unknown or reserved terminal types.
    fn try_from(value: u16) -> Result<Self, u16> {
        Ok(match value {
            0x0100 => Self::UsbUndefined,
            0x0101 => Self::UsbStreaming,
            0x01ff => Self::UsbVendor,
            0x0200 => Self::InUndefined,
            0x0201 => Self::InMicrophone,
            0x0202 => Self::InDesktopMicrophone,
            0x0203 => Self::InPersonalMicrophone,
            0x0204 => Self::InOmniDirectionalMicrophone,
            0x0205 => Self::InMicrophoneArray,
            0x0206 => Self::InProcessingMicrophoneArray,
            0x0300 => Self::OutUndefined,
            0x0301 => Self::OutSpeaker,
            0x0302 => Self::OutHeadphones,
            0x0303 => Self::OutHeadMountedDisplayAudio,
            0x0304 => Self::OutDesktopSpeaker,
            0x0305 => Self::OutRoomSpeaker,
            0x0306 => Self::OutCommunicationSpeaker,
            0x0307 => Self::OutLowFrequencyEffectsSpeaker,
            0x0400 => Self::BiUndefined,
            0x0401 => Self::BiHandset,
            0x0402 => Self::BiHeadset,
            0x0403 => Self::BiSpeakerphoneNoEcho,
            0x0404 => Self::BiEchoSuppressingSpeakerphone,
            0x0405 => Self::BiEchoCancelingSpeakerphone,
            0x0500 => Self::TelUndefined,
            0x0501 => Self::TelPhoneLine,
            0x0502 => Self::TelTelephone,
            0x0503 => Self::TelDownLinePhone,
            0x0600 => Self::ExtUndefined,
            0x0601 => Self::ExtAnalogConnector,
            0x0602 => Self::ExtDigitalAudioInterface,
            0x0603 => Self::ExtLineConnector,
            0x0604 => Self::ExtLegacyAudioConnector,
            0x0605 => Self::ExtSpdifConnector,
            0x0606 => Self::Ext1394DaStream,
            0x0607 => Self::Ext1394DvStreamSoundtrack,
            0x0608 => Self::ExtAdatLightpipe,
            0x0609 => Self::ExtTdif,
            0x060A => Self::ExtMadi,
            0x0700 => Self::Undefined,
            0x0701 => Self::LevelCalibrationNoiseSource,
            0x0702 => Self::EqualizationNoise,
            0x0703 => Self::CdPlayer,
            0x0704 => Self::DAT,
            0x0705 => Self::DCC,
            0x0706 => Self::MiniDisk,
            0x0707 => Self::AnalogTape,
            0x0708 => Self::Phonograph,
            0x0709 => Self::VcrAudio,
            0x070A => Self::VideoDiscAudio,
            0x070B => Self::DvdAudio,
            0x070C => Self::TvTunerAudio,
            0x070D => Self::SatelliteReceiverAudio,
            0x070E => Self::CableTunerAudio,
            0x070F => Self::DssAudio,
            0x0710 => Self::RadioReceiver,
            0x0711 => Self::RadioTransmitter,
            0x0712 => Self::MultiTrackRecorder,
            0x0713 => Self::Synthesizer,
            0x0714 => Self::Piano,
            0x0715 => Self::Guitar,
            0x0716 => Self::DrumsRhythm,
            0x0717 => Self::OtherMusicalInstrument,
            other => return Err(other),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_every_code() {
        let mut known = 0;
        for value in 0..=u16::MAX {
            if let Ok(terminal_type) = TerminalType::try_from(value) {
                assert_eq!(u16::from(terminal_type), value);
                known += 1;
            }
        }
        assert_eq!(known, 63);
    }
}

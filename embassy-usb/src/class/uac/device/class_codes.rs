//! Audio Device Class Codes as defined in Universal Serial Bus Device Class
//! Definition for Audio Devices, Release 1.0, Appendix A and Universal Serial
//! Bus Device Class Definition for Audio Data Formats, Release 1.0, Appendix
//! A.1.1 (Audio Data Format Type I Codes)
#![allow(dead_code)]

/// The current version of the ADC specification (1.0)
pub const ADC_VERSION: u16 = 0x0100;

/// The current version of the USB device (1.0)
pub const DEVICE_VERSION: u16 = 0x0100;

/// Audio Interface Class Code
pub const USB_AUDIO_CLASS: u8 = 0x01;

// Audio Interface Subclass Codes
pub const USB_UNDEFINED_SUBCLASS: u8 = 0x00;
pub const USB_AUDIOCONTROL_SUBCLASS: u8 = 0x01;
pub const USB_AUDIOSTREAMING_SUBCLASS: u8 = 0x02;
pub const USB_MIDISTREAMING_SUBCLASS: u8 = 0x03;

// Audio Protocol Code
pub const PROTOCOL_NONE: u8 = 0x00;

// Audio Class-Specific Descriptor Types
pub const CS_UNDEFINED: u8 = 0x20;
pub const CS_DEVICE: u8 = 0x21;
pub const CS_CONFIGURATION: u8 = 0x22;
pub const CS_STRING: u8 = 0x23;
pub const CS_INTERFACE: u8 = 0x24;
pub const CS_ENDPOINT: u8 = 0x25;

// Descriptor Subtype
pub const AC_DESCRIPTOR_UNDEFINED: u8 = 0x00;
pub const HEADER_SUBTYPE: u8 = 0x01;
pub const INPUT_TERMINAL: u8 = 0x02;
pub const OUTPUT_TERMINAL: u8 = 0x03;
pub const MIXER_UNIT: u8 = 0x04;
pub const SELECTOR_UNIT: u8 = 0x05;
pub const FEATURE_UNIT: u8 = 0x06;
pub const PROCESSING_UNIT: u8 = 0x07;
pub const EXTENSION_UNIT: u8 = 0x08;

// Audio Class-Specific AS Interface Descriptor Subtypes
pub const AS_DESCRIPTOR_UNDEFINED: u8 = 0x00;
pub const AS_GENERAL: u8 = 0x01;
pub const FORMAT_TYPE: u8 = 0x02;
pub const FORMAT_SPECIFIC: u8 = 0x03;

// Processing Unit Process Types
pub const PROCESS_UNDEFINED: u16 = 0x00;
pub const UP_DOWNMIX_PROCESS: u16 = 0x01;
pub const DOLBY_PROLOGIC_PROCESS: u16 = 0x02;
pub const DDD_STEREO_EXTENDER_PROCESS: u16 = 0x03;
pub const REVERBERATION_PROCESS: u16 = 0x04;
pub const CHORUS_PROCESS: u16 = 0x05;
pub const DYN_RANGE_COMP_PROCESS: u16 = 0x06;

// Audio Class-Specific Endpoint Descriptor Subtypes
pub const EP_DESCRIPTOR_UNDEFINED: u8 = 0x00;
pub const EP_GENERAL: u8 = 0x01;

// Audio Class-Specific Request Codes
pub const REQUEST_CODE_UNDEFINED: u8 = 0x00;
pub const SET_CUR: u8 = 0x01;
pub const GET_CUR: u8 = 0x81;
pub const SET_MIN: u8 = 0x02;
pub const GET_MIN: u8 = 0x82;
pub const SET_MAX: u8 = 0x03;
pub const GET_MAX: u8 = 0x83;
pub const SET_RES: u8 = 0x04;
pub const GET_RES: u8 = 0x84;
pub const SET_MEM: u8 = 0x05;
pub const GET_MEM: u8 = 0x85;
pub const GET_STAT: u8 = 0xFF;

// Terminal Control Selectors
pub const TE_CONTROL_UNDEFINED: u8 = 0x00;
pub const COPY_PROTECT_CONTROL: u8 = 0x01;

// Feature Unit Control Selectors
pub const FU_CONTROL_UNDEFINED: u8 = 0x00;
pub const MUTE_CONTROL: u8 = 0x01;
pub const VOLUME_CONTROL: u8 = 0x02;
pub const BASS_CONTROL: u8 = 0x03;
pub const MID_CONTROL: u8 = 0x04;
pub const TREBLE_CONTROL: u8 = 0x05;
pub const GRAPHIC_EQUALIZER_CONTROL: u8 = 0x06;
pub const AUTOMATIC_GAIN_CONTROL: u8 = 0x07;
pub const DELAY_CONTROL: u8 = 0x08;
pub const BASS_BOOST_CONTROL: u8 = 0x09;
pub const LOUDNESS_CONTROL: u8 = 0x0A;

// Up/Down-mix Processing Unit Control Selectors
pub const UD_CONTROL_UNDEFINED: u8 = 0x00;
pub const UD_ENABLE_CONTROL: u8 = 0x01;
pub const UD_MODE_SELECT_CONTROL: u8 = 0x02;

// Dolby Prologic Processing Unit Control Selectors
pub const DP_CONTROL_UNDEFINED: u8 = 0x00;
pub const DP_ENABLE_CONTROL: u8 = 0x01;
pub const DP_MODE_SELECT_CONTROL: u8 = 0x2;

// 3D Stereo Extender Processing Unit Control Selectors
pub const DDD_CONTROL_UNDEFINED: u8 = 0x00;
pub const DDD_ENABLE_CONTROL: u8 = 0x01;
pub const DDD_SPACIOUSNESS_CONTROL: u8 = 0x03;

// Reverberation Processing Unit Control Selectors
pub const RV_CONTROL_UNDEFINED: u8 = 0x00;
pub const RV_ENABLE_CONTROL: u8 = 0x01;
pub const REVERB_LEVEL_CONTROL: u8 = 0x02;
pub const REVERB_TIME_CONTROL: u8 = 0x03;
pub const REVERB_FEEDBACK_CONTROL: u8 = 0x04;

// Chorus Processing Unit Control Selectors
pub const CH_CONTROL_UNDEFINED: u8 = 0x00;
pub const CH_ENABLE_CONTROL: u8 = 0x01;
pub const CHORUS_LEVEL_CONTROL: u8 = 0x02;
pub const CHORUS_RATE_CONTROL: u8 = 0x03;
pub const CHORUS_DEPTH_CONTROL: u8 = 0x04;

// Dynamic Range Compressor Processing Unit Control Selectors
pub const DR_CONTROL_UNDEFINED: u8 = 0x00;
pub const DR_ENABLE_CONTROL: u8 = 0x01;
pub const COMPRESSION_RATE_CONTROL: u8 = 0x02;
pub const MAXAMPL_CONTROL: u8 = 0x03;
pub const THRESHOLD_CONTROL: u8 = 0x04;
pub const ATTACK_TIME: u8 = 0x05;
pub const RELEASE_TIME: u8 = 0x06;

// Extension Unit Control Selectors
pub const XU_CONTROL_UNDEFINED: u16 = 0x00;
pub const XU_ENABLE_CONTROL: u16 = 0x01;

// Endpoint Control Selectors
pub const EP_CONTROL_UNDEFINED: u8 = 0x00;
pub const SAMPLING_FREQ_CONTROL: u8 = 0x01;
pub const PITCH_CONTROL: u8 = 0x02;

// Format Type Codes
pub const FORMAT_TYPE_UNDEFINED: u8 = 0x00;
pub const FORMAT_TYPE_I: u8 = 0x01;

// Audio Data Format Type I Codes
pub const TYPE_I_UNDEFINED: u16 = 0x0000;
pub const PCM: u16 = 0x0001;
pub const PCM8: u16 = 0x0002;
pub const IEEE_FLOAT: u16 = 0x0003;
pub const ALAW: u16 = 0x0004;
pub const MULAW: u16 = 0x0005;

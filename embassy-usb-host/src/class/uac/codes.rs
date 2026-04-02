//! USB Audio Device Class constants from UAC2 specification
//! Based on USB Device Class Definition for Audio Devices, Release 2.0 (May 31, 2006)

/// Audio Function Class Code
pub const AUDIO_FUNCTION: u8 = 0x01;

/// Audio Function Subclass Codes
pub const FUNCTION_SUBCLASS_UNDEFINED: u8 = 0x00;

/// Audio Function Protocol Codes
pub mod function_protocol {
    pub const UNDEFINED: u8 = 0x00;
    pub const AF_VERSION_02_00: u8 = 0x20;
}

/// Audio Interface Class/Subclass Codes
pub mod interface {
    pub const AUDIO: u8 = 0x01;

    pub mod subclass {
        pub const UNDEFINED: u8 = 0x00;
        pub const AUDIOCONTROL: u8 = 0x01;
        pub const AUDIOSTREAMING: u8 = 0x02;
        pub const MIDISTREAMING: u8 = 0x03;
    }

    pub mod protocol {
        pub const UNDEFINED: u8 = 0x00;
        pub const IP_VERSION_02_00: u8 = 0x20;
    }
}

/// Audio Function Category Codes
pub mod function_category {
    pub const UNDEFINED: u8 = 0x00;
    pub const DESKTOP_SPEAKER: u8 = 0x01;
    pub const HOME_THEATER: u8 = 0x02;
    pub const MICROPHONE: u8 = 0x03;
    pub const HEADSET: u8 = 0x04;
    pub const TELEPHONE: u8 = 0x05;
    pub const CONVERTER: u8 = 0x06;
    pub const VOICE_SOUND_RECORDER: u8 = 0x07;
    pub const IO_BOX: u8 = 0x08;
    pub const MUSICAL_INSTRUMENT: u8 = 0x09;
    pub const PRO_AUDIO: u8 = 0x0A;
    pub const AUDIO_VIDEO: u8 = 0x0B;
    pub const CONTROL_PANEL: u8 = 0x0C;
    pub const OTHER: u8 = 0xFF;
}

/// Audio Class-Specific Descriptor Types
pub mod descriptor_type {
    pub const CS_UNDEFINED: u8 = 0x20;
    pub const CS_DEVICE: u8 = 0x21;
    pub const CS_CONFIGURATION: u8 = 0x22;
    pub const CS_STRING: u8 = 0x23;
    pub const CS_INTERFACE: u8 = 0x24;
    pub const CS_ENDPOINT: u8 = 0x25;
}

/// Audio Class-Specific AC Interface Descriptor Subtypes
pub mod ac_descriptor {
    pub const UNDEFINED: u8 = 0x00;
    pub const HEADER: u8 = 0x01;
    pub const INPUT_TERMINAL: u8 = 0x02;
    pub const OUTPUT_TERMINAL: u8 = 0x03;
    pub const MIXER_UNIT: u8 = 0x04;
    pub const SELECTOR_UNIT: u8 = 0x05;
    pub const FEATURE_UNIT: u8 = 0x06;
    pub const EFFECT_UNIT: u8 = 0x07;
    pub const PROCESSING_UNIT: u8 = 0x08;
    pub const EXTENSION_UNIT: u8 = 0x09;
    pub const CLOCK_SOURCE: u8 = 0x0A;
    pub const CLOCK_SELECTOR: u8 = 0x0B;
    pub const CLOCK_MULTIPLIER: u8 = 0x0C;
    pub const SAMPLE_RATE_CONVERTER: u8 = 0x0D;
}

/// Audio Class-Specific AS Interface Descriptor Subtypes
pub mod as_descriptor {
    pub const UNDEFINED: u8 = 0x00;
    pub const GENERAL: u8 = 0x01;
    pub const FORMAT_TYPE: u8 = 0x02;
    pub const ENCODER: u8 = 0x03;
    pub const DECODER: u8 = 0x04;
}

/// Effect Unit Effect Types
pub mod effect_type {
    pub const UNDEFINED: u8 = 0x00;
    pub const PARAM_EQ_SECTION_EFFECT: u8 = 0x01;
    pub const REVERBERATION_EFFECT: u8 = 0x02;
    pub const MOD_DELAY_EFFECT: u8 = 0x03;
    pub const DYN_RANGE_COMP_EFFECT: u8 = 0x04;
}

/// Processing Unit Process Types
pub mod process_type {
    pub const UNDEFINED: u8 = 0x00;
    pub const UP_DOWNMIX_PROCESS: u8 = 0x01;
    pub const DOLBY_PROLOGIC_PROCESS: u8 = 0x02;
    pub const STEREO_EXTENDER_PROCESS: u8 = 0x03;
}

/// Audio Class-Specific Endpoint Descriptor Subtypes
pub mod endpoint_descriptor {
    pub const UNDEFINED: u8 = 0x00;
    pub const EP_GENERAL: u8 = 0x01;
}

/// Audio Class-Specific Request Codes
pub mod request_code {
    pub const UNDEFINED: u8 = 0x00;
    pub const CUR: u8 = 0x01;
    pub const RANGE: u8 = 0x02;
    pub const MEM: u8 = 0x03;
}

/// Encoder Type Codes
pub mod encoder_type {
    pub const UNDEFINED: u8 = 0x00;
    pub const OTHER_ENCODER: u8 = 0x01;
    pub const MPEG_ENCODER: u8 = 0x02;
    pub const AC_3_ENCODER: u8 = 0x03;
    pub const WMA_ENCODER: u8 = 0x04;
    pub const DTS_ENCODER: u8 = 0x05;
}

/// Decoder Type Codes
pub mod decoder_type {
    pub const UNDEFINED: u8 = 0x00;
    pub const OTHER_DECODER: u8 = 0x01;
    pub const MPEG_DECODER: u8 = 0x02;
    pub const AC_3_DECODER: u8 = 0x03;
    pub const WMA_DECODER: u8 = 0x04;
    pub const DTS_DECODER: u8 = 0x05;
}

/// Control Selector Codes
pub mod control_selector {
    /// Clock Source Control Selectors
    pub mod clock_source {
        pub const UNDEFINED: u16 = 0x00 << 8;
        pub const SAMPLING_FREQ_CONTROL: u16 = 0x01 << 8;
        pub const CLOCK_VALID_CONTROL: u16 = 0x02 << 8;
    }

    /// Clock Selector Control Selectors
    pub mod clock_selector {
        pub const UNDEFINED: u16 = 0x00 << 8;
        pub const CLOCK_SELECTOR_CONTROL: u16 = 0x01 << 8;
    }

    /// Clock Multiplier Control Selectors
    pub mod clock_multiplier {
        pub const UNDEFINED: u16 = 0x00 << 8;
        pub const NUMERATOR_CONTROL: u16 = 0x01 << 8;
        pub const DENOMINATOR_CONTROL: u16 = 0x02 << 8;
    }

    /// Terminal Control Selectors
    pub mod terminal {
        pub const UNDEFINED: u16 = 0x00 << 8;
        pub const COPY_PROTECT_CONTROL: u16 = 0x01 << 8;
        pub const CONNECTOR_CONTROL: u16 = 0x02 << 8;
        pub const OVERLOAD_CONTROL: u16 = 0x03 << 8;
        pub const CLUSTER_CONTROL: u16 = 0x04 << 8;
        pub const UNDERFLOW_CONTROL: u16 = 0x05 << 8;
        pub const OVERFLOW_CONTROL: u16 = 0x06 << 8;
        pub const LATENCY_CONTROL: u16 = 0x07 << 8;
    }

    /// Mixer Control Selectors
    pub mod mixer {
        pub const UNDEFINED: u16 = 0x00 << 8;
        pub const MIXER_CONTROL: u16 = 0x01 << 8;
        pub const CLUSTER_CONTROL: u16 = 0x02 << 8;
        pub const UNDERFLOW_CONTROL: u16 = 0x03 << 8;
        pub const OVERFLOW_CONTROL: u16 = 0x04 << 8;
        pub const LATENCY_CONTROL: u16 = 0x05 << 8;
    }

    /// Selector Control Selectors
    pub mod selector {
        pub const UNDEFINED: u16 = 0x00 << 8;
        pub const SELECTOR_CONTROL: u16 = 0x01 << 8;
        pub const LATENCY_CONTROL: u16 = 0x02 << 8;
    }

    /// Feature Unit Control Selectors
    pub mod feature_unit {
        pub const UNDEFINED: u16 = 0x00 << 8;
        pub const MUTE_CONTROL: u16 = 0x01 << 8;
        pub const VOLUME_CONTROL: u16 = 0x02 << 8;
        pub const BASS_CONTROL: u16 = 0x03 << 8;
        pub const MID_CONTROL: u16 = 0x04 << 8;
        pub const TREBLE_CONTROL: u16 = 0x05 << 8;
        pub const GRAPHIC_EQUALIZER_CONTROL: u8 = 0x06;
        pub const AUTOMATIC_GAIN_CONTROL: u16 = 0x07 << 8;
        pub const DELAY_CONTROL: u16 = 0x08 << 8;
        pub const BASS_BOOST_CONTROL: u16 = 0x09 << 8;
        pub const LOUDNESS_CONTROL: u16 = 0x0A << 8;
        pub const INPUT_GAIN_CONTROL: u16 = 0x0B << 8;
        pub const INPUT_GAIN_PAD_CONTROL: u16 = 0x0C << 8;
        pub const PHASE_INVERTER_CONTROL: u16 = 0x0D << 8;
        pub const UNDERFLOW_CONTROL: u16 = 0x0E << 8;
        pub const OVERFLOW_CONTROL: u16 = 0x0F << 8;
        pub const LATENCY_CONTROL: u16 = 0x10 << 8;
    }

    /// Effect Unit Control Selectors
    pub mod effect_unit {
        /// Parametric Equalizer Section Effect Unit Control Selectors
        pub mod parametric_equalizer {
            pub const UNDEFINED: u16 = 0x00 << 8;
            pub const ENABLE_CONTROL: u16 = 0x01 << 8;
            pub const CENTERFREQ_CONTROL: u16 = 0x02 << 8;
            pub const QFACTOR_CONTROL: u16 = 0x03 << 8;
            pub const GAIN_CONTROL: u16 = 0x04 << 8;
            pub const UNDERFLOW_CONTROL: u16 = 0x05 << 8;
            pub const OVERFLOW_CONTROL: u16 = 0x06 << 8;
            pub const LATENCY_CONTROL: u16 = 0x07 << 8;
        }

        /// Reverberation Effect Unit Control Selectors
        pub mod reverberation {
            pub const UNDEFINED: u16 = 0x00 << 8;
            pub const ENABLE_CONTROL: u16 = 0x01 << 8;
            pub const TYPE_CONTROL: u16 = 0x02 << 8;
            pub const LEVEL_CONTROL: u16 = 0x03 << 8;
            pub const TIME_CONTROL: u16 = 0x04 << 8;
            pub const FEEDBACK_CONTROL: u16 = 0x05 << 8;
            pub const PREDELAY_CONTROL: u16 = 0x06 << 8;
            pub const DENSITY_CONTROL: u16 = 0x07 << 8;
            pub const HIFREQ_ROLLOFF_CONTROL: u16 = 0x08 << 8;
            pub const UNDERFLOW_CONTROL: u16 = 0x09 << 8;
            pub const OVERFLOW_CONTROL: u16 = 0x0A << 8;
            pub const LATENCY_CONTROL: u16 = 0x0B << 8;
        }

        /// Modulation Delay Effect Unit Control Selectors
        pub mod modulation_delay {
            pub const UNDEFINED: u16 = 0x00 << 8;
            pub const ENABLE_CONTROL: u16 = 0x01 << 8;
            pub const BALANCE_CONTROL: u16 = 0x02 << 8;
            pub const RATE_CONTROL: u16 = 0x03 << 8;
            pub const DEPTH_CONTROL: u16 = 0x04 << 8;
            pub const TIME_CONTROL: u16 = 0x05 << 8;
            pub const FEEDBACK_CONTROL: u16 = 0x06 << 8;
            pub const UNDERFLOW_CONTROL: u16 = 0x07 << 8;
            pub const OVERFLOW_CONTROL: u16 = 0x08 << 8;
            pub const LATENCY_CONTROL: u16 = 0x09 << 8;
        }

        /// Dynamic Range Compressor Effect Unit Control Selectors
        pub mod dynamic_range_compressor {
            pub const UNDEFINED: u16 = 0x00 << 8;
            pub const ENABLE_CONTROL: u16 = 0x01 << 8;
            pub const COMPRESSION_RATE_CONTROL: u16 = 0x02 << 8;
            pub const MAXAMPL_CONTROL: u16 = 0x03 << 8;
            pub const THRESHOLD_CONTROL: u16 = 0x04 << 8;
            pub const ATTACK_TIME_CONTROL: u16 = 0x05 << 8;
            pub const RELEASE_TIME_CONTROL: u16 = 0x06 << 8;
            pub const UNDERFLOW_CONTROL: u16 = 0x07 << 8;
            pub const OVERFLOW_CONTROL: u16 = 0x08 << 8;
            pub const LATENCY_CONTROL: u16 = 0x09 << 8;
        }
    }

    /// Processing Unit Control Selectors
    pub mod processing_unit {
        /// Up/Down-mix Processing Unit Control Selectors
        pub mod up_downmix {
            pub const UNDEFINED: u16 = 0x00 << 8;
            pub const ENABLE_CONTROL: u16 = 0x01 << 8;
            pub const MODE_SELECT_CONTROL: u16 = 0x02 << 8;
            pub const CLUSTER_CONTROL: u16 = 0x03 << 8;
            pub const UNDERFLOW_CONTROL: u16 = 0x04 << 8;
            pub const OVERFLOW_CONTROL: u16 = 0x05 << 8;
            pub const LATENCY_CONTROL: u16 = 0x06 << 8;
        }

        /// Dolby Prologic Processing Unit Control Selectors
        pub mod dolby_prologic {
            pub const UNDEFINED: u16 = 0x00 << 8;
            pub const ENABLE_CONTROL: u16 = 0x01 << 8;
            pub const MODE_SELECT_CONTROL: u16 = 0x02 << 8;
            pub const CLUSTER_CONTROL: u16 = 0x03 << 8;
            pub const UNDERFLOW_CONTROL: u16 = 0x04 << 8;
            pub const OVERFLOW_CONTROL: u16 = 0x05 << 8;
            pub const LATENCY_CONTROL: u16 = 0x06 << 8;
        }

        /// Stereo Extender Processing Unit Control Selectors
        pub mod stereo_extender {
            pub const UNDEFINED: u16 = 0x00 << 8;
            pub const ENABLE_CONTROL: u16 = 0x01 << 8;
            pub const WIDTH_CONTROL: u16 = 0x02 << 8;
            pub const UNDERFLOW_CONTROL: u16 = 0x03 << 8;
            pub const OVERFLOW_CONTROL: u16 = 0x04 << 8;
            pub const LATENCY_CONTROL: u16 = 0x05 << 8;
        }
    }

    /// Extension Unit Control Selectors
    pub mod extension_unit {
        pub const UNDEFINED: u16 = 0x00 << 8;
        pub const ENABLE_CONTROL: u16 = 0x01 << 8;
        pub const CLUSTER_CONTROL: u16 = 0x02 << 8;
        pub const UNDERFLOW_CONTROL: u16 = 0x03 << 8;
        pub const OVERFLOW_CONTROL: u16 = 0x04 << 8;
        pub const LATENCY_CONTROL: u16 = 0x05 << 8;
    }

    /// AudioStreaming Interface Control Selectors
    pub mod audio_streaming {
        pub const UNDEFINED: u16 = 0x00 << 8;
        pub const ACT_ALT_SETTING_CONTROL: u16 = 0x01 << 8;
        pub const VAL_ALT_SETTINGS_CONTROL: u16 = 0x02 << 8;
        pub const AUDIO_DATA_FORMAT_CONTROL: u16 = 0x03 << 8;
    }

    /// Encoder Control Selectors
    pub mod encoder {
        pub const UNDEFINED: u16 = 0x00 << 8;
        pub const BIT_RATE_CONTROL: u16 = 0x01 << 8;
        pub const QUALITY_CONTROL: u16 = 0x02 << 8;
        pub const VBR_CONTROL: u16 = 0x03 << 8;
        pub const TYPE_CONTROL: u16 = 0x04 << 8;
        pub const UNDERFLOW_CONTROL: u16 = 0x05 << 8;
        pub const OVERFLOW_CONTROL: u16 = 0x06 << 8;
        pub const ENCODER_ERROR_CONTROL: u16 = 0x07 << 8;
        pub const PARAM1_CONTROL: u16 = 0x08 << 8;
        pub const PARAM2_CONTROL: u16 = 0x09 << 8;
        pub const PARAM3_CONTROL: u16 = 0x0A << 8;
        pub const PARAM4_CONTROL: u16 = 0x0B << 8;
        pub const PARAM5_CONTROL: u16 = 0x0C << 8;
        pub const PARAM6_CONTROL: u16 = 0x0D << 8;
        pub const PARAM7_CONTROL: u16 = 0x0E << 8;
        pub const PARAM8_CONTROL: u16 = 0x0F << 8;
    }

    /// Decoder Control Selectors
    pub mod decoder {
        /// MPEG Decoder Control Selectors
        pub mod mpeg {
            pub const UNDEFINED: u16 = 0x00 << 8;
            pub const DUAL_CHANNEL_CONTROL: u16 = 0x01 << 8;
            pub const SECOND_STEREO_CONTROL: u16 = 0x02 << 8;
            pub const MULTILINGUAL_CONTROL: u16 = 0x03 << 8;
            pub const DYN_RANGE_CONTROL: u16 = 0x04 << 8;
            pub const SCALING_CONTROL: u16 = 0x05 << 8;
            pub const HILO_SCALING_CONTROL: u16 = 0x06 << 8;
            pub const UNDERFLOW_CONTROL: u16 = 0x07 << 8;
            pub const OVERFLOW_CONTROL: u16 = 0x08 << 8;
            pub const DECODER_ERROR_CONTROL: u16 = 0x09 << 8;
        }

        /// AC-3 Decoder Control Selectors
        pub mod ac_3 {
            pub const UNDEFINED: u16 = 0x00 << 8;
            pub const MODE_CONTROL: u16 = 0x01 << 8;
            pub const DYN_RANGE_CONTROL: u16 = 0x02 << 8;
            pub const SCALING_CONTROL: u16 = 0x03 << 8;
            pub const HILO_SCALING_CONTROL: u16 = 0x04 << 8;
            pub const UNDERFLOW_CONTROL: u16 = 0x05 << 8;
            pub const OVERFLOW_CONTROL: u16 = 0x06 << 8;
            pub const DECODER_ERROR_CONTROL: u16 = 0x07 << 8;
        }

        /// WMA Decoder Control Selectors
        pub mod wma {
            pub const UNDEFINED: u16 = 0x00 << 8;
            pub const UNDERFLOW_CONTROL: u16 = 0x01 << 8;
            pub const OVERFLOW_CONTROL: u16 = 0x02 << 8;
            pub const DECODER_ERROR_CONTROL: u16 = 0x03 << 8;
        }

        /// DTS Decoder Control Selectors
        pub mod dts {
            pub const UNDEFINED: u16 = 0x00 << 8;
            pub const UNDERFLOW_CONTROL: u16 = 0x01 << 8;
            pub const OVERFLOW_CONTROL: u16 = 0x02 << 8;
            pub const DECODER_ERROR_CONTROL: u16 = 0x03 << 8;
        }
    }

    /// Endpoint Control Selectors
    pub mod endpoint {
        pub const UNDEFINED: u16 = 0x00 << 8;
        pub const PITCH_CONTROL: u16 = 0x01 << 8;
        pub const DATA_OVERRUN_CONTROL: u16 = 0x02 << 8;
        pub const DATA_UNDERRUN_CONTROL: u16 = 0x03 << 8;
    }
}

pub mod format_type {

    macro_rules! bitflags {
        ($($tt:tt)*) => {
            #[cfg(feature = "defmt")]
            defmt::bitflags! { $($tt)* }
            #[cfg(not(feature = "defmt"))]
            bitflags::bitflags! { #[derive(Debug, Clone, PartialEq)] $($tt)* }
        };
    }

    pub const UNDEFINED: u8 = 0x00;
    pub const I: u8 = 0x01;
    pub const II: u8 = 0x02;
    pub const III: u8 = 0x03;
    pub const IV: u8 = 0x04;
    pub const EXT_I: u8 = 0x81;
    pub const EXT_II: u8 = 0x82;
    pub const EXT_III: u8 = 0x83;

    bitflags! {
        pub struct Type1: u32 {
            const PCM = 1 << 0;
            const PCM8 = 1 << 1;
            const IEEE_FLOAT = 1 << 2;
            const ALAW = 1 << 3;
            const MULAW = 1 << 4;
            // Reserved. Must be set to 0. D30..D5
            const TYPE_I_RAW_DATA = 1 << 31;
        }
    }
    bitflags! {

        pub struct Type2: u32 {
            const MPEG = 1 << 0;
            const AC_3 = 1 << 1;
            const WMA = 1 << 2;
            const DTS = 1 << 3;
            // Reserved. Must be set to 0. D30..D4
            const TYPE_II_RAW_DATA = 1 << 31;
        }
    }

    bitflags! {
        pub struct Type3: u32 {
            const IEC61937_AC_3 = 1 << 0;
            const IEC61937_MPEG_1_LAYER1 = 1 << 1;
            const IEC61937_MPEG_1_LAYER2_3 = 1 << 2;
            const IEC61937_MPEG_2_NOEXT = 1 << 2;
            const IEC61937_MPEG_2_EXT = 1 << 3;
            const IEC61937_MPEG_2_AAC_ADTS = 1 << 4;
            const IEC61937_MPEG_2_LAYER1_LS = 1 << 5;
            const IEC61937_MPEG_2_LAYER2_3_LS = 1 << 6;
            const IEC61937_DTS_I = 1 << 7;
            const IEC61937_DTS_II = 1 << 8;
            const IEC61937_DTS_III = 1 << 9;
            const IEC61937_ATRAC = 1 << 10;
            const IEC61937_ATRAC2_3 = 1 << 11;
            const TYPE_III_WMA = 1 << 12;
            // Reserved. Must be set to 0. D31..D13
        }
    }

    bitflags! {
        pub struct Type4: u32 {
            const PCM = 1 << 0;
            const PCM8 = 1 << 1;
            const IEEE_FLOAT = 1 << 2;
            const ALAW = 1 << 3;
            const MULAW = 1 << 4;
            const MPEG = 1 << 5;
            const AC_3 = 1 << 6;
            const WMA = 1 << 7;
            const IEC61937_AC_3 = 1 << 8;
            const IEC61937_MPEG_1_LAYER1 = 1 << 9;
            const IEC61937_MPEG_1_LAYER2_3 = 1 << 10;
            const IEC61937_MPEG_2_NOEXT = 1 << 10;
            const IEC61937_MPEG_2_EXT = 1 << 11;
            const IEC61937_MPEG_2_AAC_ADTS = 1 << 12;
            const IEC61937_MPEG_2_LAYER1_LS = 1 << 13;
            const IEC61937_MPEG_2_LAYER2_3_LS = 1 << 14;
            const IEC61937_DTS_I = 1 << 15;
            const IEC61937_DTS_II = 1 << 16;
            const IEC61937_DTS_III = 1 << 17;
            const IEC61937_ATRAC = 1 << 18;
            const IEC61937_ATRAC2_3 = 1 << 19;
            const TYPE_III_WMA = 1 << 20;
            const IEC60958_PCM = 1 << 21;
            // Reserved. Must be set to 0. D31..D22
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    pub enum Format {
        Type1(Type1),
        Type2(Type2),
        Type3(Type3),
        Type4(Type4),
        Type1Extended(Type1),
        Type2Extended(Type2),
        Type3Extended(Type3),
    }

    impl Format {
        pub fn from_u32(format_type: u8, bitmap: u32) -> Option<Self> {
            match format_type {
                I => Some(Self::Type1(Type1::from_bits_truncate(bitmap))),
                II => Some(Self::Type2(Type2::from_bits_truncate(bitmap))),
                III => Some(Self::Type3(Type3::from_bits_truncate(bitmap))),
                IV => Some(Self::Type4(Type4::from_bits_truncate(bitmap))),
                EXT_I => Some(Self::Type1Extended(Type1::from_bits_truncate(bitmap))),
                EXT_II => Some(Self::Type2Extended(Type2::from_bits_truncate(bitmap))),
                EXT_III => Some(Self::Type3Extended(Type3::from_bits_truncate(bitmap))),
                _ => None,
            }
        }
    }
}

pub mod terminal_type {
    // USB Terminal Types (0x01xx)
    pub mod usb {
        pub const UNDEFINED: u16 = 0x0100;
        pub const STREAMING: u16 = 0x0101;
        pub const VENDOR_SPECIFIC: u16 = 0x01FF;
    }

    // Input Terminal Types (0x02xx)
    pub mod input {
        pub const UNDEFINED: u16 = 0x0200;
        pub const MICROPHONE: u16 = 0x0201;
        pub const DESKTOP_MICROPHONE: u16 = 0x0202;
        pub const PERSONAL_MICROPHONE: u16 = 0x0203;
        pub const OMNI_DIRECTIONAL_MICROPHONE: u16 = 0x0204;
        pub const MICROPHONE_ARRAY: u16 = 0x0205;
        pub const PROCESSING_MICROPHONE_ARRAY: u16 = 0x0206;
    }

    // Output Terminal Types (0x03xx)
    pub mod output {
        pub const UNDEFINED: u16 = 0x0300;
        pub const SPEAKER: u16 = 0x0301;
        pub const HEADPHONES: u16 = 0x0302;
        pub const HEAD_MOUNTED_DISPLAY_AUDIO: u16 = 0x0303;
        pub const DESKTOP_SPEAKER: u16 = 0x0304;
        pub const ROOM_SPEAKER: u16 = 0x0305;
        pub const COMMUNICATION_SPEAKER: u16 = 0x0306;
        pub const LOW_FREQUENCY_EFFECTS_SPEAKER: u16 = 0x0307;
    }

    // Bi-directional Terminal Types (0x04xx)
    pub mod bidirectional {
        pub const UNDEFINED: u16 = 0x0400;
        pub const HANDSET: u16 = 0x0401;
        pub const HEADSET: u16 = 0x0402;
        pub const SPEAKERPHONE_NO_ECHO: u16 = 0x0403;
        pub const ECHO_SUPPRESSING_SPEAKERPHONE: u16 = 0x0404;
        pub const ECHO_CANCELING_SPEAKERPHONE: u16 = 0x0405;
    }

    // Telephony Terminal Types (0x05xx)
    pub mod telephony {
        pub const UNDEFINED: u16 = 0x0500;
        pub const PHONE_LINE: u16 = 0x0501;
        pub const TELEPHONE: u16 = 0x0502;
        pub const DOWN_LINE_PHONE: u16 = 0x0503;
    }

    // External Terminal Types (0x06xx)
    pub mod external {
        pub const UNDEFINED: u16 = 0x0600;
        pub const ANALOG_CONNECTOR: u16 = 0x0601;
        pub const DIGITAL_AUDIO_INTERFACE: u16 = 0x0602;
        pub const LINE_CONNECTOR: u16 = 0x0603;
        pub const LEGACY_AUDIO_CONNECTOR: u16 = 0x0604;
        pub const SPDIF_INTERFACE: u16 = 0x0605;
        pub const DA_STREAM_1394: u16 = 0x0606;
        pub const DV_STREAM_SOUNDTRACK_1394: u16 = 0x0607;
        pub const ADAT_LIGHTPIPE: u16 = 0x0608;
        pub const TDIF: u16 = 0x0609;
        pub const MADI: u16 = 0x060A;
    }

    // Embedded Function Terminal Types (0x07xx)
    pub mod embedded {
        pub const UNDEFINED: u16 = 0x0700;
        pub const LEVEL_CALIBRATION_NOISE_SOURCE: u16 = 0x0701;
        pub const EQUALIZATION_NOISE: u16 = 0x0702;
        pub const CD_PLAYER: u16 = 0x0703;
        pub const DAT: u16 = 0x0704;
        pub const DCC: u16 = 0x0705;
        pub const COMPRESSED_AUDIO_PLAYER: u16 = 0x0706;
        pub const ANALOG_TAPE: u16 = 0x0707;
        pub const PHONOGRAPH: u16 = 0x0708;
        pub const VCR_AUDIO: u16 = 0x0709;
        pub const VIDEO_DISC_AUDIO: u16 = 0x070A;
        pub const DVD_AUDIO: u16 = 0x070B;
        pub const TV_TUNER_AUDIO: u16 = 0x070C;
        pub const SATELLITE_RECEIVER_AUDIO: u16 = 0x070D;
        pub const CABLE_TUNER_AUDIO: u16 = 0x070E;
        pub const DSS_AUDIO: u16 = 0x070F;
        pub const RADIO_RECEIVER: u16 = 0x0710;
        pub const RADIO_TRANSMITTER: u16 = 0x0711;
        pub const MULTI_TRACK_RECORDER: u16 = 0x0712;
        pub const SYNTHESIZER: u16 = 0x0713;
        pub const PIANO: u16 = 0x0714;
        pub const GUITAR: u16 = 0x0715;
        pub const DRUMS_RHYTHM: u16 = 0x0716;
        pub const OTHER_MUSICAL_INSTRUMENT: u16 = 0x0717;
    }
}

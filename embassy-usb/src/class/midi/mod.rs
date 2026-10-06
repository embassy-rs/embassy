//! USB MIDI 1.0 class.

mod packet;

pub mod device;
pub mod host;

pub use packet::*;

/// This should be used as `device_class` when building the `UsbDevice`.
pub const USB_AUDIO_CLASS: u8 = 0x01;
/// Maximum jacks or virtual cables per MIDIStreaming endpoint (4-bit cable number).
pub const MAX_MIDI_JACKS: usize = 16;

pub(crate) const USB_MIDISTREAMING_SUBCLASS: u8 = 0x03;
pub(crate) const PROTOCOL_NONE: u8 = 0x00;

// Class-specific MIDIStreaming interface descriptor subtypes (USB-MIDI 1.0 Table A-1).
pub(crate) const MS_HEADER_SUBTYPE: u8 = 0x01;
pub(crate) const MIDI_IN_JACK_SUBTYPE: u8 = 0x02;
pub(crate) const MIDI_OUT_JACK_SUBTYPE: u8 = 0x03;
// Class-specific MIDIStreaming endpoint descriptor subtype (Table A-2).
pub(crate) const MS_GENERAL: u8 = 0x01;

/// USB-MIDI specification release 1.0, in binary-coded decimal.
pub(crate) const MIDI_VERSION: u16 = 0x0100;

// Class-specific descriptor lengths, including bLength and bDescriptorType.
pub(crate) const MS_HEADER_LEN: u8 = 7;
pub(crate) const MIDI_IN_JACK_LEN: u8 = 6;
pub(crate) const MIDI_OUT_JACK_BASE_LEN: u8 = 7; // Plus 2 bytes per source pin.
pub(crate) const MS_GENERAL_BASE_LEN: u8 = 4; // Plus 1 byte per associated jack.

/// Whether a MIDI jack is inside the USB device or connected externally (USB-MIDI 1.0 Table A-3).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MidiJackType {
    /// Jack connected to a USB endpoint.
    Embedded = 0x01,
    /// Jack representing a physical or otherwise external MIDI connection.
    External = 0x02,
}

impl TryFrom<u8> for MidiJackType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, u8> {
        match value {
            0x01 => Ok(Self::Embedded),
            0x02 => Ok(Self::External),
            other => Err(other),
        }
    }
}

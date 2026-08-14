//! USB-MIDI 1.0 class-specific descriptor parsing.

use embassy_usb_driver::EndpointType;
use heapless::Vec;

use super::{USB_CLASS_AUDIO, USB_MIDI_1_PROTOCOL, USB_SUBCLASS_MIDI_STREAMING};
use crate::descriptor::descriptor_type::{CS_ENDPOINT, CS_INTERFACE, ENDPOINT};
use crate::descriptor::{
    ConfigurationDescriptorChain, EndpointDescriptor, InterfaceDescriptorChain, RawDescriptorIterator, USBDescriptor,
};

const MS_HEADER: u8 = 0x01;
const MIDI_IN_JACK: u8 = 0x02;
const MIDI_OUT_JACK: u8 = 0x03;
const MS_GENERAL: u8 = 0x01;

/// Maximum MIDIStreaming interfaces described by one configuration.
pub const MAX_MIDI_INTERFACES: usize = 8;
/// Maximum jacks or cables described by one MIDIStreaming interface.
pub const MAX_MIDI_JACKS: usize = 16;
/// Maximum physical bulk endpoints described by one MIDIStreaming interface.
pub const MAX_MIDI_ENDPOINTS: usize = 4;
/// Maximum input pins feeding one MIDI OUT jack.
pub const MAX_MIDI_JACK_SOURCES: usize = 16;

/// USB-MIDI class-specific descriptor parsing error.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MidiDescriptorError {
    /// A descriptor is truncated, inconsistent, or has an unexpected type.
    InvalidDescriptor,
    /// A configuration exceeds one of this driver's fixed capacities.
    Capacity,
}

/// Class-specific MIDIStreaming header descriptor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MidiStreamingHeader {
    /// USB-MIDI class specification release in binary-coded decimal.
    pub midi_version: u16,
    /// Total size of the class-specific MIDIStreaming descriptors.
    pub total_length: u16,
}

impl MidiStreamingHeader {
    /// Parse a class-specific MIDIStreaming header descriptor.
    pub fn try_from_bytes(raw: &[u8]) -> Result<Self, MidiDescriptorError> {
        check_descriptor_exact(raw, CS_INTERFACE, MS_HEADER, 7)?;
        Ok(Self {
            midi_version: u16::from_le_bytes([raw[3], raw[4]]),
            total_length: u16::from_le_bytes([raw[5], raw[6]]),
        })
    }
}

/// Whether a MIDI jack is inside the USB device or connected externally.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MidiJackType {
    /// Jack connected to a USB endpoint.
    Embedded,
    /// Jack representing a physical or otherwise external MIDI connection.
    External,
}

impl MidiJackType {
    fn try_from_byte(value: u8) -> Result<Self, MidiDescriptorError> {
        match value {
            0x01 => Ok(Self::Embedded),
            0x02 => Ok(Self::External),
            _ => Err(MidiDescriptorError::InvalidDescriptor),
        }
    }
}

/// MIDI IN jack descriptor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MidiInJack {
    /// Embedded or external jack.
    pub jack_type: MidiJackType,
    /// Jack identifier referenced by other class-specific descriptors.
    pub jack_id: u8,
    /// String descriptor index, or zero when absent.
    pub string_index: u8,
}

impl MidiInJack {
    /// Parse a class-specific MIDI IN jack descriptor.
    pub fn try_from_bytes(raw: &[u8]) -> Result<Self, MidiDescriptorError> {
        check_descriptor_exact(raw, CS_INTERFACE, MIDI_IN_JACK, 6)?;
        Ok(Self {
            jack_type: MidiJackType::try_from_byte(raw[3])?,
            jack_id: raw[4],
            string_index: raw[5],
        })
    }
}

/// Source pin feeding a MIDI OUT jack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MidiJackSource {
    /// Identifier of the source entity or jack.
    pub jack_id: u8,
    /// One-based output pin on the source entity.
    pub pin: u8,
}

/// MIDI OUT jack descriptor.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MidiOutJack {
    /// Embedded or external jack.
    pub jack_type: MidiJackType,
    /// Jack identifier referenced by other class-specific descriptors.
    pub jack_id: u8,
    /// Source pins feeding this jack.
    pub sources: Vec<MidiJackSource, MAX_MIDI_JACK_SOURCES>,
    /// String descriptor index, or zero when absent.
    pub string_index: u8,
}

impl MidiOutJack {
    /// Parse a class-specific MIDI OUT jack descriptor.
    pub fn try_from_bytes(raw: &[u8]) -> Result<Self, MidiDescriptorError> {
        check_descriptor(raw, CS_INTERFACE, MIDI_OUT_JACK, 7)?;
        let source_count = raw[5] as usize;
        let expected_len = 7usize
            .checked_add(
                source_count
                    .checked_mul(2)
                    .ok_or(MidiDescriptorError::InvalidDescriptor)?,
            )
            .ok_or(MidiDescriptorError::InvalidDescriptor)?;
        if raw.len() != expected_len {
            return Err(MidiDescriptorError::InvalidDescriptor);
        }

        let mut sources = Vec::new();
        for source in raw[6..6 + source_count * 2].chunks_exact(2) {
            sources
                .push(MidiJackSource {
                    jack_id: source[0],
                    pin: source[1],
                })
                .map_err(|_| MidiDescriptorError::Capacity)?;
        }

        Ok(Self {
            jack_type: MidiJackType::try_from_byte(raw[3])?,
            jack_id: raw[4],
            sources,
            string_index: raw[expected_len - 1],
        })
    }
}

/// Jack IDs associated with a class-specific MIDIStreaming endpoint descriptor.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MidiEndpointJackAssociations {
    /// Embedded jack IDs ordered by USB-MIDI cable number.
    pub jack_ids: Vec<u8, MAX_MIDI_JACKS>,
}

impl MidiEndpointJackAssociations {
    /// Parse a class-specific MIDIStreaming endpoint descriptor.
    pub fn try_from_bytes(raw: &[u8]) -> Result<Self, MidiDescriptorError> {
        check_descriptor(raw, CS_ENDPOINT, MS_GENERAL, 4)?;
        let expected_len = 4usize
            .checked_add(raw[3] as usize)
            .ok_or(MidiDescriptorError::InvalidDescriptor)?;
        if raw.len() != expected_len {
            return Err(MidiDescriptorError::InvalidDescriptor);
        }
        Ok(Self {
            jack_ids: Vec::from_slice(&raw[4..]).map_err(|_| MidiDescriptorError::Capacity)?,
        })
    }
}

/// A recognized descriptor within one USB-MIDI 1.0 streaming interface.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MidiDescriptor {
    /// Class-specific MIDIStreaming header.
    Header(MidiStreamingHeader),
    /// MIDI IN jack.
    InJack(MidiInJack),
    /// MIDI OUT jack.
    OutJack(MidiOutJack),
    /// Standard USB endpoint.
    Endpoint(EndpointDescriptor),
    /// Jack associations for the preceding standard endpoint.
    EndpointJackAssociations(MidiEndpointJackAssociations),
}

/// Iterator over recognized descriptors in one USB-MIDI 1.0 interface.
pub struct MidiDescriptorIterator<'a> {
    descriptors: RawDescriptorIterator<'a>,
}

impl<'a> MidiDescriptorIterator<'a> {
    /// Create an iterator when `interface` is a USB-MIDI 1.0 streaming interface.
    pub fn new(interface: &'a InterfaceDescriptorChain<'a>) -> Option<Self> {
        is_midi_streaming_interface(interface).then(|| Self {
            descriptors: interface.iter_descriptors(),
        })
    }
}

impl Iterator for MidiDescriptorIterator<'_> {
    type Item = Result<MidiDescriptor, MidiDescriptorError>;

    fn next(&mut self) -> Option<Self::Item> {
        for (_, raw) in self.descriptors.by_ref() {
            if raw.len() < 2 {
                return Some(Err(MidiDescriptorError::InvalidDescriptor));
            }
            let descriptor = match (raw[1], raw.get(2).copied()) {
                (CS_INTERFACE, Some(MS_HEADER)) => MidiStreamingHeader::try_from_bytes(raw).map(MidiDescriptor::Header),
                (CS_INTERFACE, Some(MIDI_IN_JACK)) => MidiInJack::try_from_bytes(raw).map(MidiDescriptor::InJack),
                (CS_INTERFACE, Some(MIDI_OUT_JACK)) => MidiOutJack::try_from_bytes(raw).map(MidiDescriptor::OutJack),
                (ENDPOINT, _) => EndpointDescriptor::try_from_bytes(raw)
                    .map(MidiDescriptor::Endpoint)
                    .map_err(|_| MidiDescriptorError::InvalidDescriptor),
                (CS_ENDPOINT, Some(MS_GENERAL)) => {
                    MidiEndpointJackAssociations::try_from_bytes(raw).map(MidiDescriptor::EndpointJackAssociations)
                }
                _ => continue,
            };
            return Some(descriptor);
        }
        None
    }
}

/// One physical USB bulk endpoint and its ordered virtual-cable jack IDs.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MidiEndpointDescriptor {
    /// USB endpoint address, including its direction bit.
    pub address: u8,
    /// Maximum USB packet size.
    pub max_packet_size: u16,
    /// Associated embedded jack IDs. Their positions are USB-MIDI cable numbers.
    pub jack_ids: Vec<u8, MAX_MIDI_JACKS>,
}

impl MidiEndpointDescriptor {
    /// Whether this endpoint transfers data from the device to the host.
    pub const fn is_in(&self) -> bool {
        self.address & 0x80 != 0
    }
}

/// Parsed USB-MIDI 1.0 streaming interface and its jack topology.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MidiStreamingInterface {
    /// Standard USB interface number.
    pub interface_number: u8,
    /// Standard USB alternate-setting number.
    pub alternate_setting: u8,
    /// Class-specific streaming header, when supplied by the device.
    pub header: Option<MidiStreamingHeader>,
    /// MIDI IN jacks declared by this interface.
    pub in_jacks: Vec<MidiInJack, MAX_MIDI_JACKS>,
    /// MIDI OUT jacks declared by this interface.
    pub out_jacks: Vec<MidiOutJack, MAX_MIDI_JACKS>,
    /// Physical bulk endpoints and their ordered cable associations.
    pub endpoints: Vec<MidiEndpointDescriptor, MAX_MIDI_ENDPOINTS>,
}

impl MidiStreamingInterface {
    /// Parse one MIDIStreaming interface and associate each endpoint with its jacks.
    pub fn try_from_interface(interface: &InterfaceDescriptorChain<'_>) -> Result<Self, MidiDescriptorError> {
        let descriptors = MidiDescriptorIterator::new(interface).ok_or(MidiDescriptorError::InvalidDescriptor)?;
        let mut result = Self {
            interface_number: interface.interface_number,
            alternate_setting: interface.alternate_setting,
            header: None,
            in_jacks: Vec::new(),
            out_jacks: Vec::new(),
            endpoints: Vec::new(),
        };
        let mut current_endpoint = None;

        for descriptor in descriptors {
            match descriptor? {
                MidiDescriptor::Header(header) => result.header = Some(header),
                MidiDescriptor::InJack(jack) => {
                    result.in_jacks.push(jack).map_err(|_| MidiDescriptorError::Capacity)?
                }
                MidiDescriptor::OutJack(jack) => {
                    result.out_jacks.push(jack).map_err(|_| MidiDescriptorError::Capacity)?
                }
                MidiDescriptor::Endpoint(endpoint) => {
                    current_endpoint = None;
                    if endpoint.ep_type() == EndpointType::Bulk {
                        result
                            .endpoints
                            .push(MidiEndpointDescriptor {
                                address: endpoint.endpoint_address,
                                max_packet_size: endpoint.max_packet_size,
                                jack_ids: Vec::new(),
                            })
                            .map_err(|_| MidiDescriptorError::Capacity)?;
                        current_endpoint = Some(result.endpoints.len() - 1);
                    }
                }
                MidiDescriptor::EndpointJackAssociations(associations) => {
                    let endpoint = current_endpoint.ok_or(MidiDescriptorError::InvalidDescriptor)?;
                    result.endpoints[endpoint].jack_ids = associations.jack_ids;
                }
            }
        }
        Ok(result)
    }
}

/// Parse all USB-MIDI 1.0 streaming interfaces in a configuration descriptor.
pub fn parse_midi_interfaces(
    config_desc: &[u8],
) -> Result<Vec<MidiStreamingInterface, MAX_MIDI_INTERFACES>, MidiDescriptorError> {
    let config = ConfigurationDescriptorChain::try_from_slice(config_desc)
        .map_err(|_| MidiDescriptorError::InvalidDescriptor)?;
    let mut result = Vec::new();
    for interface in config.iter_interface().filter(is_midi_streaming_interface) {
        result
            .push(MidiStreamingInterface::try_from_interface(&interface)?)
            .map_err(|_| MidiDescriptorError::Capacity)?;
    }
    Ok(result)
}

fn is_midi_streaming_interface(interface: &InterfaceDescriptorChain<'_>) -> bool {
    interface.interface_class == USB_CLASS_AUDIO
        && interface.interface_subclass == USB_SUBCLASS_MIDI_STREAMING
        && interface.interface_protocol == USB_MIDI_1_PROTOCOL
}

fn check_descriptor(raw: &[u8], descriptor_type: u8, subtype: u8, min_len: usize) -> Result<(), MidiDescriptorError> {
    if raw.len() < min_len || raw[0] as usize != raw.len() || raw[1] != descriptor_type || raw[2] != subtype {
        return Err(MidiDescriptorError::InvalidDescriptor);
    }
    Ok(())
}

fn check_descriptor_exact(
    raw: &[u8],
    descriptor_type: u8,
    subtype: u8,
    expected_len: usize,
) -> Result<(), MidiDescriptorError> {
    check_descriptor(raw, descriptor_type, subtype, expected_len)?;
    if raw.len() != expected_len {
        return Err(MidiDescriptorError::InvalidDescriptor);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Synthetic minimal configuration used to exercise interface and endpoint discovery.
    const MINIMAL_MIDI_CONFIG: &[u8] = &[
        9, 2, 32, 0, 1, 1, 0, 0x80, 50, 9, 4, 1, 0, 2, 1, 3, 0, 0, 7, 5, 0x81, 2, 64, 0, 0, 7, 5, 0x02, 2, 64, 0, 0,
    ];

    /// Yamaha DGX205 MIDI keyboard configuration descriptor.
    ///
    /// Copied from Cotton's `cotton-usb-host/src/tests/wire.rs`. Its endpoint
    /// descriptors are deliberately oversized at nine bytes.
    #[rustfmt::skip]
    const YAMAHA_DGX205_CONFIG: &[u8] = &[
        9, 2, 101, 0, 2, 1, 0, 128, 50, 9, 4, 0, 0, 0, 1, 1, 0, 0, 9, 36, 1, 0, 1, 9, 0, 1, 1, 9, 4, 1, 0, 2, 1, 3, 0,
        0, 7, 36, 1, 0, 1, 65, 0, 6, 36, 2, 1, 1, 0, 6, 36, 2, 2, 2, 0, 9, 36, 3, 1, 3, 1, 2, 1, 0, 9, 36, 3, 2, 4, 1,
        1, 1, 0, 9, 5, 2, 2, 32, 0, 0, 0, 0, 5, 37, 1, 1, 1, 9, 5, 129, 2, 32, 0, 0, 0, 0, 5, 37, 1, 1, 3,
    ];

    #[test]
    fn finds_midi_streaming_bulk_endpoints() {
        let interfaces = parse_midi_interfaces(MINIMAL_MIDI_CONFIG).unwrap();
        assert_eq!(interfaces.len(), 1);
        assert_eq!(interfaces[0].interface_number, 1);
        assert_eq!(interfaces[0].endpoints[0].address, 0x81);
        assert_eq!(interfaces[0].endpoints[1].address, 0x02);
    }

    #[test]
    fn rejects_non_midi_1_interface() {
        let mut config = MINIMAL_MIDI_CONFIG.to_vec();
        config[16] = 0x02;
        assert!(parse_midi_interfaces(&config).unwrap().is_empty());
    }

    #[test]
    fn parses_midi_jacks_and_endpoint_associations() {
        let interfaces = parse_midi_interfaces(YAMAHA_DGX205_CONFIG).unwrap();
        assert_eq!(interfaces.len(), 1);

        let midi = &interfaces[0];
        assert_eq!(midi.interface_number, 1);
        assert_eq!(
            midi.header,
            Some(MidiStreamingHeader {
                midi_version: 0x0100,
                total_length: 65
            })
        );
        assert_eq!(midi.in_jacks.len(), 2);
        assert_eq!(midi.in_jacks[0].jack_type, MidiJackType::Embedded);
        assert_eq!(midi.in_jacks[0].jack_id, 1);
        assert_eq!(midi.in_jacks[1].jack_type, MidiJackType::External);
        assert_eq!(midi.in_jacks[1].jack_id, 2);
        assert_eq!(midi.out_jacks.len(), 2);
        assert_eq!(midi.out_jacks[0].jack_id, 3);
        assert_eq!(
            midi.out_jacks[0].sources.as_slice(),
            &[MidiJackSource { jack_id: 2, pin: 1 }]
        );
        assert_eq!(midi.endpoints.len(), 2);
        assert_eq!(midi.endpoints[0].address, 0x02);
        assert_eq!(midi.endpoints[0].jack_ids.as_slice(), &[1]);
        assert_eq!(midi.endpoints[1].address, 0x81);
        assert_eq!(midi.endpoints[1].jack_ids.as_slice(), &[3]);
    }

    #[test]
    fn iterates_typed_midi_descriptors_in_wire_order() {
        let config = ConfigurationDescriptorChain::try_from_slice(YAMAHA_DGX205_CONFIG).unwrap();
        let interface = config.iter_interface().nth(1).unwrap();
        let descriptors: heapless::Vec<_, 9> = MidiDescriptorIterator::new(&interface)
            .unwrap()
            .map(Result::unwrap)
            .collect();

        assert!(matches!(descriptors[0], MidiDescriptor::Header(_)));
        assert!(matches!(descriptors[1], MidiDescriptor::InJack(_)));
        assert!(matches!(descriptors[2], MidiDescriptor::InJack(_)));
        assert!(matches!(descriptors[3], MidiDescriptor::OutJack(_)));
        assert!(matches!(descriptors[4], MidiDescriptor::OutJack(_)));
        assert!(matches!(descriptors[5], MidiDescriptor::Endpoint(_)));
        assert!(matches!(descriptors[6], MidiDescriptor::EndpointJackAssociations(_)));
        assert!(matches!(descriptors[7], MidiDescriptor::Endpoint(_)));
        assert!(matches!(descriptors[8], MidiDescriptor::EndpointJackAssociations(_)));
    }

    #[test]
    fn rejects_inconsistent_class_specific_lengths() {
        assert_eq!(
            MidiStreamingHeader::try_from_bytes(&[6, 36, 1, 0, 1, 7]),
            Err(MidiDescriptorError::InvalidDescriptor)
        );
        assert_eq!(
            MidiOutJack::try_from_bytes(&[9, 36, 3, 1, 3, 2, 1, 1, 0]),
            Err(MidiDescriptorError::InvalidDescriptor)
        );
        assert_eq!(
            MidiEndpointJackAssociations::try_from_bytes(&[5, 37, 1, 2, 1]),
            Err(MidiDescriptorError::InvalidDescriptor)
        );
        assert_eq!(
            parse_midi_interfaces(&MINIMAL_MIDI_CONFIG[..MINIMAL_MIDI_CONFIG.len() - 1]),
            Err(MidiDescriptorError::InvalidDescriptor)
        );
    }

    #[test]
    fn rejects_invalid_jack_types_and_excessive_associations() {
        assert_eq!(
            MidiInJack::try_from_bytes(&[6, 36, 2, 3, 1, 0]),
            Err(MidiDescriptorError::InvalidDescriptor)
        );

        let raw = [21, 37, 1, 17, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17];
        assert_eq!(
            MidiEndpointJackAssociations::try_from_bytes(&raw),
            Err(MidiDescriptorError::Capacity)
        );
    }

    #[test]
    fn rejects_endpoint_associations_without_a_bulk_endpoint() {
        let config = [9, 2, 23, 0, 1, 1, 0, 128, 50, 9, 4, 1, 0, 0, 1, 3, 0, 0, 5, 37, 1, 1, 1];
        assert_eq!(
            parse_midi_interfaces(&config),
            Err(MidiDescriptorError::InvalidDescriptor)
        );
    }
}

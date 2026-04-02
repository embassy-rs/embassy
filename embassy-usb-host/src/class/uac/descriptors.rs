//! USB Audio Class (UAC) descriptor parsing and management.

use core::iter::Peekable;

use heapless::Vec;
use heapless::index_map::FnvIndexMap;

use super::codes::*;
use crate::descriptor::{ConfigurationDescriptor, EndpointDescriptor, StringIndex, USBDescriptor};

const MAX_AUDIO_STREAMING_INTERFACES: usize = 16;
const MAX_ALTERNATE_SETTINGS: usize = 4;
const MAX_CLOCK_DESCRIPTORS: usize = 8;
const MAX_UNIT_DESCRIPTORS: usize = 16;
const MAX_TERMINAL_DESCRIPTORS: usize = 16;

/// Collection of audio interfaces representing a complete UAC audio function.
///
/// This struct contains all the interfaces that make up a USB Audio Class device,
/// including the interface association descriptor, control interface, and streaming interfaces.
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AudioInterfaceCollection {
    /// Interface association descriptor that groups the audio interfaces together.
    pub interface_association_descriptor: InterfaceAssociationDescriptor,
    /// Audio control interface containing clocks, terminals, and units.
    pub control_interface: AudioControlInterface,
    /// Collection of audio streaming interfaces for data transfer.
    pub audio_streaming_interfaces: Vec<AudioStreamingInterface, MAX_AUDIO_STREAMING_INTERFACES>,
}

/// Errors that can occur during audio interface parsing.
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AudioInterfaceError {
    /// A buffer is full and cannot accept more items.
    BufferFull(&'static str),
    /// No audio control interface was found in the configuration.
    MissingControlInterface,
    /// Audio control interface header descriptor is missing.
    MissingControlInterfaceHeader,
    /// An invalid descriptor was encountered during parsing.
    InvalidDescriptor,
    /// No audio configuration was found in the device.
    NoAudioConfiguration,
    /// Audio streaming class descriptor is missing.
    MissingAudioStreamingClassDescriptor,
}

impl AudioInterfaceCollection {
    /// Attempts to parse an audio interface collection from a configuration descriptor.
    ///
    /// This method searches for an interface association descriptor for audio,
    /// then parses the control interface and all streaming interfaces.
    ///
    /// Returns an [`AudioInterfaceCollection`] on success, or an [`AudioInterfaceError`] if parsing fails.
    pub fn try_from_configuration(cfg: &ConfigurationDescriptor) -> Result<Self, AudioInterfaceError> {
        let mut descriptors = cfg.iter_descriptors().peekable();

        // Find Interface Association Descriptor for Audio Function
        let iad = Self::find_interface_association_descriptor(&mut descriptors)?;

        // Find and parse Audio Control Interface
        let mut control_interface = None;
        let mut streaming_interfaces = Vec::new();

        while let Some(interfaces) =
            Self::next_interface_descriptors(&mut descriptors, iad.first_interface, iad.num_interfaces)?
        {
            trace!("Found interfaces: {:?}", interfaces);
            let first_interface = interfaces.first().unwrap();
            if first_interface.interface_class == interface::AUDIO {
                match first_interface.interface_subclass {
                    interface::subclass::AUDIOCONTROL => {
                        if control_interface.is_some() {
                            warn!("Audio Control Interface already parsed, skipping");
                            continue;
                        }
                        if first_interface.interface_protocol != function_protocol::AF_VERSION_02_00 {
                            debug!(
                                "Skipping interface with unsupported protocol: {:#04x}",
                                first_interface.interface_protocol
                            );
                            continue;
                        }
                        control_interface = Some(Self::collect_audio_control_interface(&mut descriptors, interfaces)?);
                    }
                    interface::subclass::AUDIOSTREAMING => {
                        if first_interface.interface_protocol != function_protocol::AF_VERSION_02_00 {
                            debug!(
                                "Skipping interface with unsupported protocol: {:#04x}",
                                first_interface.interface_protocol
                            );
                            continue;
                        }
                        streaming_interfaces
                            .push(Self::collect_audio_streaming_interface(&mut descriptors, interfaces)?)
                            .map_err(|_| AudioInterfaceError::BufferFull("Too many audio streaming interfaces"))?;
                    }
                    _ => {
                        trace!(
                            "Skipping unknown audio subclass: {:#04x}",
                            first_interface.interface_subclass
                        );
                    }
                }
            }
        }

        // Create the audio interface collection
        Ok(Self {
            interface_association_descriptor: iad,
            control_interface: control_interface.ok_or(AudioInterfaceError::MissingControlInterface)?,
            audio_streaming_interfaces: streaming_interfaces,
        })
    }

    fn find_interface_association_descriptor<'a>(
        descriptors: &mut Peekable<impl Iterator<Item = (usize, &'a [u8])>>,
    ) -> Result<InterfaceAssociationDescriptor, AudioInterfaceError> {
        loop {
            let (_, desc) = descriptors.next().ok_or(AudioInterfaceError::NoAudioConfiguration)?;
            if let Ok(iad) = InterfaceAssociationDescriptor::try_from_bytes(desc) {
                if iad.is_audio_association() {
                    return Ok(iad);
                }
            }
        }
    }

    fn next_interface_descriptors<'a>(
        descriptors: &mut Peekable<impl Iterator<Item = (usize, &'a [u8])>>,
        first_interface: u8,
        num_interfaces: u8,
    ) -> Result<Option<Vec<InterfaceDescriptor, MAX_ALTERNATE_SETTINGS>>, AudioInterfaceError> {
        let mut interface_descriptors = Vec::new();

        while let Some((_, desc)) = descriptors.next() {
            if let Ok(interface) = InterfaceDescriptor::try_from_bytes(desc) {
                trace!(
                    "Found interface: number={}, class={:#04x}, subclass={:#04x}, protocol={:#04x} from {:?}",
                    interface.interface_number,
                    interface.interface_class,
                    interface.interface_subclass,
                    interface.interface_protocol,
                    desc,
                );

                // Check if we're still within the audio function's interfaces
                if interface.interface_number >= first_interface + num_interfaces {
                    trace!(
                        "Interface number {} outside of IAD range, stopping",
                        interface.interface_number
                    );
                    break;
                }
                interface_descriptors
                    .push(interface)
                    .map_err(|_| AudioInterfaceError::BufferFull("Too many interfaces"))?;

                // Do we have contiguous (alternate) interfaces?
                while let Some((_, next_desc)) = descriptors.peek() {
                    if let Ok(next_interface) = InterfaceDescriptor::try_from_bytes(next_desc) {
                        if next_interface.interface_number == interface.interface_number {
                            // The next descriptor is the same interface, so we collect all descriptors for this interface
                            trace!(
                                "Found next interface with same number: {:#04x}",
                                next_interface.interface_number
                            );
                            descriptors.next(); // consume the next descriptor
                            interface_descriptors
                                .push(next_interface)
                                .map_err(|_| AudioInterfaceError::BufferFull("Too many interfaces"))?;
                            continue;
                        }
                    }
                    break;
                }
                break;
            }
        }

        if interface_descriptors.is_empty() {
            Ok(None)
        } else {
            Ok(Some(interface_descriptors))
        }
    }

    fn collect_audio_control_interface<'a>(
        descriptors: &mut Peekable<impl Iterator<Item = (usize, &'a [u8])>>,
        interfaces: Vec<InterfaceDescriptor, MAX_ALTERNATE_SETTINGS>,
    ) -> Result<AudioControlInterface, AudioInterfaceError> {
        trace!("Processing Audio Control Interface");
        let (_, header) = descriptors
            .next()
            .ok_or(AudioInterfaceError::MissingControlInterfaceHeader)?;
        if let Ok(header) = AudioControlHeaderDescriptor::try_from_bytes(header) {
            debug!(
                "Found Audio Control Header: version={}.{}",
                header.audio_device_class.0, header.audio_device_class.1
            );
            let mut clock_descriptors = FnvIndexMap::new();
            let mut unit_descriptors = FnvIndexMap::new();
            let mut terminal_descriptors = FnvIndexMap::new();
            let mut interrupt_endpoint_descriptor = None;
            while let Some((_, desc)) = descriptors.peek() {
                if desc.len() > 2 {
                    match desc[1] {
                        descriptor_type::CS_INTERFACE => {
                            let (_, desc) = descriptors.next().unwrap();
                            match ClockDescriptor::try_from_bytes(desc) {
                                Ok(clock) => {
                                    clock_descriptors
                                        .insert(clock.clock_id(), clock)
                                        .map_err(|_| AudioInterfaceError::BufferFull("Too many clock descriptors"))?;
                                }
                                // Ignore invalid descriptors: We don't know if we even have a clock descriptor
                                Err(AudioInterfaceError::InvalidDescriptor) => {}
                                Err(e) => return Err(e),
                            }
                            if let Ok(terminal) = TerminalDescriptor::try_from_bytes(desc) {
                                terminal_descriptors
                                    .insert(terminal.terminal_id(), terminal)
                                    .map_err(|_| AudioInterfaceError::BufferFull("Too many terminal descriptors"))?;
                            }
                            if let Ok(unit) = UnitDescriptor::try_from_bytes(desc) {
                                unit_descriptors
                                    .insert(unit.unit_id(), unit)
                                    .map_err(|_| AudioInterfaceError::BufferFull("Too many unit descriptors"))?;
                            }
                        }
                        descriptor_type::CS_ENDPOINT => {
                            if let Ok(desc) = EndpointDescriptor::try_from_bytes(descriptors.next().unwrap().1) {
                                interrupt_endpoint_descriptor = Some(desc);
                            }
                        }
                        _ => break,
                    }
                } else {
                    break;
                }
            }
            Ok(AudioControlInterface {
                interface_descriptors: interfaces,
                header_descriptor: header,
                interrupt_endpoint_descriptor: interrupt_endpoint_descriptor,
                clock_descriptors: clock_descriptors,
                unit_descriptors: unit_descriptors,
                terminal_descriptors: terminal_descriptors,
            })
        } else {
            Err(AudioInterfaceError::MissingControlInterfaceHeader)
        }
    }

    fn collect_audio_streaming_interface<'a>(
        descriptors: &mut Peekable<impl Iterator<Item = (usize, &'a [u8])>>,
        interfaces: Vec<InterfaceDescriptor, MAX_ALTERNATE_SETTINGS>,
    ) -> Result<AudioStreamingInterface, AudioInterfaceError> {
        trace!("Processing Audio Streaming Interface");
        let (_, class_descriptor) = descriptors
            .next()
            .ok_or(AudioInterfaceError::MissingAudioStreamingClassDescriptor)?;
        if let Ok(class_descriptor) = AudioStreamingClassDescriptor::try_from_bytes(class_descriptor) {
            trace!("Found Audio Streaming Class Descriptor: {:?}", class_descriptor.format);
            let mut streaming_interface = AudioStreamingInterface {
                interface_descriptors: interfaces,
                class_descriptor: class_descriptor,
                endpoint_descriptor: None,
                feedback_endpoint_descriptor: None,
                audio_endpoint_descriptor: None,
                format_type_descriptor: None,
            };
            loop {
                if let Some((_, desc)) = descriptors.peek() {
                    if InterfaceDescriptor::try_from_bytes(desc).is_ok() {
                        break;
                    }
                    let (_, desc) = descriptors.next().unwrap();
                    if let Ok(endpoint) = EndpointDescriptor::try_from_bytes(desc) {
                        if endpoint.attributes == 0b010001 {
                            streaming_interface.feedback_endpoint_descriptor = Some(endpoint);
                        } else {
                            streaming_interface.endpoint_descriptor = Some(endpoint);
                        }
                    }
                    if let Ok(audio_endpoint) = AudioEndpointDescriptor::try_from_bytes(desc) {
                        streaming_interface.audio_endpoint_descriptor = Some(audio_endpoint);
                    }
                    if let Ok(format_type) = FormatTypeDescriptor::try_from_bytes(desc) {
                        streaming_interface.format_type_descriptor = Some(format_type);
                    }
                } else {
                    break;
                }
            }
            Ok(streaming_interface)
        } else {
            Err(AudioInterfaceError::MissingAudioStreamingClassDescriptor)
        }
    }
}

/// USB interface association descriptor for grouping related interfaces.
///
/// This descriptor is used to associate multiple interfaces that belong to the same function,
/// such as an audio function with control and streaming interfaces.
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InterfaceAssociationDescriptor {
    /// First interface number in the association.
    pub first_interface: u8,
    /// Number of interfaces in the association.
    pub num_interfaces: u8,
    /// Function class code.
    pub class: u8,
    /// Function subclass code.
    pub subclass: u8,
    /// Function protocol code.
    pub protocol: u8,
    /// Index of string descriptor describing the function.
    pub interface_name: StringIndex,
}

impl USBDescriptor for InterfaceAssociationDescriptor {
    const SIZE: usize = 8;
    const DESC_TYPE: u8 = 0x0B;
    type Error = ();

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::SIZE {
            return Err(());
        }
        if bytes[1] != Self::DESC_TYPE {
            return Err(());
        }
        Ok(Self {
            first_interface: bytes[2],
            num_interfaces: bytes[3],
            class: bytes[4],
            subclass: bytes[5],
            protocol: bytes[6],
            interface_name: bytes[7],
        })
    }
}

impl InterfaceAssociationDescriptor {
    /// Returns true if this interface association is for an audio function.
    pub fn is_audio_association(&self) -> bool {
        self.class == AUDIO_FUNCTION && self.protocol == function_protocol::AF_VERSION_02_00
    }
}

/// USB interface descriptor for audio interfaces.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InterfaceDescriptor {
    /// Length of this descriptor in bytes.
    pub len: u8,
    /// Type of this descriptor. Must be 0x04.
    pub descriptor_type: u8,
    /// Number of this interface.
    pub interface_number: u8,
    /// Value used to select this alternate setting for the interface.
    pub alternate_setting: u8,
    /// Number of endpoints used by this interface.
    pub num_endpoints: u8,
    /// USB interface class code.
    pub interface_class: u8,
    /// USB interface subclass code.
    pub interface_subclass: u8,
    /// USB interface protocol code.
    pub interface_protocol: u8,
    /// Index of string descriptor describing this interface.
    pub interface_name: StringIndex,
}

impl USBDescriptor for InterfaceDescriptor {
    const SIZE: usize = 9;
    const DESC_TYPE: u8 = 0x04;
    type Error = ();

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::SIZE {
            return Err(());
        }
        if bytes[1] != Self::DESC_TYPE {
            return Err(());
        }
        Ok(Self {
            len: bytes[0],
            descriptor_type: bytes[1],
            interface_number: bytes[2],
            alternate_setting: bytes[3],
            num_endpoints: bytes[4],
            interface_class: bytes[5],
            interface_subclass: bytes[6],
            interface_protocol: bytes[7],
            interface_name: bytes[8],
        })
    }
}

//--------------------------------------------------------------------------------------------------
// Audio Control

/// Audio control interface containing all control-related descriptors.
///
/// This struct contains the header descriptor, clock descriptors, unit descriptors,
/// terminal descriptors, and optional interrupt endpoint descriptor.
#[derive(Debug, PartialEq)]
pub struct AudioControlInterface {
    /// Interface descriptors for this control interface.
    pub interface_descriptors: Vec<InterfaceDescriptor, MAX_ALTERNATE_SETTINGS>,
    /// Audio control header descriptor.
    pub header_descriptor: AudioControlHeaderDescriptor,
    /// Optional interrupt endpoint descriptor for control notifications.
    pub interrupt_endpoint_descriptor: Option<EndpointDescriptor>,
    /// Map of clock descriptors indexed by clock ID.
    pub clock_descriptors: FnvIndexMap<u8, ClockDescriptor, MAX_CLOCK_DESCRIPTORS>,
    /// Map of unit descriptors indexed by unit ID.
    pub unit_descriptors: FnvIndexMap<u8, UnitDescriptor, MAX_UNIT_DESCRIPTORS>,
    /// Map of terminal descriptors indexed by terminal ID.
    pub terminal_descriptors: FnvIndexMap<u8, TerminalDescriptor, MAX_TERMINAL_DESCRIPTORS>,
}

// heapless::IndexMap does not implement `defmt::Format`; summarize maps by length.
#[cfg(feature = "defmt")]
impl defmt::Format for AudioControlInterface {
    fn format(&self, fmt: defmt::Formatter<'_>) {
        defmt::write!(
            fmt,
            "AudioControlInterface {{ interface_descriptors: {=?}, header_descriptor: {=?}, interrupt_endpoint_descriptor: {=?}, clock_descriptors_len: {=usize}, unit_descriptors_len: {=usize}, terminal_descriptors_len: {=usize} }}",
            self.interface_descriptors,
            self.header_descriptor,
            self.interrupt_endpoint_descriptor,
            self.clock_descriptors.len(),
            self.unit_descriptors.len(),
            self.terminal_descriptors.len(),
        );
    }
}

/// Audio control header descriptor containing version and category information.
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AudioControlHeaderDescriptor {
    /// Audio device class version (major, minor).
    pub audio_device_class: (u8, u8),
    /// Category of the audio device.
    pub category: u8,
    /// Bitmap of supported controls.
    pub controls_bitmap: u8,
}

impl USBDescriptor for AudioControlHeaderDescriptor {
    const SIZE: usize = 9;
    const DESC_TYPE: u8 = descriptor_type::CS_INTERFACE;
    type Error = ();

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::SIZE {
            return Err(());
        }
        if bytes[1] != Self::DESC_TYPE {
            return Err(());
        }
        if bytes[2] != ac_descriptor::HEADER {
            return Err(());
        }
        Ok(Self {
            audio_device_class: (bytes[4], bytes[3]),
            category: bytes[5],
            controls_bitmap: bytes[8],
        })
    }
}

/// Enumeration of clock descriptor types.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClockDescriptor {
    /// Clock source descriptor.
    Source(ClockSourceDescriptor),
    /// Clock selector descriptor.
    Selector(ClockSelectorDescriptor),
    /// Clock multiplier descriptor.
    Multiplier(ClockMultiplierDescriptor),
}

impl ClockDescriptor {
    fn try_from_bytes(bytes: &[u8]) -> Result<Self, AudioInterfaceError> {
        if bytes.len() < 4 {
            return Err(AudioInterfaceError::InvalidDescriptor);
        }
        if bytes[1] != descriptor_type::CS_INTERFACE {
            return Err(AudioInterfaceError::InvalidDescriptor);
        }
        match bytes[2] {
            ac_descriptor::CLOCK_SOURCE => Ok(Self::Source(ClockSourceDescriptor::try_from_bytes(bytes)?)),
            ac_descriptor::CLOCK_SELECTOR => Ok(Self::Selector(ClockSelectorDescriptor::try_from_bytes(bytes)?)),
            ac_descriptor::CLOCK_MULTIPLIER => Ok(Self::Multiplier(ClockMultiplierDescriptor::try_from_bytes(bytes)?)),
            _ => Err(AudioInterfaceError::InvalidDescriptor),
        }
    }

    /// Returns the clock ID for this descriptor.
    pub fn clock_id(&self) -> u8 {
        match self {
            Self::Source(desc) => desc.clock_id,
            Self::Selector(desc) => desc.clock_id,
            Self::Multiplier(desc) => desc.clock_id,
        }
    }
}

/// Clock source descriptor defining an audio clock source.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ClockSourceDescriptor {
    /// Unique identifier for this clock source.
    pub clock_id: u8,
    /// Bitmap of clock source attributes.
    pub attributes_bitmap: u8,
    /// Bitmap of supported controls.
    pub controls_bitmap: u8,
    /// Associated terminal ID.
    pub associated_terminal: u8,
    /// Index of string descriptor describing this clock source.
    pub clock_name: StringIndex,
}

impl USBDescriptor for ClockSourceDescriptor {
    const SIZE: usize = 8;
    const DESC_TYPE: u8 = descriptor_type::CS_INTERFACE;
    type Error = AudioInterfaceError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::SIZE {
            return Err(AudioInterfaceError::InvalidDescriptor);
        }
        if bytes[1] != Self::DESC_TYPE {
            return Err(AudioInterfaceError::InvalidDescriptor);
        }
        if bytes[2] != ac_descriptor::CLOCK_SOURCE {
            return Err(AudioInterfaceError::InvalidDescriptor);
        }
        Ok(Self {
            clock_id: bytes[3],
            attributes_bitmap: bytes[4],
            controls_bitmap: bytes[5],
            associated_terminal: bytes[6],
            clock_name: bytes[7],
        })
    }
}

/// Clock selector descriptor for selecting between multiple clock sources.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ClockSelectorDescriptor {
    /// Unique identifier for this clock selector.
    pub clock_id: u8,
    /// List of source clock IDs that can be selected.
    pub source_ids: Vec<u8, MAX_CLOCK_DESCRIPTORS>,
    /// Bitmap of supported controls.
    pub controls_bitmap: u8,
    /// Index of string descriptor describing this clock selector.
    pub clock_name: StringIndex,
}

impl ClockSelectorDescriptor {
    fn try_from_bytes(bytes: &[u8]) -> Result<Self, AudioInterfaceError> {
        if bytes.len() < 7 {
            return Err(AudioInterfaceError::InvalidDescriptor);
        }
        if bytes[1] != descriptor_type::CS_INTERFACE {
            return Err(AudioInterfaceError::InvalidDescriptor);
        }
        if bytes[2] != ac_descriptor::CLOCK_SELECTOR {
            return Err(AudioInterfaceError::InvalidDescriptor);
        }
        let mut source_ids = Vec::new();
        let num_source_ids = bytes[4] as usize;
        for i in 0..num_source_ids {
            source_ids
                .push(bytes[5 + i])
                .map_err(|_| AudioInterfaceError::BufferFull("Too many clock source ids"))?;
        }
        Ok(Self {
            clock_id: bytes[3],
            source_ids,
            controls_bitmap: bytes[5 + num_source_ids as usize],
            clock_name: bytes[6 + num_source_ids as usize],
        })
    }
}

/// Clock multiplier descriptor for frequency multiplication.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ClockMultiplierDescriptor {
    /// Unique identifier for this clock multiplier.
    pub clock_id: u8,
    /// Source clock ID.
    pub source_id: u8,
    /// Bitmap of supported controls.
    pub controls_bitmap: u8,
    /// Index of string descriptor describing this clock multiplier.
    pub clock_name: StringIndex,
}

impl USBDescriptor for ClockMultiplierDescriptor {
    const SIZE: usize = 7;
    const DESC_TYPE: u8 = descriptor_type::CS_INTERFACE;
    type Error = AudioInterfaceError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, AudioInterfaceError> {
        if bytes.len() < Self::SIZE {
            return Err(AudioInterfaceError::InvalidDescriptor);
        }
        if bytes[1] != Self::DESC_TYPE {
            return Err(AudioInterfaceError::InvalidDescriptor);
        }
        if bytes[2] != ac_descriptor::CLOCK_MULTIPLIER {
            return Err(AudioInterfaceError::InvalidDescriptor);
        }
        Ok(Self {
            clock_id: bytes[3],
            source_id: bytes[4],
            controls_bitmap: bytes[5],
            clock_name: bytes[6],
        })
    }
}

/// Enumeration of terminal descriptor types.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TerminalDescriptor {
    /// Input terminal descriptor.
    Input(InputTerminalDescriptor),
    /// Output terminal descriptor.
    Output(OutputTerminalDescriptor),
}

impl TerminalDescriptor {
    fn try_from_bytes(bytes: &[u8]) -> Result<Self, ()> {
        if bytes.len() < 3 {
            return Err(());
        }
        if bytes[1] != descriptor_type::CS_INTERFACE {
            return Err(());
        }
        match bytes[2] {
            ac_descriptor::INPUT_TERMINAL => Ok(Self::Input(InputTerminalDescriptor::try_from_bytes(bytes)?)),
            ac_descriptor::OUTPUT_TERMINAL => Ok(Self::Output(OutputTerminalDescriptor::try_from_bytes(bytes)?)),
            _ => Err(()),
        }
    }

    /// Returns the terminal ID for this descriptor.
    pub fn terminal_id(&self) -> u8 {
        match self {
            Self::Input(desc) => desc.terminal_id,
            Self::Output(desc) => desc.terminal_id,
        }
    }

    /// Returns the terminal type for this descriptor.
    pub fn terminal_type(&self) -> TerminalType {
        match self {
            Self::Input(desc) => desc.terminal_type,
            Self::Output(desc) => desc.terminal_type,
        }
    }

    /// Returns the clock source ID associated with this terminal.
    pub fn clock_source_id(&self) -> u8 {
        match self {
            Self::Input(desc) => desc.clock_source_id,
            Self::Output(desc) => desc.clock_source_id,
        }
    }

    /// Returns the terminal name string index.
    pub fn terminal_name(&self) -> StringIndex {
        match self {
            Self::Input(desc) => desc.terminal_name,
            Self::Output(desc) => desc.terminal_name,
        }
    }
}

/// Enumeration of terminal types as defined by the USB Audio Class specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TerminalType {
    /// Unknown terminal type with raw value.
    Unknown(u16),

    // USB Terminal Types
    /// USB undefined terminal.
    UsbUndefined,
    /// USB streaming terminal.
    UsbStreaming,
    /// USB vendor-specific terminal.
    UsbVendorSpecific,

    // Input Terminal Types
    /// Input undefined terminal.
    InputUndefined,
    /// Microphone terminal.
    Microphone,
    /// Desktop microphone terminal.
    DesktopMicrophone,
    /// Personal microphone terminal.
    PersonalMicrophone,
    /// Omni-directional microphone terminal.
    OmniMicrophone,
    /// Microphone array terminal.
    MicrophoneArray,
    /// Processing microphone array terminal.
    ProcessingMicrophoneArray,

    // Output Terminal Types
    /// Output undefined terminal.
    OutputUndefined,
    /// Speaker terminal.
    Speaker,
    /// Headphones terminal.
    Headphones,
    /// Head-mounted display audio terminal.
    HeadMountedDisplay,
    /// Desktop speaker terminal.
    DesktopSpeaker,
    /// Room speaker terminal.
    RoomSpeaker,
    /// Communication speaker terminal.
    CommunicationSpeaker,
    /// Low frequency effects speaker terminal.
    LowFrequencyEffectsSpeaker,

    // Bi-directional Terminal Types
    /// Bi-directional undefined terminal.
    BiDirectionalUndefined,
    /// Handset terminal.
    Handset,
    /// Headset terminal.
    Headset,
    /// Speakerphone terminal.
    SpeakerPhone,
    /// Echo suppressing speakerphone terminal.
    EchoSuppressing,
    /// Echo canceling speakerphone terminal.
    EchoCanceling,

    // Telephony Terminal Types
    /// Telephony undefined terminal.
    TelephonyUndefined,
    /// Phone line terminal.
    PhoneLine,
    /// Telephone terminal.
    Telephone,
    /// Down line phone terminal.
    DownLinePhone,

    // External Terminal Types
    /// External undefined terminal.
    ExternalUndefined,
    /// Analog connector terminal.
    AnalogConnector,
    /// Digital audio interface terminal.
    DigitalAudioInterface,
    /// Line connector terminal.
    LineConnector,
    /// Legacy audio connector terminal.
    LegacyAudioConnector,
    /// SPDIF interface terminal.
    SpdifInterface,
    /// DA 1394 stream terminal.
    Da1394Stream,
    /// DVD audio stream terminal.
    DvdAudioStream,
    /// AVC stream terminal.
    AvcStream,
}

fn terminal_type_from_u16(terminal_type: u16) -> TerminalType {
    use TerminalType::*;

    use crate::class::uac::codes::terminal_type::*;

    match terminal_type {
        usb::UNDEFINED => UsbUndefined,
        usb::STREAMING => UsbStreaming,
        usb::VENDOR_SPECIFIC => UsbVendorSpecific,

        input::UNDEFINED => InputUndefined,
        input::MICROPHONE => Microphone,
        input::DESKTOP_MICROPHONE => DesktopMicrophone,
        input::PERSONAL_MICROPHONE => PersonalMicrophone,
        input::OMNI_DIRECTIONAL_MICROPHONE => OmniMicrophone,
        input::MICROPHONE_ARRAY => MicrophoneArray,
        input::PROCESSING_MICROPHONE_ARRAY => ProcessingMicrophoneArray,

        output::UNDEFINED => OutputUndefined,
        output::SPEAKER => Speaker,
        output::HEADPHONES => Headphones,
        output::HEAD_MOUNTED_DISPLAY_AUDIO => HeadMountedDisplay,
        output::DESKTOP_SPEAKER => DesktopSpeaker,
        output::ROOM_SPEAKER => RoomSpeaker,
        output::COMMUNICATION_SPEAKER => CommunicationSpeaker,
        output::LOW_FREQUENCY_EFFECTS_SPEAKER => LowFrequencyEffectsSpeaker,

        bidirectional::UNDEFINED => BiDirectionalUndefined,
        bidirectional::HANDSET => Handset,
        bidirectional::HEADSET => Headset,
        bidirectional::SPEAKERPHONE_NO_ECHO => SpeakerPhone,
        bidirectional::ECHO_SUPPRESSING_SPEAKERPHONE => EchoSuppressing,
        bidirectional::ECHO_CANCELING_SPEAKERPHONE => EchoCanceling,

        telephony::UNDEFINED => TelephonyUndefined,
        telephony::PHONE_LINE => PhoneLine,
        telephony::TELEPHONE => Telephone,
        telephony::DOWN_LINE_PHONE => DownLinePhone,

        external::UNDEFINED => ExternalUndefined,
        external::ANALOG_CONNECTOR => AnalogConnector,
        external::DIGITAL_AUDIO_INTERFACE => DigitalAudioInterface,
        external::LINE_CONNECTOR => LineConnector,
        external::LEGACY_AUDIO_CONNECTOR => LegacyAudioConnector,
        external::SPDIF_INTERFACE => SpdifInterface,
        external::DA_STREAM_1394 => Da1394Stream,
        external::DV_STREAM_SOUNDTRACK_1394 => DvdAudioStream,
        external::ADAT_LIGHTPIPE => AvcStream,

        _ => Unknown(terminal_type),
    }
}

/// Input terminal descriptor for audio input sources.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InputTerminalDescriptor {
    /// Unique identifier for this input terminal.
    pub terminal_id: u8,
    /// Type of this input terminal.
    pub terminal_type: TerminalType,
    /// Associated terminal ID.
    pub associated_terminal_id: u8,
    /// Clock source ID associated with this terminal.
    pub clock_source_id: u8,
    /// Number of channels supported by this terminal.
    pub num_channels: u8,
    /// Bitmap of channel configuration.
    pub channel_config_bitmap: u32,
    /// Index of string descriptor for channel names.
    pub channel_names: StringIndex,
    /// Bitmap of supported controls.
    pub controls_bitmap: u16,
    /// Index of string descriptor describing this terminal.
    pub terminal_name: StringIndex,
}

impl USBDescriptor for InputTerminalDescriptor {
    const SIZE: usize = 17;
    const DESC_TYPE: u8 = descriptor_type::CS_INTERFACE;
    type Error = ();

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() != Self::SIZE {
            return Err(());
        }
        if bytes[1] != Self::DESC_TYPE {
            return Err(());
        }
        if bytes[2] != ac_descriptor::INPUT_TERMINAL {
            return Err(());
        }
        Ok(Self {
            terminal_id: bytes[3],
            terminal_type: terminal_type_from_u16(u16::from_le_bytes([bytes[4], bytes[5]])),
            associated_terminal_id: bytes[6],
            clock_source_id: bytes[7],
            num_channels: bytes[8],
            channel_config_bitmap: u32::from_le_bytes([bytes[9], bytes[10], bytes[11], bytes[12]]),
            channel_names: bytes[13],
            controls_bitmap: u16::from_le_bytes([bytes[14], bytes[15]]),
            terminal_name: bytes[16],
        })
    }
}

/// Output terminal descriptor for audio output destinations.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OutputTerminalDescriptor {
    /// Unique identifier for this output terminal.
    pub terminal_id: u8,
    /// Type of this output terminal.
    pub terminal_type: TerminalType,
    /// Associated terminal ID.
    pub associated_terminal_id: u8,
    /// Source unit or terminal ID.
    pub source_id: u8,
    /// Clock source ID associated with this terminal.
    pub clock_source_id: u8,
    /// Bitmap of supported controls.
    pub controls_bitmap: u16,
    /// Index of string descriptor describing this terminal.
    pub terminal_name: StringIndex,
}

impl USBDescriptor for OutputTerminalDescriptor {
    const SIZE: usize = 12;
    const DESC_TYPE: u8 = descriptor_type::CS_INTERFACE;
    type Error = ();

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() != Self::SIZE {
            return Err(());
        }
        if bytes[1] != Self::DESC_TYPE {
            return Err(());
        }
        if bytes[2] != ac_descriptor::OUTPUT_TERMINAL {
            return Err(());
        }
        Ok(Self {
            terminal_id: bytes[3],
            terminal_type: terminal_type_from_u16(u16::from_le_bytes([bytes[4], bytes[5]])),
            associated_terminal_id: bytes[6],
            source_id: bytes[7],
            clock_source_id: bytes[8],
            controls_bitmap: u16::from_le_bytes([bytes[9], bytes[10]]),
            terminal_name: bytes[11],
        })
    }
}

/// Enumeration of unit descriptor types for audio processing units.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum UnitDescriptor {
    /// Mixer unit with unit ID.
    Mixer(u8),
    /// Selector unit with unit ID.
    Selector(u8),
    /// Feature unit with unit ID.
    Feature(u8),
    /// Processing unit with unit ID.
    Processing(u8),
    /// Effect unit with unit ID.
    Effect(u8),
    /// Sample rate converter unit with unit ID.
    SampleRateConverter(u8),
    /// Extension unit with unit ID.
    Extension(u8),
}

impl USBDescriptor for UnitDescriptor {
    const SIZE: usize = 4; // This is not the true size; Will become variable
    const DESC_TYPE: u8 = descriptor_type::CS_INTERFACE;
    type Error = ();

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::SIZE {
            return Err(());
        }
        if bytes[1] != Self::DESC_TYPE {
            return Err(());
        }
        match bytes[2] {
            ac_descriptor::MIXER_UNIT => Ok(Self::Mixer(bytes[3])),
            ac_descriptor::SELECTOR_UNIT => Ok(Self::Selector(bytes[3])),
            ac_descriptor::FEATURE_UNIT => Ok(Self::Feature(bytes[3])),
            ac_descriptor::PROCESSING_UNIT => Ok(Self::Processing(bytes[3])),
            ac_descriptor::EFFECT_UNIT => Ok(Self::Effect(bytes[3])),
            ac_descriptor::SAMPLE_RATE_CONVERTER => Ok(Self::SampleRateConverter(bytes[3])),
            ac_descriptor::EXTENSION_UNIT => Ok(Self::Extension(bytes[3])),
            _ => Err(()),
        }
    }
}

impl UnitDescriptor {
    /// Returns the unit ID for this descriptor.
    pub fn unit_id(&self) -> u8 {
        match self {
            Self::Mixer(id) => *id,
            Self::Selector(id) => *id,
            Self::Feature(id) => *id,
            Self::Processing(id) => *id,
            Self::Effect(id) => *id,
            Self::SampleRateConverter(id) => *id,
            Self::Extension(id) => *id,
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Audio Streaming

/// Audio streaming interface containing streaming-related descriptors.
///
/// This struct contains the interface descriptors, class descriptor, endpoint descriptors,
/// and format type descriptor for an audio streaming interface.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AudioStreamingInterface {
    /// Interface descriptors for this streaming interface.
    pub interface_descriptors: Vec<InterfaceDescriptor, MAX_ALTERNATE_SETTINGS>,
    /// Audio streaming class descriptor.
    pub class_descriptor: AudioStreamingClassDescriptor,
    /// Main endpoint descriptor for audio data.
    pub endpoint_descriptor: Option<EndpointDescriptor>,
    /// Optional feedback endpoint descriptor for clock synchronization.
    pub feedback_endpoint_descriptor: Option<EndpointDescriptor>,
    /// Audio-specific endpoint descriptor.
    pub audio_endpoint_descriptor: Option<AudioEndpointDescriptor>,
    /// Format type descriptor defining the audio format.
    pub format_type_descriptor: Option<FormatTypeDescriptor>,
    // TODO: Encoder, decoder descriptors
}

/// Audio streaming class descriptor containing format and channel information.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AudioStreamingClassDescriptor {
    /// Terminal link ID connecting to the control interface.
    pub terminal_link_id: u8,
    /// Bitmap of supported controls.
    pub controls_bitmap: u8,
    /// Audio format type supported by this interface.
    pub format: format_type::Format,
    /// Number of channels supported.
    pub num_channels: u8,
    /// Bitmap of channel configuration.
    pub channel_config_bitmap: u32,
    /// Index of string descriptor for channel names.
    pub channel_name: StringIndex,
}

impl USBDescriptor for AudioStreamingClassDescriptor {
    const SIZE: usize = 16;
    const DESC_TYPE: u8 = descriptor_type::CS_INTERFACE;
    type Error = ();

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::SIZE {
            return Err(());
        }
        if bytes[1] != Self::DESC_TYPE {
            return Err(());
        }
        if bytes[2] != as_descriptor::GENERAL {
            return Err(());
        }
        let format =
            format_type::Format::from_u32(bytes[5], u32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]));
        if format.is_none() {
            error!("Invalid format type descriptor: type {:?}", bytes[5]);
            return Err(());
        }
        Ok(Self {
            terminal_link_id: bytes[3],
            controls_bitmap: bytes[4],
            format: format.unwrap(),
            num_channels: bytes[10],
            channel_config_bitmap: u32::from_le_bytes([bytes[11], bytes[12], bytes[13], bytes[14]]),
            channel_name: bytes[15],
        })
    }
}

/// Audio-specific endpoint descriptor containing audio endpoint attributes.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AudioEndpointDescriptor {
    /// Bitmap of endpoint attributes.
    pub attributes_bitmap: u8,
    /// Bitmap of supported controls.
    pub controls_bitmap: u8,
    /// Units for lock delay (1=milliseconds, 2=samples).
    pub lock_delay_units: u8,
    /// Lock delay value in the specified units.
    pub lock_delay: u16,
}

impl USBDescriptor for AudioEndpointDescriptor {
    const SIZE: usize = 6;
    const DESC_TYPE: u8 = descriptor_type::CS_ENDPOINT;
    type Error = ();

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < Self::SIZE {
            return Err(());
        }
        if bytes[1] != Self::DESC_TYPE {
            return Err(());
        }
        if bytes[2] != as_descriptor::GENERAL {
            return Err(());
        }
        Ok(Self {
            attributes_bitmap: bytes[3],
            controls_bitmap: bytes[4],
            lock_delay_units: bytes[5],
            lock_delay: u16::from_le_bytes([bytes[6], bytes[7]]),
        })
    }
}

/// Enumeration of format type descriptors for different audio formats.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FormatTypeDescriptor {
    /// Type I format (PCM, PCM8, etc.).
    I(FormatTypeI),
    /// Type II format (MPEG, AC-3, etc.).
    II(FormatTypeII),
    /// Type III format (IEC1937_AC-3, IEC1937_MPEG-1_Layer1, etc.).
    III(FormatTypeIII),
    /// Type IV format.
    IV,
    /// Extended Type I format.
    ExtendedI(FormatTypeExtendedI),
    /// Extended Type II format.
    ExtendedII(FormatTypeExtendedII),
    /// Extended Type III format.
    ExtendedIII(FormatTypeExtendedIII),
}

impl FormatTypeDescriptor {
    fn try_from_bytes(bytes: &[u8]) -> Result<Self, ()> {
        if bytes.len() < 4 {
            // minimum length of a format type descriptor
            return Err(());
        }
        let len = bytes[0] as usize;
        if bytes[1] != descriptor_type::CS_INTERFACE {
            return Err(());
        }
        if bytes[2] != as_descriptor::FORMAT_TYPE {
            return Err(());
        }
        match bytes[3] {
            format_type::I => {
                if len != 6 {
                    return Err(());
                }
                Ok(Self::I(FormatTypeI {
                    subslot_size: bytes[4],
                    bit_resolution: bytes[5],
                }))
            }
            format_type::II => {
                if len != 8 {
                    return Err(());
                }
                Ok(Self::II(FormatTypeII {
                    max_bit_rate: u16::from_le_bytes([bytes[4], bytes[5]]),
                    slots_per_frame: u16::from_le_bytes([bytes[6], bytes[7]]),
                }))
            }
            format_type::III => {
                if len != 6 {
                    return Err(());
                }
                Ok(Self::III(FormatTypeIII {
                    subslot_size: bytes[4],
                    bit_resolution: bytes[5],
                }))
            }
            format_type::IV => Ok(Self::IV),
            format_type::EXT_I => {
                if len != 9 {
                    return Err(());
                }
                Ok(Self::ExtendedI(FormatTypeExtendedI {
                    subslot_size: bytes[4],
                    bit_resolution: bytes[5],
                    header_length: bytes[6],
                    control_size: bytes[7],
                    sideband_protocol: bytes[8],
                }))
            }
            format_type::EXT_II => {
                if len != 10 {
                    return Err(());
                }
                Ok(Self::ExtendedII(FormatTypeExtendedII {
                    max_bit_rate: u16::from_le_bytes([bytes[4], bytes[5]]),
                    samples_per_frame: u16::from_le_bytes([bytes[6], bytes[7]]),
                    header_length: bytes[8],
                    sideband_protocol: bytes[9],
                }))
            }
            format_type::EXT_III => {
                if len != 8 {
                    return Err(());
                }
                Ok(Self::ExtendedIII(FormatTypeExtendedIII {
                    subslot_size: bytes[4],
                    bit_resolution: bytes[5],
                    header_length: bytes[6],
                    sideband_protocol: bytes[7],
                }))
            }
            _ => Err(()),
        }
    }
}

/// Type I format descriptor for PCM-like formats.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FormatTypeI {
    /// Size of each subslot in bytes.
    pub subslot_size: u8,
    /// Bit resolution of the audio data.
    pub bit_resolution: u8,
}

/// Type II format descriptor for compressed formats.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FormatTypeII {
    /// Maximum bit rate in bits per second.
    pub max_bit_rate: u16,
    /// Number of slots per frame.
    pub slots_per_frame: u16,
}

/// Type III format descriptor for IEC formats.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FormatTypeIII {
    /// Size of each subslot in bytes.
    pub subslot_size: u8,
    /// Bit resolution of the audio data.
    pub bit_resolution: u8,
}

/// Extended Type I format descriptor.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FormatTypeExtendedI {
    /// Size of each subslot in bytes.
    pub subslot_size: u8,
    /// Bit resolution of the audio data.
    pub bit_resolution: u8,
    /// Length of the format-specific header.
    pub header_length: u8,
    /// Size of control data.
    pub control_size: u8,
    /// Sideband protocol identifier.
    pub sideband_protocol: u8,
}

/// Extended Type II format descriptor.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FormatTypeExtendedII {
    /// Maximum bit rate in bits per second.
    pub max_bit_rate: u16,
    /// Number of samples per frame.
    pub samples_per_frame: u16,
    /// Length of the format-specific header.
    pub header_length: u8,
    /// Sideband protocol identifier.
    pub sideband_protocol: u8,
}

/// Extended Type III format descriptor.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FormatTypeExtendedIII {
    /// Size of each subslot in bytes.
    pub subslot_size: u8,
    /// Bit resolution of the audio data.
    pub bit_resolution: u8,
    /// Length of the format-specific header.
    pub header_length: u8,
    /// Sideband protocol identifier.
    pub sideband_protocol: u8,
}

//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod test {
    use env_logger;
    use heapless::Vec;

    use super::*;
    use crate::descriptor::ConfigurationDescriptor;

    #[test]
    fn test_parse() {
        // Initialize logger
        env_logger::init();

        let mut buffer: [u8; 512] = [0; 512];
        let descriptors = [
            8, 11, 0, 4, 1, 0, 32, 0, // Interface Association Descriptor
            9, 4, 0, 0, 0, 1, 1, 32, 7, // Audio Control Interface
            9, 36, 1, 0, 2, 8, 223, 0, 0, // Audio Control Header Descriptor
            8, 36, 10, 40, 1, 7, 0, 16, // Clock Source Descriptor
            17, 36, 2, 2, 1, 1, 0, 40, 16, 0, 0, 0, 0, 18, 0, 0, 2, // Input Terminal Descriptor
            // Feature Unit Descriptor
            74, 36, 6, 10, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 15, //
            12, 36, 3, 20, 1, 3, 0, 10, 40, 0, 0, 5, // Output Terminal Descriptor
            17, 36, 2, 1, 1, 2, 0, 40, 16, 0, 0, 0, 0, 50, 0, 0, 3, // Input Terminal Descriptor
            // Feature Unit Descriptor
            74, 36, 6, 11, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 14, //
            12, 36, 3, 22, 1, 1, 0, 11, 40, 0, 0, 4, // Output Terminal Descriptor
            9, 4, 1, 0, 0, 1, 2, 32, 8, // Audio Streaming Interface Descriptor (Alt Setting 0)
            9, 4, 1, 1, 2, 1, 2, 32, 9, // Audio Streaming Interface Descriptor (Alt Setting 1)
            16, 36, 1, 2, 0, 1, 1, 0, 0, 0, 16, 0, 0, 0, 0, 18, // AS Interface Descriptor (General)
            6, 36, 2, 1, 4, 24, // Format Type Descriptor
            7, 5, 1, 5, 0, 2, 1, // Endpoint Descriptor
            8, 37, 1, 0, 0, 2, 8, 0, // AS Endpoint Descriptor
            7, 5, 129, 17, 4, 0, 4, // Endpoint Descriptor (Feedback)
            9, 4, 2, 0, 0, 1, 2, 32, 10, // Audio Streaming Interface Descriptor (Alt Setting 0)
            9, 4, 2, 1, 1, 1, 2, 32, 11, // Audio Streaming Interface Descriptor (Alt Setting 1)
            16, 36, 1, 22, 0, 1, 1, 0, 0, 0, 16, 0, 0, 0, 0, 50, // AS Interface Descriptor (General)
            6, 36, 2, 1, 4, 24, // Format Type Descriptor
            7, 5, 130, 5, 0, 2, 1, // Endpoint Descriptor
            8, 37, 1, 0, 0, 2, 8, 0, // AS Endpoint Descriptor
            9, 4, 3, 0, 0, 1, 1, 0, 0, // Audio Control Interface (UAC 1)
            9, 36, 1, 0, 1, 9, 0, 1, 1, // Audio Control Header Descriptor
            9, 4, 4, 0, 2, 1, 3, 0, 0, // MIDI Streaming Interface Descriptor
            7, 36, 1, 0, 1, 61, 0, // MS Interface Header Descriptor
            6, 36, 2, 1, 51, 0, // MIDI IN Jack Descriptor (Embedded)
            6, 36, 2, 2, 52, 82, // MIDI IN Jack Descriptor (External)
            9, 36, 3, 1, 55, 1, 52, 1, 0, // MIDI OUT Jack Descriptor (Embedded)
            9, 36, 3, 2, 56, 1, 51, 1, 83, // MIDI OUT Jack Descriptor (External)
            7, 5, 131, 2, 0, 2, 0, // Endpoint Descriptor (OUT)
            5, 37, 1, 1, 55, // MS Endpoint Descriptor
            7, 5, 2, 2, 0, 2, 0, // Endpoint Descriptor (IN)
            5, 37, 1, 1, 51, // MS Endpoint Descriptor
            9, 4, 5, 0, 0, 254, 1, 1, 0, // DFU Interface Descriptor
            7, 33, 7, 250, 0, 64, 0, // DFU Functional Descriptor
        ];
        buffer[..descriptors.len()].copy_from_slice(&descriptors);
        let descriptor = ConfigurationDescriptor {
            len: 0,
            descriptor_type: 0,
            total_len: 0,
            num_interfaces: 0,
            configuration_value: 1,
            configuration_name: 0,
            attributes: 0,
            max_power: 0,
            buffer: &buffer,
        };
        let mut expected_clock_descriptors = FnvIndexMap::<u8, ClockDescriptor, MAX_CLOCK_DESCRIPTORS>::new();
        let mut expected_unit_descriptors = FnvIndexMap::<u8, UnitDescriptor, MAX_UNIT_DESCRIPTORS>::new();
        let mut expected_terminal_descriptors = FnvIndexMap::<u8, TerminalDescriptor, MAX_TERMINAL_DESCRIPTORS>::new();
        expected_clock_descriptors
            .insert(
                40,
                ClockDescriptor::Source(ClockSourceDescriptor {
                    clock_id: 40,
                    attributes_bitmap: 1,
                    controls_bitmap: 7,
                    associated_terminal: 0,
                    clock_name: 16,
                }),
            )
            .unwrap();
        expected_unit_descriptors
            .insert(10, UnitDescriptor::Feature(10))
            .unwrap();
        expected_unit_descriptors
            .insert(11, UnitDescriptor::Feature(11))
            .unwrap();
        expected_terminal_descriptors
            .insert(
                2,
                TerminalDescriptor::Input(InputTerminalDescriptor {
                    terminal_id: 2,
                    terminal_type: TerminalType::UsbStreaming,
                    associated_terminal_id: 0,
                    clock_source_id: 40,
                    num_channels: 16,
                    channel_config_bitmap: 0,
                    channel_names: 18,
                    controls_bitmap: 0,
                    terminal_name: 2,
                }),
            )
            .unwrap();
        expected_terminal_descriptors
            .insert(
                20,
                TerminalDescriptor::Output(OutputTerminalDescriptor {
                    terminal_id: 20,
                    terminal_type: TerminalType::Speaker,
                    associated_terminal_id: 0,
                    source_id: 10,
                    clock_source_id: 40,
                    controls_bitmap: 0,
                    terminal_name: 5,
                }),
            )
            .unwrap();
        expected_terminal_descriptors
            .insert(
                1,
                TerminalDescriptor::Input(InputTerminalDescriptor {
                    terminal_id: 1,
                    terminal_type: TerminalType::Microphone,
                    associated_terminal_id: 0,
                    clock_source_id: 40,
                    num_channels: 16,
                    channel_config_bitmap: 0,
                    channel_names: 50,
                    controls_bitmap: 0,
                    terminal_name: 3,
                }),
            )
            .unwrap();
        expected_terminal_descriptors
            .insert(
                22,
                TerminalDescriptor::Output(OutputTerminalDescriptor {
                    terminal_id: 22,
                    terminal_type: TerminalType::UsbStreaming,
                    associated_terminal_id: 0,
                    source_id: 11,
                    clock_source_id: 40,
                    controls_bitmap: 0,
                    terminal_name: 4,
                }),
            )
            .unwrap();

        let expected = AudioInterfaceCollection {
            interface_association_descriptor: InterfaceAssociationDescriptor {
                first_interface: 0,
                num_interfaces: 4,
                class: 1,
                subclass: 0,
                protocol: 32,
                interface_name: 0,
            },
            control_interface: AudioControlInterface {
                interface_descriptors: Vec::from_slice(&[InterfaceDescriptor {
                    len: 9,
                    descriptor_type: 4,
                    interface_number: 0,
                    alternate_setting: 0,
                    num_endpoints: 0,
                    interface_class: 1,
                    interface_subclass: 1,
                    interface_protocol: 32,
                    interface_name: 7,
                }])
                .unwrap(),
                header_descriptor: AudioControlHeaderDescriptor {
                    audio_device_class: (2, 0),
                    category: 8,
                    controls_bitmap: 0,
                },
                interrupt_endpoint_descriptor: None,
                clock_descriptors: expected_clock_descriptors,
                unit_descriptors: expected_unit_descriptors,
                terminal_descriptors: expected_terminal_descriptors,
            },
            audio_streaming_interfaces: Vec::from_slice(&[
                AudioStreamingInterface {
                    interface_descriptors: Vec::from_slice(&[
                        InterfaceDescriptor {
                            len: 9,
                            descriptor_type: 4,
                            interface_number: 1,
                            alternate_setting: 0,
                            num_endpoints: 0,
                            interface_class: 1,
                            interface_subclass: 2,
                            interface_protocol: 32,
                            interface_name: 8,
                        },
                        InterfaceDescriptor {
                            len: 9,
                            descriptor_type: 4,
                            interface_number: 1,
                            alternate_setting: 1,
                            num_endpoints: 2,
                            interface_class: 1,
                            interface_subclass: 2,
                            interface_protocol: 32,
                            interface_name: 9,
                        },
                    ])
                    .unwrap(),
                    class_descriptor: AudioStreamingClassDescriptor {
                        terminal_link_id: 2,
                        controls_bitmap: 0,
                        format: format_type::Format::Type1(format_type::Type1::PCM),
                        num_channels: 16,
                        channel_config_bitmap: 0,
                        channel_name: 18,
                    },
                    endpoint_descriptor: Some(EndpointDescriptor {
                        len: 7,
                        descriptor_type: 5,
                        endpoint_address: 1,
                        attributes: 5,
                        max_packet_size: 512,
                        interval: 1,
                    }),
                    feedback_endpoint_descriptor: Some(EndpointDescriptor {
                        len: 7,
                        descriptor_type: 5,
                        endpoint_address: 129,
                        attributes: 17,
                        max_packet_size: 4,
                        interval: 4,
                    }),
                    audio_endpoint_descriptor: Some(AudioEndpointDescriptor {
                        attributes_bitmap: 0,
                        controls_bitmap: 0,
                        lock_delay_units: 2,
                        lock_delay: 8,
                    }),
                    format_type_descriptor: Some(FormatTypeDescriptor::I(FormatTypeI {
                        subslot_size: 4,
                        bit_resolution: 24,
                    })),
                },
                AudioStreamingInterface {
                    interface_descriptors: Vec::from_slice(&[
                        InterfaceDescriptor {
                            len: 9,
                            descriptor_type: 4,
                            interface_number: 2,
                            alternate_setting: 0,
                            num_endpoints: 0,
                            interface_class: 1,
                            interface_subclass: 2,
                            interface_protocol: 32,
                            interface_name: 10,
                        },
                        InterfaceDescriptor {
                            len: 9,
                            descriptor_type: 4,
                            interface_number: 2,
                            alternate_setting: 1,
                            num_endpoints: 1,
                            interface_class: 1,
                            interface_subclass: 2,
                            interface_protocol: 32,
                            interface_name: 11,
                        },
                    ])
                    .unwrap(),
                    class_descriptor: AudioStreamingClassDescriptor {
                        terminal_link_id: 22,
                        controls_bitmap: 0,
                        format: format_type::Format::Type1(format_type::Type1::PCM),
                        num_channels: 16,
                        channel_config_bitmap: 0,
                        channel_name: 50,
                    },
                    endpoint_descriptor: Some(EndpointDescriptor {
                        len: 7,
                        descriptor_type: 5,
                        endpoint_address: 130,
                        attributes: 5,
                        max_packet_size: 512,
                        interval: 1,
                    }),
                    feedback_endpoint_descriptor: None,
                    audio_endpoint_descriptor: Some(AudioEndpointDescriptor {
                        attributes_bitmap: 0,
                        controls_bitmap: 0,
                        lock_delay_units: 2,
                        lock_delay: 8,
                    }),
                    format_type_descriptor: Some(FormatTypeDescriptor::I(FormatTypeI {
                        subslot_size: 4,
                        bit_resolution: 24,
                    })),
                },
            ])
            .unwrap(),
        };
        let audio_interface_collection = AudioInterfaceCollection::try_from_configuration(&descriptor).unwrap();
        // info!("{:#?}", audio_interface_collection);
        assert_eq!(audio_interface_collection, expected);
    }
}

//! USB Audio Class (UAC) descriptor parsing and management.

use heapless::Vec;
use heapless::index_map::FnvIndexMap;

use super::codes::*;
use crate::descriptor::descriptor_type::{CS_ENDPOINT, CS_INTERFACE, INTERFACE_ASSOCIATION};
use crate::descriptor::{
    ConfigurationDescriptorChain, DescriptorError, DescriptorVisitor, EndpointDescriptor, ExtendableDescriptor,
    InterfaceDescriptor, InterfaceDescriptorChain, StringIndex, USBDescriptor, VariableSizeDescriptor, VisitError,
};

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

impl From<DescriptorError> for AudioInterfaceError {
    fn from(_err: DescriptorError) -> Self {
        Self::InvalidDescriptor
    }
}

struct AudioCollectionBuilder {
    iad: Option<InterfaceAssociationDescriptor>,
    // Interface group cached until the CS_INTERFACE header arrives in on_other.
    // Non-empty means a group is pending; cleared when the header is consumed.
    interfaces: Vec<InterfaceDescriptor, MAX_ALTERNATE_SETTINGS>,
    control: Option<AudioControlInterface>,
    streaming: Vec<AudioStreamingInterface, MAX_AUDIO_STREAMING_INTERFACES>,
    error: Option<AudioInterfaceError>,
}

impl AudioCollectionBuilder {
    fn new() -> Self {
        Self {
            iad: None,
            interfaces: Vec::new(),
            control: None,
            streaming: Vec::new(),
            error: None,
        }
    }

    fn build(self) -> Result<AudioInterfaceCollection, AudioInterfaceError> {
        if let Some(e) = self.error {
            return Err(e);
        }
        Ok(AudioInterfaceCollection {
            interface_association_descriptor: self.iad.ok_or(AudioInterfaceError::NoAudioConfiguration)?,
            control_interface: self.control.ok_or(AudioInterfaceError::MissingControlInterface)?,
            audio_streaming_interfaces: self.streaming,
        })
    }
}

impl<'a> DescriptorVisitor<'a> for AudioCollectionBuilder {
    type Error = AudioInterfaceError;

    fn on_interface(&mut self, iface: &InterfaceDescriptorChain) -> bool {
        let Some(ref iad) = self.iad else {
            return true;
        };

        if iface.interface_number >= iad.first_interface + iad.num_interfaces {
            return false; // stop iteration
        }

        if iface.interface_class != interface::AUDIO {
            return true; // ignore
        }

        if iface.interface_protocol != function_protocol::AF_VERSION_02_00 {
            debug!(
                "Skipping interface with unsupported protocol: {:#04x}",
                iface.interface_protocol
            );
            return true;
        }

        match iface.interface_subclass {
            interface::subclass::AUDIOCONTROL => {
                if self.control.is_some() {
                    warn!("Audio Control Interface already parsed, skipping");
                    return true;
                }
                if !self.interfaces.is_empty() {
                    self.error = Some(AudioInterfaceError::MissingAudioStreamingClassDescriptor);
                    return false;
                }
                trace!("Processing Audio Control Interface");
                self.interfaces = Vec::from_slice(&[InterfaceDescriptor::from(iface)]).unwrap();
            }
            interface::subclass::AUDIOSTREAMING => {
                let is_alternate = self
                    .interfaces
                    .first()
                    .is_some_and(|i| i.interface_number == iface.interface_number);
                if is_alternate {
                    if self.interfaces.push(InterfaceDescriptor::from(iface)).is_err() {
                        self.error = Some(AudioInterfaceError::BufferFull("Too many interfaces"));
                        return false;
                    }
                    return true;
                }
                if !self.interfaces.is_empty() {
                    self.error = Some(match self.interfaces[0].interface_subclass {
                        interface::subclass::AUDIOCONTROL => AudioInterfaceError::MissingControlInterfaceHeader,
                        _ => AudioInterfaceError::MissingAudioStreamingClassDescriptor,
                    });
                    return false;
                }
                trace!("Processing Audio Streaming Interface");
                self.interfaces = Vec::from_slice(&[InterfaceDescriptor::from(iface)]).unwrap();
            }
            _ => {
                trace!("Skipping unknown audio subclass: {:#04x}", iface.interface_subclass);
            }
        }
        true
    }

    fn on_endpoint(&mut self, iface: &InterfaceDescriptorChain, ep: &EndpointDescriptor) -> bool {
        match iface.interface_subclass {
            interface::subclass::AUDIOSTREAMING => {
                if let Some(si) = self.streaming.last_mut() {
                    if ep.attributes == 0b010001 {
                        si.feedback_endpoint_descriptor = Some(*ep);
                    } else {
                        si.endpoint_descriptor = Some(*ep);
                    }
                }
            }
            interface::subclass::AUDIOCONTROL => {
                if let Some(ac) = &mut self.control {
                    ac.interrupt_endpoint_descriptor = Some(*ep);
                }
            }
            _ => {}
        }
        true
    }

    fn on_other(&mut self, _iface: Option<&InterfaceDescriptorChain>, raw: &[u8]) -> Result<bool, Self::Error> {
        if raw.len() < 2 {
            return Ok(true);
        }
        match raw[1] {
            INTERFACE_ASSOCIATION => {
                if self.iad.is_none() {
                    if let Ok(iad) = InterfaceAssociationDescriptor::try_from_bytes(raw) {
                        if iad.is_audio_association() {
                            self.iad = Some(iad);
                        }
                    }
                }
            }
            CS_INTERFACE => {
                if !self.interfaces.is_empty() {
                    match self.interfaces[0].interface_subclass {
                        interface::subclass::AUDIOCONTROL => {
                            if let Ok(header) = AudioControlHeaderDescriptor::try_from_bytes(raw) {
                                let interfaces = core::mem::take(&mut self.interfaces);
                                debug!(
                                    "Found Audio Control Header: version={}.{}",
                                    header.audio_device_class.0, header.audio_device_class.1
                                );
                                self.control = Some(AudioControlInterface::new(interfaces, header));
                                return Ok(true);
                            }
                        }
                        interface::subclass::AUDIOSTREAMING => {
                            if let Ok(class_desc) = AudioStreamingClassDescriptor::try_from_bytes(raw) {
                                let interfaces = core::mem::take(&mut self.interfaces);
                                trace!("Found Audio Streaming Class Descriptor: {:?}", class_desc.format);
                                self.streaming
                                    .push(AudioStreamingInterface::new(interfaces, class_desc))
                                    .map_err(|_| {
                                        AudioInterfaceError::BufferFull("Too many audio streaming interfaces")
                                    })?;
                                return Ok(true);
                            }
                        }
                        _ => {}
                    }
                }
                // Accumulate into whichever interface is currently in progress.
                if let Some(si) = self.streaming.last_mut() {
                    if let Ok(format_type) = FormatTypeDescriptor::try_from_bytes(raw) {
                        si.format_type_descriptor = Some(format_type);
                    }
                } else if let Some(ac) = &mut self.control {
                    ac.add_cs_interface(raw)?;
                }
            }
            CS_ENDPOINT => {
                if let Some(si) = self.streaming.last_mut() {
                    if let Ok(audio_ep) = AudioEndpointDescriptor::try_from_bytes(raw) {
                        si.audio_endpoint_descriptor = Some(audio_ep);
                    }
                }
            }
            _ => {}
        }
        Ok(true)
    }
}

impl AudioInterfaceCollection {
    /// Attempts to parse an audio interface collection from a configuration descriptor.
    ///
    /// This method searches for an interface association descriptor for audio,
    /// then parses the control interface and all streaming interfaces.
    ///
    /// Returns an [`AudioInterfaceCollection`] on success, or an [`AudioInterfaceError`] if parsing fails.
    pub fn try_from_configuration(cfg: &ConfigurationDescriptorChain) -> Result<Self, AudioInterfaceError> {
        let mut builder = AudioCollectionBuilder::new();
        cfg.visit_descriptors(&mut builder).map_err(|e| match e {
            VisitError::BadDescriptor => AudioInterfaceError::InvalidDescriptor,
            VisitError::Visitor(e) => e,
        })?;
        builder.build()
    }
}

/// USB interface association descriptor for grouping related interfaces.
///
/// This descriptor is used to associate multiple interfaces that belong to the same function,
/// such as an audio function with control and streaming interfaces. (USB Audio Devices 2.0 §4.6)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

impl ExtendableDescriptor for InterfaceAssociationDescriptor {
    const MIN_LEN: u8 = 8;
}

impl USBDescriptor for InterfaceAssociationDescriptor {
    const BUF_SIZE: usize = Self::MIN_LEN as usize;
    const DESC_TYPE: u8 = INTERFACE_ASSOCIATION;
    type Error = DescriptorError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::match_bytes(bytes)?;
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

impl AudioControlInterface {
    fn new(interfaces: Vec<InterfaceDescriptor, MAX_ALTERNATE_SETTINGS>, header: AudioControlHeaderDescriptor) -> Self {
        Self {
            interface_descriptors: interfaces,
            header_descriptor: header,
            interrupt_endpoint_descriptor: None,
            clock_descriptors: FnvIndexMap::new(),
            unit_descriptors: FnvIndexMap::new(),
            terminal_descriptors: FnvIndexMap::new(),
        }
    }

    fn add_cs_interface(&mut self, raw: &[u8]) -> Result<(), AudioInterfaceError> {
        match ClockDescriptor::try_from_bytes(raw) {
            Ok(clock) => {
                self.clock_descriptors
                    .insert(clock.clock_id(), clock)
                    .map_err(|_| AudioInterfaceError::BufferFull("Too many clock descriptors"))?;
            }
            Err(AudioInterfaceError::InvalidDescriptor) => {}
            Err(e) => return Err(e),
        }
        if let Ok(terminal) = TerminalDescriptor::try_from_bytes(raw) {
            self.terminal_descriptors
                .insert(terminal.terminal_id(), terminal)
                .map_err(|_| AudioInterfaceError::BufferFull("Too many terminal descriptors"))?;
        }
        if let Ok(unit) = UnitDescriptor::try_from_bytes(raw) {
            self.unit_descriptors
                .insert(unit.unit_id(), unit)
                .map_err(|_| AudioInterfaceError::BufferFull("Too many unit descriptors"))?;
        }
        Ok(())
    }
}

/// Audio control header descriptor containing version and category information. (USB Audio Devices 2.0 §4.7.2)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AudioControlHeaderDescriptor {
    /// Audio device class version (major, minor).
    pub audio_device_class: (u8, u8),
    /// Category of the audio device.
    pub category: u8,
    /// Bitmap of supported controls.
    pub controls_bitmap: u8,
}

impl ExtendableDescriptor for AudioControlHeaderDescriptor {
    const MIN_LEN: u8 = 9;
}

impl USBDescriptor for AudioControlHeaderDescriptor {
    const BUF_SIZE: usize = Self::MIN_LEN as usize;
    const DESC_TYPE: u8 = CS_INTERFACE;
    const DESC_SUBTYPE: Option<u8> = Some(ac_descriptor::HEADER);
    type Error = DescriptorError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::match_bytes(bytes)?;
        Ok(Self {
            audio_device_class: (bytes[4], bytes[3]),
            category: bytes[5],
            controls_bitmap: bytes[8],
        })
    }
}

/// Enumeration of clock descriptor types.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ClockDescriptor {
    /// Clock source descriptor.
    Source(ClockSourceDescriptor),
    /// Clock selector descriptor.
    Selector(ClockSelectorDescriptor),
    /// Clock multiplier descriptor.
    Multiplier(ClockMultiplierDescriptor),
}

impl ExtendableDescriptor for ClockDescriptor {
    const MIN_LEN: u8 = 3;
}

impl USBDescriptor for ClockDescriptor {
    // can hold any subdescriptor that is supported.
    const BUF_SIZE: usize = const_max![
        ClockSourceDescriptor::BUF_SIZE,
        ClockSelectorDescriptor::BUF_SIZE,
        ClockMultiplierDescriptor::BUF_SIZE,
    ];
    const DESC_TYPE: u8 = CS_INTERFACE;
    type Error = AudioInterfaceError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::match_bytes(bytes)?;
        match bytes[2] {
            ac_descriptor::CLOCK_SOURCE => Ok(Self::Source(ClockSourceDescriptor::try_from_bytes(bytes)?)),
            ac_descriptor::CLOCK_SELECTOR => Ok(Self::Selector(ClockSelectorDescriptor::try_from_bytes(bytes)?)),
            ac_descriptor::CLOCK_MULTIPLIER => Ok(Self::Multiplier(ClockMultiplierDescriptor::try_from_bytes(bytes)?)),
            _ => Err(AudioInterfaceError::InvalidDescriptor),
        }
    }
}

impl ClockDescriptor {
    /// Returns the clock ID for this descriptor.
    pub fn clock_id(&self) -> u8 {
        match self {
            Self::Source(desc) => desc.clock_id,
            Self::Selector(desc) => desc.clock_id,
            Self::Multiplier(desc) => desc.clock_id,
        }
    }
}

/// Clock source descriptor defining an audio clock source. (USB Audio Devices 2.0 §4.7.2.1)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

impl ExtendableDescriptor for ClockSourceDescriptor {
    const MIN_LEN: u8 = 8;
}

impl USBDescriptor for ClockSourceDescriptor {
    const BUF_SIZE: usize = Self::MIN_LEN as usize;
    const DESC_TYPE: u8 = CS_INTERFACE;
    const DESC_SUBTYPE: Option<u8> = Some(ac_descriptor::CLOCK_SOURCE);
    type Error = DescriptorError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::match_bytes(bytes)?;
        Ok(Self {
            clock_id: bytes[3],
            attributes_bitmap: bytes[4],
            controls_bitmap: bytes[5],
            associated_terminal: bytes[6],
            clock_name: bytes[7],
        })
    }
}

/// Clock selector descriptor for selecting between multiple clock sources. (USB Audio Devices 2.0 §4.7.2.2)
#[derive(Clone, Debug, PartialEq, Eq)]
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
    /// Maximum number of source ids that we support (at most 248).
    pub const SUPPORTED_SOURCE_IDS: u8 = MAX_CLOCK_DESCRIPTORS as u8;
}

impl VariableSizeDescriptor for ClockSelectorDescriptor {
    const MIN_LEN: u8 = 7;
    const MAX_LEN: u8 = u8::MAX;

    /// Matches length with the number of source ids.
    #[inline(always)]
    fn match_bytes_len(bytes: &[u8]) -> bool {
        if bytes.len() < 4 {
            return false;
        }
        let len = bytes[0] as usize;
        let num_source_ids = bytes[4] as usize;
        len == 7 + num_source_ids
    }
}

impl USBDescriptor for ClockSelectorDescriptor {
    const BUF_SIZE: usize = 7 + Self::SUPPORTED_SOURCE_IDS as usize;
    const DESC_TYPE: u8 = CS_INTERFACE;
    const DESC_SUBTYPE: Option<u8> = Some(ac_descriptor::CLOCK_SELECTOR);
    type Error = AudioInterfaceError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, AudioInterfaceError> {
        Self::match_bytes(bytes)?;
        let mut source_ids = Vec::new();
        let num_source_ids = bytes[4];
        if num_source_ids as usize > source_ids.capacity() {
            return Err(AudioInterfaceError::BufferFull("Too many clock source ids"));
        }
        for i in 0..num_source_ids as usize {
            if let Some(&source_id) = bytes.get(5 + i) {
                let result = source_ids.push(source_id);
                debug_assert!(result.is_ok(), "push must work");
            } else {
                debug_assert!(false, "source_id must exist");
            }
        }
        Ok(Self {
            clock_id: bytes[3],
            source_ids,
            controls_bitmap: *bytes.get(5 + num_source_ids as usize).unwrap_or(&0),
            clock_name: *bytes.get(6 + num_source_ids as usize).unwrap_or(&0),
        })
    }
}

/// Clock multiplier descriptor for frequency multiplication. (USB Audio Devices 2.0 §4.7.2.3)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

impl ExtendableDescriptor for ClockMultiplierDescriptor {
    const MIN_LEN: u8 = 7;
}

impl USBDescriptor for ClockMultiplierDescriptor {
    const BUF_SIZE: usize = Self::MIN_LEN as usize;
    const DESC_TYPE: u8 = CS_INTERFACE;
    const DESC_SUBTYPE: Option<u8> = Some(ac_descriptor::CLOCK_MULTIPLIER);
    type Error = DescriptorError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, DescriptorError> {
        Self::match_bytes(bytes)?;
        Ok(Self {
            clock_id: bytes[3],
            source_id: bytes[4],
            controls_bitmap: bytes[5],
            clock_name: bytes[6],
        })
    }
}

/// Enumeration of terminal descriptor types.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TerminalDescriptor {
    /// Input terminal descriptor.
    Input(InputTerminalDescriptor),
    /// Output terminal descriptor.
    Output(OutputTerminalDescriptor),
}

impl ExtendableDescriptor for TerminalDescriptor {
    const MIN_LEN: u8 = 3;
}

impl USBDescriptor for TerminalDescriptor {
    // can hold any subdescriptor that is supported
    const BUF_SIZE: usize = const_max![InputTerminalDescriptor::BUF_SIZE, OutputTerminalDescriptor::BUF_SIZE,];
    const DESC_TYPE: u8 = CS_INTERFACE;
    type Error = DescriptorError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::match_bytes(bytes)?;
        match bytes[2] {
            ac_descriptor::INPUT_TERMINAL => Ok(Self::Input(InputTerminalDescriptor::try_from_bytes(bytes)?)),
            ac_descriptor::OUTPUT_TERMINAL => Ok(Self::Output(OutputTerminalDescriptor::try_from_bytes(bytes)?)),
            _ => Err(DescriptorError::BadDescriptorType),
        }
    }
}

impl TerminalDescriptor {
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

/// Input terminal descriptor for audio input sources. (USB Audio Devices 2.0 §4.7.2.4)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

impl ExtendableDescriptor for InputTerminalDescriptor {
    const MIN_LEN: u8 = 17;
}

impl USBDescriptor for InputTerminalDescriptor {
    const BUF_SIZE: usize = Self::MIN_LEN as usize;
    const DESC_TYPE: u8 = CS_INTERFACE;
    const DESC_SUBTYPE: Option<u8> = Some(ac_descriptor::INPUT_TERMINAL);
    type Error = DescriptorError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::match_bytes(bytes)?;
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

/// Output terminal descriptor for audio output destinations. (USB Audio Devices 2.0 §4.7.2.5)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

impl ExtendableDescriptor for OutputTerminalDescriptor {
    const MIN_LEN: u8 = 12;
}

impl USBDescriptor for OutputTerminalDescriptor {
    const BUF_SIZE: usize = Self::MIN_LEN as usize;
    const DESC_TYPE: u8 = CS_INTERFACE;
    const DESC_SUBTYPE: Option<u8> = Some(ac_descriptor::OUTPUT_TERMINAL);
    type Error = DescriptorError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::match_bytes(bytes)?;
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
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum UnitDescriptor {
    /// Mixer unit with unit ID. (USB Audio Devices 2.0 §4.7.2.6)
    Mixer(u8),
    /// Selector unit with unit ID. (USB Audio Devices 2.0 §4.7.2.7)
    Selector(u8),
    /// Feature unit with unit ID. (USB Audio Devices 2.0 §4.7.2.8)
    Feature(u8),
    /// Processing unit with unit ID. (USB Audio Devices 2.0 §4.7.2.11)
    Processing(u8),
    /// Effect unit with unit ID. (USB Audio Devices 2.0 §4.7.2.10)
    Effect(u8),
    /// Sample rate converter unit with unit ID. (USB Audio Devices 2.0 §4.7.2.9)
    SampleRateConverter(u8),
    /// Extension unit with unit ID. (USB Audio Devices 2.0 §4.7.2.12)
    Extension(u8),
}

impl ExtendableDescriptor for UnitDescriptor {
    const MIN_LEN: u8 = 4;
}

impl USBDescriptor for UnitDescriptor {
    // This is not the true size; Will become variable
    const BUF_SIZE: usize = Self::MIN_LEN as usize;
    const DESC_TYPE: u8 = CS_INTERFACE;
    type Error = DescriptorError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::match_bytes(bytes)?;
        match bytes[2] {
            ac_descriptor::MIXER_UNIT => Ok(Self::Mixer(bytes[3])),
            ac_descriptor::SELECTOR_UNIT => Ok(Self::Selector(bytes[3])),
            ac_descriptor::FEATURE_UNIT => Ok(Self::Feature(bytes[3])),
            ac_descriptor::PROCESSING_UNIT => Ok(Self::Processing(bytes[3])),
            ac_descriptor::EFFECT_UNIT => Ok(Self::Effect(bytes[3])),
            ac_descriptor::SAMPLE_RATE_CONVERTER => Ok(Self::SampleRateConverter(bytes[3])),
            ac_descriptor::EXTENSION_UNIT => Ok(Self::Extension(bytes[3])),
            _ => Err(DescriptorError::BadDescriptorType),
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

impl AudioStreamingInterface {
    fn new(
        interfaces: Vec<InterfaceDescriptor, MAX_ALTERNATE_SETTINGS>,
        class_desc: AudioStreamingClassDescriptor,
    ) -> Self {
        Self {
            interface_descriptors: interfaces,
            class_descriptor: class_desc,
            endpoint_descriptor: None,
            feedback_endpoint_descriptor: None,
            audio_endpoint_descriptor: None,
            format_type_descriptor: None,
        }
    }
}

/// Audio streaming class descriptor containing format and channel information. (USB Audio Devices 2.0 §4.9.2)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

impl ExtendableDescriptor for AudioStreamingClassDescriptor {
    const MIN_LEN: u8 = 16;
}

impl USBDescriptor for AudioStreamingClassDescriptor {
    const BUF_SIZE: usize = Self::MIN_LEN as usize;
    const DESC_TYPE: u8 = CS_INTERFACE;
    const DESC_SUBTYPE: Option<u8> = Some(as_descriptor::GENERAL);
    type Error = DescriptorError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::match_bytes(bytes)?;
        let format =
            format_type::Format::from_u32(bytes[5], u32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]));
        if format.is_none() {
            error!("Invalid format type descriptor: type {:?}", bytes[5]);
            return Err(DescriptorError::BadDescriptorData);
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

/// Audio-specific endpoint descriptor containing audio endpoint attributes. (USB Audio Devices 2.0 §4.10.1.2)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

impl ExtendableDescriptor for AudioEndpointDescriptor {
    const MIN_LEN: u8 = 8;
}

impl USBDescriptor for AudioEndpointDescriptor {
    const BUF_SIZE: usize = Self::MIN_LEN as usize;
    const DESC_TYPE: u8 = descriptor_type::CS_ENDPOINT;
    const DESC_SUBTYPE: Option<u8> = Some(as_descriptor::GENERAL);
    type Error = DescriptorError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::match_bytes(bytes)?;
        Ok(Self {
            attributes_bitmap: bytes[3],
            controls_bitmap: bytes[4],
            lock_delay_units: bytes[5],
            lock_delay: u16::from_le_bytes([bytes[6], bytes[7]]),
        })
    }
}

/// Enumeration of format type descriptors for different audio formats.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FormatTypeDescriptor {
    /// Type I format (PCM, PCM8, etc.). (USB Audio Data Formats 2.0 §2.3.1.6)
    I(FormatTypeI),
    /// Type II format (MPEG, AC-3, etc.). (USB Audio Data Formats 2.0 §2.3.2.6)
    II(FormatTypeII),
    /// Type III format (IEC1937_AC-3, IEC1937_MPEG-1_Layer1, etc.). (USB Audio Data Formats 2.0 §2.3.3.1)
    III(FormatTypeIII),
    /// Type IV format. (USB Audio Data Formats 2.0 §2.3.4.1)
    IV,
    /// Extended Type I format. (USB Audio Data Formats 2.0 §2.4.1.1)
    ExtendedI(FormatTypeExtendedI),
    /// Extended Type II format. (USB Audio Data Formats 2.0 §2.4.2.1)
    ExtendedII(FormatTypeExtendedII),
    /// Extended Type III format. (USB Audio Data Formats 2.0 §2.4.3.1)
    ExtendedIII(FormatTypeExtendedIII),
}

impl FormatTypeDescriptor {
    /// Size of the byte buffer for [FormatTypeDescriptor::I].
    pub const BUF_SIZE_I: usize = 6;
    /// Size of the byte buffer for [FormatTypeDescriptor::II].
    pub const BUF_SIZE_II: usize = 8;
    /// Size of the byte buffer for [FormatTypeDescriptor::III].
    pub const BUF_SIZE_III: usize = 6;
    /// Size of the byte buffer for [FormatTypeDescriptor::IV].
    pub const BUF_SIZE_IV: usize = 4;
    /// Size of the byte buffer for [FormatTypeDescriptor::ExtendedI].
    pub const BUF_SIZE_EXTENDED_I: usize = 9;
    /// Size of the byte buffer for [FormatTypeDescriptor::ExtendedII].
    pub const BUF_SIZE_EXTENDED_II: usize = 10;
    /// Size of the byte buffer for [FormatTypeDescriptor::ExtendedIII].
    pub const BUF_SIZE_EXTENDED_III: usize = 8;
}

impl ExtendableDescriptor for FormatTypeDescriptor {
    const MIN_LEN: u8 = 4;
}

impl USBDescriptor for FormatTypeDescriptor {
    const BUF_SIZE: usize = const_max![
        Self::BUF_SIZE_I,
        Self::BUF_SIZE_II,
        Self::BUF_SIZE_III,
        Self::BUF_SIZE_IV,
        Self::BUF_SIZE_EXTENDED_I,
        Self::BUF_SIZE_EXTENDED_II,
        Self::BUF_SIZE_EXTENDED_III,
    ];
    const DESC_TYPE: u8 = CS_INTERFACE;
    const DESC_SUBTYPE: Option<u8> = Some(as_descriptor::FORMAT_TYPE);
    type Error = DescriptorError;

    fn try_from_bytes(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::match_bytes(bytes)?;
        let len = bytes[0] as usize;
        if bytes.len() < len {
            return Err(DescriptorError::UnexpectedEndOfBuffer);
        }
        match bytes[3] {
            format_type::I => {
                if len != Self::BUF_SIZE_I {
                    return Err(DescriptorError::BadDescriptorSize);
                }
                Ok(Self::I(FormatTypeI {
                    subslot_size: bytes[4],
                    bit_resolution: bytes[5],
                }))
            }
            format_type::II => {
                if len != Self::BUF_SIZE_II {
                    return Err(DescriptorError::BadDescriptorSize);
                }
                Ok(Self::II(FormatTypeII {
                    max_bit_rate: u16::from_le_bytes([bytes[4], bytes[5]]),
                    slots_per_frame: u16::from_le_bytes([bytes[6], bytes[7]]),
                }))
            }
            format_type::III => {
                if len != Self::BUF_SIZE_III {
                    return Err(DescriptorError::BadDescriptorSize);
                }
                Ok(Self::III(FormatTypeIII {
                    subslot_size: bytes[4],
                    bit_resolution: bytes[5],
                }))
            }
            format_type::IV => Ok(Self::IV),
            format_type::EXT_I => {
                if len != Self::BUF_SIZE_EXTENDED_I {
                    return Err(DescriptorError::BadDescriptorSize);
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
                if len != Self::BUF_SIZE_EXTENDED_II {
                    return Err(DescriptorError::BadDescriptorSize);
                }
                Ok(Self::ExtendedII(FormatTypeExtendedII {
                    max_bit_rate: u16::from_le_bytes([bytes[4], bytes[5]]),
                    samples_per_frame: u16::from_le_bytes([bytes[6], bytes[7]]),
                    header_length: bytes[8],
                    sideband_protocol: bytes[9],
                }))
            }
            format_type::EXT_III => {
                if len != Self::BUF_SIZE_EXTENDED_III {
                    return Err(DescriptorError::BadDescriptorSize);
                }
                Ok(Self::ExtendedIII(FormatTypeExtendedIII {
                    subslot_size: bytes[4],
                    bit_resolution: bytes[5],
                    header_length: bytes[6],
                    sideband_protocol: bytes[7],
                }))
            }
            _ => Err(DescriptorError::BadDescriptorData),
        }
    }
}

/// Type I format descriptor for PCM-like formats.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FormatTypeI {
    /// Size of each subslot in bytes.
    pub subslot_size: u8,
    /// Bit resolution of the audio data.
    pub bit_resolution: u8,
}

/// Type II format descriptor for compressed formats.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FormatTypeII {
    /// Maximum bit rate in bits per second.
    pub max_bit_rate: u16,
    /// Number of slots per frame.
    pub slots_per_frame: u16,
}

/// Type III format descriptor for IEC formats.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct FormatTypeIII {
    /// Size of each subslot in bytes.
    pub subslot_size: u8,
    /// Bit resolution of the audio data.
    pub bit_resolution: u8,
}

/// Extended Type I format descriptor.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
        let _ = env_logger::try_init();

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
        let descriptor = ConfigurationDescriptorChain {
            descriptor: ConfigurationDescriptor {
                total_len: 0,
                num_interfaces: 0,
                configuration_value: 1,
                configuration_name: 0,
                attributes: 0,
                max_power: 0,
            },
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
                            interface_number: 1,
                            alternate_setting: 0,
                            num_endpoints: 0,
                            interface_class: 1,
                            interface_subclass: 2,
                            interface_protocol: 32,
                            interface_name: 8,
                        },
                        InterfaceDescriptor {
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
                        endpoint_address: 1,
                        attributes: 5,
                        max_packet_size: 512,
                        interval: 1,
                    }),
                    feedback_endpoint_descriptor: Some(EndpointDescriptor {
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
                            interface_number: 2,
                            alternate_setting: 0,
                            num_endpoints: 0,
                            interface_class: 1,
                            interface_subclass: 2,
                            interface_protocol: 32,
                            interface_name: 10,
                        },
                        InterfaceDescriptor {
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

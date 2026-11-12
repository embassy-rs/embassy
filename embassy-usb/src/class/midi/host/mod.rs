//! USB MIDI 1.0 host class driver.
//!
//! USB MIDI 1.0 transports one or more virtual MIDI cables over bulk
//! endpoints. Each transfer consists of four-byte USB-MIDI event packets.

use core::marker::PhantomData;

use embassy_usb_driver::host::{PipeError, UsbHostAllocator, UsbPipe, pipe};
use heapless::Vec;

use crate::class::midi::{
    CS_GR_TRM_BLOCK, GR_TRM_BLOCK_HEADER_LEN, GR_TRM_BLOCK_LEN, GroupTerminalBlock, GtbError, MAX_GTBS, MAX_MIDI_JACKS,
    MAX_UMP_BYTES, MIDI_PACKET_SIZE, MessageType, MidiPacket, MidiPacketError, MidiPacketReader, USB_AUDIO_CLASS,
    USB_MIDISTREAMING_SUBCLASS, UmpError, UmpPacket, UmpReader, endpoint_groups, parse_gtb_descriptors,
};
use crate::host::class::{ClassError, ClassMatch, HostClass};
use crate::host::descriptor::{ClassCode, InterfaceDescriptor};
use crate::host::handler::EnumerationInfo;

/// Matcher for the MIDI streaming interface.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct MidiMatcher;

impl ClassMatch for MidiMatcher {
    const NAME: &'static str = "MIDI";
    const CODE: Option<ClassCode> = Some(ClassCode::new(USB_AUDIO_CLASS, USB_MIDISTREAMING_SUBCLASS, 0));

    fn matches(iface: &InterfaceDescriptor) -> bool {
        iface.interface_class == USB_AUDIO_CLASS && iface.interface_subclass == USB_MIDISTREAMING_SUBCLASS
    }
}

mod descriptors;
mod transport;

use descriptors::{
    MidiDescriptorError, MidiEndpointDescriptor, MidiStreamingInterface, parse_midi_interfaces_for_device,
};

/// Advanced descriptor and endpoint-level USB-MIDI APIs.
pub mod raw {
    pub use super::descriptors::*;
    pub use super::transport::{MidiInputPipe, MidiOutputPipe, UmpInputPipe, UmpOutputPipe};
}

/// USB MIDI host error.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MidiError {
    /// A USB transfer failed.
    Transfer(PipeError),
    /// No MIDIStreaming interface was found.
    NoInterface,
    /// The requested transfer direction is not exposed by the interface.
    NoEndpoint,
    /// An endpoint was opened using the wrong transfer direction.
    WrongDirection,
    /// The controller could not allocate an endpoint pipe.
    NoPipe,
    /// The MIDI descriptors are malformed or exceed a fixed capacity.
    Descriptor(MidiDescriptorError),
    /// The device uses a topology unsupported by the friendly host API.
    UnsupportedTopology,
    /// A port does not belong to this MIDI device direction.
    InvalidPort,
    /// A MIDI message could not be encoded, or a transfer is not made of complete event packets.
    Packet(MidiPacketError),
    /// A Universal MIDI Packet error occurred.
    Ump(UmpError),
    /// A Group Terminal Block descriptor error occurred.
    Gtb(GtbError),
    /// A packet is addressed to a group the function does not accept.
    InvalidGroup,
}

impl MidiError {
    /// Whether MIDI 2.0 is unusable on this device, so MIDI 1.0 is worth trying (not for resource or link errors).
    pub fn is_midi2_failure(&self) -> bool {
        matches!(
            self,
            Self::Transfer(PipeError::Stall)
                | Self::NoInterface
                | Self::UnsupportedTopology
                | Self::Descriptor(_)
                | Self::Gtb(_)
        )
    }
}

impl From<PipeError> for MidiError {
    fn from(error: PipeError) -> Self {
        Self::Transfer(error)
    }
}

impl From<MidiDescriptorError> for MidiError {
    fn from(error: MidiDescriptorError) -> Self {
        Self::Descriptor(error)
    }
}

impl From<MidiPacketError> for MidiError {
    fn from(error: MidiPacketError) -> Self {
        Self::Packet(error)
    }
}

impl From<UmpError> for MidiError {
    fn from(error: UmpError) -> Self {
        Self::Ump(error)
    }
}

impl From<GtbError> for MidiError {
    fn from(error: GtbError) -> Self {
        Self::Gtb(error)
    }
}

impl From<MidiError> for ClassError {
    fn from(error: MidiError) -> Self {
        match error {
            MidiError::Transfer(error) => Self::Transfer(error),
            MidiError::NoInterface => Self::NoMatch,
            MidiError::NoPipe => Self::NoPipe,
            MidiError::Descriptor(_) | MidiError::Gtb(_) => Self::InvalidDescriptor,
            MidiError::Packet(_) | MidiError::Ump(_) => Self::InvalidResponse,
            MidiError::UnsupportedTopology => Self::Unsupported("MIDI topology"),
            _ => Self::Unsupported("MIDI"),
        }
    }
}

impl core::fmt::Display for MidiError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Transfer(_) => write!(f, "USB MIDI transfer failed"),
            Self::NoInterface => write!(f, "no USB MIDIStreaming interface found"),
            Self::NoEndpoint => write!(f, "USB MIDI endpoint direction unavailable"),
            Self::WrongDirection => write!(f, "USB MIDI endpoint has the wrong direction"),
            Self::NoPipe => write!(f, "no free USB host pipe"),
            Self::Descriptor(_) => write!(f, "invalid USB MIDI descriptors"),
            Self::UnsupportedTopology => write!(f, "unsupported USB MIDI topology"),
            Self::InvalidPort => write!(f, "USB MIDI port does not belong to this device direction"),
            Self::Packet(_) => write!(f, "invalid MIDI message or USB-MIDI packet"),
            Self::Ump(_) => write!(f, "invalid Universal MIDI Packet"),
            Self::Gtb(_) => write!(f, "invalid Group Terminal Block descriptor"),
            Self::InvalidGroup => write!(f, "UMP group not accepted by the function"),
        }
    }
}

impl core::error::Error for MidiError {}

/// Marker for device-to-host ports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct In;

/// Marker for host-to-device ports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Out;

/// A logical device-to-host MIDI port discovered from a USB cable association.
pub type MidiInputPort = MidiPort<In>;

/// A logical host-to-device MIDI port discovered from a USB cable association.
pub type MidiOutputPort = MidiPort<Out>;

/// A logical MIDI port discovered from a USB cable association; `D` is [`In`] or [`Out`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MidiPort<D> {
    device_address: u8,
    interface_number: u8,
    endpoint_address: u8,
    cable: u8,
    jack_id: u8,
    direction: PhantomData<D>,
}

impl<D> MidiPort<D> {
    /// One-based port number suitable for display.
    pub const fn number(&self) -> u8 {
        self.cable + 1
    }

    /// Zero-based USB-MIDI virtual cable number.
    pub const fn cable(&self) -> u8 {
        self.cable
    }

    /// Associated embedded jack identifier.
    pub const fn jack_id(&self) -> u8 {
        self.jack_id
    }

    /// MIDIStreaming interface number.
    pub const fn interface_number(&self) -> u8 {
        self.interface_number
    }
}

/// Friendly host driver for one USB MIDI 1.0 streaming interface.
///
/// Input-only and output-only devices are represented by an empty port slice
/// on the unsupported direction.
///
/// It uses alternate setting 0 and sends no control request, so a device last used as MIDI 2.0
/// must first go through [`MidiInfo::deactivate`].
///
/// This convenience API supports one alternate-setting-zero MIDIStreaming
/// interface with at most one bulk endpoint per direction. Use [`raw`] for
/// devices with multiple streaming interfaces or same-direction endpoints.
pub struct MidiHost<'d, A: UsbHostAllocator<'d>> {
    sender: Option<MidiSender<'d, A>>,
    receiver: Option<MidiReceiver<'d, A>>,
    info: MidiInfo,
}

impl<'d, A: UsbHostAllocator<'d>> MidiHost<'d, A> {
    /// Discover the MIDI ports and allocate the available bulk pipes.
    pub fn new(alloc: &A, config_desc: &[u8], enum_info: &EnumerationInfo) -> Result<Self, MidiError> {
        Self::from_info(alloc, MidiInfo::midi1_from_config(config_desc, enum_info)?, enum_info)
    }

    /// Allocate the bulk pipes for a MIDI 1.0 function described by `info`.
    pub fn from_info(alloc: &A, info: MidiInfo, enum_info: &EnumerationInfo) -> Result<Self, MidiError> {
        if info.version != MidiVersion::Midi1 {
            return Err(MidiError::NoInterface);
        }
        let receiver = info
            .input_endpoint
            .as_ref()
            .map(|e| {
                MidiReceiver::open(
                    alloc,
                    info.interface_number,
                    &e.to_descriptor(MidiVersion::Midi1),
                    enum_info,
                )
            })
            .transpose()?;
        let sender = info
            .output_endpoint
            .as_ref()
            .map(|e| {
                MidiSender::open(
                    alloc,
                    info.interface_number,
                    &e.to_descriptor(MidiVersion::Midi1),
                    enum_info,
                )
            })
            .transpose()?;
        Ok(Self { sender, receiver, info })
    }

    /// The function this host was opened from.
    pub fn info(&self) -> &MidiInfo {
        &self.info
    }

    /// Logical device-to-host MIDI ports.
    pub fn input_ports(&self) -> &[MidiInputPort] {
        self.receiver.as_ref().map_or(&[], |receiver| receiver.ports())
    }

    /// Logical host-to-device MIDI ports.
    pub fn output_ports(&self) -> &[MidiOutputPort] {
        self.sender.as_ref().map_or(&[], |sender| sender.ports())
    }

    /// Receive one bulk transfer containing USB-MIDI event packets.
    pub async fn read_transfer(&mut self, buf: &mut [u8]) -> Result<usize, MidiError> {
        self.receiver
            .as_mut()
            .ok_or(MidiError::NoEndpoint)?
            .read_transfer(buf)
            .await
    }

    /// Send one or more complete USB-MIDI event packets.
    pub async fn write_packet(&mut self, packets: &[u8]) -> Result<(), MidiError> {
        self.sender
            .as_mut()
            .ok_or(MidiError::NoEndpoint)?
            .write_packet(packets)
            .await
    }

    /// Encode and send one MIDI message, or a SysEx fragment of up to three bytes, to a logical port.
    pub async fn send(&mut self, port: MidiOutputPort, message: &[u8]) -> Result<(), MidiError> {
        self.sender
            .as_mut()
            .ok_or(MidiError::NoEndpoint)?
            .send(port, message)
            .await
    }

    /// See [`MidiSender::send_sysex`].
    pub async fn send_sysex(&mut self, port: MidiOutputPort, message: &[u8]) -> Result<(), MidiError> {
        self.sender
            .as_mut()
            .ok_or(MidiError::NoEndpoint)?
            .send_sysex(port, message)
            .await
    }

    /// Split into independently owned sender and receiver halves, plus the discovery info.
    ///
    /// This allows sending and receiving from separate tasks.
    pub fn split(self) -> (Option<MidiSender<'d, A>>, Option<MidiReceiver<'d, A>>, MidiInfo) {
        (self.sender, self.receiver, self.info)
    }
}

fn select_streaming_interface(
    interfaces: &[MidiStreamingInterface],
) -> Result<
    (
        &MidiStreamingInterface,
        Option<&MidiEndpointDescriptor>,
        Option<&MidiEndpointDescriptor>,
    ),
    MidiError,
> {
    // Only bulk endpoints can be opened, so others are ignored.
    let bulk = |endpoint: &&MidiEndpointDescriptor| endpoint.ep_type == embassy_usb_driver::EndpointType::Bulk;
    let mut matching = interfaces.iter().filter(|interface| {
        interface.alternate_setting == 0 && !interface.is_midi2() && interface.endpoints.iter().any(|e| bulk(&e))
    });
    let interface = matching.next().ok_or(MidiError::NoInterface)?;
    if matching.next().is_some() {
        return Err(MidiError::UnsupportedTopology);
    }

    let mut input_endpoint = None;
    let mut output_endpoint = None;
    for endpoint in interface.endpoints.iter().filter(bulk) {
        let slot = if endpoint.is_in() {
            &mut input_endpoint
        } else {
            &mut output_endpoint
        };
        if slot.replace(endpoint).is_some() {
            return Err(MidiError::UnsupportedTopology);
        }
    }

    Ok((interface, input_endpoint, output_endpoint))
}

/// USB-MIDI host-to-device packet sender.
pub struct MidiSender<'d, A: UsbHostAllocator<'d>> {
    pipe: transport::MidiOutputPipe<'d, A>,
    ports: Vec<MidiOutputPort, MAX_MIDI_JACKS>,
    max_packet_size: usize,
}

impl<'d, A: UsbHostAllocator<'d>> MidiSender<'d, A> {
    /// Allocate the pipe of `endpoint`, whose cable associations give the ports.
    pub fn open(
        alloc: &A,
        interface_number: u8,
        endpoint: &MidiEndpointDescriptor,
        enum_info: &EnumerationInfo,
    ) -> Result<Self, MidiError> {
        let pipe = transport::MidiOutputPipe::open(alloc, endpoint, enum_info)?;
        Self::new(pipe, interface_number, endpoint, enum_info.device_address)
    }

    /// Wrap an output pipe; the ports come from `endpoint`'s cable associations.
    pub fn new(
        pipe: transport::MidiOutputPipe<'d, A>,
        interface_number: u8,
        endpoint: &MidiEndpointDescriptor,
        device_address: u8,
    ) -> Result<Self, MidiError> {
        let ports = cable_ports(interface_number, endpoint, device_address)?;
        Ok(Self {
            pipe,
            ports,
            max_packet_size: usize::from(endpoint.max_packet_size),
        })
    }

    /// Logical ports that can receive MIDI from the host.
    pub fn ports(&self) -> &[MidiOutputPort] {
        &self.ports
    }

    /// Mask of the cables the host may send to; cable `n` is group `n`.
    pub fn groups(&self) -> u16 {
        cable_mask(&self.ports)
    }

    /// Send one or more complete USB-MIDI event packets.
    pub async fn write_packet(&mut self, packets: &[u8]) -> Result<(), MidiError> {
        self.pipe.write(packets).await
    }

    /// Encode and send one MIDI message, or a SysEx fragment of up to three bytes, to a logical port.
    pub async fn send(&mut self, port: MidiOutputPort, message: &[u8]) -> Result<(), MidiError> {
        if !self.ports.contains(&port) {
            return Err(MidiError::InvalidPort);
        }
        let packet = MidiPacket::try_encode_exact(port.cable, message)?;
        self.write_packet(packet.as_bytes()).await
    }

    /// Send a complete `SysEx` message (`F0` to `F7`) to a logical port, as three-byte event packets
    /// in transfers of at most the max packet size.
    pub async fn send_sysex(&mut self, port: MidiOutputPort, message: &[u8]) -> Result<(), MidiError> {
        if !self.ports.contains(&port) {
            return Err(MidiError::InvalidPort);
        }
        let limit = self.max_packet_size.min(64) & !3;
        if limit == 0 {
            return Err(MidiPacketError::InvalidPacketLength.into());
        }
        let mut buf = [0u8; 64];
        let mut len = 0;
        for packet in SysexPackets::new(port.cable, message)? {
            if len + MIDI_PACKET_SIZE > limit {
                self.write_packet(&buf[..len]).await?;
                len = 0;
            }
            buf[len..len + MIDI_PACKET_SIZE].copy_from_slice(packet?.as_bytes());
            len += MIDI_PACKET_SIZE;
        }
        self.write_packet(&buf[..len]).await
    }
}

/// The USB-MIDI event packets of one complete `SysEx` message, three bytes each.
struct SysexPackets<'a> {
    cable: u8,
    rest: &'a [u8],
}

impl<'a> SysexPackets<'a> {
    /// `message` runs from `F0` to `F7` with only data bytes between.
    fn new(cable: u8, message: &'a [u8]) -> Result<Self, MidiPacketError> {
        let valid = message.len() >= 2
            && message[0] == 0xf0
            && message[message.len() - 1] == 0xf7
            && message[1..message.len() - 1].iter().all(|&byte| byte < 0x80);
        if !valid {
            return Err(MidiPacketError::InvalidEventData);
        }
        Ok(Self { cable, rest: message })
    }
}

impl Iterator for SysexPackets<'_> {
    type Item = Result<MidiPacket, MidiPacketError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rest.is_empty() {
            return None;
        }
        let (chunk, rest) = self.rest.split_at(self.rest.len().min(3));
        self.rest = rest;
        Some(MidiPacket::try_encode(self.cable, chunk).map(|(packet, _)| packet))
    }
}

/// USB-MIDI device-to-host packet receiver.
pub struct MidiReceiver<'d, A: UsbHostAllocator<'d>> {
    pipe: transport::MidiInputPipe<'d, A>,
    ports: Vec<MidiInputPort, MAX_MIDI_JACKS>,
}

impl<'d, A: UsbHostAllocator<'d>> MidiReceiver<'d, A> {
    /// Allocate the pipe of `endpoint`, whose cable associations give the ports.
    pub fn open(
        alloc: &A,
        interface_number: u8,
        endpoint: &MidiEndpointDescriptor,
        enum_info: &EnumerationInfo,
    ) -> Result<Self, MidiError> {
        let pipe = transport::MidiInputPipe::open(alloc, endpoint, enum_info)?;
        Self::new(pipe, interface_number, endpoint, enum_info.device_address)
    }

    /// Wrap an input pipe; the ports come from `endpoint`'s cable associations.
    pub fn new(
        pipe: transport::MidiInputPipe<'d, A>,
        interface_number: u8,
        endpoint: &MidiEndpointDescriptor,
        device_address: u8,
    ) -> Result<Self, MidiError> {
        let ports = cable_ports(interface_number, endpoint, device_address)?;
        Ok(Self { pipe, ports })
    }

    /// Logical ports that can send MIDI to the host.
    pub fn ports(&self) -> &[MidiInputPort] {
        &self.ports
    }

    /// Mask of the cables the function sends to the host; cable `n` is group `n`.
    pub fn groups(&self) -> u16 {
        cable_mask(&self.ports)
    }

    /// Receive one bulk transfer containing complete USB-MIDI event packets.
    pub async fn read_transfer(&mut self, buf: &mut [u8]) -> Result<usize, MidiError> {
        self.pipe.read(buf).await
    }

    /// Receive one transfer and map each event packet to its logical input port.
    pub async fn receive<'a>(&'a mut self, buffer: &'a mut [u8]) -> Result<ReceivedMidiPackets<'a>, MidiError> {
        let length = self.read_transfer(buffer).await?;
        // `read_transfer` checked the length, so this is `None` only for a zero-length transfer.
        let packets = MidiPacketReader::new(&buffer[..length]).ok();
        for packet in packets.clone().into_iter().flatten() {
            if !self.ports.iter().any(|port| port.cable == packet.cable_number()) {
                return Err(MidiError::InvalidPort);
            }
        }
        Ok(ReceivedMidiPackets {
            packets,
            ports: &self.ports,
        })
    }
}

/// One received USB-MIDI event packet and its logical input port.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ReceivedMidiPacket {
    port: MidiInputPort,
    packet: MidiPacket,
}

impl ReceivedMidiPacket {
    /// Logical input port selected by the packet's virtual cable number.
    pub const fn port(&self) -> MidiInputPort {
        self.port
    }

    /// USB-MIDI event packet.
    pub const fn packet(&self) -> &MidiPacket {
        &self.packet
    }

    /// MIDI bytes carried by the packet.
    pub fn data(&self) -> &[u8] {
        self.packet.event()
    }
}

/// Iterator over the logical MIDI packets in one received USB transfer.
pub struct ReceivedMidiPackets<'a> {
    packets: Option<MidiPacketReader<'a>>,
    ports: &'a [MidiInputPort],
}

impl Iterator for ReceivedMidiPackets<'_> {
    type Item = ReceivedMidiPacket;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let packet = self.packets.as_mut()?.next()?;
            // Reserved CINs carry no MIDI bytes.
            if packet.event_len() == 0 {
                continue;
            }
            let port = self
                .ports
                .iter()
                .find(|port| port.cable == packet.cable_number())
                .copied()?;
            return Some(ReceivedMidiPacket { port, packet });
        }
    }
}

fn cable_mask<D>(ports: &[MidiPort<D>]) -> u16 {
    ports.iter().fold(0, |mask, port| mask | 1 << port.cable)
}

fn cable_ports<D>(
    interface_number: u8,
    endpoint: &MidiEndpointDescriptor,
    device_address: u8,
) -> Result<Vec<MidiPort<D>, MAX_MIDI_JACKS>, MidiError> {
    let mut ports = Vec::new();
    for (cable, &jack_id) in cable_jacks(endpoint).enumerate() {
        ports
            .push(MidiPort {
                device_address,
                interface_number,
                endpoint_address: endpoint.address,
                cable: cable as u8,
                jack_id,
                direction: PhantomData,
            })
            .map_err(|_| MidiDescriptorError::Capacity)?;
    }
    Ok(ports)
}

fn cable_jacks(endpoint: &MidiEndpointDescriptor) -> impl Iterator<Item = &u8> {
    static IMPLICIT_CABLE: [u8; 1] = [0];
    if endpoint.jack_ids.is_empty() {
        IMPLICIT_CABLE.iter()
    } else {
        endpoint.jack_ids.iter()
    }
}

/// Most jack or block IDs one endpoint refers to: 16 cables (MIDI 1.0) or 16 blocks (MIDI 2.0 §5.3.2).
const MAX_ASSOCIATED_IDS: usize = 16;
const _: () = core::assert!(MAX_MIDI_JACKS <= MAX_ASSOCIATED_IDS);

/// Which version of USB MIDI a streaming function speaks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MidiVersion {
    /// USB-MIDI 1.0 event packets, on alternate setting 0.
    Midi1,
    /// MIDI 2.0 Universal MIDI Packets, on an alternate setting of 1 or higher.
    Midi2,
}

/// A MIDI data endpoint and the jack or Group Terminal Block IDs it carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MidiEndpoint {
    /// USB endpoint address, including its direction bit.
    pub address: u8,
    /// Maximum USB packet size.
    pub max_packet_size: u16,
    /// Transfer type. USB MIDI 2.0 §5.3 allows bulk or interrupt.
    pub ep_type: embassy_usb_driver::EndpointType,
    /// Raw `bInterval`. Zero for bulk (§5.3.1); the polling interval for interrupt (Appendix B.8).
    pub interval: u8,
    associated_ids: [u8; MAX_ASSOCIATED_IDS],
    associated_count: u8,
}

impl MidiEndpoint {
    /// Whether this endpoint transfers data from the device to the host.
    pub const fn is_in(&self) -> bool {
        self.address & 0x80 != 0
    }

    /// Jack IDs (MIDI 1.0, position = cable) or Group Terminal Block IDs (MIDI 2.0) of this endpoint.
    pub fn associated_ids(&self) -> &[u8] {
        &self.associated_ids[..self.associated_count as usize]
    }

    fn from_descriptor(endpoint: &MidiEndpointDescriptor, version: MidiVersion) -> Result<Self, MidiError> {
        let ids: &[u8] = match version {
            MidiVersion::Midi1 => &endpoint.jack_ids,
            MidiVersion::Midi2 => &endpoint.group_terminal_block_ids,
        };
        let mut array = [0; MAX_ASSOCIATED_IDS];
        array
            .get_mut(..ids.len())
            .ok_or(MidiDescriptorError::Capacity)?
            .copy_from_slice(ids);
        Ok(Self {
            address: endpoint.address,
            max_packet_size: endpoint.max_packet_size,
            ep_type: endpoint.ep_type,
            interval: endpoint.interval,
            associated_ids: array,
            associated_count: ids.len() as u8,
        })
    }

    fn to_descriptor(&self, version: MidiVersion) -> MidiEndpointDescriptor {
        let mut descriptor = MidiEndpointDescriptor {
            address: self.address,
            max_packet_size: self.max_packet_size,
            ep_type: self.ep_type,
            interval: self.interval,
            jack_ids: Vec::new(),
            group_terminal_block_ids: Vec::new(),
        };
        // The capacity is MAX_GTBS or MAX_MIDI_JACKS, both at least `MAX_ASSOCIATED_IDS`.
        let _ = match version {
            MidiVersion::Midi1 => descriptor.jack_ids.extend_from_slice(self.associated_ids()),
            MidiVersion::Midi2 => descriptor
                .group_terminal_block_ids
                .extend_from_slice(self.associated_ids()),
        };
        descriptor
    }
}

/// The USB MIDI function of a device, as plain `Copy` data: version, alternate setting and endpoints.
///
/// Use [`discover`](Self::discover), [`activate`](Self::activate), then [`MidiHost::from_info`] or
/// [`Midi2Host::new`]. If `activate` fails with [`MidiError::is_midi2_failure`], the device is back
/// on alternate setting 0: open [`midi1_from_config`](Self::midi1_from_config) as MIDI 1.0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MidiInfo {
    /// Version of the function.
    pub version: MidiVersion,
    interface_number: u8,
    alternate_setting: u8,
    input_endpoint: Option<MidiEndpoint>,
    output_endpoint: Option<MidiEndpoint>,
}

impl MidiInfo {
    /// Discover the function to use, preferring MIDI 2.0; parses descriptors only.
    pub fn discover(config_desc: &[u8], enum_info: &EnumerationInfo) -> Result<Self, MidiError> {
        Self::midi2_from_config(config_desc, enum_info).or_else(|_| Self::midi1_from_config(config_desc, enum_info))
    }

    /// Discover the MIDI 1.0 function: one interface on alternate setting 0, one bulk endpoint per direction.
    pub fn midi1_from_config(config_desc: &[u8], enum_info: &EnumerationInfo) -> Result<Self, MidiError> {
        let interfaces = parse_midi_interfaces_for_device(&enum_info.device_desc, config_desc)?;
        let (interface, input, output) = select_streaming_interface(&interfaces)?;
        Ok(Self {
            version: MidiVersion::Midi1,
            interface_number: interface.interface_number,
            alternate_setting: interface.alternate_setting,
            input_endpoint: input
                .map(|e| MidiEndpoint::from_descriptor(e, MidiVersion::Midi1))
                .transpose()?,
            output_endpoint: output
                .map(|e| MidiEndpoint::from_descriptor(e, MidiVersion::Midi1))
                .transpose()?,
        })
    }

    /// Discover the first MIDI 2.0 function: an alternate setting reporting `bcdMSC >= 0x0200`,
    /// at most one endpoint per direction.
    ///
    /// Endpoints may be bulk or interrupt (USB MIDI 2.0 §3.3.2, §5.3). Appendix B.8 is an
    /// interrupt IN endpoint beside a bulk OUT endpoint.
    pub fn midi2_from_config(config_desc: &[u8], enum_info: &EnumerationInfo) -> Result<Self, MidiError> {
        use embassy_usb_driver::EndpointType;

        let interfaces = parse_midi_interfaces_for_device(&enum_info.device_desc, config_desc)?;
        let interface = interfaces
            .iter()
            .find(|iface| iface.is_midi2())
            .ok_or(MidiError::NoInterface)?;

        let mut input_endpoint = None;
        let mut output_endpoint = None;
        for endpoint in &interface.endpoints {
            // §4: one or more MIDI endpoints, each bulk or interrupt (§5.3.1).
            if !matches!(endpoint.ep_type, EndpointType::Bulk | EndpointType::Interrupt) {
                return Err(MidiError::UnsupportedTopology);
            }
            let slot = if endpoint.is_in() {
                &mut input_endpoint
            } else {
                &mut output_endpoint
            };
            if slot
                .replace(MidiEndpoint::from_descriptor(endpoint, MidiVersion::Midi2)?)
                .is_some()
            {
                return Err(MidiError::UnsupportedTopology);
            }
        }
        if input_endpoint.is_none() && output_endpoint.is_none() {
            return Err(MidiError::NoInterface);
        }

        Ok(Self {
            version: MidiVersion::Midi2,
            interface_number: interface.interface_number,
            alternate_setting: interface.alternate_setting,
            input_endpoint,
            output_endpoint,
        })
    }

    /// Fetch this MIDI 2.0 function's Group Terminal Blocks and the groups each endpoint carries.
    ///
    /// `buffer` needs `GR_TRM_BLOCK_HEADER_LEN + MAX_GTBS * GR_TRM_BLOCK_LEN` bytes to always fit.
    /// Fails with [`MidiError::UnsupportedTopology`] for a MIDI 1.0 function.
    pub async fn fetch_group_terminal_blocks<P>(
        &self,
        control_pipe: &mut P,
        buffer: &mut [u8],
    ) -> Result<Midi2Topology, MidiError>
    where
        P: UsbPipe<pipe::Control, pipe::InOut>,
    {
        if self.version != MidiVersion::Midi2 {
            return Err(MidiError::UnsupportedTopology);
        }
        let blocks = crate::class::midi::host::fetch_group_terminal_blocks::<_, MAX_GTBS>(
            control_pipe,
            self.interface_number,
            self.alternate_setting,
            buffer,
        )
        .await?;
        let groups = |endpoint: &Option<MidiEndpoint>| match endpoint {
            Some(endpoint) => endpoint_groups(&blocks, endpoint.associated_ids(), !endpoint.is_in()),
            None => Ok(0),
        };
        Ok(Midi2Topology {
            input_groups: groups(&self.input_endpoint)?,
            output_groups: groups(&self.output_endpoint)?,
            blocks,
        })
    }

    /// Select a MIDI 2.0 function's alternate setting and read its topology, before opening pipes.
    ///
    /// On failure the device is put back on alternate setting 0. A MIDI 1.0 function needs nothing:
    /// this sends no request and returns an empty topology.
    pub async fn activate<P>(&self, control_pipe: &mut P) -> Result<Midi2Topology, MidiError>
    where
        P: UsbPipe<pipe::Control, pipe::InOut>,
    {
        use crate::host::control::SetupPacket;

        if self.version == MidiVersion::Midi1 {
            return Ok(Midi2Topology::default());
        }
        let set_interface = SetupPacket::set_interface(self.interface_number as u16, self.alternate_setting as u16);
        control_pipe.control_out(&set_interface.to_bytes(), &[]).await?;
        let mut buffer = [0u8; GR_TRM_BLOCK_HEADER_LEN as usize + MAX_GTBS * GR_TRM_BLOCK_LEN as usize];
        match self.fetch_group_terminal_blocks(control_pipe, &mut buffer).await {
            Ok(topology) => Ok(topology),
            Err(error) => {
                // Best effort: the error being reported is the one that matters.
                let _ = self.deactivate(control_pipe).await;
                Err(error)
            }
        }
    }

    /// Put a MIDI 2.0 function back on alternate setting 0 before reopening it as MIDI 1.0; no request for MIDI 1.0.
    pub async fn deactivate<P>(&self, control_pipe: &mut P) -> Result<(), MidiError>
    where
        P: UsbPipe<pipe::Control, pipe::InOut>,
    {
        if self.version == MidiVersion::Midi2 {
            let set_interface = crate::host::control::SetupPacket::set_interface(self.interface_number as u16, 0);
            control_pipe.control_out(&set_interface.to_bytes(), &[]).await?;
        }
        Ok(())
    }

    /// Standard interface number of this function.
    pub fn interface_number(&self) -> u8 {
        self.interface_number
    }

    /// Alternate setting that carries this function's protocol.
    pub fn alternate_setting(&self) -> u8 {
        self.alternate_setting
    }

    /// Device-to-host (IN) bulk endpoint, if the function has one.
    pub fn input_endpoint(&self) -> Option<&MidiEndpoint> {
        self.input_endpoint.as_ref()
    }

    /// Host-to-device (OUT) bulk endpoint, if the function has one.
    pub fn output_endpoint(&self) -> Option<&MidiEndpoint> {
        self.output_endpoint.as_ref()
    }
}

/// Group Terminal Blocks of a MIDI 2.0 function and the groups each endpoint carries; empty for MIDI 1.0.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Midi2Topology {
    blocks: Vec<GroupTerminalBlock, MAX_GTBS>,
    input_groups: u16,
    output_groups: u16,
}

impl Midi2Topology {
    /// The Group Terminal Blocks, in descriptor order.
    pub fn group_terminal_blocks(&self) -> &[GroupTerminalBlock] {
        &self.blocks
    }

    /// Bit mask of the groups carried device to host, bit `n` for group `n`.
    pub const fn input_group_mask(&self) -> u16 {
        self.input_groups
    }

    /// Bit mask of the groups carried host to device, bit `n` for group `n`.
    pub const fn output_group_mask(&self) -> u16 {
        self.output_groups
    }

    /// Group numbers carried on the device-to-host (IN) endpoint.
    pub fn input_groups(&self) -> impl Iterator<Item = u8> {
        groups(self.input_groups)
    }

    /// Group numbers carried on the host-to-device (OUT) endpoint.
    pub fn output_groups(&self) -> impl Iterator<Item = u8> {
        groups(self.output_groups)
    }
}

fn groups(mask: u16) -> impl Iterator<Item = u8> {
    (0..16).filter(move |group| mask & (1 << group) != 0)
}

/// Size of a UMP word in bytes.
const UMP_WORD_BYTES: usize = 4;

/// Device-to-host half of a [`Midi2Host`].
pub struct Midi2Receiver<'d, A: UsbHostAllocator<'d>> {
    pipe: raw::UmpInputPipe<'d, A>,
    groups: u16,
    /// Start of a packet that a transfer ended in the middle of.
    carry: [u8; MAX_UMP_BYTES - 4],
    carried: usize,
}

impl<'d, A: UsbHostAllocator<'d>> Midi2Receiver<'d, A> {
    /// Wrap a UMP input pipe; `groups` is the mask of groups the function sends to the host.
    pub fn new(pipe: raw::UmpInputPipe<'d, A>, groups: u16) -> Self {
        Self {
            pipe,
            groups,
            carry: [0; MAX_UMP_BYTES - 4],
            carried: 0,
        }
    }

    /// Bit mask of the groups the function sends to the host, bit `n` for group `n`.
    pub const fn groups(&self) -> u16 {
        self.groups
    }

    /// Receive one transfer of raw UMP bytes, preceded by any partial packet [`read_packets`](Self::read_packets) holds.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, MidiError> {
        if buf.len() < self.carried + UMP_WORD_BYTES {
            return Err(UmpError::BufferTooSmall.into());
        }
        buf[..self.carried].copy_from_slice(&self.carry[..self.carried]);
        let len = self.pipe.read(&mut buf[self.carried..]).await?;
        let total = self.carried + len;
        self.carried = 0;
        Ok(total)
    }

    /// Receive into `buf` and return the length of the complete UMPs at its start; a packet cut by
    /// the end of a transfer is held for the next call. `buf` must be a multiple of 4 and at least 32 bytes.
    pub async fn read_packets(&mut self, buf: &mut [u8]) -> Result<usize, MidiError> {
        if buf.len() < 2 * MAX_UMP_BYTES {
            return Err(UmpError::BufferTooSmall.into());
        }
        loop {
            buf[..self.carried].copy_from_slice(&self.carry[..self.carried]);
            let len = self.pipe.read(&mut buf[self.carried..]).await?;
            let total = self.carried + len;
            let complete = complete_len(&buf[..total]);
            self.carried = total - complete;
            self.carry[..self.carried].copy_from_slice(&buf[complete..total]);
            // A zero-length transfer completes nothing, so wait for the next one.
            if complete != 0 {
                return Ok(complete);
            }
        }
    }

    /// Like [`read_packets`](Self::read_packets), returning the packets as a reader over `buf`.
    pub async fn read_ump<'a>(&mut self, buf: &'a mut [u8]) -> Result<UmpReader<'a>, MidiError> {
        let len = self.read_packets(buf).await?;
        Ok(UmpReader::new(&buf[..len]))
    }
}

/// Host-to-device half of a [`Midi2Host`].
pub struct Midi2Sender<'d, A: UsbHostAllocator<'d>> {
    pipe: raw::UmpOutputPipe<'d, A>,
    groups: u16,
    max_packet_size: usize,
}

impl<'d, A: UsbHostAllocator<'d>> Midi2Sender<'d, A> {
    /// Wrap a UMP output pipe; `groups` is the mask of groups the host may send, where 0 skips the check.
    pub fn new(pipe: raw::UmpOutputPipe<'d, A>, groups: u16, max_packet_size: u16) -> Self {
        Self {
            pipe,
            groups,
            max_packet_size: max_packet_size as usize,
        }
    }

    /// Bit mask of the groups the host may send, bit `n` for group `n`.
    pub const fn groups(&self) -> u16 {
        self.groups
    }

    /// Send complete UMPs, each for a group the function accepts; nothing is sent if one is rejected.
    /// Transfers are cut between packets at the max packet size (§3.2.3).
    pub async fn write(&mut self, data: &[u8]) -> Result<(), MidiError> {
        for packet in UmpReader::new(data) {
            let packet = packet?;
            if self.groups != 0 && !packet.in_groups(self.groups) {
                return Err(MidiError::InvalidGroup);
            }
            if packet.byte_count() > self.max_packet_size {
                return Err(UmpError::BufferTooSmall.into());
            }
        }
        let mut start = 0;
        while start < data.len() {
            let len = chunk_len(&data[start..], self.max_packet_size);
            self.pipe.write(&data[start..start + len]).await?;
            start += len;
        }
        Ok(())
    }

    /// Send a single Universal MIDI Packet.
    pub async fn write_packet(&mut self, packet: &UmpPacket) -> Result<(), MidiError> {
        let mut buf = [0u8; MAX_UMP_BYTES];
        let len = packet.to_bytes(&mut buf)?;
        self.write(&buf[..len]).await
    }
}

/// Friendly host driver for one USB MIDI 2.0 streaming function.
pub struct Midi2Host<'d, A: UsbHostAllocator<'d>> {
    sender: Option<Midi2Sender<'d, A>>,
    receiver: Option<Midi2Receiver<'d, A>>,
    info: MidiInfo,
    topology: Midi2Topology,
}

impl<'d, A: UsbHostAllocator<'d>> Midi2Host<'d, A> {
    /// Allocate UMP pipes for a MIDI 2.0 function; call [`MidiInfo::activate`] first.
    pub fn new(alloc: &A, info: MidiInfo, enum_info: &EnumerationInfo) -> Result<Self, MidiError> {
        if info.version != MidiVersion::Midi2 {
            return Err(MidiError::NoInterface);
        }
        let receiver = info
            .input_endpoint
            .as_ref()
            .map(|endpoint| {
                raw::UmpInputPipe::open(alloc, &endpoint.to_descriptor(MidiVersion::Midi2), enum_info)
                    .map(|pipe| Midi2Receiver::new(pipe, 0))
            })
            .transpose()?;
        let sender = info
            .output_endpoint
            .as_ref()
            .map(|endpoint| {
                raw::UmpOutputPipe::open(alloc, &endpoint.to_descriptor(MidiVersion::Midi2), enum_info)
                    .map(|pipe| Midi2Sender::new(pipe, 0, endpoint.max_packet_size))
            })
            .transpose()?;

        Ok(Self {
            sender,
            receiver,
            info,
            topology: Midi2Topology::default(),
        })
    }

    /// Attach the topology from [`MidiInfo::activate`]; its masks become the halves' groups.
    pub fn with_topology(mut self, topology: Midi2Topology) -> Self {
        if let Some(receiver) = &mut self.receiver {
            receiver.groups = topology.input_group_mask();
        }
        if let Some(sender) = &mut self.sender {
            sender.groups = topology.output_group_mask();
        }
        self.topology = topology;
        self
    }

    /// Discover, activate and open the first MIDI 2.0 function, with its topology.
    ///
    /// `control_pipe` stays allocated, so a bidirectional function needs three pipes at once; a constrained
    /// host can call [`MidiInfo::activate`], drop its control pipe, then [`Midi2Host::new`].
    pub async fn connect<P>(
        alloc: &A,
        control_pipe: &mut P,
        config_desc: &[u8],
        enum_info: &EnumerationInfo,
    ) -> Result<Self, MidiError>
    where
        P: UsbPipe<pipe::Control, pipe::InOut>,
    {
        let info = MidiInfo::midi2_from_config(config_desc, enum_info)?;
        let topology = info.activate(control_pipe).await?;
        match Self::new(alloc, info, enum_info) {
            Ok(host) => Ok(host.with_topology(topology)),
            Err(error) => {
                // Best effort; the allocation error is the one reported.
                let _ = info.deactivate(control_pipe).await;
                Err(error)
            }
        }
    }

    /// Activate on a control pipe of its own, free it, then allocate the data pipes, so two free pipes
    /// suffice. If that fails, a fresh control pipe puts the device back on alternate setting 0.
    async fn connect_with_own_control_pipe(
        alloc: &A,
        info: MidiInfo,
        enum_info: &EnumerationInfo,
    ) -> Result<Self, MidiError> {
        let alloc_control = || {
            alloc
                .alloc_pipe::<pipe::Control, pipe::InOut>(
                    enum_info.device_address,
                    &embassy_usb_driver::EndpointInfo {
                        addr: 0.into(),
                        ep_type: embassy_usb_driver::EndpointType::Control,
                        max_packet_size: (enum_info.device_desc.max_packet_size0 as u16)
                            .min(enum_info.speed().max_packet_size()),
                        interval_ms: 0,
                        max_burst: 0,
                    },
                    enum_info.split(),
                )
                .map_err(|_| MidiError::NoPipe)
        };
        let mut control = alloc_control()?;
        let topology = info.activate(&mut control).await?;
        drop(control);
        match Self::new(alloc, info, enum_info) {
            Ok(host) => Ok(host.with_topology(topology)),
            Err(error) => {
                // Best effort, on a fresh control pipe.
                if let Ok(mut control) = alloc_control() {
                    let _ = info.deactivate(&mut control).await;
                }
                Err(error)
            }
        }
    }

    /// The function this host was opened from.
    pub fn info(&self) -> &MidiInfo {
        &self.info
    }

    /// The function's topology; empty unless attached with [`with_topology`](Self::with_topology).
    pub fn topology(&self) -> &Midi2Topology {
        &self.topology
    }

    /// Returns the interface number of this MIDI 2.0 streaming interface.
    pub fn interface_number(&self) -> u8 {
        self.info.interface_number
    }

    /// Receive one transfer of raw UMP bytes into `buf`.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, MidiError> {
        self.receiver.as_mut().ok_or(MidiError::NoEndpoint)?.read(buf).await
    }

    /// See [`Midi2Receiver::read_ump`].
    pub async fn read_ump<'a>(&mut self, buf: &'a mut [u8]) -> Result<UmpReader<'a>, MidiError> {
        self.receiver.as_mut().ok_or(MidiError::NoEndpoint)?.read_ump(buf).await
    }

    /// See [`Midi2Sender::write`].
    pub async fn write(&mut self, data: &[u8]) -> Result<(), MidiError> {
        self.sender.as_mut().ok_or(MidiError::NoEndpoint)?.write(data).await
    }

    /// Send a single Universal MIDI Packet.
    pub async fn write_packet(&mut self, packet: &UmpPacket) -> Result<(), MidiError> {
        self.sender
            .as_mut()
            .ok_or(MidiError::NoEndpoint)?
            .write_packet(packet)
            .await
    }

    /// Split into independently owned sender and receiver halves, plus the discovery info.
    pub fn split(self) -> (Option<Midi2Sender<'d, A>>, Option<Midi2Receiver<'d, A>>, MidiInfo) {
        (self.sender, self.receiver, self.info)
    }
}

/// Length in bytes of the whole UMPs at the start of `data`, ignoring a trailing partial one.
fn complete_len(data: &[u8]) -> usize {
    let mut len = 0;
    while let Some(&top) = data.get(len + 3) {
        let packet = MessageType::from_nibble(top >> 4).byte_count();
        if len + packet > data.len() {
            break;
        }
        len += packet;
    }
    len
}

/// Length of the leading whole UMPs of `data` that fit in `max` bytes, at least one packet.
fn chunk_len(data: &[u8], max: usize) -> usize {
    let mut len = 0;
    while len < data.len() {
        let packet = MessageType::from_nibble(data[len + 3] >> 4).byte_count();
        if len != 0 && len + packet > max {
            break;
        }
        len += packet;
    }
    len
}

impl<'d, A: UsbHostAllocator<'d>> HostClass<'d, A> for Midi2Host<'d, A> {
    const NAME: &'static str = "MIDI 2.0";
    type Info = MidiInfo;

    fn probe(enum_info: &EnumerationInfo, cfg: &[u8]) -> Option<MidiInfo> {
        MidiInfo::midi2_from_config(cfg, enum_info).ok()
    }

    async fn open(alloc: &A, _cfg: &[u8], info: MidiInfo, enum_info: &EnumerationInfo) -> Result<Self, ClassError> {
        Ok(Self::connect_with_own_control_pipe(alloc, info, enum_info).await?)
    }
}

impl<'d, A: UsbHostAllocator<'d>> HostClass<'d, A> for MidiHost<'d, A> {
    const NAME: &'static str = "MIDI 1.0";
    type Info = MidiInfo;

    fn probe(enum_info: &EnumerationInfo, cfg: &[u8]) -> Option<MidiInfo> {
        MidiInfo::midi1_from_config(cfg, enum_info).ok()
    }

    async fn open(alloc: &A, _cfg: &[u8], info: MidiInfo, enum_info: &EnumerationInfo) -> Result<Self, ClassError> {
        Ok(Self::from_info(alloc, info, enum_info)?)
    }
}

/// Read a string descriptor (such as a block name) as UTF-8 truncated to `N` bytes, in the device's
/// first language. Index 0 means no string and gives an empty one.
pub async fn fetch_string<P, const N: usize>(control_pipe: &mut P, index: u8) -> Result<heapless::String<N>, MidiError>
where
    P: UsbPipe<pipe::Control, pipe::InOut>,
{
    use crate::host::control::SetupPacket;

    let mut name = heapless::String::new();
    if index == 0 {
        return Ok(name);
    }
    let mut raw = [0u8; 255];
    let get = |index: u8, lang_id: u16| SetupPacket::get_string_descriptor(index, lang_id, 255).to_bytes();

    // String descriptor 0 lists the supported language IDs.
    let len = control_pipe.control_in(&get(0, 0), &mut raw).await?;
    if len < 4 || raw[1] != 0x03 {
        return Err(MidiDescriptorError::InvalidDescriptor.into());
    }
    let lang_id = u16::from_le_bytes([raw[2], raw[3]]);

    let len = control_pipe.control_in(&get(index, lang_id), &mut raw).await?;
    if len < 2 || raw[1] != 0x03 || raw[0] as usize > len {
        return Err(MidiDescriptorError::InvalidDescriptor.into());
    }
    let units = raw[2..raw[0] as usize]
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]));
    for c in char::decode_utf16(units) {
        if name.push(c.unwrap_or(char::REPLACEMENT_CHARACTER)).is_err() {
            break;
        }
    }
    Ok(name)
}

/// Query the Group Terminal Blocks of an alternate setting: standard `GET_DESCRIPTOR`, with
/// `CS_GR_TRM_BLOCK` and the alternate setting in `wValue`, and the interface in `wIndex` (§6).
pub async fn fetch_group_terminal_blocks<P, const N: usize>(
    control_pipe: &mut P,
    interface_number: u8,
    alternate_setting: u8,
    buffer: &mut [u8],
) -> Result<Vec<GroupTerminalBlock, N>, MidiError>
where
    P: UsbPipe<pipe::Control, pipe::InOut>,
{
    use crate::host::control::SetupPacket;

    let request = |length: usize| {
        SetupPacket::standard_interface_in(
            crate::control::Request::GET_DESCRIPTOR,
            ((CS_GR_TRM_BLOCK as u16) << 8) | alternate_setting as u16,
            interface_number as u16,
            length as u16,
        )
        .to_bytes()
    };

    // The reply length is only known from the header, so read that first (§6).
    let header_len = GR_TRM_BLOCK_HEADER_LEN as usize;
    let header = buffer.get_mut(..header_len).ok_or(GtbError::BufferTooSmall)?;
    if control_pipe.control_in(&request(header_len), header).await? != header_len {
        return Err(GtbError::InvalidDescriptor.into());
    }
    let total = u16::from_le_bytes([header[3], header[4]]) as usize;
    let reply = buffer.get_mut(..total).ok_or(GtbError::BufferTooSmall)?;
    let len = control_pipe.control_in(&request(total), reply).await?;
    Ok(parse_gtb_descriptors(&reply[..len])?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_event_packet_fields_and_lengths() {
        let note_on = MidiPacket::new([0x39, 0x90, 60, 100]);
        assert_eq!(note_on.cable_number(), 3);
        assert_eq!(note_on.cin(), 9);
        assert_eq!(note_on.event(), &[0x90, 60, 100]);

        let program_change = MidiPacket::new([0x0c, 0xc0, 12, 0]);
        assert_eq!(program_change.event(), &[0xc0, 12]);

        let clock = MidiPacket::new([0x0f, 0xf8, 0, 0]);
        assert_eq!(clock.event(), &[0xf8]);

        let reserved = MidiPacket::new([0x00, 0, 0, 0]);
        assert_eq!(reserved.event_len(), 0);
    }

    #[test]
    fn iterates_complete_event_packets() {
        let data = [0x09, 0x90, 60, 100, 0x08, 0x80, 60, 0];
        let packets: heapless::Vec<_, 2> = MidiPacketReader::new(&data).unwrap().collect();
        assert_eq!(packets.len(), 2);
        assert_eq!(packets[0].event(), &[0x90, 60, 100]);
        assert_eq!(packets[1].event(), &[0x80, 60, 0]);
        assert!(MidiPacketReader::new(&data[..7]).is_err());
        assert!(MidiPacketReader::new(&[]).is_err());
    }

    #[test]
    fn encodes_channel_voice_event_packets() {
        let messages: &[(&[u8], u8)] = &[
            (&[0x80, 60, 0], 0x08),
            (&[0x90, 60, 100], 0x09),
            (&[0xa0, 60, 50], 0x0a),
            (&[0xb0, 74, 90], 0x0b),
            (&[0xc0, 12], 0x0c),
            (&[0xd0, 77], 0x0d),
            (&[0xe0, 0, 64], 0x0e),
        ];
        for &(message, cin) in messages {
            let packet = MidiPacket::try_encode_exact(3, message).unwrap();
            assert_eq!(packet.cable_number(), 3);
            assert_eq!(packet.cin(), cin);
            assert_eq!(packet.event(), message);
        }
    }

    #[test]
    fn encodes_system_common_and_realtime_event_packets() {
        let messages: &[(&[u8], u8)] = &[
            (&[0xf1, 1], 0x02),
            (&[0xf2, 1, 2], 0x03),
            (&[0xf3, 3], 0x02),
            (&[0xf6], 0x05),
            (&[0xf7], 0x05),
            (&[0xf8], 0x0f),
            (&[0xfa], 0x0f),
            (&[0xfb], 0x0f),
            (&[0xfc], 0x0f),
            (&[0xfe], 0x0f),
            (&[0xff], 0x0f),
        ];
        for &(message, cin) in messages {
            let packet = MidiPacket::try_encode_exact(15, message).unwrap();
            assert_eq!(packet.cin(), cin);
            assert_eq!(packet.event(), message);
            for &padding in &packet.as_bytes()[1 + message.len()..] {
                assert_eq!(padding, 0);
            }
        }
    }

    #[test]
    fn rejects_invalid_midi_event_packet_inputs() {
        assert_eq!(
            MidiPacket::try_encode_exact(16, &[0x90, 60, 100]),
            Err(MidiPacketError::InvalidCableNumber)
        );
        for status in [0xf4, 0xf5, 0xfd] {
            assert_eq!(
                MidiPacket::try_encode_exact(0, &[status]),
                Err(MidiPacketError::InvalidEventStatus)
            );
        }
        assert_eq!(MidiPacket::try_encode_exact(0, &[]), Err(MidiPacketError::EmptyEvent));
        for message in [&[0xf0][..], &[0x90, 60][..], &[0xc0, 12, 0][..], &[0xf8, 0][..]] {
            assert_eq!(
                MidiPacket::try_encode_exact(0, message),
                Err(MidiPacketError::InvalidEventLength)
            );
        }
        assert_eq!(
            MidiPacket::try_encode_exact(0, &[0x90, 0x80, 0]),
            Err(MidiPacketError::InvalidEventData)
        );
    }

    #[test]
    fn encodes_sysex_and_single_byte_event_packets() {
        let messages: &[(&[u8], u8)] = &[
            (&[0xf0, 0x7e, 0x7f], 0x04),
            (&[0x01, 0x02, 0x03], 0x04),
            (&[0x01, 0xf7], 0x06),
            (&[0x01, 0x02, 0xf7], 0x07),
            (&[0x00], 0x0f),
            (&[0x7f], 0x0f),
            (&[0xf9], 0x0f),
        ];
        for &(message, cin) in messages {
            let packet = MidiPacket::try_encode_exact(0, message).unwrap();
            assert_eq!(packet.cin(), cin);
            assert_eq!(packet.event(), message);
        }
    }

    #[test]
    fn maps_endpoint_cables_to_typed_logical_ports() {
        let endpoint = MidiEndpointDescriptor {
            address: 0x81,
            max_packet_size: 64,
            ep_type: embassy_usb_driver::EndpointType::Bulk,
            interval: 0,
            jack_ids: Vec::from_slice(&[3, 7]).unwrap(),
            group_terminal_block_ids: Vec::new(),
        };
        let ports = cable_ports::<In>(2, &endpoint, 5).unwrap();
        assert_eq!(cable_mask(&ports), 0b11);

        assert_eq!(ports.len(), 2);
        assert_eq!(ports[0].number(), 1);
        assert_eq!(ports[0].cable(), 0);
        assert_eq!(ports[0].jack_id(), 3);
        assert_eq!(ports[1].number(), 2);
        assert_eq!(ports[1].cable(), 1);
        assert_eq!(ports[1].jack_id(), 7);
        assert_eq!(ports[1].interface_number(), 2);
    }

    #[test]
    fn maps_received_cables_to_input_ports() {
        let endpoint = MidiEndpointDescriptor {
            address: 0x81,
            max_packet_size: 64,
            ep_type: embassy_usb_driver::EndpointType::Bulk,
            interval: 0,
            jack_ids: Vec::from_slice(&[3, 7]).unwrap(),
            group_terminal_block_ids: Vec::new(),
        };
        let ports = cable_ports::<In>(2, &endpoint, 5).unwrap();
        let transfer = [0, 0, 0, 0, 0x19, 0x90, 60, 100, 0, 0, 0, 0];
        let mut packets = ReceivedMidiPackets {
            packets: MidiPacketReader::new(&transfer).ok(),
            ports: &ports,
        };

        let received = packets.next().unwrap();
        assert_eq!(received.port(), ports[1]);
        assert_eq!(received.data(), &[0x90, 60, 100]);
        assert!(packets.next().is_none());
    }

    #[test]
    fn supplies_one_implicit_port_without_jack_associations() {
        let endpoint = MidiEndpointDescriptor {
            address: 0x02,
            max_packet_size: 64,
            ep_type: embassy_usb_driver::EndpointType::Bulk,
            interval: 0,
            jack_ids: Vec::new(),
            group_terminal_block_ids: Vec::new(),
        };
        let ports = cable_ports::<Out>(1, &endpoint, 4).unwrap();
        assert_eq!(cable_mask(&ports), 0b1);

        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0].cable(), 0);
        assert_eq!(ports[0].jack_id(), 0);
    }

    fn interface(alternate_setting: u8, endpoints: &[MidiEndpointDescriptor]) -> MidiStreamingInterface {
        MidiStreamingInterface {
            interface_number: 1,
            alternate_setting,
            header: None,
            in_jacks: Vec::new(),
            out_jacks: Vec::new(),
            endpoints: Vec::from_slice(endpoints).unwrap(),
        }
    }

    fn endpoint(address: u8) -> MidiEndpointDescriptor {
        MidiEndpointDescriptor {
            address,
            max_packet_size: 64,
            ep_type: embassy_usb_driver::EndpointType::Bulk,
            interval: 0,
            jack_ids: Vec::new(),
            group_terminal_block_ids: Vec::new(),
        }
    }

    #[test]
    fn selects_input_output_and_duplex_topologies() {
        let input = endpoint(0x81);
        let output = endpoint(0x02);

        let interfaces = [interface(0, core::slice::from_ref(&input))];
        let (_, selected_input, selected_output) = select_streaming_interface(&interfaces).unwrap();
        assert_eq!(selected_input.unwrap().address, input.address);
        assert!(selected_output.is_none());

        let interfaces = [interface(0, core::slice::from_ref(&output))];
        let (_, selected_input, selected_output) = select_streaming_interface(&interfaces).unwrap();
        assert!(selected_input.is_none());
        assert_eq!(selected_output.unwrap().address, output.address);

        let interfaces = [interface(0, &[input.clone(), output.clone()])];
        let (_, selected_input, selected_output) = select_streaming_interface(&interfaces).unwrap();
        assert_eq!(selected_input.unwrap().address, input.address);
        assert_eq!(selected_output.unwrap().address, output.address);
    }

    #[test]
    fn rejects_missing_and_ambiguous_topologies() {
        assert!(matches!(select_streaming_interface(&[]), Err(MidiError::NoInterface)));

        let input = endpoint(0x81);
        let output = endpoint(0x02);
        let interfaces = [interface(1, core::slice::from_ref(&input))];
        assert!(matches!(
            select_streaming_interface(&interfaces),
            Err(MidiError::NoInterface)
        ));

        // Alternate setting 0 declaring bcdMSC 0x0200 is a MIDI 2.0 function, not MIDI 1.0.
        let mut midi2 = interface(0, core::slice::from_ref(&input));
        midi2.header = Some(descriptors::MidiStreamingHeader {
            midi_version: crate::class::midi::MIDI2_VERSION,
            total_length: 7,
        });
        assert!(matches!(
            select_streaming_interface(&[midi2]),
            Err(MidiError::NoInterface)
        ));

        let interfaces = [
            interface(0, core::slice::from_ref(&input)),
            interface(0, core::slice::from_ref(&output)),
        ];
        assert!(matches!(
            select_streaming_interface(&interfaces),
            Err(MidiError::UnsupportedTopology)
        ));

        let interfaces = [interface(0, &[input.clone(), input])];
        assert!(matches!(
            select_streaming_interface(&interfaces),
            Err(MidiError::UnsupportedTopology)
        ));
    }

    // Minimal dual-protocol MIDIStreaming interface: interface 1, alternate settings 0
    // (MIDI 1.0) and 1 (MIDI 2.0), both referencing bulk endpoints 0x01 and 0x81.
    #[rustfmt::skip]
    const MIDI2_CONFIG: &[u8] = &[
        9, 2, 83, 0, 1, 1, 0, 0x80, 50,
        // Interface 1, Alternate Setting 0 (MIDI 1.0).
        9, 4, 1, 0, 2, 1, 3, 0, 0,
        7, 5, 0x01, 2, 64, 0, 0,
        5, 0x25, 0x01, 0x01, 0x01,
        7, 5, 0x81, 2, 64, 0, 0,
        5, 0x25, 0x01, 0x01, 0x02,
        // Interface 1, Alternate Setting 1 (MIDI 2.0).
        9, 4, 1, 1, 2, 1, 3, 0, 0,
        7, 0x24, 0x01, 0x00, 0x02, 0x07, 0x00,
        7, 5, 0x01, 2, 64, 0, 0,
        5, 0x25, 0x02, 0x01, 0x01,
        7, 5, 0x81, 2, 64, 0, 0,
        // The IN endpoint carries two Group Terminal Blocks (1 and 2).
        6, 0x25, 0x02, 0x02, 0x01, 0x02,
    ];

    // Group Terminal Blocks matching MIDI2_CONFIG: block 1 is bidirectional and covers
    // groups 0-1; block 2 is output-only (device to host) and covers group 2.
    #[rustfmt::skip]
    const MIDI2_GTB: &[u8] = &[
        5, 0x26, 0x01, 31, 0,
        13, 0x26, 0x02, 1, 0x00, 0, 2, 0, 0x11, 0, 0, 0, 0,
        13, 0x26, 0x02, 2, 0x02, 2, 1, 0, 0x01, 0, 0, 0, 0,
    ];

    #[test]
    fn midi2_host_discovers_topology_and_selects_alt_setting() {
        use embassy_futures::block_on;
        use embassy_usb_driver::{EndpointInfo, EndpointType};

        use crate::class::midi::MidiProtocol;
        use crate::host::test_utils::{MockHost, MockState};

        let enum_info = crate::host::test_utils::enumeration_info();
        let interfaces = parse_midi_interfaces_for_device(&enum_info.device_desc, MIDI2_CONFIG).unwrap();
        assert_eq!(interfaces.len(), 2);

        let midi2 = interfaces.iter().find(|iface| iface.is_midi2()).unwrap();
        assert_eq!(midi2.alternate_setting, 1);
        let midi2_addrs = [midi2.endpoints[0].address, midi2.endpoints[1].address];
        assert_eq!(midi2_addrs, [0x01, 0x81]);

        // The MIDI 1.0 setting references the same endpoints.
        let midi1 = interfaces.iter().find(|iface| iface.alternate_setting == 0).unwrap();
        let midi1_addrs = [midi1.endpoints[0].address, midi1.endpoints[1].address];
        assert_eq!(midi1_addrs, midi2_addrs);

        // Discovery is independent of the device and of pipe allocation.
        let info = MidiInfo::discover(MIDI2_CONFIG, &enum_info).unwrap();
        assert_eq!(info.version, MidiVersion::Midi2);
        assert_eq!(info.interface_number(), 1);
        assert_eq!(info.alternate_setting(), 1);
        assert_eq!(info.input_endpoint().unwrap().address, 0x81);
        assert_eq!(info.output_endpoint().unwrap().address, 0x01);
        assert_eq!(info.input_endpoint().unwrap().associated_ids(), &[1, 2]);

        let state = MockState::new();
        state.set_gtb_descriptors(MIDI2_GTB);
        let host = MockHost::new(&state);
        let control_ep = EndpointInfo {
            addr: 0.into(),
            ep_type: EndpointType::Control,
            max_packet_size: 64,
            interval_ms: 0,
            max_burst: 0,
        };
        let mut control = host
            .alloc_pipe::<pipe::Control, pipe::InOut>(enum_info.device_address, &control_ep, enum_info.split())
            .unwrap();

        // Activation selects the alternate setting, then reads the blocks; wValue carries the setting.
        let topology = block_on(info.activate(&mut control)).unwrap();
        assert_eq!(state.set_interface(), Some((1, 1)));
        assert_eq!(state.gtb_request(), Some((0x2601, 1)));

        let blocks = topology.group_terminal_blocks();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].protocol, MidiProtocol::Midi2_0);
        assert_eq!(blocks[1].protocol, MidiProtocol::Midi1_0_64);
        assert!(topology.input_groups().eq([0, 1, 2]));
        assert!(topology.output_groups().eq([0, 1]));
        assert_eq!(
            (topology.input_group_mask(), topology.output_group_mask()),
            (0b111, 0b11)
        );

        // Pipe allocation is the final, separate step.
        let midi = Midi2Host::new(&host, info, &enum_info).unwrap().with_topology(topology);
        assert_eq!(midi.interface_number(), 1);
        assert!(midi.topology().input_groups().eq([0, 1, 2]));

        let (sender, receiver, info) = midi.split();
        assert_eq!(sender.as_ref().unwrap().groups(), 0b11);
        assert_eq!(receiver.as_ref().unwrap().groups(), 0b111);
        assert_eq!(info.interface_number(), 1);

        // Once the host is gone, `deactivate` returns the device to alternate setting 0.
        drop((sender, receiver));
        block_on(info.deactivate(&mut control)).unwrap();
        assert_eq!(state.set_interface(), Some((1, 0)));
    }

    #[test]
    fn midi2_host_class_opens_with_topology_and_rolls_back_on_failure() {
        use embassy_futures::block_on;

        use crate::host::test_utils::{MockHost, MockState};

        let enum_info = crate::host::test_utils::enumeration_info();
        let info = <Midi2Host<'_, MockHost<'_>> as HostClass<'_, MockHost<'_>>>::probe(&enum_info, MIDI2_CONFIG)
            .expect("MIDI 2.0 function");

        let state = MockState::new();
        state.set_gtb_descriptors(MIDI2_GTB);
        let host = MockHost::new(&state);
        let midi = block_on(Midi2Host::open(&host, MIDI2_CONFIG, info, &enum_info)).unwrap();
        assert_eq!(state.set_interface(), Some((1, 1)));
        assert!(midi.topology().input_groups().eq([0, 1, 2]));

        // Blocks that cannot be read put the device back on MIDI 1.0.
        let state = MockState::new();
        let host = MockHost::new(&state);
        assert!(block_on(Midi2Host::open(&host, MIDI2_CONFIG, info, &enum_info)).is_err());
        assert_eq!(state.set_interface(), Some((1, 0)));
    }

    #[test]
    fn failed_midi2_activation_leaves_the_device_usable_as_midi1() {
        use embassy_futures::block_on;
        use embassy_usb_driver::{EndpointInfo, EndpointType};

        use crate::host::test_utils::{MockHost, MockState};

        let enum_info = crate::host::test_utils::enumeration_info();
        let info = MidiInfo::discover(MIDI2_CONFIG, &enum_info).unwrap();
        assert_eq!(info.version, MidiVersion::Midi2);

        // No blocks to read: activation fails and puts the device back on alternate setting 0.
        let state = MockState::new();
        let host = MockHost::new(&state);
        let control_ep = EndpointInfo {
            addr: 0.into(),
            ep_type: EndpointType::Control,
            max_packet_size: 64,
            interval_ms: 0,
            max_burst: 0,
        };
        let mut control = host
            .alloc_pipe::<pipe::Control, pipe::InOut>(enum_info.device_address, &control_ep, enum_info.split())
            .unwrap();
        let error = block_on(info.activate(&mut control)).unwrap_err();
        assert!(error.is_midi2_failure());
        assert_eq!(state.set_interface(), Some((1, 0)));

        let midi1 = MidiInfo::midi1_from_config(MIDI2_CONFIG, &enum_info).unwrap();
        assert_eq!((midi1.version, midi1.alternate_setting()), (MidiVersion::Midi1, 0));
        let (sender, receiver, info) = MidiHost::from_info(&host, midi1, &enum_info).unwrap().split();
        assert_eq!(sender.unwrap().groups(), 0b1);
        assert_eq!(receiver.unwrap().groups(), 0b1);
        assert_eq!(info.version, MidiVersion::Midi1);
    }

    #[test]
    fn midi2_interrupt_in_endpoint_is_discovered() {
        use embassy_usb_driver::EndpointType;

        use crate::host::test_utils::{MockHost, MockState};

        let enum_info = crate::host::test_utils::enumeration_info();
        let mut config = [0u8; MIDI2_CONFIG.len()];
        config.copy_from_slice(MIDI2_CONFIG);
        // Appendix B.8: the MIDI 2.0 IN endpoint is interrupt, bInterval 1 (full speed, 1 ms).
        let in_endpoint = config.windows(4).rposition(|w| w == [7, 5, 0x81, 2]).unwrap();
        config[in_endpoint + 3] = 3;
        config[in_endpoint + 6] = 1;

        let info = MidiInfo::midi2_from_config(&config, &enum_info).unwrap();
        assert_eq!(info.version, MidiVersion::Midi2);
        let input = info.input_endpoint().unwrap();
        assert_eq!((input.ep_type, input.interval), (EndpointType::Interrupt, 1));
        assert_eq!(info.output_endpoint().unwrap().ep_type, EndpointType::Bulk);
        assert_eq!(
            MidiInfo::discover(&config, &enum_info).unwrap().version,
            MidiVersion::Midi2
        );

        let state = MockState::new();
        let host = MockHost::new(&state);
        assert!(Midi2Host::new(&host, info, &enum_info).is_ok());
    }

    #[test]
    fn ump_framing_finds_whole_packets_and_transfer_chunks() {
        // A 32-bit, a 64-bit and a 128-bit packet.
        let mut data = [0u8; 28];
        data[3] = 0x20;
        data[7] = 0x40;
        data[15] = 0x50;
        assert_eq!(complete_len(&data), 28);
        assert_eq!(complete_len(&data[..27]), 12);
        assert_eq!(complete_len(&data[..3]), 0);
        assert_eq!(chunk_len(&data, 16), 12);
        assert_eq!(chunk_len(&data[12..], 16), 16);
    }

    #[test]
    fn midi2_host_checks_groups_and_reassembles_split_packets() {
        use embassy_futures::block_on;

        use crate::host::test_utils::{MockHost, MockState};

        let enum_info = crate::host::test_utils::enumeration_info();
        let info = MidiInfo::midi2_from_config(MIDI2_CONFIG, &enum_info).unwrap();
        let state = MockState::new();
        state.set_gtb_descriptors(MIDI2_GTB);
        let alloc = MockHost::new(&state);
        let mut host = block_on(Midi2Host::connect_with_own_control_pipe(&alloc, info, &enum_info)).unwrap();

        // The function accepts groups 0 and 1 from the host.
        let note = |group: u32| UmpPacket::from_u32(0x2000_3c7f | group << 24).unwrap();
        assert!(matches!(
            block_on(host.write_packet(&note(5))),
            Err(MidiError::InvalidGroup)
        ));
        let mut buf = [0u8; 32];
        assert_eq!(block_on(host.read(&mut buf)).unwrap(), 0);
        block_on(host.write_packet(&note(1))).unwrap();
        assert_eq!(state.out_ends_transaction(), Some(true));
        assert_eq!(block_on(host.read_ump(&mut buf)).unwrap().next(), Some(Ok(note(1))));

        // A 128-bit packet arrives in two transfers behind a 32-bit one.
        let long = UmpPacket::from_u32_quad(0x5000_0001, 2, 3, 4).unwrap();
        let mut bytes = [0u8; 20];
        note(0).to_bytes(&mut bytes).unwrap();
        long.to_bytes(&mut bytes[4..]).unwrap();
        let out = info.output_endpoint().unwrap().to_descriptor(MidiVersion::Midi2);
        let mut sender = raw::UmpOutputPipe::open(&alloc, &out, &enum_info).unwrap();
        block_on(sender.write(&bytes[..12])).unwrap();
        let mut packets = block_on(host.read_ump(&mut buf)).unwrap();
        assert_eq!(packets.next(), Some(Ok(note(0))));
        assert_eq!(packets.next(), None);
        block_on(sender.write(&bytes[12..])).unwrap();
        assert_eq!(block_on(host.read_ump(&mut buf)).unwrap().next(), Some(Ok(long)));
    }

    #[test]
    fn fetch_string_reads_a_utf16_name() {
        use embassy_futures::block_on;
        use embassy_usb_driver::{EndpointInfo, EndpointType};

        use crate::host::test_utils::{MockHost, MockState};

        let enum_info = crate::host::test_utils::enumeration_info();
        let state = MockState::new();
        state.set_string("Main \u{e9}");
        let host = MockHost::new(&state);
        let control_ep = EndpointInfo {
            addr: 0.into(),
            ep_type: EndpointType::Control,
            max_packet_size: 64,
            interval_ms: 0,
            max_burst: 0,
        };
        let mut control = host
            .alloc_pipe::<pipe::Control, pipe::InOut>(enum_info.device_address, &control_ep, enum_info.split())
            .unwrap();

        assert_eq!(
            block_on(fetch_string::<_, 16>(&mut control, 4)).unwrap().as_str(),
            "Main \u{e9}"
        );
        // Truncates at a character boundary; index 0 is no string.
        assert_eq!(
            block_on(fetch_string::<_, 6>(&mut control, 4)).unwrap().as_str(),
            "Main "
        );
        assert!(block_on(fetch_string::<_, 16>(&mut control, 0)).unwrap().is_empty());
    }

    #[test]
    fn sysex_is_cut_into_event_packets_of_three_bytes() {
        let packets = |message: &[u8]| -> heapless::Vec<[u8; 4], 8> {
            SysexPackets::new(2, message)
                .unwrap()
                .map(|packet| packet.unwrap().to_bytes())
                .collect()
        };
        // Start, continue and end packets; then the three-byte and shortest messages.
        assert_eq!(
            packets(&[0xf0, 1, 2, 3, 4, 5, 6, 0xf7]),
            [[0x24, 0xf0, 1, 2], [0x24, 3, 4, 5], [0x26, 6, 0xf7, 0]]
        );
        assert_eq!(packets(&[0xf0, 1, 0xf7]), [[0x27, 0xf0, 1, 0xf7]]);
        assert_eq!(packets(&[0xf0, 0xf7]), [[0x26, 0xf0, 0xf7, 0]]);
        // A lone `F7` ends it when the previous chunk was full.
        assert_eq!(packets(&[0xf0, 1, 2, 0xf7]), [[0x24, 0xf0, 1, 2], [0x25, 0xf7, 0, 0]]);

        // Unframed messages and status bytes in the data are refused.
        for bad in [&[1u8, 2][..], &[0xf0, 1][..], &[0xf0, 0x90, 0xf7][..], &[][..]] {
            assert!(SysexPackets::new(0, bad).is_err());
        }
    }

    #[test]
    fn a_failed_pipe_allocation_puts_the_device_back_on_alternate_setting_0() {
        use embassy_futures::block_on;
        use embassy_usb_driver::host::{HostError, SplitInfo};
        use embassy_usb_driver::{EndpointInfo, EndpointType};

        use crate::host::test_utils::{MockHost, MockPipe, MockState};

        // No bulk pipes left.
        #[derive(Clone)]
        struct NoBulk<'d>(MockHost<'d>);
        impl<'d> UsbHostAllocator<'d> for NoBulk<'d> {
            type Pipe<T: pipe::Type, D: pipe::Direction> = MockPipe<'d>;

            fn alloc_pipe<T: pipe::Type, D: pipe::Direction>(
                &self,
                addr: u8,
                endpoint: &EndpointInfo,
                split: Option<SplitInfo>,
            ) -> Result<Self::Pipe<T, D>, HostError> {
                if endpoint.ep_type == EndpointType::Bulk {
                    return Err(HostError::OutOfPipes);
                }
                self.0.alloc_pipe::<T, D>(addr, endpoint, split)
            }
        }

        let enum_info = crate::host::test_utils::enumeration_info();
        let info = MidiInfo::midi2_from_config(MIDI2_CONFIG, &enum_info).unwrap();
        let state = MockState::new();
        state.set_gtb_descriptors(MIDI2_GTB);
        let alloc = NoBulk(MockHost::new(&state));
        let result = block_on(Midi2Host::connect_with_own_control_pipe(&alloc, info, &enum_info));
        assert!(matches!(result, Err(MidiError::NoPipe)));
        // Activated, then put back.
        assert_eq!(state.set_interface(), Some((1, 0)));
    }

    #[test]
    fn a_raw_read_returns_the_held_bytes_so_nothing_stale_is_left() {
        use embassy_futures::block_on;

        use crate::host::test_utils::{MockHost, MockState};

        let enum_info = crate::host::test_utils::enumeration_info();
        let info = MidiInfo::midi2_from_config(MIDI2_CONFIG, &enum_info).unwrap();
        let state = MockState::new();
        state.set_gtb_descriptors(MIDI2_GTB);
        let alloc = MockHost::new(&state);
        let mut host = block_on(Midi2Host::connect_with_own_control_pipe(&alloc, info, &enum_info)).unwrap();
        let out = info.output_endpoint().unwrap().to_descriptor(MidiVersion::Midi2);
        let mut sender = raw::UmpOutputPipe::open(&alloc, &out, &enum_info).unwrap();

        // A 32-bit packet and the first half of a 128-bit one: `read_packets` holds the half.
        let long = UmpPacket::from_u32_quad(0x5000_0001, 2, 3, 4).unwrap();
        let mut bytes = [0u8; 20];
        UmpPacket::from_u32(0x2090_3c7f).unwrap().to_bytes(&mut bytes).unwrap();
        long.to_bytes(&mut bytes[4..]).unwrap();
        block_on(sender.write(&bytes[..12])).unwrap();
        let mut buf = [0u8; 32];
        assert_eq!(
            block_on(host.receiver.as_mut().unwrap().read_packets(&mut buf)).unwrap(),
            4
        );

        // A raw read starts with the held half.
        block_on(sender.write(&bytes[12..])).unwrap();
        let n = block_on(host.read(&mut buf)).unwrap();
        assert_eq!(&buf[..n], &bytes[4..]);

        // Packets that follow are not joined to the old fragment.
        block_on(sender.write(&bytes[..4])).unwrap();
        let mut packets = block_on(host.read_ump(&mut buf)).unwrap();
        assert_eq!(packets.next(), Some(Ok(UmpPacket::from_u32(0x2090_3c7f).unwrap())));
        assert_eq!(packets.next(), None);
    }

    #[test]
    fn opening_needs_only_two_pipes_for_a_bidirectional_function() {
        use core::cell::Cell;

        use embassy_futures::block_on;
        use embassy_usb_driver::EndpointInfo;
        use embassy_usb_driver::host::{HostError, PipeError, SplitInfo, TimeoutConfig};

        use crate::host::test_utils::{MockHost, MockPipe, MockState};

        // Holds at most `max` pipes at a time.
        #[derive(Clone)]
        struct Limited<'d> {
            host: MockHost<'d>,
            live: &'d Cell<usize>,
            max: usize,
        }
        struct Counted<'d> {
            pipe: MockPipe<'d>,
            live: &'d Cell<usize>,
        }
        impl Drop for Counted<'_> {
            fn drop(&mut self) {
                self.live.set(self.live.get() - 1);
            }
        }
        impl<'d> UsbHostAllocator<'d> for Limited<'d> {
            type Pipe<T: pipe::Type, D: pipe::Direction> = Counted<'d>;

            fn alloc_pipe<T: pipe::Type, D: pipe::Direction>(
                &self,
                addr: u8,
                endpoint: &EndpointInfo,
                split: Option<SplitInfo>,
            ) -> Result<Self::Pipe<T, D>, HostError> {
                if self.live.get() >= self.max {
                    return Err(HostError::OutOfPipes);
                }
                let pipe = self.host.alloc_pipe::<T, D>(addr, endpoint, split)?;
                self.live.set(self.live.get() + 1);
                Ok(Counted { pipe, live: self.live })
            }
        }
        impl<T: pipe::Type, D: pipe::Direction> UsbPipe<T, D> for Counted<'_> {
            async fn control_in(&mut self, setup: &[u8; 8], buf: &mut [u8]) -> Result<usize, PipeError>
            where
                T: pipe::IsControl,
                D: pipe::IsIn,
            {
                UsbPipe::<T, D>::control_in(&mut self.pipe, setup, buf).await
            }
            async fn control_out(&mut self, setup: &[u8; 8], buf: &[u8]) -> Result<(), PipeError>
            where
                T: pipe::IsControl,
                D: pipe::IsOut,
            {
                UsbPipe::<T, D>::control_out(&mut self.pipe, setup, buf).await
            }
            async fn request_in(&mut self, buf: &mut [u8]) -> Result<usize, PipeError>
            where
                D: pipe::IsIn,
            {
                UsbPipe::<T, D>::request_in(&mut self.pipe, buf).await
            }
            async fn request_out(&mut self, buf: &[u8], ensure_transaction_end: bool) -> Result<(), PipeError>
            where
                D: pipe::IsOut,
            {
                UsbPipe::<T, D>::request_out(&mut self.pipe, buf, ensure_transaction_end).await
            }
            fn set_timeout(&mut self, timeout: TimeoutConfig)
            where
                T: pipe::IsControl,
            {
                UsbPipe::<T, D>::set_timeout(&mut self.pipe, timeout)
            }
            fn reset_data_toggle(&mut self)
            where
                T: pipe::IsBulkOrInterrupt,
            {
                UsbPipe::<T, D>::reset_data_toggle(&mut self.pipe)
            }
        }

        let enum_info = crate::host::test_utils::enumeration_info();
        let info = MidiInfo::midi2_from_config(MIDI2_CONFIG, &enum_info).unwrap();
        let open = |max| {
            let state = MockState::new();
            state.set_gtb_descriptors(MIDI2_GTB);
            let live = Cell::new(0);
            let alloc = Limited {
                host: MockHost::new(&state),
                live: &live,
                max,
            };
            let result = block_on(Midi2Host::connect_with_own_control_pipe(&alloc, info, &enum_info));
            (result.map(|_| ()), state.set_interface())
        };

        // Two slots suffice: the control pipe is freed before the data pipes.
        let (result, alternate_setting) = open(2);
        assert!(result.is_ok());
        assert_eq!(alternate_setting, Some((1, 1)));
        // One does not, and the device is put back on alternate setting 0.
        let (result, alternate_setting) = open(1);
        assert!(matches!(result, Err(MidiError::NoPipe)));
        assert_eq!(alternate_setting, Some((1, 0)));
    }
}

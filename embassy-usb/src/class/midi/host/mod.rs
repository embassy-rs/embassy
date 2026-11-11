//! USB MIDI 1.0 host class driver.
//!
//! USB MIDI 1.0 transports one or more virtual MIDI cables over bulk
//! endpoints. Each transfer consists of four-byte USB-MIDI event packets.

use core::marker::PhantomData;

use embassy_usb_driver::host::{PipeError, UsbHostAllocator, UsbPipe, pipe};
use heapless::Vec;

use crate::class::midi::{
    CS_GR_TRM_BLOCK, GroupTerminalBlock, GtbError, MAX_GTBS, MAX_MIDI_JACKS, MidiPacket, MidiPacketError,
    MidiPacketReader, USB_AUDIO_CLASS, USB_MIDISTREAMING_SUBCLASS, UmpError, UmpPacket, UmpReader,
    parse_gtb_descriptors,
};
use crate::host::class::ClassMatch;
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
/// This convenience API supports one alternate-setting-zero MIDIStreaming
/// interface with at most one bulk endpoint per direction. Use [`raw`] for
/// devices with multiple streaming interfaces or same-direction endpoints.
pub struct MidiHost<'d, A: UsbHostAllocator<'d>> {
    sender: Sender<'d, A>,
    receiver: Receiver<'d, A>,
}

impl<'d, A: UsbHostAllocator<'d>> MidiHost<'d, A> {
    /// Discover the MIDI ports and allocate the available bulk pipes.
    pub fn new(alloc: &A, config_desc: &[u8], enum_info: &EnumerationInfo) -> Result<Self, MidiError> {
        let interfaces = parse_midi_interfaces_for_device(&enum_info.device_desc, config_desc)?;
        let (interface, input_endpoint, output_endpoint) = select_streaming_interface(&interfaces)?;

        let receiver = Receiver::open(alloc, interface, input_endpoint, enum_info)?;
        let sender = Sender::open(alloc, interface, output_endpoint, enum_info)?;
        Ok(Self { sender, receiver })
    }

    /// Logical device-to-host MIDI ports.
    pub fn input_ports(&self) -> &[MidiInputPort] {
        self.receiver.ports()
    }

    /// Logical host-to-device MIDI ports.
    pub fn output_ports(&self) -> &[MidiOutputPort] {
        self.sender.ports()
    }

    /// Receive one bulk transfer containing USB-MIDI event packets.
    pub async fn read_transfer(&mut self, buf: &mut [u8]) -> Result<usize, MidiError> {
        self.receiver.read_transfer(buf).await
    }

    /// Send one or more complete USB-MIDI event packets.
    pub async fn write_packet(&mut self, packets: &[u8]) -> Result<(), MidiError> {
        self.sender.write_packet(packets).await
    }

    /// Encode and send one MIDI message, or a SysEx fragment of up to three bytes, to a logical port.
    pub async fn send(&mut self, port: MidiOutputPort, message: &[u8]) -> Result<(), MidiError> {
        self.sender.send(port, message).await
    }

    /// Split the class into independently owned sender and receiver halves.
    ///
    /// This allows sending and receiving from separate tasks.
    pub fn split(self) -> (Sender<'d, A>, Receiver<'d, A>) {
        (self.sender, self.receiver)
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
    let mut matching = interfaces
        .iter()
        .filter(|interface| interface.alternate_setting == 0 && !interface.endpoints.is_empty());
    let interface = matching.next().ok_or(MidiError::NoInterface)?;
    if matching.next().is_some() {
        return Err(MidiError::UnsupportedTopology);
    }

    let mut input_endpoint = None;
    let mut output_endpoint = None;
    for endpoint in &interface.endpoints {
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
pub struct Sender<'d, A: UsbHostAllocator<'d>> {
    pipe: Option<transport::MidiOutputPipe<'d, A>>,
    ports: Vec<MidiOutputPort, MAX_MIDI_JACKS>,
}

impl<'d, A: UsbHostAllocator<'d>> Sender<'d, A> {
    fn open(
        alloc: &A,
        interface: &MidiStreamingInterface,
        endpoint: Option<&MidiEndpointDescriptor>,
        enum_info: &EnumerationInfo,
    ) -> Result<Self, MidiError> {
        let Some(endpoint) = endpoint else {
            return Ok(Self {
                pipe: None,
                ports: Vec::new(),
            });
        };
        let ports = cable_ports(interface.interface_number, endpoint, enum_info.device_address)?;
        let pipe = transport::MidiOutputPipe::open(alloc, endpoint, enum_info)?;
        Ok(Self {
            pipe: Some(pipe),
            ports,
        })
    }

    /// Logical ports that can receive MIDI from the host.
    pub fn ports(&self) -> &[MidiOutputPort] {
        &self.ports
    }

    /// Send one or more complete USB-MIDI event packets.
    pub async fn write_packet(&mut self, packets: &[u8]) -> Result<(), MidiError> {
        self.pipe.as_mut().ok_or(MidiError::NoEndpoint)?.write(packets).await
    }

    /// Encode and send one MIDI message, or a SysEx fragment of up to three bytes, to a logical port.
    pub async fn send(&mut self, port: MidiOutputPort, message: &[u8]) -> Result<(), MidiError> {
        if !self.ports.contains(&port) {
            return Err(MidiError::InvalidPort);
        }
        let packet = MidiPacket::try_encode_exact(port.cable, message)?;
        self.write_packet(packet.as_bytes()).await
    }
}

/// USB-MIDI device-to-host packet receiver.
pub struct Receiver<'d, A: UsbHostAllocator<'d>> {
    pipe: Option<transport::MidiInputPipe<'d, A>>,
    ports: Vec<MidiInputPort, MAX_MIDI_JACKS>,
}

impl<'d, A: UsbHostAllocator<'d>> Receiver<'d, A> {
    fn open(
        alloc: &A,
        interface: &MidiStreamingInterface,
        endpoint: Option<&MidiEndpointDescriptor>,
        enum_info: &EnumerationInfo,
    ) -> Result<Self, MidiError> {
        let Some(endpoint) = endpoint else {
            return Ok(Self {
                pipe: None,
                ports: Vec::new(),
            });
        };
        let ports = cable_ports(interface.interface_number, endpoint, enum_info.device_address)?;
        let pipe = transport::MidiInputPipe::open(alloc, endpoint, enum_info)?;
        Ok(Self {
            pipe: Some(pipe),
            ports,
        })
    }

    /// Logical ports that can send MIDI to the host.
    pub fn ports(&self) -> &[MidiInputPort] {
        &self.ports
    }

    /// Receive one bulk transfer containing complete USB-MIDI event packets.
    pub async fn read_transfer(&mut self, buf: &mut [u8]) -> Result<usize, MidiError> {
        self.pipe.as_mut().ok_or(MidiError::NoEndpoint)?.read(buf).await
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

/// Discovered description of a MIDI 2.0 streaming function.
///
/// Discovery, alternate-setting activation, and pipe allocation are separate
/// steps so they can run in different tasks:
///
/// 1. [`Midi2Info::discover`] parses the configuration descriptor.
/// 2. [`Midi2Info::activate`] selects the MIDI 2.0 alternate setting.
/// 3. [`Midi2Host::new`] allocates the UMP pipes from the [`Midi2Info`].
///
/// [`Midi2Info::fetch_group_terminal_blocks`] additionally reads the device's
/// Group Terminal Blocks so the group topology can be inspected.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Midi2Info {
    interface_number: u8,
    alternate_setting: u8,
    input_endpoint: Option<MidiEndpointDescriptor>,
    output_endpoint: Option<MidiEndpointDescriptor>,
    group_terminal_blocks: Vec<GroupTerminalBlock, MAX_GTBS>,
}

impl Midi2Info {
    /// Discover the first MIDI 2.0 streaming function in a configuration descriptor.
    ///
    /// This only parses descriptors and does not touch the device. A MIDI 2.0
    /// function is an alternate setting >= 1 whose class-specific header reports
    /// `bcdMSC >= 0x0200`.
    pub fn discover(config_desc: &[u8], enum_info: &EnumerationInfo) -> Result<Self, MidiError> {
        let interfaces = parse_midi_interfaces_for_device(&enum_info.device_desc, config_desc)?;
        let interface = interfaces
            .iter()
            .find(|iface| iface.is_midi2())
            .ok_or(MidiError::NoInterface)?;

        let mut input_endpoint = None;
        let mut output_endpoint = None;
        for endpoint in &interface.endpoints {
            let slot = if endpoint.is_in() {
                &mut input_endpoint
            } else {
                &mut output_endpoint
            };
            if slot.replace(endpoint.clone()).is_some() {
                return Err(MidiError::UnsupportedTopology);
            }
        }

        Ok(Self {
            interface_number: interface.interface_number,
            alternate_setting: interface.alternate_setting,
            input_endpoint,
            output_endpoint,
            group_terminal_blocks: Vec::new(),
        })
    }

    /// Fetch and store this function's Group Terminal Blocks.
    ///
    /// `buffer` must be large enough for the GTB header and all block descriptors;
    /// `GR_TRM_BLOCK_HEADER_LEN + MAX_GTBS * GR_TRM_BLOCK_LEN` bytes always fit.
    pub async fn fetch_group_terminal_blocks<P>(
        &mut self,
        control_pipe: &mut P,
        buffer: &mut [u8],
    ) -> Result<(), MidiError>
    where
        P: UsbPipe<pipe::Control, pipe::InOut>,
    {
        self.group_terminal_blocks = crate::class::midi::host::fetch_group_terminal_blocks::<_, MAX_GTBS>(
            control_pipe,
            self.interface_number,
            self.alternate_setting,
            buffer,
        )
        .await?;
        Ok(())
    }

    /// Select this function's MIDI 2.0 alternate setting.
    ///
    /// Must be called before the UMP pipes are used: the MIDI 1.0 and MIDI 2.0
    /// settings share their bulk endpoints, and only the selected setting's
    /// protocol is carried over them.
    pub async fn activate<P>(&self, control_pipe: &mut P) -> Result<(), MidiError>
    where
        P: UsbPipe<pipe::Control, pipe::InOut>,
    {
        if self.alternate_setting != 0 {
            let setup = crate::host::control::SetupPacket::set_interface(
                self.interface_number as u16,
                self.alternate_setting as u16,
            );
            control_pipe.control_out(&setup.to_bytes(), &[]).await?;
        }
        Ok(())
    }

    /// Standard interface number of this MIDI 2.0 function.
    pub fn interface_number(&self) -> u8 {
        self.interface_number
    }

    /// Alternate setting that carries MIDI 2.0 / UMP.
    pub fn alternate_setting(&self) -> u8 {
        self.alternate_setting
    }

    /// Device-to-host (IN) bulk endpoint, if the function has one.
    pub fn input_endpoint(&self) -> Option<&MidiEndpointDescriptor> {
        self.input_endpoint.as_ref()
    }

    /// Host-to-device (OUT) bulk endpoint, if the function has one.
    pub fn output_endpoint(&self) -> Option<&MidiEndpointDescriptor> {
        self.output_endpoint.as_ref()
    }

    /// Group Terminal Blocks fetched by [`Midi2Info::fetch_group_terminal_blocks`].
    pub fn group_terminal_blocks(&self) -> &[GroupTerminalBlock] {
        &self.group_terminal_blocks
    }

    /// Group numbers carried on the device-to-host (IN) endpoint.
    ///
    /// Empty until the Group Terminal Blocks have been fetched.
    pub fn input_groups(&self) -> impl Iterator<Item = u8> + '_ {
        self.groups_for(self.input_endpoint.as_ref())
    }

    /// Group numbers carried on the host-to-device (OUT) endpoint.
    ///
    /// Empty until the Group Terminal Blocks have been fetched.
    pub fn output_groups(&self) -> impl Iterator<Item = u8> + '_ {
        self.groups_for(self.output_endpoint.as_ref())
    }

    fn groups_for<'a>(&'a self, endpoint: Option<&'a MidiEndpointDescriptor>) -> impl Iterator<Item = u8> + 'a {
        endpoint
            .into_iter()
            .flat_map(|endpoint| endpoint.group_terminal_block_ids.iter().copied())
            .flat_map(move |block_id| {
                self.group_terminal_blocks
                    .iter()
                    .filter(move |block| block.block_id == block_id)
                    .flat_map(|block| block.first_group..block.first_group.saturating_add(block.num_groups))
            })
    }
}

/// Friendly host driver for one USB MIDI 2.0 streaming function.
pub struct Midi2Host<'d, A: UsbHostAllocator<'d>> {
    sender: Option<raw::UmpOutputPipe<'d, A>>,
    receiver: Option<raw::UmpInputPipe<'d, A>>,
    info: Midi2Info,
}

impl<'d, A: UsbHostAllocator<'d>> Midi2Host<'d, A> {
    /// Allocate UMP pipes for a discovered MIDI 2.0 function.
    ///
    /// Call [`Midi2Info::activate`] first so the device carries UMP on the shared
    /// bulk endpoints.
    pub fn new(alloc: &A, info: Midi2Info, enum_info: &EnumerationInfo) -> Result<Self, MidiError> {
        let receiver = info
            .input_endpoint
            .as_ref()
            .map(|endpoint| raw::UmpInputPipe::open(alloc, endpoint, enum_info))
            .transpose()?;
        let sender = info
            .output_endpoint
            .as_ref()
            .map(|endpoint| raw::UmpOutputPipe::open(alloc, endpoint, enum_info))
            .transpose()?;

        Ok(Self { sender, receiver, info })
    }

    /// Discover, activate, and construct a host for the first MIDI 2.0 function.
    ///
    /// This is a convenience for the common case. Use [`Midi2Info::discover`],
    /// [`Midi2Info::activate`], and [`Midi2Host::new`] separately to run discovery
    /// and activation in different tasks. Group Terminal Blocks are not fetched;
    /// use [`Midi2Info::fetch_group_terminal_blocks`] (or [`Midi2Host::info_mut`])
    /// when the topology is needed.
    pub async fn connect<P>(
        alloc: &A,
        control_pipe: &mut P,
        config_desc: &[u8],
        enum_info: &EnumerationInfo,
    ) -> Result<Self, MidiError>
    where
        P: UsbPipe<pipe::Control, pipe::InOut>,
    {
        let info = Midi2Info::discover(config_desc, enum_info)?;
        info.activate(control_pipe).await?;
        Self::new(alloc, info, enum_info)
    }

    /// The discovered function description.
    pub fn info(&self) -> &Midi2Info {
        &self.info
    }

    /// Mutable access to the discovered description, for fetching Group Terminal Blocks.
    pub fn info_mut(&mut self) -> &mut Midi2Info {
        &mut self.info
    }

    /// Returns the interface number of this MIDI 2.0 streaming interface.
    pub fn interface_number(&self) -> u8 {
        self.info.interface_number
    }

    /// Receive raw UMP bytes into `buf`.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, MidiError> {
        let receiver = self.receiver.as_mut().ok_or(MidiError::NoEndpoint)?;
        receiver.read(buf).await
    }

    /// Receive UMP packets and return a `UmpReader` to iterate through them.
    pub async fn read_ump<'a>(&mut self, buf: &'a mut [u8]) -> Result<UmpReader<'a>, MidiError> {
        let receiver = self.receiver.as_mut().ok_or(MidiError::NoEndpoint)?;
        receiver.read_ump(buf).await
    }

    /// Write raw UMP bytes.
    pub async fn write(&mut self, data: &[u8]) -> Result<(), MidiError> {
        let sender = self.sender.as_mut().ok_or(MidiError::NoEndpoint)?;
        sender.write(data).await
    }

    /// Send a single Universal MIDI Packet.
    pub async fn write_packet(&mut self, packet: &UmpPacket) -> Result<(), MidiError> {
        let sender = self.sender.as_mut().ok_or(MidiError::NoEndpoint)?;
        sender.write_packet(packet).await
    }

    /// Split into independently owned sender and receiver halves, plus the discovery info.
    pub fn split(
        self,
    ) -> (
        Option<raw::UmpOutputPipe<'d, A>>,
        Option<raw::UmpInputPipe<'d, A>>,
        Midi2Info,
    ) {
        (self.sender, self.receiver, self.info)
    }
}

/// Query the Group Terminal Blocks for an interface's MIDI 2.0 alternate setting.
///
/// This is the standard `GET_DESCRIPTOR` request defined by USB-MIDI 2.0 §6:
/// `wValue` carries `CS_GR_TRM_BLOCK (0x26)` in the high byte and the alternate
/// setting in the low byte, and `wIndex` is the interface number.
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

    let setup = SetupPacket::standard_interface_in(
        crate::control::Request::GET_DESCRIPTOR,
        ((CS_GR_TRM_BLOCK as u16) << 8) | alternate_setting as u16,
        interface_number as u16,
        buffer.len() as u16,
    );
    let len = control_pipe.control_in(&setup.to_bytes(), buffer).await?;
    let blocks = parse_gtb_descriptors(&buffer[..len])?;
    Ok(blocks)
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
            jack_ids: Vec::from_slice(&[3, 7]).unwrap(),
            group_terminal_block_ids: Vec::new(),
        };
        let ports = cable_ports::<In>(2, &endpoint, 5).unwrap();

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
            jack_ids: Vec::new(),
            group_terminal_block_ids: Vec::new(),
        };
        let ports = cable_ports::<Out>(1, &endpoint, 4).unwrap();

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

    // Group Terminal Blocks matching MIDI2_CONFIG: block 1 is bidirectional and
    // covers groups 0-1; block 2 is input-only and covers group 2.
    #[rustfmt::skip]
    const MIDI2_GTB: &[u8] = &[
        5, 0x26, 0x01, 31, 0,
        13, 0x26, 0x02, 1, 0x00, 0, 2, 0, 0x11, 0, 0, 0, 0,
        13, 0x26, 0x02, 2, 0x01, 2, 1, 0, 0x01, 0, 0, 0, 0,
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
        let mut info = Midi2Info::discover(MIDI2_CONFIG, &enum_info).unwrap();
        assert_eq!(info.interface_number(), 1);
        assert_eq!(info.alternate_setting(), 1);
        assert_eq!(info.input_endpoint().unwrap().address, 0x81);
        assert_eq!(info.output_endpoint().unwrap().address, 0x01);
        assert!(info.group_terminal_blocks().is_empty());
        assert_eq!(info.input_groups().count(), 0);

        let state = MockState::new();
        state.set_gtb_descriptors(MIDI2_GTB);
        let host = MockHost::new(&state);
        let control_ep = EndpointInfo {
            addr: 0.into(),
            ep_type: EndpointType::Control,
            max_packet_size: 64,
            interval_ms: 0,
        };
        let mut control = host
            .alloc_pipe::<pipe::Control, pipe::InOut>(enum_info.device_address, &control_ep, enum_info.split())
            .unwrap();

        // Activation is a separate step from discovery.
        block_on(info.activate(&mut control)).unwrap();
        assert_eq!(state.set_interface(), Some((1, 1)));

        // GTB retrieval uses the standard request with the alternate setting in wValue.
        let mut gtb_buf = [0u8; 64];
        block_on(info.fetch_group_terminal_blocks(&mut control, &mut gtb_buf)).unwrap();
        assert_eq!(state.gtb_request(), Some((0x2601, 1)));

        let blocks = info.group_terminal_blocks();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].protocol, MidiProtocol::Midi2_0);
        assert_eq!(blocks[1].protocol, MidiProtocol::Midi1_0_64);
        assert!(info.input_groups().eq([0, 1, 2]));
        assert!(info.output_groups().eq([0, 1]));

        // Pipe allocation is the final, separate step.
        let midi = Midi2Host::new(&host, info, &enum_info).unwrap();
        assert_eq!(midi.interface_number(), 1);
        assert!(midi.info().input_groups().eq([0, 1, 2]));

        let (sender, receiver, info) = midi.split();
        assert!(sender.is_some());
        assert!(receiver.is_some());
        assert_eq!(info.group_terminal_blocks().len(), 2);
    }
}

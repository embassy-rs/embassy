//! USB MIDI 1.0 host class driver.
//!
//! USB MIDI 1.0 transports one or more virtual MIDI cables over bulk
//! endpoints. Each transfer consists of four-byte USB-MIDI event packets.

use core::marker::PhantomData;

use embassy_usb_driver::host::{PipeError, UsbHostAllocator};
use heapless::Vec;

use crate::class::midi::{MAX_MIDI_JACKS, MidiPacket, MidiPacketError, MidiPacketReader};
use crate::host::handler::EnumerationInfo;

mod descriptors;
mod transport;

use descriptors::{
    MidiDescriptorError, MidiEndpointDescriptor, MidiStreamingInterface, parse_midi_interfaces_for_device,
};

/// Advanced descriptor and endpoint-level USB-MIDI APIs.
pub mod raw {
    pub use super::descriptors::*;
    pub use super::transport::{MidiInputPipe, MidiOutputPipe};
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
}

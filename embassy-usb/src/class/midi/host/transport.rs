//! Low-level USB-MIDI endpoint transports.
//!
//! MIDI 1.0 opens bulk endpoints. MIDI 2.0 opens bulk or interrupt endpoints
//! (USB MIDI 2.0 §3.3.2 and §5.3; Appendix B.7 is bulk OUT and Appendix B.8 is
//! interrupt IN). The MIDI 1.0 and UMP pipes differ only in their framing checks.

use embassy_usb_driver::host::{UsbHostAllocator, UsbPipe, pipe};
use embassy_usb_driver::{EndpointAddress, EndpointInfo, EndpointType};

use super::{MidiEndpointDescriptor, MidiError};
use crate::class::midi::packet::check_transfer_len;
use crate::class::midi::{MAX_UMP_BYTES, MIDI_PACKET_SIZE, MidiPacketError, UmpPacket, UmpReader};
use crate::host::handler::EnumerationInfo;

/// Size of a UMP word in bytes.
const UMP_WORD_SIZE: usize = 4;

fn check_ump_len(len: usize) -> Result<(), MidiPacketError> {
    if len == 0 || !len.is_multiple_of(UMP_WORD_SIZE) {
        return Err(MidiPacketError::InvalidPacketLength);
    }
    Ok(())
}

fn endpoint_info(endpoint: &MidiEndpointDescriptor) -> EndpointInfo {
    EndpointInfo {
        addr: EndpointAddress::from(endpoint.address),
        ep_type: endpoint.ep_type,
        max_packet_size: endpoint.max_packet_size,
        // Full-speed interrupt `bInterval` is a frame count (Appendix B.8). Bulk leaves it at 0 (§5.3.1).
        interval_ms: endpoint.interval,
        max_burst: 0,
    }
}

enum DataIn<'d, A: UsbHostAllocator<'d>> {
    Bulk(A::Pipe<pipe::Bulk, pipe::In>),
    Interrupt(A::Pipe<pipe::Interrupt, pipe::In>),
}

enum DataOut<'d, A: UsbHostAllocator<'d>> {
    Bulk(A::Pipe<pipe::Bulk, pipe::Out>),
    Interrupt(A::Pipe<pipe::Interrupt, pipe::Out>),
}

impl<'d, A: UsbHostAllocator<'d>> DataIn<'d, A> {
    fn open(alloc: &A, endpoint: &MidiEndpointDescriptor, enum_info: &EnumerationInfo) -> Result<Self, MidiError> {
        if !endpoint.is_in() {
            return Err(MidiError::WrongDirection);
        }
        let info = endpoint_info(endpoint);
        match endpoint.ep_type {
            EndpointType::Interrupt => alloc
                .alloc_pipe::<pipe::Interrupt, pipe::In>(enum_info.device_address, &info, enum_info.split())
                .map(Self::Interrupt)
                .map_err(|_| MidiError::NoPipe),
            EndpointType::Bulk => alloc
                .alloc_pipe::<pipe::Bulk, pipe::In>(enum_info.device_address, &info, enum_info.split())
                .map(Self::Bulk)
                .map_err(|_| MidiError::NoPipe),
            _ => Err(MidiError::UnsupportedTopology),
        }
    }

    async fn request_in(&mut self, buf: &mut [u8]) -> Result<usize, MidiError> {
        match self {
            Self::Bulk(pipe) => Ok(pipe.request_in(buf).await?),
            Self::Interrupt(pipe) => Ok(pipe.request_in(buf).await?),
        }
    }
}

impl<'d, A: UsbHostAllocator<'d>> DataOut<'d, A> {
    fn open(alloc: &A, endpoint: &MidiEndpointDescriptor, enum_info: &EnumerationInfo) -> Result<Self, MidiError> {
        if endpoint.is_in() {
            return Err(MidiError::WrongDirection);
        }
        let info = endpoint_info(endpoint);
        match endpoint.ep_type {
            EndpointType::Interrupt => alloc
                .alloc_pipe::<pipe::Interrupt, pipe::Out>(enum_info.device_address, &info, enum_info.split())
                .map(Self::Interrupt)
                .map_err(|_| MidiError::NoPipe),
            EndpointType::Bulk => alloc
                .alloc_pipe::<pipe::Bulk, pipe::Out>(enum_info.device_address, &info, enum_info.split())
                .map(Self::Bulk)
                .map_err(|_| MidiError::NoPipe),
            _ => Err(MidiError::UnsupportedTopology),
        }
    }

    async fn request_out(&mut self, buf: &[u8], ensure_transaction_end: bool) -> Result<(), MidiError> {
        match self {
            Self::Bulk(pipe) => Ok(pipe.request_out(buf, ensure_transaction_end).await?),
            Self::Interrupt(pipe) => Ok(pipe.request_out(buf, ensure_transaction_end).await?),
        }
    }
}

/// A transfer ending on a full-size packet is terminated with a zero-length packet.
const ENSURE_TRANSACTION_END: bool = true;

/// Device-to-host USB-MIDI transport.
pub struct MidiInputPipe<'d, A: UsbHostAllocator<'d>> {
    pipe: DataIn<'d, A>,
}

impl<'d, A: UsbHostAllocator<'d>> MidiInputPipe<'d, A> {
    /// Use an already allocated bulk IN pipe.
    pub fn from_pipe(pipe: A::Pipe<pipe::Bulk, pipe::In>) -> Self {
        Self {
            pipe: DataIn::Bulk(pipe),
        }
    }

    /// Use an already allocated interrupt IN pipe.
    pub fn from_interrupt_pipe(pipe: A::Pipe<pipe::Interrupt, pipe::In>) -> Self {
        Self {
            pipe: DataIn::Interrupt(pipe),
        }
    }

    /// Allocate a pipe for a device-to-host MIDI endpoint.
    pub fn open(alloc: &A, endpoint: &MidiEndpointDescriptor, enum_info: &EnumerationInfo) -> Result<Self, MidiError> {
        Ok(Self {
            pipe: DataIn::open(alloc, endpoint, enum_info)?,
        })
    }

    /// Receive one USB transfer containing complete event packets.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, MidiError> {
        check_transfer_len(buffer.len())?;
        let length = self.pipe.request_in(buffer).await?;
        // A zero-length transfer is valid and carries no packets.
        if !length.is_multiple_of(MIDI_PACKET_SIZE) {
            return Err(MidiPacketError::InvalidPacketLength.into());
        }
        Ok(length)
    }
}

/// Host-to-device USB-MIDI transport.
pub struct MidiOutputPipe<'d, A: UsbHostAllocator<'d>> {
    pipe: DataOut<'d, A>,
}

impl<'d, A: UsbHostAllocator<'d>> MidiOutputPipe<'d, A> {
    /// Use an already allocated bulk OUT pipe.
    pub fn from_pipe(pipe: A::Pipe<pipe::Bulk, pipe::Out>) -> Self {
        Self {
            pipe: DataOut::Bulk(pipe),
        }
    }

    /// Use an already allocated interrupt OUT pipe.
    pub fn from_interrupt_pipe(pipe: A::Pipe<pipe::Interrupt, pipe::Out>) -> Self {
        Self {
            pipe: DataOut::Interrupt(pipe),
        }
    }

    /// Allocate a pipe for a host-to-device MIDI endpoint.
    pub fn open(alloc: &A, endpoint: &MidiEndpointDescriptor, enum_info: &EnumerationInfo) -> Result<Self, MidiError> {
        Ok(Self {
            pipe: DataOut::open(alloc, endpoint, enum_info)?,
        })
    }

    /// Send one USB transfer containing complete event packets.
    pub async fn write(&mut self, packets: &[u8]) -> Result<(), MidiError> {
        check_transfer_len(packets.len())?;
        self.pipe.request_out(packets, ENSURE_TRANSACTION_END).await?;
        Ok(())
    }
}

/// Device-to-host Universal MIDI Packet (UMP / MIDI 2.0) transport.
pub struct UmpInputPipe<'d, A: UsbHostAllocator<'d>> {
    pipe: DataIn<'d, A>,
}

impl<'d, A: UsbHostAllocator<'d>> UmpInputPipe<'d, A> {
    /// Use an already allocated bulk IN pipe.
    pub fn from_pipe(pipe: A::Pipe<pipe::Bulk, pipe::In>) -> Self {
        Self {
            pipe: DataIn::Bulk(pipe),
        }
    }

    /// Use an already allocated interrupt IN pipe.
    pub fn from_interrupt_pipe(pipe: A::Pipe<pipe::Interrupt, pipe::In>) -> Self {
        Self {
            pipe: DataIn::Interrupt(pipe),
        }
    }

    /// Allocate a pipe for a device-to-host MIDI 2.0 endpoint.
    pub fn open(alloc: &A, endpoint: &MidiEndpointDescriptor, enum_info: &EnumerationInfo) -> Result<Self, MidiError> {
        Ok(Self {
            pipe: DataIn::open(alloc, endpoint, enum_info)?,
        })
    }

    /// Receive one USB transfer containing UMP data.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, MidiError> {
        check_ump_len(buffer.len())?;
        let length = self.pipe.request_in(buffer).await?;
        // A zero-length transfer is valid and carries no packets.
        if !length.is_multiple_of(UMP_WORD_SIZE) {
            return Err(MidiPacketError::InvalidPacketLength.into());
        }
        Ok(length)
    }

    /// Receive UMP packets and return an iterator.
    pub async fn read_ump<'a>(&mut self, buffer: &'a mut [u8]) -> Result<UmpReader<'a>, MidiError> {
        let length = self.read(buffer).await?;
        Ok(UmpReader::new(&buffer[..length]))
    }
}

/// Host-to-device Universal MIDI Packet (UMP / MIDI 2.0) transport.
pub struct UmpOutputPipe<'d, A: UsbHostAllocator<'d>> {
    pipe: DataOut<'d, A>,
}

impl<'d, A: UsbHostAllocator<'d>> UmpOutputPipe<'d, A> {
    /// Use an already allocated bulk OUT pipe.
    pub fn from_pipe(pipe: A::Pipe<pipe::Bulk, pipe::Out>) -> Self {
        Self {
            pipe: DataOut::Bulk(pipe),
        }
    }

    /// Use an already allocated interrupt OUT pipe.
    pub fn from_interrupt_pipe(pipe: A::Pipe<pipe::Interrupt, pipe::Out>) -> Self {
        Self {
            pipe: DataOut::Interrupt(pipe),
        }
    }

    /// Allocate a pipe for a host-to-device MIDI 2.0 endpoint.
    pub fn open(alloc: &A, endpoint: &MidiEndpointDescriptor, enum_info: &EnumerationInfo) -> Result<Self, MidiError> {
        Ok(Self {
            pipe: DataOut::open(alloc, endpoint, enum_info)?,
        })
    }

    /// Send raw UMP bytes.
    pub async fn write(&mut self, bytes: &[u8]) -> Result<(), MidiError> {
        check_ump_len(bytes.len())?;
        self.pipe.request_out(bytes, ENSURE_TRANSACTION_END).await?;
        Ok(())
    }

    /// Send a single Universal MIDI Packet.
    pub async fn write_packet(&mut self, packet: &UmpPacket) -> Result<(), MidiError> {
        let mut buf = [0u8; MAX_UMP_BYTES];
        let n = packet
            .to_bytes(&mut buf)
            .map_err(|_| MidiPacketError::InvalidPacketLength)?;
        self.write(&buf[..n]).await
    }
}

// Valid only when both alternate settings use the same endpoint.
impl<'d, A: UsbHostAllocator<'d>> From<MidiInputPipe<'d, A>> for UmpInputPipe<'d, A> {
    fn from(pipe: MidiInputPipe<'d, A>) -> Self {
        Self { pipe: pipe.pipe }
    }
}

impl<'d, A: UsbHostAllocator<'d>> From<UmpInputPipe<'d, A>> for MidiInputPipe<'d, A> {
    fn from(pipe: UmpInputPipe<'d, A>) -> Self {
        Self { pipe: pipe.pipe }
    }
}

impl<'d, A: UsbHostAllocator<'d>> From<MidiOutputPipe<'d, A>> for UmpOutputPipe<'d, A> {
    fn from(pipe: MidiOutputPipe<'d, A>) -> Self {
        Self { pipe: pipe.pipe }
    }
}

impl<'d, A: UsbHostAllocator<'d>> From<UmpOutputPipe<'d, A>> for MidiOutputPipe<'d, A> {
    fn from(pipe: UmpOutputPipe<'d, A>) -> Self {
        Self { pipe: pipe.pipe }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_one_or_more_complete_event_packets() {
        assert!(check_transfer_len(MIDI_PACKET_SIZE).is_ok());
        assert!(check_transfer_len(MIDI_PACKET_SIZE * 4).is_ok());
    }

    #[test]
    fn rejects_empty_and_partial_event_packets() {
        for len in [0, MIDI_PACKET_SIZE - 1, MIDI_PACKET_SIZE + 1] {
            assert_eq!(check_transfer_len(len), Err(MidiPacketError::InvalidPacketLength));
        }
    }
}

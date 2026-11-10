//! Low-level USB-MIDI bulk endpoint transports.

use embassy_usb_driver::host::{UsbHostAllocator, UsbPipe, pipe};
use embassy_usb_driver::{EndpointAddress, EndpointInfo, EndpointType};

use super::{MidiEndpointDescriptor, MidiError};
use crate::class::midi::packet::check_transfer_len;
use crate::class::midi::{MAX_UMP_BYTES, MIDI_PACKET_SIZE, MidiPacketError, UmpPacket, UmpReader};
use crate::host::handler::EnumerationInfo;

/// Device-to-host USB-MIDI bulk transport.
pub struct MidiInputPipe<'d, A: UsbHostAllocator<'d>> {
    pipe: A::Pipe<pipe::Bulk, pipe::In>,
}

impl<'d, A: UsbHostAllocator<'d>> MidiInputPipe<'d, A> {
    /// Allocate a pipe for a device-to-host MIDI endpoint.
    pub fn open(alloc: &A, endpoint: &MidiEndpointDescriptor, enum_info: &EnumerationInfo) -> Result<Self, MidiError> {
        if !endpoint.is_in() {
            return Err(MidiError::WrongDirection);
        }
        let endpoint = EndpointInfo {
            addr: EndpointAddress::from(endpoint.address),
            ep_type: EndpointType::Bulk,
            max_packet_size: endpoint.max_packet_size,
            interval_ms: 0,
        };
        let pipe = alloc
            .alloc_pipe::<pipe::Bulk, pipe::In>(enum_info.device_address, &endpoint, enum_info.split())
            .map_err(|_| MidiError::NoPipe)?;
        Ok(Self { pipe })
    }

    /// Receive one USB bulk transfer containing complete event packets.
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

/// Host-to-device USB-MIDI bulk transport.
pub struct MidiOutputPipe<'d, A: UsbHostAllocator<'d>> {
    pipe: A::Pipe<pipe::Bulk, pipe::Out>,
}

impl<'d, A: UsbHostAllocator<'d>> MidiOutputPipe<'d, A> {
    /// Allocate a pipe for a host-to-device MIDI endpoint.
    pub fn open(alloc: &A, endpoint: &MidiEndpointDescriptor, enum_info: &EnumerationInfo) -> Result<Self, MidiError> {
        if endpoint.is_in() {
            return Err(MidiError::WrongDirection);
        }
        let endpoint = EndpointInfo {
            addr: EndpointAddress::from(endpoint.address),
            ep_type: EndpointType::Bulk,
            max_packet_size: endpoint.max_packet_size,
            interval_ms: 0,
        };
        let pipe = alloc
            .alloc_pipe::<pipe::Bulk, pipe::Out>(enum_info.device_address, &endpoint, enum_info.split())
            .map_err(|_| MidiError::NoPipe)?;
        Ok(Self { pipe })
    }

    /// Send one USB bulk transfer containing complete event packets.
    pub async fn write(&mut self, packets: &[u8]) -> Result<(), MidiError> {
        check_transfer_len(packets.len())?;
        self.pipe.request_out(packets, false).await?;
        Ok(())
    }
}

/// Device-to-host Universal MIDI Packet (UMP / MIDI 2.0) bulk transport.
pub struct UmpInputPipe<'d, A: UsbHostAllocator<'d>> {
    pipe: A::Pipe<pipe::Bulk, pipe::In>,
}

impl<'d, A: UsbHostAllocator<'d>> UmpInputPipe<'d, A> {
    /// Allocate a pipe for a device-to-host MIDI 2.0 endpoint.
    pub fn open(alloc: &A, endpoint: &MidiEndpointDescriptor, enum_info: &EnumerationInfo) -> Result<Self, MidiError> {
        if !endpoint.is_in() {
            return Err(MidiError::WrongDirection);
        }
        let endpoint = EndpointInfo {
            addr: EndpointAddress::from(endpoint.address),
            ep_type: EndpointType::Bulk,
            max_packet_size: endpoint.max_packet_size,
            interval_ms: 0,
        };
        let pipe = alloc
            .alloc_pipe::<pipe::Bulk, pipe::In>(enum_info.device_address, &endpoint, enum_info.split())
            .map_err(|_| MidiError::NoPipe)?;
        Ok(Self { pipe })
    }

    /// Receive one USB bulk transfer containing UMP data.
    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, MidiError> {
        let length = self.pipe.request_in(buffer).await?;
        if length % 4 != 0 {
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

/// Host-to-device Universal MIDI Packet (UMP / MIDI 2.0) bulk transport.
pub struct UmpOutputPipe<'d, A: UsbHostAllocator<'d>> {
    pipe: A::Pipe<pipe::Bulk, pipe::Out>,
}

impl<'d, A: UsbHostAllocator<'d>> UmpOutputPipe<'d, A> {
    /// Allocate a pipe for a host-to-device MIDI 2.0 endpoint.
    pub fn open(alloc: &A, endpoint: &MidiEndpointDescriptor, enum_info: &EnumerationInfo) -> Result<Self, MidiError> {
        if endpoint.is_in() {
            return Err(MidiError::WrongDirection);
        }
        let endpoint = EndpointInfo {
            addr: EndpointAddress::from(endpoint.address),
            ep_type: EndpointType::Bulk,
            max_packet_size: endpoint.max_packet_size,
            interval_ms: 0,
        };
        let pipe = alloc
            .alloc_pipe::<pipe::Bulk, pipe::Out>(enum_info.device_address, &endpoint, enum_info.split())
            .map_err(|_| MidiError::NoPipe)?;
        Ok(Self { pipe })
    }

    /// Send raw UMP bytes over USB bulk.
    pub async fn write(&mut self, bytes: &[u8]) -> Result<(), MidiError> {
        if bytes.len() % 4 != 0 {
            return Err(MidiPacketError::InvalidPacketLength.into());
        }
        self.pipe.request_out(bytes, false).await?;
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

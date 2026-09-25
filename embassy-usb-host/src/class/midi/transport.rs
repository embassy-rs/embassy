//! Low-level USB-MIDI bulk endpoint transports.

use embassy_usb_driver::host::{UsbHostAllocator, UsbPipe, pipe};
use embassy_usb_driver::{EndpointAddress, EndpointInfo, EndpointType};

use super::{EVENT_PACKET_SIZE, MidiEndpointDescriptor, MidiError};
use crate::handler::EnumerationInfo;

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
        check_transfer_buffer(buffer)?;
        let length = self.pipe.request_in(buffer).await?;
        if length % EVENT_PACKET_SIZE != 0 {
            return Err(MidiError::InvalidPacketLength);
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
        check_transfer_buffer(packets)?;
        self.pipe.request_out(packets, false).await?;
        Ok(())
    }
}

fn check_transfer_buffer(buffer: &[u8]) -> Result<(), MidiError> {
    if buffer.is_empty() || !buffer.len().is_multiple_of(EVENT_PACKET_SIZE) {
        return Err(MidiError::InvalidPacketLength);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_one_or_more_complete_event_packets() {
        assert!(check_transfer_buffer(&[0; EVENT_PACKET_SIZE]).is_ok());
        assert!(check_transfer_buffer(&[0; EVENT_PACKET_SIZE * 4]).is_ok());
    }

    #[test]
    fn rejects_empty_and_partial_event_packets() {
        assert!(matches!(
            check_transfer_buffer(&[]),
            Err(MidiError::InvalidPacketLength)
        ));
        assert!(matches!(
            check_transfer_buffer(&[0; EVENT_PACKET_SIZE - 1]),
            Err(MidiError::InvalidPacketLength)
        ));
        assert!(matches!(
            check_transfer_buffer(&[0; EVENT_PACKET_SIZE + 1]),
            Err(MidiError::InvalidPacketLength)
        ));
    }
}

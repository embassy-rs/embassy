//! USB host driver traits and data types.

use crate::EndpointType;

/// Errors returned by [`ChannelOut::write`] and [`ChannelIn::read`]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ChannelError {
    /// Either the packet to be written is too long to fit in the transmission
    /// buffer or the received packet is too long to fit in `buf`.
    BufferOverflow,

    /// The device endpoint is stalled.
    Stall,
}

/// USB endpoint descriptor as defined in the USB 2.0 specification.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EndpointDescriptor {
    /// Length of this descriptor in bytes.
    pub len: u8,
    /// Type of this descriptor. Must be 0x05.
    pub descriptor_type: u8,
    /// Endpoint address.
    pub endpoint_address: u8,
    /// Attributes of this endpoint.
    pub attributes: u8,
    /// Maximum packet size.
    pub max_packet_size: u16,
    /// Polling interval.
    pub interval: u8,
}

impl EndpointDescriptor {
    /// Returns the endpoint type as inferred from the `attributes` field.
    pub fn ep_type(&self) -> EndpointType {
        match self.attributes & 0x03 {
            0 => EndpointType::Control,
            1 => EndpointType::Isochronous,
            2 => EndpointType::Bulk,
            3 => EndpointType::Interrupt,
            _ => unreachable!(),
        }
    }
}

/// Async USB Host Driver trait.
/// To be implemented by the HAL.
pub trait USBHostDriverTrait {
    /// USB Host Channel for an IN Endpoint
    type ChannelIn: ChannelIn;

    /// USB Host Channel for an OUT Endpoint
    type ChannelOut: ChannelOut;

    /// Issue a bus reset.
    async fn bus_reset(&mut self);

    /// Wait for a device to connect.
    async fn wait_for_device_connect(&mut self);

    /// Wait for current device to disconnect.
    async fn wait_for_device_disconnect(&mut self);

    /// Issue a control request out (sending data to device).
    async fn control_request_out(&mut self, bytes: &[u8]) -> Result<(), ()>;

    /// Issue a control request in (receiving data from device).
    async fn control_request_in(&mut self, bytes: &[u8], dest: &mut [u8]) -> Result<usize, ()>;

    /// Reconfigure channel 0 (endpoint 0 in and out)
    fn reconfigure_channel0(&mut self, max_packet_size: u16, dev_addr: u8) -> Result<(), ()>;

    /// Allocate a channel to interact with an IN Endpoint
    fn alloc_channel_in(&mut self, desc: &EndpointDescriptor) -> Result<Self::ChannelIn, ()>;

    /// Allocate a channel to interact with an OUT Endpoint
    fn alloc_channel_out(&mut self, desc: &EndpointDescriptor) -> Result<Self::ChannelOut, ()>;
}

/// USB Host Channel for an IN Endpoint
pub trait ChannelIn {
    /// Attempt to read `buf.len()` bytes from an IN Endpoint.
    /// This reads multiple USB packets if `buf.len()` is larger than the maximum packet size.
    /// Returns the number of bytes read, which may be be less than `buf.len()` if the device responds with non full packet.
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, ChannelError>;
}

/// USB Host Channel for an OUT Endpoint
pub trait ChannelOut {
    /// Write `buf.len()` bytes to an OUT Endpoint.
    /// This writes multiple USB packets if `buf.len()` is larger than the maximum packet size.
    async fn write(&mut self, buf: &[u8]) -> Result<(), ChannelError>;
}

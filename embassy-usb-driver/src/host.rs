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

#[cfg(feature = "defmt")]
defmt::bitflags! {
    /// RequestType bitfields for the setup packet
    pub struct RequestType: u8 {
        // Recipient
        /// The request is intended for the entire device.
        const RECIPIENT_DEVICE    = 0;
        /// The request is intended for an interface.
        const RECIPIENT_INTERFACE = 1;
        /// The request is intended for an endpoint.
        const RECIPIENT_ENDPOINT  = 2;
        /// The recipient of the request is unspecified.
        const RECIPIENT_OTHER     = 3;

        // Type
        /// The request is a standard USB request.
        const TYPE_STANDARD = 0 << 5;
        /// The request is a class-specific request.
        const TYPE_CLASS    = 1 << 5;
        /// The request is a vendor-specific request.
        const TYPE_VENDOR   = 2 << 5;
        /// Reserved.
        const TYPE_RESERVED = 3 << 5;
        // Direction
        /// The request will send data to the device.
        const OUT = 0 << 7;
        /// The request expects to receive data from the device.
        const IN  = 1 << 7;
    }
}

#[cfg(not(feature = "defmt"))]
bitflags::bitflags! {
    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    /// RequestType bitfields for the setup packet
    pub struct RequestType: u8 {
        // Recipient
        /// The request is intended for the entire device.
        const RECIPIENT_DEVICE    = 0;
        /// The request is intended for an interface.
        const RECIPIENT_INTERFACE = 1;
        /// The request is intended for an endpoint.
        const RECIPIENT_ENDPOINT  = 2;
        /// The recipient of the request is unspecified.
        const RECIPIENT_OTHER     = 3;

        // Type
        /// The request is a standard USB request.
        const TYPE_STANDARD = 0 << 5;
        /// The request is a class-specific request.
        const TYPE_CLASS    = 1 << 5;
        /// The request is a vendor-specific request.
        const TYPE_VENDOR   = 2 << 5;
        /// Reserved.
        const TYPE_RESERVED = 3 << 5;
        // Direction
        /// The request will send data to the device.
        const OUT = 0 << 7;
        /// The request expects to receive data from the device.
        const IN  = 1 << 7;
    }
}

/// USB Control Setup Packet
#[repr(C, packed)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct SetupPacket {
    pub request_type: RequestType,
    pub request: u8,
    pub value: u16,
    pub index: u16,
    pub length: u16,
}

impl SetupPacket {
    /// Get a reference to the underlying bytes of the setup packet.
    pub fn as_bytes(&self) -> &[u8] {
        // Safe because we know that the size of SetupPacket is 8 bytes.
        unsafe { core::slice::from_raw_parts(self as *const _ as *const u8, size_of::<Self>()) }
    }
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

    /// Create descriptor for CONTROL endpoint
    pub fn control(addr: u8, max_packet_size: u16) -> Self {
        Self {
            len: 8,
            descriptor_type: 0x05,
            endpoint_address: addr,
            attributes: EndpointType::Control as u8,
            max_packet_size,
            interval: 0,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DeviceEvent {
    Connected,
    Disconnected,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HostError {
    BufferOverflow,
    DeviceDisconnected,
    RequestFailed,
    InvalidDescriptor,
    OutOfSlots,
    OutOfChannels,
    Other(&'static str),
}

/// Async USB Host Driver trait.
/// To be implemented by the HAL.
pub trait UsbHostDriver {    
    type Channel<T: channel::Type, D: channel::Direction>: UsbChannel<T, D>;
    
    /// Wait for device connect or disconnect
    async fn wait_for_device_event(&self) -> DeviceEvent;

    /// Issue a bus reset.
    async fn bus_reset(&self);

    /// Retarget control channel
    fn retarget_channel<D: channel::Direction>(
        &self, 
        channel: &mut Self::Channel<channel::Control, D>,
        addr: u8,
        max_packet_size: u8,
        pre: bool,
    ) -> Result<(), HostError>;

    /// Allocate channel for communication with device
    /// 
    /// `pre` - device is low-speed and communication is going through hub, so send PRE packet
    fn alloc_channel<T: channel::Type, D: channel::Direction>(
        &self,
        addr: u8,
        endpoint: &EndpointDescriptor,
        pre: bool,
    ) -> Result<Self::Channel<T, D>, HostError>;

    /// Drop allocated channel
    fn drop_channel<T: channel::Type, D: channel::Direction>(
        &self, 
        channel: &mut Self::Channel<T, D>
    );
}

/// [UsbChannel] Typelevel structs and traits
// TODO: Seal traits
pub mod channel {
    use super::EndpointType;
    
    pub trait Type { fn ep_type() -> EndpointType;
    }
    pub struct Control {}
    pub struct Interrupt {}
    pub struct Bulk {}
    pub struct Isochronous {}
    impl Type for Control {
        fn ep_type() -> EndpointType {
            EndpointType::Control
        }
    }
    impl Type for Interrupt {
        fn ep_type() -> EndpointType {
            EndpointType::Interrupt
        }
    }
    impl Type for Bulk {
        fn ep_type() -> EndpointType {
            EndpointType::Bulk
        }
    }
    impl Type for Isochronous {
        fn ep_type() -> EndpointType {
            EndpointType::Isochronous
        }
    }
    
    #[diagnostic::on_unimplemented(
        message = "This is not a CONTROL channel",
    )]
    pub trait IsControl {}
    impl IsControl for Control {}
    
    pub trait Direction {
        fn is_in() -> bool;
        fn is_out() -> bool;
    }
    pub struct In {}
    pub struct Out {}
    pub struct InOut {}
    impl Direction for In {
        fn is_in() -> bool { true }
        fn is_out() -> bool { false }
    }
    impl Direction for Out {
        fn is_in() -> bool { false }
        fn is_out() -> bool { true }
    }
    impl Direction for InOut {
        fn is_in() -> bool { true }
        fn is_out() -> bool { true }
    }
    
    #[diagnostic::on_unimplemented(
        message = "This is not an IN channel",
    )]
    pub trait IsIn {}
    impl IsIn for In {}
    impl IsIn for InOut {}
    
    #[diagnostic::on_unimplemented(
        message = "This is not an OUT channel",
    )]
    pub trait IsOut {}
    impl IsOut for Out {}
    impl IsOut for InOut {}
}

pub trait UsbChannel<T: channel::Type, D: channel::Direction> {
    /// Send IN control request
    async fn control_in(&mut self, setup: &SetupPacket, buf: &mut [u8]) -> Result<usize, HostError>
    where 
        T: channel::IsControl,
        D: channel::IsIn {
        unimplemented!()
    }
        
    /// Send OUT control request
    async fn control_out(&mut self, setup: &SetupPacket, buf: &[u8]) -> Result<usize, HostError>
    where 
        T: channel::IsControl,
        D: channel::IsOut {
        unimplemented!()
    }

    /// Send IN request of type other from control
    async fn request_in(&mut self, buf: &mut [u8]) -> Result<usize, HostError>
    where 
        D: channel::IsIn {
        unimplemented!()
    }

    /// Send OUT request of type other from control
    async fn request_out(&mut self, buf: &[u8]) -> Result<usize, HostError>
    where 
        D: channel::IsOut {
        unimplemented!()
    }
}

/// Convenience impl for combined inout channel
impl<T, I, O> UsbChannel<T, channel::InOut> for (O, I)
where
    T: channel::Type,
    I: UsbChannel<T, channel::In>,
    O: UsbChannel<T, channel::Out> 
{}

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

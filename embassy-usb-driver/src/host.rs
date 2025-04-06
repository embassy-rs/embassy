//! USB host driver traits and data types.

use core::time::Duration;

use crate::{EndpointInfo, EndpointType, Speed};

/// Errors returned by [`ChannelOut::write`] and [`ChannelIn::read`]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ChannelError {
    /// Either the packet to be written is too long to fit in the transmission
    /// buffer or the received packet is too long to fit in `buf`.
    BufferOverflow,

    /// Response from device/bus was not interpretable (Crc, Babble)
    BadResponse,

    /// Transaction was canceled
    Canceled,

    /// The device endpoint is stalled.
    Stall,

    /// Device did not respond in time
    Timeout,

    /// Device disconnected
    Disconnected,
}

macro_rules! bitflags {
    ($($tt:tt)*) => {
        #[cfg(feature = "defmt")]
        defmt::bitflags! { $($tt)* }
        #[cfg(not(feature = "defmt"))]
        bitflags::bitflags! { $($tt)* }
    };
}

bitflags! {
    #[cfg_attr(not(feature = "defmt"), derive(Copy, Clone, Eq, PartialEq, Debug))]
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
#[repr(C)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
        unsafe { core::slice::from_raw_parts(self as *const _ as *const u8, core::mem::size_of::<Self>()) }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DeviceEvent {
    /// Indicates a root-device has become attached
    Connected(Speed),
    Disconnected,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HostError {
    ChannelError(ChannelError),
    RequestFailed,
    InvalidDescriptor,
    OutOfSlots,
    OutOfChannels,
    NoSuchDevice,
    InsufficientMemory,
    Other(&'static str),
}

impl From<ChannelError> for HostError {
    fn from(value: ChannelError) -> Self {
        HostError::ChannelError(value)
    }
}

/// Async USB Host Driver trait.
/// To be implemented by the HAL.
pub trait UsbHostDriver: Sized {
    /// Channel implementation of this UsbHostDriver
    type Channel<T: channel::Type, D: channel::Direction>: UsbChannel<T, D>;

    /// Wait for device connect or disconnect
    ///
    /// When connected, this function must issue a bus reset before the speed is reported
    async fn wait_for_device_event(&self) -> DeviceEvent;

    /// Issue a bus reset.
    async fn bus_reset(&self);

    /// Allocate channel for communication with device
    ///
    /// This can be a scarce resource, for one-off requests please scope the channel so it's dropped after completion
    ///
    /// `pre` - device is low-speed and communication is going through hub, so send PRE packet
    fn alloc_channel<T: channel::Type, D: channel::Direction>(
        &self,
        addr: u8,
        endpoint: &EndpointInfo,
        pre: bool,
    ) -> Result<Self::Channel<T, D>, HostError>;

    // Drop happens implicitly on channel-side
    // / Drop allocated channel
    // fn drop_channel<T: channel::Type, D: channel::Direction>(&self, channel: &mut Self::Channel<T, D>);
}

/// [UsbChannel] Typelevel structs and traits
// TODO: Seal traits
pub mod channel {
    use super::EndpointType;

    pub trait Type {
        fn ep_type() -> EndpointType;
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

    #[diagnostic::on_unimplemented(message = "This is not a CONTROL channel")]
    pub trait IsControl {}
    impl IsControl for Control {}

    #[diagnostic::on_unimplemented(message = "This is not a CONTROL channel")]
    pub trait IsInterrupt {}
    impl IsInterrupt for Interrupt {}

    pub trait Direction {
        fn is_in() -> bool;
        fn is_out() -> bool;
    }
    pub struct In {}
    pub struct Out {}
    pub struct InOut {}
    impl Direction for In {
        fn is_in() -> bool {
            true
        }
        fn is_out() -> bool {
            false
        }
    }
    impl Direction for Out {
        fn is_in() -> bool {
            false
        }
        fn is_out() -> bool {
            true
        }
    }
    impl Direction for InOut {
        fn is_in() -> bool {
            true
        }
        fn is_out() -> bool {
            true
        }
    }

    #[diagnostic::on_unimplemented(message = "This is not an IN channel")]
    pub trait IsIn: Direction {}
    impl IsIn for In {}
    impl IsIn for InOut {}

    #[diagnostic::on_unimplemented(message = "This is not an OUT channel")]
    pub trait IsOut: Direction {}
    impl IsOut for Out {}
    impl IsOut for InOut {}
}

/// Specify the timeout of a channel
pub struct TimeoutConfig {
    /// Maximum response timeout for transactions with a Data Stage
    pub data_timeout: Duration,
    /// Maximum response timeout for transactions without a data stage
    pub standard_timeout: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        TimeoutConfig {
            data_timeout: Duration::from_millis(500),
            standard_timeout: Duration::from_millis(50),
        }
    }
}

/// ## Virtual USB Channels
/// These contain the required information to send a packet correctly to a device endpoint.
/// The information is carried with the channel on creation (see [`UsbHostDriver::alloc_channel`]) and can be changed with [`UsbChannel::retarget_channel`].
///
/// It is up to the hal's driver how to implement concurrent requests, some hardware IP may allow for multiple hardware channels
///  while others may only have a single channel which needs to be multiplexed in software, while others still use DMA request linked-lists.
/// Any of these are compatibile with the UsbChannel with varying degrees of sync primitives required.
pub trait UsbChannel<T: channel::Type, D: channel::Direction> {
    /// Send IN control request
    async fn control_in(&mut self, setup: &SetupPacket, buf: &mut [u8]) -> Result<usize, ChannelError>
    where
        T: channel::IsControl,
        D: channel::IsIn;

    /// Send OUT control request
    async fn control_out(&mut self, setup: &SetupPacket, buf: &[u8]) -> Result<usize, ChannelError>
    where
        T: channel::IsControl,
        D: channel::IsOut;

    /// Retargets channel to a new endpoint, may error if the underlying driver runs out of resources
    fn retarget_channel(&mut self, addr: u8, endpoint: &EndpointInfo, pre: bool) -> Result<(), HostError>;

    /// Send IN request of type other from control
    /// For interrupt channels this will return the result of the next succesful interrupt poll
    async fn request_in(&mut self, buf: &mut [u8]) -> Result<usize, ChannelError>
    where
        D: channel::IsIn;

    /// Send OUT request of type other from control
    async fn request_out(&mut self, buf: &[u8]) -> Result<usize, ChannelError>
    where
        D: channel::IsOut;

    /// Configure the timeouts of this channel
    async fn set_timeout(&mut self, timeout: TimeoutConfig);
}

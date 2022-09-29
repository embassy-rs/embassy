#![no_std]

use core::future::Future;

/// Direction of USB traffic. Note that in the USB standard the direction is always indicated from
/// the perspective of the host, which is backward for devices, but the standard directions are used
/// for consistency.
///
/// The values of the enum also match the direction bit used in endpoint addresses and control
/// request types.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Direction {
    /// Host to device (OUT)
    Out = 0x00,
    /// Device to host (IN)
    In = 0x80,
}

/// USB endpoint transfer type. The values of this enum can be directly cast into `u8` to get the
/// transfer bmAttributes transfer type bits.
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EndpointType {
    /// Control endpoint. Used for device management. Only the host can initiate requests. Usually
    /// used only endpoint 0.
    Control = 0b00,
    /// Isochronous endpoint. Used for time-critical unreliable data. Not implemented yet.
    Isochronous = 0b01,
    /// Bulk endpoint. Used for large amounts of best-effort reliable data.
    Bulk = 0b10,
    /// Interrupt endpoint. Used for small amounts of time-critical reliable data.
    Interrupt = 0b11,
}

/// Type-safe endpoint address.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EndpointAddress(u8);

impl From<u8> for EndpointAddress {
    #[inline]
    fn from(addr: u8) -> EndpointAddress {
        EndpointAddress(addr)
    }
}

impl From<EndpointAddress> for u8 {
    #[inline]
    fn from(addr: EndpointAddress) -> u8 {
        addr.0
    }
}

impl EndpointAddress {
    const INBITS: u8 = Direction::In as u8;

    /// Constructs a new EndpointAddress with the given index and direction.
    #[inline]
    pub fn from_parts(index: usize, dir: Direction) -> Self {
        EndpointAddress(index as u8 | dir as u8)
    }

    /// Gets the direction part of the address.
    #[inline]
    pub fn direction(&self) -> Direction {
        if (self.0 & Self::INBITS) != 0 {
            Direction::In
        } else {
            Direction::Out
        }
    }

    /// Returns true if the direction is IN, otherwise false.
    #[inline]
    pub fn is_in(&self) -> bool {
        (self.0 & Self::INBITS) != 0
    }

    /// Returns true if the direction is OUT, otherwise false.
    #[inline]
    pub fn is_out(&self) -> bool {
        (self.0 & Self::INBITS) == 0
    }

    /// Gets the index part of the endpoint address.
    #[inline]
    pub fn index(&self) -> usize {
        (self.0 & !Self::INBITS) as usize
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EndpointInfo {
    pub addr: EndpointAddress,
    pub ep_type: EndpointType,
    pub max_packet_size: u16,
    pub interval: u8,
}

/// Driver for a specific USB peripheral. Implement this to add support for a new hardware
/// platform.
pub trait Driver<'a> {
    type EndpointOut: EndpointOut + 'a;
    type EndpointIn: EndpointIn + 'a;
    type ControlPipe: ControlPipe + 'a;
    type Bus: Bus + 'a;

    /// Allocates an endpoint and specified endpoint parameters. This method is called by the device
    /// and class implementations to allocate endpoints, and can only be called before
    /// [`start`](Self::start) is called.
    ///
    /// # Arguments
    ///
    /// * `ep_addr` - A static endpoint address to allocate. If Some, the implementation should
    ///   attempt to return an endpoint with the specified address. If None, the implementation
    ///   should return the next available one.
    /// * `max_packet_size` - Maximum packet size in bytes.
    /// * `interval` - Polling interval parameter for interrupt endpoints.
    fn alloc_endpoint_out(
        &mut self,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval: u8,
    ) -> Result<Self::EndpointOut, EndpointAllocError>;

    fn alloc_endpoint_in(
        &mut self,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval: u8,
    ) -> Result<Self::EndpointIn, EndpointAllocError>;

    /// Start operation of the USB device.
    ///
    /// This returns the `Bus` and `ControlPipe` instances that are used to operate
    /// the USB device. Additionally, this makes all the previously allocated endpoints
    /// start operating.
    ///
    /// This consumes the `Driver` instance, so it's no longer possible to allocate more
    /// endpoints.
    fn start(self, control_max_packet_size: u16) -> (Self::Bus, Self::ControlPipe);

    /// Indicates that `set_device_address` must be called before accepting the corresponding
    /// control transfer, not after.
    ///
    /// The default value for this constant is `false`, which corresponds to the USB 2.0 spec, 9.4.6
    const QUIRK_SET_ADDRESS_BEFORE_STATUS: bool = false;
}

pub trait Bus {
    type EnableFuture<'a>: Future<Output = ()> + 'a
    where
        Self: 'a;
    type DisableFuture<'a>: Future<Output = ()> + 'a
    where
        Self: 'a;
    type PollFuture<'a>: Future<Output = Event> + 'a
    where
        Self: 'a;
    type RemoteWakeupFuture<'a>: Future<Output = Result<(), Unsupported>> + 'a
    where
        Self: 'a;

    /// Enables the USB peripheral. Soon after enabling the device will be reset, so
    /// there is no need to perform a USB reset in this method.
    fn enable(&mut self) -> Self::EnableFuture<'_>;

    /// Disables and powers down the USB peripheral.
    fn disable(&mut self) -> Self::DisableFuture<'_>;

    fn poll<'a>(&'a mut self) -> Self::PollFuture<'a>;

    /// Sets the device USB address to `addr`.
    fn set_address(&mut self, addr: u8);

    /// Enables or disables an endpoint.
    fn endpoint_set_enabled(&mut self, ep_addr: EndpointAddress, enabled: bool);

    /// Sets or clears the STALL condition for an endpoint. If the endpoint is an OUT endpoint, it
    /// should be prepared to receive data again. Only used during control transfers.
    fn endpoint_set_stalled(&mut self, ep_addr: EndpointAddress, stalled: bool);

    /// Gets whether the STALL condition is set for an endpoint. Only used during control transfers.
    fn endpoint_is_stalled(&mut self, ep_addr: EndpointAddress) -> bool;

    /// Simulates a disconnect from the USB bus, causing the host to reset and re-enumerate the
    /// device.
    ///
    /// The default implementation just returns `Unsupported`.
    ///
    /// # Errors
    ///
    /// * [`Unsupported`](crate::driver::Unsupported) - This UsbBus implementation doesn't support
    ///   simulating a disconnect or it has not been enabled at creation time.
    fn force_reset(&mut self) -> Result<(), Unsupported> {
        Err(Unsupported)
    }

    /// Initiates a remote wakeup of the host by the device.
    ///
    /// # Errors
    ///
    /// * [`Unsupported`](crate::driver::Unsupported) - This UsbBus implementation doesn't support
    ///   remote wakeup or it has not been enabled at creation time.
    fn remote_wakeup(&mut self) -> Self::RemoteWakeupFuture<'_>;
}

pub trait Endpoint {
    type WaitEnabledFuture<'a>: Future<Output = ()> + 'a
    where
        Self: 'a;

    /// Get the endpoint address
    fn info(&self) -> &EndpointInfo;

    /// Waits for the endpoint to be enabled.
    fn wait_enabled(&mut self) -> Self::WaitEnabledFuture<'_>;
}

pub trait EndpointOut: Endpoint {
    type ReadFuture<'a>: Future<Output = Result<usize, EndpointError>> + 'a
    where
        Self: 'a;

    /// Reads a single packet of data from the endpoint, and returns the actual length of
    /// the packet.
    ///
    /// This should also clear any NAK flags and prepare the endpoint to receive the next packet.
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a>;
}

pub trait ControlPipe {
    type SetupFuture<'a>: Future<Output = [u8; 8]> + 'a
    where
        Self: 'a;
    type DataOutFuture<'a>: Future<Output = Result<usize, EndpointError>> + 'a
    where
        Self: 'a;
    type DataInFuture<'a>: Future<Output = Result<(), EndpointError>> + 'a
    where
        Self: 'a;
    type AcceptFuture<'a>: Future<Output = ()> + 'a
    where
        Self: 'a;
    type RejectFuture<'a>: Future<Output = ()> + 'a
    where
        Self: 'a;

    /// Maximum packet size for the control pipe
    fn max_packet_size(&self) -> usize;

    /// Reads a single setup packet from the endpoint.
    fn setup<'a>(&'a mut self) -> Self::SetupFuture<'a>;

    /// Reads a DATA OUT packet into `buf` in response to a control write request.
    ///
    /// Must be called after `setup()` for requests with `direction` of `Out`
    /// and `length` greater than zero.
    fn data_out<'a>(&'a mut self, buf: &'a mut [u8], first: bool, last: bool) -> Self::DataOutFuture<'a>;

    /// Sends a DATA IN packet with `data` in response to a control read request.
    ///
    /// If `last_packet` is true, the STATUS packet will be ACKed following the transfer of `data`.
    fn data_in<'a>(&'a mut self, data: &'a [u8], first: bool, last: bool) -> Self::DataInFuture<'a>;

    /// Accepts a control request.
    ///
    /// Causes the STATUS packet for the current request to be ACKed.
    fn accept<'a>(&'a mut self) -> Self::AcceptFuture<'a>;

    /// Rejects a control request.
    ///
    /// Sets a STALL condition on the pipe to indicate an error.
    fn reject<'a>(&'a mut self) -> Self::RejectFuture<'a>;
}

pub trait EndpointIn: Endpoint {
    type WriteFuture<'a>: Future<Output = Result<(), EndpointError>> + 'a
    where
        Self: 'a;

    /// Writes a single packet of data to the endpoint.
    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a>;
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Event returned by [`Bus::poll`].
pub enum Event {
    /// The USB reset condition has been detected.
    Reset,

    /// A USB suspend request has been detected or, in the case of self-powered devices, the device
    /// has been disconnected from the USB bus.
    Suspend,

    /// A USB resume request has been detected after being suspended or, in the case of self-powered
    /// devices, the device has been connected to the USB bus.
    Resume,

    /// The USB power has been detected.
    PowerDetected,

    /// The USB power has been removed. Not supported by all devices.
    PowerRemoved,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EndpointAllocError;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Operation is unsupported by the driver.
pub struct Unsupported;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Errors returned by [`EndpointIn::write`] and [`EndpointOut::read`]
pub enum EndpointError {
    /// Either the packet to be written is too long to fit in the transmission
    /// buffer or the received packet is too long to fit in `buf`.
    BufferOverflow,

    /// The endpoint is disabled.
    Disabled,
}

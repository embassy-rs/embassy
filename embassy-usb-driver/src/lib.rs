#![no_std]
#![allow(async_fn_in_trait)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

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
    Out,
    /// Device to host (IN)
    In,
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
    const INBITS: u8 = 0x80;

    /// Constructs a new EndpointAddress with the given index and direction.
    #[inline]
    pub fn from_parts(index: usize, dir: Direction) -> Self {
        let dir_u8 = match dir {
            Direction::Out => 0x00,
            Direction::In => Self::INBITS,
        };
        EndpointAddress(index as u8 | dir_u8)
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

/// Information for an endpoint.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EndpointInfo {
    /// Endpoint's address.
    pub addr: EndpointAddress,
    /// Endpoint's type.
    pub ep_type: EndpointType,
    /// Max packet size, in bytes.
    pub max_packet_size: u16,
    /// Polling interval, in milliseconds.
    pub interval_ms: u8,
}

/// Main USB driver trait.
///
/// Implement this to add support for a new hardware platform.
pub trait Driver<'a> {
    /// Type of the OUT endpoints for this driver.
    type EndpointOut: EndpointOut + 'a;
    /// Type of the IN endpoints for this driver.
    type EndpointIn: EndpointIn + 'a;
    /// Type of the control pipe for this driver.
    type ControlPipe: ControlPipe + 'a;
    /// Type for bus control for this driver.
    type Bus: Bus + 'a;

    /// Allocates an OUT endpoint.
    ///
    /// This method is called by the USB stack to allocate endpoints.
    /// It can only be called before [`start`](Self::start) is called.
    ///
    /// # Arguments
    ///
    /// * `ep_type` - the endpoint's type.
    /// * `max_packet_size` - Maximum packet size in bytes.
    /// * `interval_ms` - Polling interval parameter for interrupt endpoints.
    fn alloc_endpoint_out(
        &mut self,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval_ms: u8,
    ) -> Result<Self::EndpointOut, EndpointAllocError>;

    /// Allocates an IN endpoint.
    ///
    /// This method is called by the USB stack to allocate endpoints.
    /// It can only be called before [`start`](Self::start) is called.
    ///
    /// # Arguments
    ///
    /// * `ep_type` - the endpoint's type.
    /// * `max_packet_size` - Maximum packet size in bytes.
    /// * `interval_ms` - Polling interval parameter for interrupt endpoints.
    fn alloc_endpoint_in(
        &mut self,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval_ms: u8,
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
}

/// USB bus trait.
///
/// This trait provides methods that act on the whole bus. It is kept owned by
/// the main USB task, and used to manage the bus.
pub trait Bus {
    /// Enable the USB peripheral.
    async fn enable(&mut self);

    /// Disable and powers down the USB peripheral.
    async fn disable(&mut self);

    /// Wait for a bus-related event.
    ///
    /// This method should asynchronously wait for an event to happen, then
    /// return it. See [`Event`] for the list of events this method should return.
    async fn poll(&mut self) -> Event;

    /// Enable or disable an endpoint.
    fn endpoint_set_enabled(&mut self, ep_addr: EndpointAddress, enabled: bool);

    /// Set or clear the STALL condition for an endpoint.
    ///
    /// If the endpoint is an OUT endpoint, it should be prepared to receive data again.
    fn endpoint_set_stalled(&mut self, ep_addr: EndpointAddress, stalled: bool);

    /// Get whether the STALL condition is set for an endpoint.
    fn endpoint_is_stalled(&mut self, ep_addr: EndpointAddress) -> bool;

    /// Simulate a disconnect from the USB bus, causing the host to reset and re-enumerate the
    /// device.
    ///
    /// The default implementation just returns `Unsupported`.
    ///
    /// # Errors
    ///
    /// * [`Unsupported`](crate::Unsupported) - This UsbBus implementation doesn't support
    ///   simulating a disconnect or it has not been enabled at creation time.
    fn force_reset(&mut self) -> Result<(), Unsupported> {
        Err(Unsupported)
    }

    /// Initiate a remote wakeup of the host by the device.
    ///
    /// # Errors
    ///
    /// * [`Unsupported`](crate::Unsupported) - This UsbBus implementation doesn't support
    ///   remote wakeup or it has not been enabled at creation time.
    async fn remote_wakeup(&mut self) -> Result<(), Unsupported>;
}

/// Endpoint trait, common for OUT and IN.
pub trait Endpoint {
    /// Get the endpoint address
    fn info(&self) -> &EndpointInfo;

    /// Wait for the endpoint to be enabled.
    async fn wait_enabled(&mut self);
}

/// OUT Endpoint trait.
pub trait EndpointOut: Endpoint {
    /// Read a single packet of data from the endpoint, and return the actual length of
    /// the packet.
    ///
    /// This should also clear any NAK flags and prepare the endpoint to receive the next packet.
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError>;
}

/// USB control pipe trait.
///
/// The USB control pipe owns both OUT endpoint 0 and IN endpoint 0 in a single
/// unit, and manages them together to implement the control pipe state machine.
///
/// The reason this is a separate trait instead of using EndpointOut/EndpointIn is that
/// many USB peripherals treat the control pipe endpoints differently (different registers,
/// different procedures), usually to accelerate processing in hardware somehow. A separate
/// trait allows the driver to handle it specially.
///
/// The call sequences made by the USB stack to the ControlPipe are the following:
///
/// - control in/out with len=0:
///
/// ```not_rust
/// setup()
/// (...processing...)
/// accept() or reject()
/// ```
///
/// - control out for setting the device address:
///
/// ```not_rust
/// setup()
/// (...processing...)
/// accept_set_address(addr) or reject()
/// ```
///
/// - control out with len != 0:
///
/// ```not_rust
/// setup()
/// data_out(first=true, last=false)
/// data_out(first=false, last=false)
/// ...
/// data_out(first=false, last=false)
/// data_out(first=false, last=true)
/// (...processing...)
/// accept() or reject()
/// ```
///
/// - control in with len != 0, accepted:
///
/// ```not_rust
/// setup()
/// (...processing...)
/// data_in(first=true, last=false)
/// data_in(first=false, last=false)
/// ...
/// data_in(first=false, last=false)
/// data_in(first=false, last=true)
/// (NO `accept()`!!! This is because calling `data_in` already implies acceptance.)
/// ```
///
/// - control in with len != 0, rejected:
///
/// ```not_rust
/// setup()
/// (...processing...)
/// reject()
/// ```
///
/// The driver is responsible for handling the status stage. The stack DOES NOT do zero-length
/// calls to `data_in` or `data_out` for the status zero-length packet. The status stage should
/// be triggered by either `accept()`, or `data_in` with `last = true`.
///
/// Note that the host can abandon a control request and send a new SETUP packet any time. If
/// a SETUP packet arrives at any time during `data_out`, `data_in`, `accept` or `reject`,
/// the driver must immediately return (with `EndpointError::Disabled` from `data_in`, `data_out`)
/// to let the stack call `setup()` again to start handling the new control request. Not doing
/// so will cause things to get stuck, because the host will never read/send the packet we're
/// waiting for.
pub trait ControlPipe {
    /// Maximum packet size for the control pipe
    fn max_packet_size(&self) -> usize;

    /// Read a single setup packet from the endpoint.
    async fn setup(&mut self) -> [u8; 8];

    /// Read a DATA OUT packet into `buf` in response to a control write request.
    ///
    /// Must be called after `setup()` for requests with `direction` of `Out`
    /// and `length` greater than zero.
    async fn data_out(&mut self, buf: &mut [u8], first: bool, last: bool) -> Result<usize, EndpointError>;

    /// Send a DATA IN packet with `data` in response to a control read request.
    ///
    /// If `last_packet` is true, the STATUS packet will be ACKed following the transfer of `data`.
    async fn data_in(&mut self, data: &[u8], first: bool, last: bool) -> Result<(), EndpointError>;

    /// Accept a control request.
    ///
    /// Causes the STATUS packet for the current request to be ACKed.
    async fn accept(&mut self);

    /// Reject a control request.
    ///
    /// Sets a STALL condition on the pipe to indicate an error.
    async fn reject(&mut self);

    /// Accept SET_ADDRESS control and change bus address.
    ///
    /// For most drivers this function should firstly call `accept()` and then change the bus address.
    /// However, there are peripherals (Synopsys USB OTG) that have reverse order.
    async fn accept_set_address(&mut self, addr: u8);
}

/// IN Endpoint trait.
pub trait EndpointIn: Endpoint {
    /// Write a single packet of data to the endpoint.
    async fn write(&mut self, buf: &[u8]) -> Result<(), EndpointError>;
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

/// Allocating an endpoint failed.
///
/// This can be due to running out of endpoints, or out of endpoint memory,
/// or because the hardware doesn't support the requested combination of features.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EndpointAllocError;

/// Operation is unsupported by the driver.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Unsupported;

/// Errors returned by [`EndpointIn::write`] and [`EndpointOut::read`]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EndpointError {
    /// Either the packet to be written is too long to fit in the transmission
    /// buffer or the received packet is too long to fit in `buf`.
    BufferOverflow,

    /// The endpoint is disabled.
    Disabled,
}

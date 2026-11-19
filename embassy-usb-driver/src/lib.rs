#![no_std]
#![allow(async_fn_in_trait)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

pub mod host;
mod interval;

pub use interval::EndpointInterval;

/// Speed of a device or port
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Speed {
    /// 1.5 Mbit/s
    Low,
    /// 12 Mbit/s
    Full,
    /// 480 Mbit/s
    High,
    /// 5 Gbit/s
    Super,
    /// 10 Gbit/s
    SuperPlus,
}

impl Speed {
    /// Provides the default max_packet_size for a given port speed
    pub const fn max_packet_size(self) -> u16 {
        match self {
            Speed::Low => 8,
            Speed::Full => 64,
            Speed::High => 64, // USB 2.0 spec requires 64 for high-speed control EP0
            Speed::Super | Speed::SuperPlus => 512, // USB 3.0 spec requires 512 for SuperSpeed control EP0
        }
    }
}

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
    /// Isochronous endpoint. Used for time-critical unreliable data.
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
    pub const fn from_parts(index: usize, dir: Direction) -> Self {
        let dir_u8 = match dir {
            Direction::Out => 0x00,
            Direction::In => Self::INBITS,
        };
        EndpointAddress(index as u8 | dir_u8)
    }

    /// Gets the direction part of the address.
    #[inline]
    pub const fn direction(&self) -> Direction {
        if (self.0 & Self::INBITS) != 0 {
            Direction::In
        } else {
            Direction::Out
        }
    }

    /// Returns true if the direction is IN, otherwise false.
    #[inline]
    pub const fn is_in(&self) -> bool {
        (self.0 & Self::INBITS) != 0
    }

    /// Returns true if the direction is OUT, otherwise false.
    #[inline]
    pub const fn is_out(&self) -> bool {
        (self.0 & Self::INBITS) == 0
    }

    /// Gets the index part of the endpoint address.
    #[inline]
    pub const fn index(&self) -> usize {
        (self.0 & !Self::INBITS) as usize
    }
}

/// Information for an endpoint.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct EndpointInfo {
    /// Endpoint's address.
    pub addr: EndpointAddress,
    /// Endpoint's type.
    pub ep_type: EndpointType,
    /// Max packet size per transaction, in bytes (0..=1024).
    pub max_packet_size: u16,
    /// Polling interval.
    pub interval: EndpointInterval,
    /// SuperSpeed `bMaxBurst`: the endpoint bursts up to `max_burst + 1` packets. 0 below
    /// SuperSpeed.
    pub max_burst: u8,
    /// High-speed additional transaction opportunities per microframe (`wMaxPacketSize` bits
    /// 12..11, `0..=2`) for isochronous and interrupt endpoints. 0 otherwise.
    pub additional_transactions: u8,
}

impl EndpointInfo {
    /// An endpoint with `max_burst` and `additional_transactions` 0.
    pub const fn new(
        addr: EndpointAddress,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval: EndpointInterval,
    ) -> Self {
        Self {
            addr,
            ep_type,
            max_packet_size,
            interval,
            max_burst: 0,
            additional_transactions: 0,
        }
    }
}

/// Configuration to allocate an endpoint with.
///
/// The driver must allocate an endpoint with exactly the given `addr` (if any), `ep_type`,
/// `max_packet_size` and `interval`, or fail. The docs of the other fields say whether the
/// driver may deviate from them.
///
/// The SuperSpeed fields are those of the endpoint's SuperSpeed Endpoint Companion descriptor, and 0
/// for an endpoint that has none.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct EndpointConfig {
    /// Endpoint's address, or `None` for the driver to pick one.
    pub addr: Option<EndpointAddress>,
    /// Endpoint's type.
    pub ep_type: EndpointType,
    /// Max packet size per transaction, in bytes (0..=1024).
    pub max_packet_size: u16,
    /// Polling interval.
    pub interval: EndpointInterval,
    /// Number of packets the hardware may buffer for the endpoint.
    ///
    /// More buffers let the hardware move the next packet while software is still busy with the
    /// previous one, at the cost of more packet memory.
    ///
    /// This is just a best-effort hint for the driver. Drivers may use a lower value
    /// if there is not enough packet memory, or completely ignore it.
    pub buffers: u8,
    /// SuperSpeed `bMaxBurst`: the endpoint bursts up to `max_burst + 1` packets.
    ///
    /// Drivers may use a lower value, and report it in [`EndpointInfo::max_burst`].
    pub max_burst: u8,
    /// SuperSpeed bulk `MaxStreams`: the endpoint has `2^max_streams` streams, or none when 0.
    pub max_streams: u8,
    /// SuperSpeed isochronous `Mult`: the endpoint moves up to `mult + 1` bursts per service
    /// interval.
    pub mult: u8,
    /// SuperSpeed periodic `wBytesPerInterval`, or the `dwBytesPerInterval` of a SuperSpeedPlus
    /// isochronous endpoint: the bytes the endpoint moves per service interval.
    pub bytes_per_interval: u32,
    /// High-speed additional transaction opportunities per microframe (`wMaxPacketSize` bits
    /// 12..11, `0..=2`) for isochronous and interrupt endpoints.
    ///
    /// Drivers that do not support multi-transaction periodic endpoints may fail with
    /// [`EndpointAllocError`] or use 0, and report the allocated value in
    /// [`EndpointInfo::additional_transactions`].
    pub additional_transactions: u8,
}

impl EndpointConfig {
    /// An endpoint with one buffer and all SuperSpeed/high-bandwidth fields 0.
    pub const fn new(
        addr: Option<EndpointAddress>,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval: EndpointInterval,
    ) -> Self {
        Self {
            addr,
            ep_type,
            max_packet_size,
            interval,
            buffers: 1,
            max_burst: 0,
            max_streams: 0,
            mult: 0,
            bytes_per_interval: 0,
            additional_transactions: 0,
        }
    }
}

/// Main USB device driver trait.
///
/// Implement this to add support for a new hardware platform.
pub trait Driver<'a> {
    /// Type of the OUT endpoints for this driver.
    type EndpointOut: EndpointOut + 'a;
    /// Type of the IN endpoints for this driver.
    type EndpointIn: EndpointIn + 'a;
    /// Type for bus control for this driver.
    type Bus: Bus + 'a;

    /// Allocates an OUT endpoint.
    ///
    /// This method is called by the USB stack to allocate endpoints.
    /// It can only be called before [`start`](Self::start) is called.
    fn alloc_endpoint_out(&mut self, config: &EndpointConfig) -> Result<Self::EndpointOut, EndpointAllocError>;

    /// Allocates an IN endpoint.
    ///
    /// This method is called by the USB stack to allocate endpoints.
    /// It can only be called before [`start`](Self::start) is called.
    fn alloc_endpoint_in(&mut self, config: &EndpointConfig) -> Result<Self::EndpointIn, EndpointAllocError>;

    /// Start operation of the USB device.
    ///
    /// This returns the `Bus` instance that is used to operate the USB device, including its
    /// control pipe. Additionally, this makes all the previously allocated endpoints
    /// start operating.
    ///
    /// This consumes the `Driver` instance, so it's no longer possible to allocate more
    /// endpoints.
    fn start(self, control_max_packet_size: u16) -> Self::Bus;
}

/// USB bus trait.
///
/// This trait provides methods that act on the whole bus, and on the control pipe. It is kept
/// owned by the main USB task, and used to manage the bus.
///
/// # Control pipe
///
/// The control pipe owns both OUT endpoint 0 and IN endpoint 0 in a single unit, and manages them
/// together to implement the control pipe state machine. It is part of `Bus` instead of
/// [`Endpoint`]/[`EndpointIn`]/[`EndpointOut`] because many USB peripherals treat the control
/// endpoints differently (different registers, different procedures), usually to accelerate
/// processing in hardware somehow.
///
/// A control request starts with [`Event::Setup`] from [`poll`](Self::poll). The stack then
/// calls `control_data_out`/`control_data_in` for the data stage, if any, and finishes the
/// request with `control_accept` or `control_reject`. The driver is responsible for handling the
/// status stage. The stack DOES NOT do zero-length calls to `control_data_in` or
/// `control_data_out` for the status zero-length packet. The status stage should be triggered by
/// either `control_accept()`, or following `control_data_in`.
///
/// ## Aborting a control transfer
///
/// The stack does not call [`poll`](Self::poll) while it is in the middle of a control request,
/// so it cannot notice by itself that something happened to the bus. Therefore, if the transfer
/// can no longer complete, `control_data_out`, `control_data_in`, `control_accept`, `control_reject` and
/// `control_accept_set_address` must **immediately** return, with [`EndpointError::Disabled`] or
/// [`EndpointDisabledError`] where they have an error to return. This is the case when:
///
/// * the bus is reset ([`Event::Reset`]),
/// * the bus is suspended ([`Event::Suspend`]), or power is removed ([`Event::PowerRemoved`]) in
///   drivers that detect it,
/// * the host abandoned the request and sent a new SETUP packet.
///
/// Returning early must not consume the event: the next call to [`poll`](Self::poll) must still
/// return it (for a new SETUP packet, [`Event::Setup`]). Not doing so will cause things to get
/// stuck, because the stack never gets to see the event, and the host will never read/send the
/// packet we're waiting for.
pub trait Bus {
    /// Enable the USB peripheral.
    async fn enable(&mut self);

    /// Disable and powers down the USB peripheral.
    async fn disable(&mut self);

    /// Wait for a bus-related event.
    ///
    /// This method should asynchronously wait for an event to happen, then
    /// return it. See [`Event`] for the list of events this method should return.
    ///
    /// If a bus event and a SETUP packet are both pending, the bus event must be returned first,
    /// because a reset or a suspend makes a SETUP packet received before it stale.
    ///
    /// This future must be cancel-safe: dropping it must not lose an event.
    async fn poll(&mut self) -> Event;

    /// Maximum packet size for the control pipe.
    fn control_max_packet_size(&self) -> usize;

    /// Read a DATA OUT transfer into `buf` in response to a control write request.
    ///
    /// Must be called after [`Event::Setup`] for requests with `direction` of `Out`
    /// and request length greater than zero.
    ///
    /// `buf` must be **at most** as large as the request length.
    ///
    /// # Errors
    ///
    /// * [`EndpointError::Disabled`] - The control transfer was aborted, see
    ///   [aborting a control transfer](Bus#aborting-a-control-transfer).
    /// * [`EndpointError::BufferOverflow`] - A received packet is larger than the remaining space
    ///   in `buf`.
    async fn control_data_out(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError>;

    /// Send a DATA IN transfer with `data` in response to a control read request.
    ///
    /// If `needs_zlp` is true and `data.len()` is an exact non-zero multiple of the control
    /// max packet size, a zero-length packet must be transmitted.
    /// The driver is responsible for handling the status stage following the transfer.
    ///
    /// # Errors
    ///
    /// * [`EndpointDisabledError`] - The control transfer was aborted, see
    ///   [aborting a control transfer](Bus#aborting-a-control-transfer).
    async fn control_data_in(&mut self, data: &[u8], needs_zlp: bool) -> Result<(), EndpointDisabledError>;

    /// Accept a control request.
    ///
    /// Causes the STATUS packet for the current request to be ACKed. Returns early if the transfer
    /// is aborted, see [aborting a control transfer](Bus#aborting-a-control-transfer).
    async fn control_accept(&mut self);

    /// Reject a control request.
    ///
    /// Sets a STALL condition on the pipe to indicate an error.
    async fn control_reject(&mut self);

    /// Accept SET_ADDRESS control and change bus address.
    ///
    /// For most drivers this function should firstly call `control_accept()` and then change the bus address.
    /// However, there are peripherals (Synopsys USB OTG) that have reverse order.
    async fn control_accept_set_address(&mut self, addr: u8);

    /// Enable or disable an endpoint.
    ///
    /// All non-control endpoints are initially disabled after [`Driver::start`], and a USB bus
    /// reset ([`Event::Reset`]) disables all non-control endpoints without the stack calling this
    /// method. Enabling an endpoint clears any STALL condition on it, unblocks
    /// [`Endpoint::wait_enabled`], and sets [`Endpoint::is_enabled`] to `true`. Disabling an
    /// endpoint must abort or prevent pending transfers with [`EndpointError::Disabled`].
    async fn endpoint_set_enabled(&mut self, ep_addr: EndpointAddress, enabled: bool);

    /// Get whether an endpoint is enabled.
    fn endpoint_is_enabled(&mut self, ep_addr: EndpointAddress) -> bool;

    /// Set or clear the STALL condition for an endpoint.
    ///
    /// If the endpoint is an OUT endpoint, it should be prepared to receive data again.
    async fn endpoint_set_stalled(&mut self, ep_addr: EndpointAddress, stalled: bool);

    /// Get whether the STALL condition is set for an endpoint.
    fn endpoint_is_stalled(&mut self, ep_addr: EndpointAddress) -> bool;

    /// Simulate a disconnect from the USB bus, causing the host to reset and re-enumerate the
    /// device.
    ///
    /// The default implementation just returns `Unsupported`.
    ///
    /// # Errors
    ///
    /// * [`Unsupported`] - This UsbBus implementation doesn't support
    ///   simulating a disconnect or it has not been enabled at creation time.
    async fn force_reset(&mut self) -> Result<(), Unsupported> {
        Err(Unsupported)
    }

    /// Initiate a remote wakeup of the host by the device.
    ///
    /// Whether this waits for the remote wakeup signaling to complete or only initiates it is
    /// implementation-defined.
    ///
    /// # Errors
    ///
    /// * [`Unsupported`] - This UsbBus implementation doesn't support
    ///   remote wakeup or it has not been enabled at creation time.
    async fn remote_wakeup(&mut self) -> Result<(), Unsupported>;
}

/// Endpoint trait, common for OUT and IN.
/// Endpoint is a buffer on a device that stores rx/tx data.
/// Endpoint can be thought of as one end of a pipe/channel.
pub trait Endpoint {
    /// Get the endpoint address and attributes.
    fn info(&self) -> EndpointInfo;

    /// Wait for the endpoint to be enabled.
    ///
    /// Resolves once the endpoint has been enabled via [`Bus::endpoint_set_enabled`] (and has not
    /// been disabled again, e.g. by a bus reset).
    async fn wait_enabled(&mut self);

    /// Get whether the endpoint is enabled.
    fn is_enabled(&self) -> bool;

    /// Set or clear the STALL handshake on this endpoint.
    ///
    /// While stalled, bulk and interrupt endpoints answer tokens with STALL.
    /// Control and isochronous endpoints ignore this. Clearing the stall resets
    /// the data toggle to DATA0, whether or not the endpoint was stalled.
    /// Enabling the endpoint clears a stall; stalling before the endpoint is
    /// enabled may not take effect.
    async fn set_stalled(&mut self, stalled: bool);

    /// Get whether the STALL handshake is set on this endpoint.
    ///
    /// Returns `false` for endpoints that do not have the halt feature.
    fn is_stalled(&self) -> bool;
}

/// OUT Endpoint trait.
pub trait EndpointOut: Endpoint {
    /// Read until the buffer is full or we receive a short packet from the USB host returning the
    /// actual length of the entire data block.
    ///
    /// This should also clear any NAK flags and prepare the endpoint to receive the next packet or
    /// data block.
    ///
    /// # Errors
    ///
    /// * [`EndpointError::Disabled`] - The endpoint is disabled.
    /// * [`EndpointError::BufferOverflow`] - A received packet is larger than the remaining space
    ///   in `buf` (for example if `buf.len()` is smaller than `max_packet_size` or not a multiple
    ///   of `max_packet_size`).
    async fn read_transfer(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError>;
}

/// IN Endpoint trait.
pub trait EndpointIn: Endpoint {
    /// Write all the data from `buf` to the endpoint as a transfer.
    ///
    /// The driver is responsible for splitting large transfers into packets, sending them,
    /// and optionally sending a zero-length packet (ZLP) if `needs_zlp` is true and `buf.len()`
    /// is an exact non-zero multiple of `max_packet_size`. If `buf` is empty, a single zero-length
    /// packet is sent.
    ///
    /// Drivers are allowed to buffer packets/transfers and return before the host acknowledges them.
    ///
    /// # Errors
    ///
    /// * [`EndpointDisabledError`] - The endpoint is disabled.
    async fn write_transfer(&mut self, buf: &[u8], needs_zlp: bool) -> Result<(), EndpointDisabledError>;
}

/// Event returned by [`Bus::poll`].
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Event {
    /// The USB reset condition has been detected, negotiating the given bus [`Speed`].
    Reset(Speed),

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

    /// A SETUP packet has been received on the control pipe, which starts a control request.
    ///
    /// A new SETUP packet abandons the control request in progress, if any.
    Setup([u8; 8]),
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

/// The endpoint is disabled, or a control transfer was aborted by a bus event or new SETUP packet.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EndpointDisabledError;

impl core::fmt::Display for EndpointDisabledError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Endpoint disabled")
    }
}
impl core::error::Error for EndpointDisabledError {}

/// Errors returned by [`EndpointOut::read_transfer`] and [`Bus::control_data_out`].
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum EndpointError {
    /// The received packet is too long to fit in `buf`.
    BufferOverflow,

    /// The endpoint is disabled.
    Disabled,
}

impl From<EndpointDisabledError> for EndpointError {
    fn from(_: EndpointDisabledError) -> Self {
        Self::Disabled
    }
}

impl core::fmt::Display for EndpointError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BufferOverflow => write!(f, "Buffer overflow"),
            Self::Disabled => write!(f, "Endpoint disabled"),
        }
    }
}
impl core::error::Error for EndpointError {}

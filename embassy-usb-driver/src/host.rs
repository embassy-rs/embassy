//! USB host driver traits and data types.

use core::time::Duration;

use crate::{EndpointInfo, EndpointType, Speed};

/// Speed of a low- or full-speed device reached through split transactions
/// (USB 2.0 §11.14) or a `PRE` prefix (USB 1.1 §11.8.6).
///
/// High-speed devices are not valid split targets; split metadata only applies
/// to devices operating at low or full speed behind a hub.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SplitSpeed {
    /// 1.5 Mbit/s
    Low,
    /// 12 Mbit/s
    Full,
}

/// Per-pipe information necessary to encode a split-transaction token
/// (USB 2.0 §11.14) or a legacy full-speed `PRE` packet (USB 1.1 §11.8.6).
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SplitInfo {
    hub_addr: u8,
    port: u8,
    device_speed: SplitSpeed,
}

impl SplitInfo {
    /// Create a new [`SplitInfo`].
    ///
    /// `hub_addr` is the USB address of the hub that owns the Transaction
    /// Translator; `port` is the 1-based port number on that hub where the
    /// target device is attached; `device_speed` is the speed of that target
    /// device ([`SplitSpeed::Low`] or [`SplitSpeed::Full`] only).
    pub const fn new(hub_addr: u8, port: u8, device_speed: SplitSpeed) -> Self {
        Self {
            hub_addr,
            port,
            device_speed,
        }
    }

    /// USB address of the hub that owns the Transaction Translator.
    pub const fn hub_addr(self) -> u8 {
        self.hub_addr
    }

    /// 1-based port number on the hub where the target device is attached.
    pub const fn port(self) -> u8 {
        self.port
    }

    /// Speed of the split target device (low or full only).
    pub const fn device_speed(self) -> SplitSpeed {
        self.device_speed
    }
}

/// Errors returned by [`UsbPipe`] operations.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum PipeError {
    /// The packet is too long to fit in the buffer.
    BufferOverflow,

    /// CRC or other hardware-level framing error.
    BadResponse,

    /// The device sent more data than expected (babble).
    Babble,

    /// Data toggle sequence mismatch detected.
    DataToggleError,

    /// Transaction was canceled
    Canceled,

    /// The device endpoint is stalled.
    Stall,

    /// Device did not respond in time
    Timeout,

    /// Device disconnected
    Disconnected,
}

/// Device has been attached/detached
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum DeviceEvent {
    /// Indicates a root-device has become attached
    Connected(Speed),

    /// Indicates that a device has been detached
    Disconnected,

    /// Root port overcurrent protection tripped.
    Overcurrent,
}

/// Indicates type of error of Host interface
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum HostError {
    /// A pipe-level transfer error occurred.
    PipeError(PipeError),
    /// The control request was not acknowledged by the device.
    RequestFailed,
    /// A descriptor returned by the device could not be parsed.
    InvalidDescriptor,
    /// No free device slots available.
    OutOfSlots,
    /// No free host pipes available.
    OutOfPipes,
    /// The addressed device does not exist.
    NoSuchDevice,
    /// Insufficient memory for the requested operation.
    InsufficientMemory,
    /// An unspecified error with a static description.
    Other(&'static str),
}

impl From<PipeError> for HostError {
    fn from(value: PipeError) -> Self {
        HostError::PipeError(value)
    }
}

/// Async USB Host Driver trait.
///
/// To be implemented by the HAL.
pub trait UsbHostDriver<'d>: Sized {
    /// Pipe implementation of this UsbHostDriver
    type Pipe<T: pipe::Type, D: pipe::Direction>: UsbPipe<T, D> + 'd;

    /// Wait for a root-port attach/detach.
    ///
    /// On attach, the implementation must drive a bus reset to completion
    /// before returning and must report the speed that the device settled
    /// on after reset.
    async fn wait_for_device_event(&mut self) -> DeviceEvent;

    /// Force a bus reset on the root port.
    ///
    /// Invalidates every pipe currently allocated against addresses other
    /// than 0. Used to recover from a misbehaving device or to force
    /// re-enumeration without unplug.
    async fn bus_reset(&mut self);

    /// Allocate pipe for communication with device.
    ///
    /// This can be a scarce resource, for one-off requests please scope the pipe so it's dropped after completion.
    ///
    /// `split` - when `Some`, every transfer on this pipe is routed as a
    /// split transaction through the specified hub's TT (USB 2.0 §11.14), or
    /// as a legacy PRE packet on full-speed controllers (USB 1.1 §11.8.6).
    /// Pass `None` when the device is reached directly (host at the same
    /// speed as the device, or the device is high-speed).
    fn alloc_pipe<T: pipe::Type, D: pipe::Direction>(
        &self,
        addr: u8,
        endpoint: &EndpointInfo,
        split: Option<SplitInfo>,
    ) -> Result<Self::Pipe<T, D>, HostError>;
}

/// Type-level pipe markers for endpoint type and direction.
///
/// These structs and traits are used as generic parameters on [`UsbPipe`]
/// to statically enforce correct endpoint type and direction at compile time.
///
/// All marker traits are sealed — they cannot be implemented outside this crate.
pub mod pipe {
    use super::EndpointType;

    mod sealed {
        pub trait Sealed {}
    }

    /// Marker trait for the endpoint transfer type of a pipe.
    pub trait Type: sealed::Sealed + 'static {
        /// Returns the [`EndpointType`] this marker represents.
        fn ep_type() -> EndpointType;
    }

    /// Marker for a control endpoint pipe.
    pub struct Control {}
    /// Marker for an interrupt endpoint pipe.
    pub struct Interrupt {}
    /// Marker for a bulk endpoint pipe.
    pub struct Bulk {}
    /// Marker for an isochronous endpoint pipe.
    pub struct Isochronous {}

    impl sealed::Sealed for Control {}
    impl sealed::Sealed for Interrupt {}
    impl sealed::Sealed for Bulk {}
    impl sealed::Sealed for Isochronous {}

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

    /// Trait bound satisfied only by [`Control`] pipes.
    #[diagnostic::on_unimplemented(message = "This is not a CONTROL pipe")]
    pub trait IsControl: Type {}
    impl IsControl for Control {}

    /// Trait bound satisfied only by [`Interrupt`] pipes.
    #[diagnostic::on_unimplemented(message = "This is not an INTERRUPT pipe")]
    pub trait IsInterrupt: Type {}
    impl IsInterrupt for Interrupt {}

    /// Trait bound satisfied only by [`Bulk`] or [`Interrupt`] pipes.
    #[diagnostic::on_unimplemented(message = "This is not a BULK or INTERRUPT pipe")]
    pub trait IsBulkOrInterrupt: Type {}
    impl IsBulkOrInterrupt for Bulk {}
    impl IsBulkOrInterrupt for Interrupt {}

    /// Marker trait for the transfer direction of a pipe.
    pub trait Direction: sealed::Sealed + 'static {
        /// Returns `true` if this direction supports IN (device-to-host) transfers.
        fn is_in() -> bool;
        /// Returns `true` if this direction supports OUT (host-to-device) transfers.
        fn is_out() -> bool;
    }

    /// Marker for an IN-only (device-to-host) pipe.
    pub struct In {}
    /// Marker for an OUT-only (host-to-device) pipe.
    pub struct Out {}
    /// Marker for a bidirectional pipe (used for control endpoints).
    pub struct InOut {}

    impl sealed::Sealed for In {}
    impl sealed::Sealed for Out {}
    impl sealed::Sealed for InOut {}

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

    /// Trait bound satisfied by directions that support IN transfers.
    #[diagnostic::on_unimplemented(message = "This is not an IN pipe")]
    pub trait IsIn: Direction {}
    impl IsIn for In {}
    impl IsIn for InOut {}

    /// Trait bound satisfied by directions that support OUT transfers.
    #[diagnostic::on_unimplemented(message = "This is not an OUT pipe")]
    pub trait IsOut: Direction {}
    impl IsOut for Out {}
    impl IsOut for InOut {}
}

/// Timeouts applied to a control pipe's NAK-retry behaviour.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct TimeoutConfig {
    /// Maximum response timeout for transactions with a Data Stage.
    pub data_timeout: Duration,

    /// Maximum response timeout for transactions without a Data Stage.
    pub no_data_timeout: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        TimeoutConfig {
            data_timeout: Duration::from_millis(500),
            no_data_timeout: Duration::from_millis(50),
        }
    }
}

/// ## USB Pipes
/// These contain the required information to send a packet correctly to a device endpoint.
/// The information is carried with the pipe on creation (see [`UsbHostDriver::alloc_pipe`]).
///
/// It is up to the HAL's driver how to implement concurrent requests, some hardware IP may allow for multiple hardware channels
///  while others may only have a single channel which needs to be multiplexed in software, while others still use DMA request linked-lists.
/// Any of these are compatible with the UsbPipe with varying degrees of sync primitives required.
///
/// ### NAK handling
/// Implementations must retry on NAK if appropriate for the transfer type.
/// - For **control** transfers, the implementation should retry until the configurable timeout expires (see [`UsbPipe::set_timeout`]).
/// - For **bulk** transfers, the implementation must retry indefinitely. Use `embassy_time::with_timeout` around the future to impose a deadline; dropping the future must abort the transfer.
/// - For **interrupt** transfers, a NAK indicates no data is available; the implementation should poll again at the next interval.
///
/// ### Data toggle
/// Implementations are responsible for maintaining the data toggle sequence for bulk and interrupt endpoints.
/// The toggle is initialized to DATA0 when the pipe is allocated and should advance after each successful transfer.
///
/// ### Cancellation
/// All transfer methods (`control_in`, `control_out`, `request_in`, `request_out`) are asynchronous.
/// If the returned future is dropped before completion, the implementation must abort the in-progress
/// transfer and leave the pipe in a consistent state for future requests.
pub trait UsbPipe<T: pipe::Type, D: pipe::Direction> {
    /// Send IN control request.
    ///
    /// Returns the number of bytes received into `buf`.
    async fn control_in(&mut self, setup: &[u8; 8], buf: &mut [u8]) -> Result<usize, PipeError>
    where
        T: pipe::IsControl,
        D: pipe::IsIn;

    /// Send OUT control request
    async fn control_out(&mut self, setup: &[u8; 8], buf: &[u8]) -> Result<(), PipeError>
    where
        T: pipe::IsControl,
        D: pipe::IsOut;

    /// Send IN request of type other from control
    /// For interrupt pipes this will return the result of the next successful interrupt poll
    async fn request_in(&mut self, buf: &mut [u8]) -> Result<usize, PipeError>
    where
        D: pipe::IsIn;

    /// Send OUT request of type other from control
    /// ensure_transaction_end: Send a zero length packet at the end of transaction if last packet is of max size.
    async fn request_out(&mut self, buf: &[u8], ensure_transaction_end: bool) -> Result<(), PipeError>
    where
        D: pipe::IsOut;

    /// Configure the timeouts of this pipe.
    fn set_timeout(&mut self, timeout: TimeoutConfig)
    where
        T: pipe::IsControl;

    /// Reset the host-side data toggle on this pipe to DATA0.
    ///
    /// The caller must invoke this method after:
    ///
    /// - `CLEAR_FEATURE(ENDPOINT_HALT)` successfully clears a functional
    ///   stall on this endpoint.
    /// - `SET_CONFIGURATION` succeeds (all non-control endpoints on the
    ///   affected interfaces must be reset).
    /// - `SET_INTERFACE` succeeds (all non-control endpoints on the
    ///   affected interface must be reset).
    fn reset_data_toggle(&mut self)
    where
        T: pipe::IsBulkOrInterrupt;
}

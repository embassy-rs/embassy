#![no_std]
#![allow(async_fn_in_trait)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

/// Speed of a connected USB device.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DeviceSpeed {
    /// Low speed (1.5 Mbit/s).
    Low,
    /// Full speed (12 Mbit/s).
    Full,
    /// High speed (480 Mbit/s).
    High,
}

/// Direction of USB traffic from the host's perspective.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Direction {
    /// Host to device.
    Out,
    /// Device to host.
    In,
}

/// USB endpoint transfer type.
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EndpointType {
    /// Control endpoint.
    Control = 0b00,
    /// Isochronous endpoint.
    Isochronous = 0b01,
    /// Bulk endpoint.
    Bulk = 0b10,
    /// Interrupt endpoint.
    Interrupt = 0b11,
}

/// Descriptor of a remote device's endpoint.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DeviceEndpoint {
    /// USB device address (0-127).
    pub device_address: u8,
    /// Endpoint number (0-15).
    pub ep_number: u8,
    /// Transfer direction.
    pub direction: Direction,
    /// Transfer type.
    pub ep_type: EndpointType,
    /// Maximum packet size in bytes.
    pub max_packet_size: u16,
    /// Device speed.
    pub speed: DeviceSpeed,
}

/// Event from the USB host port.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PortEvent {
    /// A device has been connected.
    Connected,
    /// A device has been disconnected.
    Disconnected,
    /// The port has been enabled after reset, with the detected speed.
    Enabled {
        /// Speed of the connected device.
        speed: DeviceSpeed,
    },
    /// Overcurrent condition detected.
    Overcurrent,
}

/// Error during a USB transfer.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TransferError {
    /// Device responded with STALL.
    Stall,
    /// No response from device (timeout).
    NoResponse,
    /// Data toggle mismatch.
    DataToggle,
    /// Babble error (data exceeds expected length).
    Babble,
    /// Transaction error (CRC, bit-stuff, etc.).
    TransactionError,
    /// Frame overrun.
    FrameOverrun,
    /// No free channel available.
    ChannelBusy,
    /// Buffer too small for received data.
    BufferOverflow,
    /// Device disconnected during transfer.
    Disconnected,
}

impl core::fmt::Display for TransferError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Stall => write!(f, "STALL"),
            Self::NoResponse => write!(f, "No response"),
            Self::DataToggle => write!(f, "Data toggle error"),
            Self::Babble => write!(f, "Babble"),
            Self::TransactionError => write!(f, "Transaction error"),
            Self::FrameOverrun => write!(f, "Frame overrun"),
            Self::ChannelBusy => write!(f, "Channel busy"),
            Self::BufferOverflow => write!(f, "Buffer overflow"),
            Self::Disconnected => write!(f, "Disconnected"),
        }
    }
}

impl core::error::Error for TransferError {}

impl embedded_io_async::Error for TransferError {
    fn kind(&self) -> embedded_io_async::ErrorKind {
        match self {
            Self::Stall => embedded_io_async::ErrorKind::Other,
            Self::NoResponse => embedded_io_async::ErrorKind::TimedOut,
            Self::Disconnected => embedded_io_async::ErrorKind::NotConnected,
            Self::BufferOverflow => embedded_io_async::ErrorKind::OutOfMemory,
            _ => embedded_io_async::ErrorKind::Other,
        }
    }
}

/// Error allocating a host channel.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ChannelAllocError;

impl core::fmt::Display for ChannelAllocError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "No free host channel")
    }
}

impl core::error::Error for ChannelAllocError {}

/// Main USB host driver trait.
///
/// Implement this to add USB host support for a new hardware platform.
/// This is the entry point that produces a [`HostBus`] for managing the port and channels.
pub trait HostDriver {
    /// The host bus type produced by this driver.
    type Bus: HostBus;

    /// Start the USB host controller and return the bus for managing connections.
    fn start(self) -> Self::Bus;
}

/// USB host bus trait.
///
/// Manages the root port (connect/disconnect/reset) and allocates channels
/// for communicating with device endpoints.
pub trait HostBus {
    /// The host channel type for transfers.
    ///
    /// Channels are released automatically when dropped. Implementations should
    /// mark the underlying hardware channel as free in the channel's `Drop` impl.
    type Channel: HostChannel;

    /// Enable the USB host controller.
    async fn enable(&mut self);

    /// Disable the USB host controller.
    async fn disable(&mut self);

    /// Wait for a port event (connect, disconnect, enable, overcurrent).
    async fn poll(&mut self) -> PortEvent;

    /// Reset the USB port. Call after detecting a connection, before enumeration.
    async fn reset(&mut self);

    /// Suspend the USB port.
    async fn suspend(&mut self);

    /// Resume the USB port from suspend.
    async fn resume(&mut self);

    /// Returns true if a device is currently connected.
    fn is_connected(&self) -> bool;

    /// Returns the speed of the connected device, if any.
    fn speed(&self) -> Option<DeviceSpeed>;

    /// Allocate a host channel for communicating with a device endpoint.
    ///
    /// The channel is released when it is dropped.
    fn alloc_channel(&self, ep: &DeviceEndpoint) -> Result<Self::Channel, ChannelAllocError>;
}

/// USB host channel trait.
///
/// A channel is a hardware resource used to communicate with a specific device endpoint.
/// The host has a limited number of channels that are multiplexed across device endpoints.
pub trait HostChannel {
    /// Change the target device address and max packet size.
    ///
    /// Used after SET_ADDRESS to update the channel without reallocating.
    fn retarget(&mut self, device_address: u8, max_packet_size: u16);

    /// Perform a complete control transfer (SETUP -> optional DATA -> STATUS).
    ///
    /// `setup` is the 8-byte SETUP packet.
    /// `direction` indicates DATA phase direction (or `Out` for no-data transfers).
    /// `data` is the buffer for the DATA phase. For IN transfers, received data is written here.
    /// For OUT transfers, data to send is read from here. For no-data transfers, pass an empty slice.
    ///
    /// Returns the number of bytes transferred in the DATA phase.
    async fn control_transfer(
        &mut self,
        setup: &[u8; 8],
        direction: Direction,
        data: &mut [u8],
    ) -> Result<usize, TransferError>;

    /// Perform an IN transfer (device to host).
    ///
    /// Reads data from the device endpoint into `buf`.
    /// Returns the number of bytes received.
    async fn in_transfer(&mut self, buf: &mut [u8]) -> Result<usize, TransferError>;

    /// Perform an OUT transfer (host to device).
    ///
    /// Writes `data` to the device endpoint.
    /// Returns the number of bytes sent.
    async fn out_transfer(&mut self, data: &[u8]) -> Result<usize, TransferError>;
}

use core::future::Future;

use crate::control::Request;

use super::types::*;

/// Driver for a specific USB peripheral. Implement this to add support for a new hardware
/// platform.
pub trait Driver<'a> {
    type EndpointOut: EndpointOut + 'a;
    type EndpointIn: EndpointIn + 'a;
    type ControlPipe: ControlPipe + 'a;
    type Bus: Bus + 'a;

    /// Allocates an endpoint and specified endpoint parameters. This method is called by the device
    /// and class implementations to allocate endpoints, and can only be called before
    /// [`enable`](UsbBus::enable) is called.
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
        ep_addr: Option<EndpointAddress>,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval: u8,
    ) -> Result<Self::EndpointOut, EndpointAllocError>;

    fn alloc_endpoint_in(
        &mut self,
        ep_addr: Option<EndpointAddress>,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval: u8,
    ) -> Result<Self::EndpointIn, EndpointAllocError>;

    fn alloc_control_pipe(
        &mut self,
        max_packet_size: u16,
    ) -> Result<Self::ControlPipe, EndpointAllocError>;

    /// Enables and initializes the USB peripheral. Soon after enabling the device will be reset, so
    /// there is no need to perform a USB reset in this method.
    fn enable(self) -> Self::Bus;

    /// Indicates that `set_device_address` must be called before accepting the corresponding
    /// control transfer, not after.
    ///
    /// The default value for this constant is `false`, which corresponds to the USB 2.0 spec, 9.4.6
    const QUIRK_SET_ADDRESS_BEFORE_STATUS: bool = false;
}

pub trait Bus {
    type PollFuture<'a>: Future<Output = Event> + 'a
    where
        Self: 'a;

    fn poll<'a>(&'a mut self) -> Self::PollFuture<'a>;

    /// Called when the host resets the device. This will be soon called after
    /// [`poll`](crate::device::UsbDevice::poll) returns [`PollResult::Reset`]. This method should
    /// reset the state of all endpoints and peripheral flags back to a state suitable for
    /// enumeration, as well as ensure that all endpoints previously allocated with alloc_ep are
    /// initialized as specified.
    fn reset(&mut self);

    /// Sets the device USB address to `addr`.
    fn set_device_address(&mut self, addr: u8);

    /// Sets the device configured state.
    fn set_configured(&mut self, configured: bool);

    /// Sets or clears the STALL condition for an endpoint. If the endpoint is an OUT endpoint, it
    /// should be prepared to receive data again. Only used during control transfers.
    fn set_stalled(&mut self, ep_addr: EndpointAddress, stalled: bool);

    /// Gets whether the STALL condition is set for an endpoint. Only used during control transfers.
    fn is_stalled(&mut self, ep_addr: EndpointAddress) -> bool;

    /// Causes the USB peripheral to enter USB suspend mode, lowering power consumption and
    /// preparing to detect a USB wakeup event. This will be called after
    /// [`poll`](crate::device::UsbDevice::poll) returns [`PollResult::Suspend`]. The device will
    /// continue be polled, and it shall return a value other than `Suspend` from `poll` when it no
    /// longer detects the suspend condition.
    fn suspend(&mut self);

    /// Resumes from suspend mode. This may only be called after the peripheral has been previously
    /// suspended.
    fn resume(&mut self);

    /// Simulates a disconnect from the USB bus, causing the host to reset and re-enumerate the
    /// device.
    ///
    /// The default implementation just returns `Unsupported`.
    ///
    /// # Errors
    ///
    /// * [`Unsupported`](crate::UsbError::Unsupported) - This UsbBus implementation doesn't support
    ///   simulating a disconnect or it has not been enabled at creation time.
    fn force_reset(&mut self) -> Result<(), Unsupported> {
        Err(Unsupported)
    }
}

pub trait Endpoint {
    type WaitEnabledFuture<'a>: Future<Output = ()> + 'a
    where
        Self: 'a;

    /// Get the endpoint address
    fn info(&self) -> &EndpointInfo;

    /// Sets or clears the STALL condition for an endpoint. If the endpoint is an OUT endpoint, it
    /// should be prepared to receive data again.
    fn set_stalled(&self, stalled: bool);

    /// Gets whether the STALL condition is set for an endpoint.
    fn is_stalled(&self) -> bool;

    /// Waits for the endpoint to be enabled.
    fn wait_enabled(&mut self) -> Self::WaitEnabledFuture<'_>;

    // TODO enable/disable?
}

pub trait EndpointOut: Endpoint {
    type ReadFuture<'a>: Future<Output = Result<usize, ReadError>> + 'a
    where
        Self: 'a;

    /// Reads a single packet of data from the endpoint, and returns the actual length of
    /// the packet.
    ///
    /// This should also clear any NAK flags and prepare the endpoint to receive the next packet.
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a>;
}

pub trait ControlPipe {
    type SetupFuture<'a>: Future<Output = Request> + 'a
    where
        Self: 'a;
    type DataOutFuture<'a>: Future<Output = Result<usize, ReadError>> + 'a
    where
        Self: 'a;
    type DataInFuture<'a>: Future<Output = ()> + 'a
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
    fn data_out<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::DataOutFuture<'a>;

    /// Sends a DATA IN packet with `data` in response to a control read request.
    ///
    /// If `last_packet` is true, the STATUS packet will be ACKed following the transfer of `data`.
    fn data_in<'a>(&'a mut self, data: &'a [u8], last_packet: bool) -> Self::DataInFuture<'a>;

    /// Accepts a control request.
    ///
    /// Causes the STATUS packet for the current request to be ACKed.
    fn accept(&mut self);

    /// Rejects a control request.
    ///
    /// Sets a STALL condition on the pipe to indicate an error.
    fn reject(&mut self);
}

pub trait EndpointIn: Endpoint {
    type WriteFuture<'a>: Future<Output = Result<(), WriteError>> + 'a
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
/// Errors returned by [`EndpointIn::write`]
pub enum WriteError {
    /// The packet is too long to fit in the
    ///   transmission buffer. This is generally an error in the class implementation, because the
    ///   class shouldn't provide more data than the `max_packet_size` it specified when allocating
    ///   the endpoint.
    BufferOverflow,
    Disabled,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Errors returned by [`EndpointOut::read`]
pub enum ReadError {
    /// The received packet is too long to
    /// fit in `buf`. This is generally an error in the class implementation, because the class
    /// should use a buffer that is large enough for the `max_packet_size` it specified when
    /// allocating the endpoint.
    BufferOverflow,
    Disabled,
}

//! Isochronous endpoint traits.

use core::num::NonZeroU32;

use crate::{Endpoint, EndpointDisabledError, EndpointError, EndpointInfo};

/// Why [`IsoInSlot::write`] did not queue the transfer.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum IsoWriteError {
    /// The service interval had already started before [`IsoInSlot::write`] was called,
    /// so the service interval was skipped.
    ///
    /// This will reset and *not* increment the skipped service interval counter.
    TooLate,
    /// `data` is larger than larger than the maximum number of
    /// bytes the endpoint is configured to transfer in a service interval.
    ///
    /// For Full Speed endpoints, this means at most
    /// [`EndpointInfo::max_packet_size`](crate::EndpointInfo::max_packet_size)
    /// bytes, but High Speed and Super Speed endpoints can be configured to allow
    /// more than one packet per service interval.
    ///
    /// This will reset and *not* increment the skipped service interval counter.
    BufferOverflow,
}

impl core::fmt::Display for IsoWriteError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TooLate => write!(f, "Isochronous transfer submitted too late"),
            Self::BufferOverflow => write!(f, "Buffer overflow"),
        }
    }
}

impl core::error::Error for IsoWriteError {}

/// A reserved isochronous IN slot in the queue, corresponding to a specific service interval
pub trait IsoInSlot {
    /// Queue one transfer for this slot without waiting.
    ///
    /// The length of `data` may be zero and must be no larger than the maximum number of
    /// bytes the endpoint is configured to transfer in a service interval.
    /// For Full Speed endpoints, this means at most
    /// [`EndpointInfo::max_packet_size`](crate::EndpointInfo::max_packet_size)
    /// bytes, but High Speed and Super Speed endpoints can be configured to allow
    /// more than one packet per service interval.
    ///
    /// A transfer the host does not poll for in its service interval is
    /// silently dropped when the interval ends and is never sent in a later interval.
    ///
    /// See [`IsoEndpointIn::ready`] for more details.
    fn write(self, data: &[u8]) -> Result<(), IsoWriteError>;
}

/// Isochronous IN endpoint trait.
///
/// Drivers implement this only when they can schedule transfers by service interval and drop
/// uncollected transfers instead of sending them late.
pub trait IsoEndpointIn: Endpoint {
    /// A reserved slot borrowing this endpoint.
    type Slot<'a>: IsoInSlot
    where
        Self: 'a;

    /// Wait until the next transfer can be queued, returning
    ///
    /// * the number of skipped service intervals (if any)
    ///   since the service interval of the last [`IsoInSlot::write`], and
    ///
    /// * an [`IsoInSlot`] for the next available service interval.
    ///
    /// Service intervals are skipped if a transfer isn't submitted in time.
    ///
    /// Skipped service intervals are handled in an implementation defined way:
    /// the device may either respond to the host's request with an empty transfer,
    /// or not respond to the host at all.
    ///
    /// After the endpoint is enabled, the skipped service interval count will stay
    /// at zero until the first call to [`IsoInSlot::write`].
    ///
    /// After that, if [`ready`](Self::ready) is called too late for the next sequential
    /// service interval, it will return the number of service intervals that were skipped.
    ///
    /// [`IsoInSlot::write`] will always reset the skipped service interval count
    /// to zero, regardless of whether it returns an error or not.
    ///
    /// If [`IsoInSlot::write`] returns an error, the service interval has been or will be skipped,
    /// but it will *not* increment the skipped service interval counter, which will stay at zero.
    ///
    /// When the endpoint is disabled, any transfers in the queue will be silently dropped.
    /// If there is an active [`IsoInSlot`], it's transfer will be dropped even if
    /// [`IsoInSlot::write`] returns [`Ok`]
    ///
    /// # Cancel safety
    ///
    /// Dropping this future will not reset the skipped service interval count
    /// or reserve a service interval and queue slot.
    ///
    /// Dropping the slot before calling [`IsoInSlot::write`] will roll back the effects
    /// of [`ready`](Self::ready). The service interval and queue slot will no longer be reserved
    /// and the skipped service interval count will not be reset.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use embassy_usb_driver::*;
    /// async fn example(mut ep: impl IsoEndpointIn, mut get_packet: impl FnMut() -> [u8; 8]) {
    ///     assert!(ep.info().max_packet_size >= 8);
    ///     ep.wait_enabled().await;
    ///     loop {
    ///         let Ok((skipped, slot)) = ep.ready().await else {
    ///             break;
    ///         };
    ///         if let Some(skipped) = skipped {
    ///             // throw away packets to avoid desynchronization
    ///             for _ in 0..skipped.get() {
    ///                 let _ = get_packet();
    ///             }
    ///         }
    ///         match slot.write(&get_packet()) {
    ///             // In this case, we don't care if we may have sent an
    ///             // empty packet instead of the real packet
    ///             Ok(()) | Err(IsoWriteError::TooLate) => {}
    ///
    ///             // We already checked that our max transfer size is big enough
    ///             Err(IsoWriteError::BufferOverflow) => unreachable!(),
    ///
    ///             // The error is non-exhaustive, so we need to handle future errors
    ///             Err(err) => panic!("{}", err),
    ///         }
    ///     }
    /// }
    /// ```
    async fn ready(&mut self) -> Result<(Option<NonZeroU32>, Self::Slot<'_>), EndpointDisabledError>;
}

/// Isochronous OUT endpoint trait.
pub trait IsoEndpointOut: Endpoint {
    /// Read one service interval's data; `buf` must hold the endpoint's maximum.
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, EndpointError>;
}

/// Isochronous endpoint of drivers without isochronous support; allocating it fails.
#[derive(Debug)]
pub enum NoIsoEndpoint {}

impl Endpoint for NoIsoEndpoint {
    fn info(&self) -> EndpointInfo {
        match *self {}
    }
    async fn wait_enabled(&mut self) {
        match *self {}
    }
    fn is_enabled(&self) -> bool {
        match *self {}
    }
    async fn set_stalled(&mut self, _stalled: bool) {
        match *self {}
    }
    fn is_stalled(&self) -> bool {
        match *self {}
    }
}

impl IsoInSlot for NoIsoEndpoint {
    fn write(self, _data: &[u8]) -> Result<(), IsoWriteError> {
        match self {}
    }
}

impl IsoEndpointIn for NoIsoEndpoint {
    type Slot<'a> = NoIsoEndpoint;

    async fn ready(&mut self) -> Result<(Option<NonZeroU32>, NoIsoEndpoint), EndpointDisabledError> {
        match *self {}
    }
}

impl IsoEndpointOut for NoIsoEndpoint {
    async fn read(&mut self, _buf: &mut [u8]) -> Result<usize, EndpointError> {
        match *self {}
    }
}

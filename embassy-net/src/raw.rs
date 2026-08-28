//! Raw sockets.

use core::future::{Future, poll_fn};
use core::task::{Context, Poll};

use xarxa::driver::PacketBuf;
pub use xarxa::driver::PacketMeta;
pub use xarxa::raw::RawMode;
use xarxa::raw::{self, RawHandle};
pub use xarxa::wire::{IpProtocol, IpVersion};

use crate::{Stack, TryError};

/// Error returned by [`RawSocket::bind`].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BindError {
    /// The socket is already bound.
    InvalidState,
    /// The interface of an Ethernet-mode bind is not an Ethernet-medium interface.
    #[cfg(feature = "medium-ethernet")]
    InvalidMedium,
}

/// Error returned by [`RawSocket::send`].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SendError {
    /// The socket is not bound.
    InvalidState,
    /// There is no route to the packet's destination.
    Unaddressable,
    /// The packet does not fit in a packet buffer.
    BufferFull,
    /// The packet is malformed, or does not match the socket's filters.
    Malformed,
}

/// Error returned by [`RawSocket::recv`].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RecvError {
    /// The socket is not bound.
    InvalidState,
    /// Provided buffer was smaller than the received packet.
    Truncated,
}

/// An Raw socket.
pub struct RawSocket<'d> {
    stack: Stack<'d>,
    handle: RawHandle,
}

impl<'d> RawSocket<'d> {
    /// Create a new raw socket using the provided stack, receiving IP packets
    /// of the given version and protocol (`None` for any).
    ///
    /// # Panics
    /// Panics if the stack has no room for another raw socket. The limit is set
    /// by the `raw-socket-count-N` feature of `xarxa`.
    pub fn new(stack: Stack<'d>, ip_version: Option<IpVersion>, ip_protocol: Option<IpProtocol>) -> Self {
        let mut this = Self::new_unbound(stack);
        unwrap!(
            this.bind(RawMode::Ip {
                version: ip_version,
                protocol: ip_protocol,
            })
            .ok()
        );
        this
    }

    /// Create a new raw socket using the provided stack, without binding it.
    ///
    /// # Panics
    /// Panics if the stack has no room for another raw socket. The limit is set
    /// by the `raw-socket-count-N` feature of `xarxa`.
    pub fn new_unbound(stack: Stack<'d>) -> Self {
        let handle = stack.with(|i| {
            unwrap!(
                i.stack.add_raw_socket().ok(),
                "too many raw sockets, raise the `raw-socket-count-N` feature of xarxa"
            )
        });
        Self { stack, handle }
    }

    /// Bind the socket to the given mode.
    pub fn bind(&mut self, mode: RawMode) -> Result<(), BindError> {
        match self.with_mut(|s| s.bind(mode)) {
            Ok(()) => Ok(()),
            Err(raw::BindError::InvalidState) => Err(BindError::InvalidState),
            #[cfg(feature = "medium-ethernet")]
            Err(raw::BindError::InvalidMedium) => Err(BindError::InvalidMedium),
        }
    }

    fn with<R>(&self, f: impl FnOnce(&mut raw::RawSocket<'_, 'd>) -> R) -> R {
        self.stack.with(|i| f(&mut i.stack.raw_socket(self.handle)))
    }

    fn with_mut<R>(&self, f: impl FnOnce(&mut raw::RawSocket<'_, 'd>) -> R) -> R {
        self.stack.with_mut(|i| f(&mut i.stack.raw_socket(self.handle)))
    }

    /// Wait until the socket becomes readable.
    ///
    /// A socket is readable when a packet has been received, or when there are queued packets in
    /// the buffer.
    pub fn wait_recv_ready(&self) -> impl Future<Output = ()> + '_ {
        poll_fn(move |cx| self.poll_recv_ready(cx))
    }

    /// Wait until a packet can be read.
    pub fn poll_recv_ready(&self, cx: &mut Context<'_>) -> Poll<()> {
        self.with_mut(|s| {
            if s.can_recv() {
                Poll::Ready(())
            } else {
                s.register_recv_waker(cx.waker());
                Poll::Pending
            }
        })
    }

    /// Receive a packet, copying it into the given buffer.
    ///
    /// This method will wait until a packet is received.
    ///
    /// If the socket is not bound, this method will return `Err(RecvError::InvalidState)`.
    ///
    /// Returns the number of bytes received.
    pub fn recv<'s>(&'s self, buf: &'s mut [u8]) -> impl Future<Output = Result<usize, RecvError>> + 's {
        poll_fn(|cx| self.poll_recv(buf, cx))
    }

    /// Receive a packet, copying it into the given buffer.
    ///
    /// This method will not wait for a packet to be received.
    ///
    /// If no packet is available, this method will return `Err(TryError::WouldBlock)`.
    pub fn try_recv(&self, buf: &mut [u8]) -> Result<usize, TryError<RecvError>> {
        self.with_mut(|s| match s.recv_slice(buf) {
            Ok(n) => Ok(n),
            Err(raw::RecvError::InvalidState) => Err(TryError::Other(RecvError::InvalidState)),
            Err(raw::RecvError::Truncated) => Err(TryError::Other(RecvError::Truncated)),
            Err(raw::RecvError::Exhausted) => Err(TryError::WouldBlock),
        })
    }

    /// Receive a packet, copying it into the given buffer.
    pub fn poll_recv(&self, buf: &mut [u8], cx: &mut Context<'_>) -> Poll<Result<usize, RecvError>> {
        self.with_mut(|s| match s.recv_slice(buf) {
            Ok(n) => Poll::Ready(Ok(n)),
            Err(raw::RecvError::InvalidState) => Poll::Ready(Err(RecvError::InvalidState)),
            Err(raw::RecvError::Truncated) => Poll::Ready(Err(RecvError::Truncated)),
            Err(raw::RecvError::Exhausted) => {
                s.register_recv_waker(cx.waker());
                Poll::Pending
            }
        })
    }

    /// Receive a packet, taking ownership of its buffer.
    ///
    /// This method will wait until a packet is received.
    ///
    /// If the socket is not bound, this method will return `Err(RecvError::InvalidState)`.
    pub async fn recv_packet(&self) -> Result<PacketBuf, RecvError> {
        poll_fn(|cx| {
            self.with_mut(|s| match s.recv() {
                Ok(packet) => Poll::Ready(Ok(packet)),
                Err(raw::RecvError::InvalidState) => Poll::Ready(Err(RecvError::InvalidState)),
                Err(raw::RecvError::Truncated) => unreachable!(),
                Err(raw::RecvError::Exhausted) => {
                    s.register_recv_waker(cx.waker());
                    Poll::Pending
                }
            })
        })
        .await
    }

    /// Receive a packet with a zero-copy function.
    ///
    /// This method will wait until a packet is received.
    pub async fn recv_with<F, R>(&mut self, f: F) -> Result<R, RecvError>
    where
        F: FnOnce(&[u8], PacketMeta) -> R,
    {
        let packet = self.recv_packet().await?;
        Ok(f(&packet, packet.meta()))
    }

    /// Wait until the socket becomes writable.
    pub fn wait_send_ready(&self) -> impl Future<Output = ()> + '_ {
        poll_fn(|cx| self.poll_send_ready(cx))
    }

    /// Wait until a packet can be sent.
    pub fn poll_send_ready(&self, cx: &mut Context<'_>) -> Poll<()> {
        self.with_mut(|s| {
            if s.is_open() {
                Poll::Ready(())
            } else {
                s.register_send_waker(cx.waker());
                Poll::Pending
            }
        })
    }

    /// Map a xarxa send result to ours. `Pending` if the send must be retried later.
    fn map_send(r: Result<(), raw::SendError>) -> Poll<Result<(), SendError>> {
        match r {
            Ok(()) => Poll::Ready(Ok(())),
            Err(raw::SendError::NoBuffer) | Err(raw::SendError::DeviceBusy) => Poll::Pending,
            Err(raw::SendError::BufferFull) => Poll::Ready(Err(SendError::BufferFull)),
            Err(raw::SendError::InvalidState) => Poll::Ready(Err(SendError::InvalidState)),
            Err(raw::SendError::Unaddressable) => Poll::Ready(Err(SendError::Unaddressable)),
            Err(raw::SendError::Malformed) => Poll::Ready(Err(SendError::Malformed)),
        }
    }

    /// Send a packet.
    ///
    /// This method will wait until the packet has been sent.
    pub async fn send(&self, buf: &[u8]) -> Result<(), SendError> {
        self.send_with_meta(buf, PacketMeta::default()).await
    }

    /// Send a packet with the given metadata attached.
    ///
    /// This method will wait until the packet has been sent.
    pub async fn send_with_meta(&self, buf: &[u8], meta: PacketMeta) -> Result<(), SendError> {
        poll_fn(move |cx| self.poll_send_with_meta(buf, meta, cx)).await
    }

    /// Send a packet.
    ///
    /// This method will not wait for a packet buffer or device room to become free.
    pub fn try_send(&self, buf: &[u8]) -> Result<(), TryError<SendError>> {
        self.with_mut(|s| match Self::map_send(s.send_slice(buf)) {
            Poll::Ready(r) => r.map_err(TryError::Other),
            Poll::Pending => Err(TryError::WouldBlock),
        })
    }

    /// Send a packet with the given metadata attached.
    pub fn poll_send_with_meta(
        &self,
        buf: &[u8],
        meta: PacketMeta,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), SendError>> {
        self.with_mut(|s| {
            let r = Self::map_send(s.send_slice_with_meta(buf, meta));
            if r.is_pending() {
                s.register_send_waker(cx.waker());
            }
            r
        })
    }

    /// Send a packet with a zero-copy function.
    ///
    /// This method will wait until a packet buffer is available before passing
    /// it to the closure. The closure returns the number of bytes written into
    /// the buffer.
    pub async fn send_with<F, R>(&mut self, max_size: usize, f: F) -> Result<R, SendError>
    where
        F: FnOnce(&mut [u8]) -> (usize, R),
    {
        let mut f = Some(f);
        poll_fn(move |cx| {
            self.with_mut(|s| {
                let mut ret = None;
                let r = s.send_with(max_size, |buf| {
                    let (size, r) = unwrap!(f.take())(buf);
                    ret = Some(r);
                    size
                });
                match Self::map_send(r) {
                    Poll::Ready(Ok(())) => Poll::Ready(Ok(unwrap!(ret))),
                    Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
                    Poll::Pending => {
                        s.register_send_waker(cx.waker());
                        Poll::Pending
                    }
                }
            })
        })
        .await
    }

    /// Returns whether the socket is bound.
    pub fn is_open(&self) -> bool {
        self.with(|s| s.is_open())
    }

    /// Close the socket.
    pub fn close(&mut self) {
        self.with_mut(|s| s.close())
    }
}

impl Drop for RawSocket<'_> {
    fn drop(&mut self) {
        self.stack.with_mut(|i| i.stack.remove_raw_socket(self.handle));
    }
}

impl core::fmt::Display for SendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidState => f.write_str("InvalidState"),
            Self::Unaddressable => f.write_str("Unaddressable"),
            Self::BufferFull => f.write_str("BufferFull"),
            Self::Malformed => f.write_str("Malformed"),
        }
    }
}
impl core::error::Error for SendError {}

impl core::fmt::Display for RecvError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidState => f.write_str("InvalidState"),
            Self::Truncated => f.write_str("Truncated"),
        }
    }
}
impl core::error::Error for RecvError {}

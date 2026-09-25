//! Raw sockets.
//!
//! A raw socket sends and receives whole packets, headers included.
//!
//! Raw sockets can be bound in two modes, each behind its own cargo feature:
//!
//! - **Ethernet mode** (`RawMode::Ethernet`, feature `raw-ethernet`): whole Ethernet
//!   frames, optionally filtered by ethertype.
//! - **IP mode** (`RawMode::Ip`, feature `raw-ip`): whole IP packets on all
//!   interfaces. The socket may be bound to an IP version and/or an IP protocol,
//!   both optional.

use core::future::{Future, poll_fn};
use core::task::{Context, Poll};

use xarxa::driver::PacketBuf;
pub use xarxa::driver::PacketMeta;
#[cfg(feature = "iface-bind")]
pub use xarxa::iface::IfaceHandle;
pub use xarxa::raw::RawMode;
use xarxa::raw::{self, RawHandle};
#[cfg(feature = "raw-ethernet")]
pub use xarxa::wire::EthernetProtocol;
#[cfg(feature = "raw-ip")]
pub use xarxa::wire::{IpProtocol, IpVersion};

use crate::error::Full;
use crate::{Stack, TryError};

/// Error returned by [`RawSocket::bind`].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BindError {
    /// The socket is already bound.
    InvalidState,
    /// An Ethernet-mode bind on a socket bound to an interface whose medium is
    /// not [`Medium::Ethernet`](crate::iface::Medium::Ethernet).
    #[cfg(feature = "raw-ethernet")]
    InvalidMedium,
}

/// Error returned by [`RawSocket::send`] and [`RawSocket::send_with`].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SendError {
    /// The socket is not bound.
    InvalidState,
    /// There is no route to the packet's destination (IP mode), or no Ethernet
    /// interface to send on (Ethernet mode).
    Unaddressable,
    /// The packet does not fit in a packet buffer.
    BufferFull,
    /// The packet fails basic validation (too short for an Ethernet header in
    /// Ethernet mode, malformed IP header in IP mode), or does not match the
    /// socket's bind filters.
    Malformed,
}

/// Error returned by [`RawSocket::recv`].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RecvError {
    /// The socket is not bound.
    InvalidState,
    /// The provided slice is smaller than the packet. The packet is dropped.
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
    /// # Errors
    /// - `Full`: if the stack has no room for another raw socket. Only possible
    ///   without the `alloc` feature, where the limit is
    ///   [`RAW_SOCKET_COUNT`](crate::config::RAW_SOCKET_COUNT), set by the
    ///   `raw-socket-count-N` feature of `xarxa`.
    #[cfg(feature = "raw-ip")]
    pub fn new(stack: Stack<'d>, ip_version: Option<IpVersion>, ip_protocol: Option<IpProtocol>) -> Result<Self, Full> {
        let mut this = Self::new_unbound(stack)?;
        // The socket was just created, so it is unbound and cannot fail to bind.
        unwrap!(
            this.bind(RawMode::Ip {
                version: ip_version,
                protocol: ip_protocol,
            })
            .ok()
        );
        Ok(this)
    }

    /// Create a new raw socket using the provided stack, without binding it.
    ///
    /// # Errors
    /// - `Full`: if the stack has no room for another raw socket. Only possible
    ///   without the `alloc` feature, where the limit is
    ///   [`RAW_SOCKET_COUNT`](crate::config::RAW_SOCKET_COUNT), set by the
    ///   `raw-socket-count-N` feature of `xarxa`.
    pub fn new_unbound(stack: Stack<'d>) -> Result<Self, Full> {
        let handle = stack.with(|i| i.stack.add_raw_socket())?;
        Ok(Self { stack, handle })
    }

    /// Bind the socket to the given mode.
    ///
    /// # Errors
    /// - `InvalidState`: if the socket is already bound (see
    ///   [is_open](#method.is_open)).
    /// - `InvalidMedium`: if an Ethernet-mode bind is made on a socket bound
    ///   (with `bind_to_iface`, feature `iface-bind`) to an interface whose
    ///   medium is not [`Medium::Ethernet`](crate::iface::Medium::Ethernet).
    ///
    /// # Panics
    /// Panics if the socket is bound to a stale interface handle.
    pub fn bind(&mut self, mode: RawMode) -> Result<(), BindError> {
        match self.with_mut(|s| s.bind(mode)) {
            Ok(()) => Ok(()),
            Err(raw::BindError::InvalidState) => Err(BindError::InvalidState),
            #[cfg(feature = "raw-ethernet")]
            Err(raw::BindError::InvalidMedium) => Err(BindError::InvalidMedium),
        }
    }

    /// Bind the socket to an interface, or unbind it with `None`.
    ///
    /// A socket bound to an interface only sends and receives packets on it:
    /// - Destinations must be on-link on that iface, or have a route through it.
    /// - Broadcast and multicast destinations go out on that iface only
    ///
    /// In Ethernet mode the bound interface's medium must be
    /// [`Medium::Ethernet`](crate::iface::Medium::Ethernet), checked at [`bind`](Self::bind).
    ///
    /// The socket must be unbound (no mode set). The binding is kept across
    /// [`close`](Self::close).
    ///
    /// # Errors
    /// - `InvalidState`: if the socket is bound.
    #[cfg(feature = "iface-bind")]
    pub fn bind_to_iface(&mut self, iface: Option<IfaceHandle>) -> Result<(), BindError> {
        match self.with_mut(|s| s.bind_to_iface(iface)) {
            Ok(()) => Ok(()),
            Err(raw::BindError::InvalidState) => Err(BindError::InvalidState),
            #[cfg(feature = "raw-ethernet")]
            Err(raw::BindError::InvalidMedium) => unreachable!(),
        }
    }

    /// Return the interface the socket is bound to, or `None`.
    ///
    /// See [`bind_to_iface`](Self::bind_to_iface).
    #[cfg(feature = "iface-bind")]
    pub fn bound_iface(&self) -> Option<IfaceHandle> {
        self.with(|s| s.bound_iface())
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

    /// Dequeue a received packet, copying it into the given slice, and return the
    /// number of octets copied.
    ///
    /// This method will wait until a packet is received.
    ///
    /// # Errors
    /// - `InvalidState`: if the socket is not bound.
    /// - `Truncated`: if `buf` is smaller than the packet. The packet is
    ///   dropped.
    pub fn recv<'s>(&'s self, buf: &'s mut [u8]) -> impl Future<Output = Result<usize, RecvError>> + 's {
        poll_fn(|cx| self.poll_recv(buf, cx))
    }

    /// Dequeue a received packet, copying it into the given slice, and return the
    /// number of octets copied.
    ///
    /// This method will not wait for a packet to be received.
    ///
    /// # Errors
    /// - `WouldBlock`: if the RX queue is empty.
    /// - `InvalidState`: if the socket is not bound.
    /// - `Truncated`: if `buf` is smaller than the packet. The packet is
    ///   dropped.
    pub fn try_recv(&self, buf: &mut [u8]) -> Result<usize, TryError<RecvError>> {
        self.with_mut(|s| match s.recv_slice(buf) {
            Ok(n) => Ok(n),
            Err(raw::RecvError::InvalidState) => Err(TryError::Other(RecvError::InvalidState)),
            Err(raw::RecvError::Truncated) => Err(TryError::Other(RecvError::Truncated)),
            Err(raw::RecvError::Exhausted) => Err(TryError::WouldBlock),
        })
    }

    /// Dequeue a received packet, copying it into the given slice, and return the
    /// number of octets copied.
    ///
    /// # Errors
    /// - `InvalidState`: if the socket is not bound.
    /// - `Truncated`: if `buf` is smaller than the packet. The packet is
    ///   dropped.
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

    /// Dequeue a received packet.
    ///
    /// The buffer holds the whole Ethernet frame (Ethernet mode) or IP packet (IP
    /// mode), headers included, exactly as received. This is zero-copy: the
    /// returned value is the buffer the packet arrived in, and dropping it frees it.
    ///
    /// This method will wait until a packet is received.
    ///
    /// # Errors
    /// - `InvalidState`: if the socket is not bound.
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
    pub async fn recv_with<R>(&mut self, f: impl FnOnce(&[u8], PacketMeta) -> R) -> Result<R, RecvError> {
        let packet = self.recv_packet().await?;
        Ok(f(&packet, packet.meta()))
    }

    /// Peek at the next received packet without dequeueing it, copying it into the
    /// given slice.
    ///
    /// This method will wait until a packet is received.
    ///
    /// # Errors
    /// - `InvalidState`: if the socket is not bound.
    /// - `Truncated`: if `buf` is smaller than the packet. No data is copied
    ///   and the packet stays in the queue.
    pub fn peek<'s>(&'s self, buf: &'s mut [u8]) -> impl Future<Output = Result<usize, RecvError>> + 's {
        poll_fn(|cx| self.poll_peek(buf, cx))
    }

    /// Peek at the next received packet without dequeueing it, copying it into the
    /// given slice.
    ///
    /// This method will not wait for a packet to be received.
    ///
    /// # Errors
    /// - `WouldBlock`: if no packet is available.
    /// - `Other(InvalidState)`: if the socket is not bound.
    /// - `Other(Truncated)`: if `buf` is smaller than the packet. No data is copied
    ///   and the packet stays in the queue.
    pub fn try_peek(&self, buf: &mut [u8]) -> Result<usize, TryError<RecvError>> {
        self.with(|s| match s.peek_slice(buf) {
            Ok(n) => Ok(n),
            Err(raw::RecvError::InvalidState) => Err(TryError::Other(RecvError::InvalidState)),
            Err(raw::RecvError::Truncated) => Err(TryError::Other(RecvError::Truncated)),
            Err(raw::RecvError::Exhausted) => Err(TryError::WouldBlock),
        })
    }

    /// Peek at the next received packet without dequeueing it, copying it into the
    /// given slice.
    ///
    /// When no packet is available, this method will return `Poll::Pending` and
    /// register the current task to be notified when a packet is received.
    ///
    /// # Errors
    /// - `InvalidState`: if the socket is not bound.
    /// - `Truncated`: if `buf` is smaller than the packet. No data is copied
    ///   and the packet stays in the queue.
    pub fn poll_peek(&self, buf: &mut [u8], cx: &mut Context<'_>) -> Poll<Result<usize, RecvError>> {
        self.with_mut(|s| match s.peek_slice(buf) {
            Ok(n) => Poll::Ready(Ok(n)),
            Err(raw::RecvError::InvalidState) => Poll::Ready(Err(RecvError::InvalidState)),
            Err(raw::RecvError::Truncated) => Poll::Ready(Err(RecvError::Truncated)),
            Err(raw::RecvError::Exhausted) => {
                s.register_recv_waker(cx.waker());
                Poll::Pending
            }
        })
    }

    /// Peek at the next received packet without dequeueing it, calling `f` with it.
    ///
    /// This method will wait until a packet is received.
    ///
    /// # Errors
    /// - `InvalidState`: if the socket is not bound.
    pub async fn peek_with<R>(&self, f: impl FnOnce(&[u8]) -> R) -> Result<R, RecvError> {
        let mut f = Some(f);
        poll_fn(|cx| {
            self.with_mut(|s| match s.peek() {
                Ok(packet) => Poll::Ready(Ok(unwrap!(f.take())(packet))),
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

    /// Peek at the next received packet without dequeueing it, calling `f` with it.
    ///
    /// This method will not wait for a packet to be received.
    ///
    /// # Errors
    /// - `WouldBlock`: if no packet is available.
    /// - `Other(InvalidState)`: if the socket is not bound.
    pub fn try_peek_with<R>(&self, f: impl FnOnce(&[u8]) -> R) -> Result<R, TryError<RecvError>> {
        self.with(|s| match s.peek() {
            Ok(packet) => Ok(f(packet)),
            Err(raw::RecvError::InvalidState) => Err(TryError::Other(RecvError::InvalidState)),
            Err(raw::RecvError::Truncated) => unreachable!(),
            Err(raw::RecvError::Exhausted) => Err(TryError::WouldBlock),
        })
    }

    /// Check whether the RX queue is not empty.
    pub fn can_recv(&self) -> bool {
        self.with(|s| s.can_recv())
    }

    /// Wait until the socket is bound.
    ///
    /// This does not check packet buffer availability or device transmit room.
    pub fn wait_send_ready(&self) -> impl Future<Output = ()> + '_ {
        poll_fn(|cx| self.poll_send_ready(cx))
    }

    /// Poll until the socket is bound.
    ///
    /// This does not check packet buffer availability or device transmit room.
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

    /// Send a packet, copying it from a slice.
    ///
    /// This method will wait until the packet has been sent.
    ///
    /// See [send_with](#method.send_with).
    pub async fn send(&self, buf: &[u8]) -> Result<(), SendError> {
        self.send_with_meta(buf, PacketMeta::default()).await
    }

    /// Send a packet with the given [`PacketMeta`] attached, copying it from a slice.
    ///
    /// The metadata is handed to the driver along with the frame. This is how a
    /// packet is tagged with an id, or a transmit timestamp is requested for it (see
    /// [`Stack::poll_tx_timestamp`]). Everything else is exactly
    /// [`send`](Self::send).
    ///
    /// This method will wait until the packet has been sent.
    pub async fn send_with_meta(&self, buf: &[u8], meta: PacketMeta) -> Result<(), SendError> {
        poll_fn(move |cx| self.poll_send_with_meta(buf, meta, cx)).await
    }

    /// Send a packet, copying it from a slice.
    ///
    /// This method will not wait for a packet buffer or device room to become free.
    ///
    /// See [send_with](#method.send_with).
    ///
    /// # Errors
    /// - `WouldBlock`: if every packet buffer is in use, or the interface the
    ///   packet would go out of has no room for it right now.
    pub fn try_send(&self, buf: &[u8]) -> Result<(), TryError<SendError>> {
        self.with_mut(|s| match Self::map_send(s.send_slice(buf)) {
            Poll::Ready(r) => r.map_err(TryError::Other),
            Poll::Pending => Err(TryError::WouldBlock),
        })
    }

    /// Send a packet with the given [`PacketMeta`] attached, copying it from a slice.
    ///
    /// See [send_with_meta](#method.send_with_meta).
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

    /// Send a packet, building it in place.
    ///
    /// The closure gets a `max_size`-byte slice inside a freshly allocated packet
    /// buffer, and returns how many bytes it wrote, along with a value that is
    /// returned from this method. The packet is then sent immediately.
    ///
    /// This method will wait until a packet buffer is available before passing
    /// it to the closure.
    ///
    /// The packet must be complete, headers included: a whole Ethernet frame (at
    /// most 1514 octets) in Ethernet mode, a whole IP packet (at most 1500 octets,
    /// or the full 1514 in a build without `medium-ethernet`, which reserves no
    /// link-layer headroom) in IP mode. It is emitted exactly as written, so the
    /// user is responsible for every header field, including the IPv4 header
    /// checksum.
    ///
    /// In Ethernet mode the frame is transmitted as-is, on the bound interface
    /// if the socket is bound to one, else on the first Ethernet interface. In
    /// IP mode the destination address is read from the IP header, and the
    /// packet is routed like any other egress packet (through the bound
    /// interface only, if the socket is bound to one). If the destination's
    /// neighbor is unresolved, the packet is queued inside the stack and sent
    /// when resolution completes. This still counts as a successful send.
    ///
    /// # Errors
    /// - `InvalidState`: if the socket is not bound.
    /// - `Unaddressable`: if there is no route to the
    ///   packet's destination or the destination is the unspecified address (IP
    ///   mode), or no Ethernet interface to send on (Ethernet mode).
    /// - `Malformed`: if the packet fails basic validation (too short for an
    ///   Ethernet header in Ethernet mode, malformed IP header in IP mode), or
    ///   does not match the socket's bind filters.
    /// - `BufferFull`: if the packet cannot fit in a packet buffer.
    ///
    /// # Panics
    /// Panics if the socket is bound to an interface that has been removed.
    pub async fn send_with<R>(
        &mut self,
        max_size: usize,
        f: impl FnOnce(&mut [u8]) -> (usize, R),
    ) -> Result<R, SendError> {
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

    /// Check whether the socket is open (bound to a mode).
    pub fn is_open(&self) -> bool {
        self.with(|s| s.is_open())
    }

    /// Return the mode the socket is bound to, or `None` if it is unbound.
    pub fn mode(&self) -> Option<RawMode> {
        self.with(|s| s.mode())
    }

    /// Close the socket, unbinding it and dropping any queued packets.
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

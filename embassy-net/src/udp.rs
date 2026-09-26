//! UDP sockets.
//!
//! How to use:
//!
//! - Create a UDP socket with [`UdpSocket::new`]
//! - [`bind`](UdpSocket::bind) it to a local address and optionally also a remote address.
//! - Send and receive packets.

use core::future::{Future, poll_fn};
use core::task::{Context, Poll};

pub use xarxa::driver::PacketMeta;
use xarxa::error::InvalidHopLimit;
#[cfg(feature = "iface-bind")]
pub use xarxa::iface::IfaceHandle;
use xarxa::udp::{self, UdpHandle};
pub use xarxa::udp::{RecvPacket, UdpMetadata};
use xarxa::wire::ListenSocketAddr;

use crate::error::Full;
use crate::wire::SocketAddr;
use crate::{NoWake, Stack, TryError, Wake, WakeRunner, wake_if};

/// Error returned by [`UdpSocket::bind`].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BindError {
    /// The socket is already bound.
    InvalidState,
    /// Another UDP socket holds an identical 4-tuple.
    InUse,
    /// No free port in the ephemeral range (only possible with tens of thousands
    /// of bound sockets).
    NoFreePorts,
    /// The local and remote addresses belong to different address families, or no
    /// local address is available for the given remote.
    Unaddressable,
}

/// Error returned by [`UdpSocket::send_to`].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SendError {
    /// The socket is not bound.
    InvalidState,
    /// The destination address or port is unspecified, or no matching source
    /// address is available.
    Unaddressable,
    /// The payload does not fit in a packet buffer.
    BufferFull,
}

/// Error returned by [`UdpSocket::recv_from`].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RecvError {
    /// The socket is not bound.
    InvalidState,
    /// The provided slice is smaller than the payload. (The packet is dropped.)
    Truncated,
    /// An ICMP error message quoting a packet this socket sent has arrived
    /// (reported once, taking it clears it).
    #[cfg(feature = "icmp-errors")]
    IcmpError {
        /// The kind of error.
        error: crate::error::IcmpError,
        /// The remote address the erring packet was sent to.
        remote: SocketAddr,
    },
}

/// An UDP socket.
pub struct UdpSocket<'d> {
    stack: Stack<'d>,
    handle: UdpHandle,
}

impl<'d> UdpSocket<'d> {
    /// Create a new UDP socket using the provided stack.
    ///
    /// Errors:
    /// - `Full` if the stack has no room for another UDP socket. The limit is set
    ///   by the `udp-socket-count-N` feature of `xarxa`.
    pub fn new(stack: Stack<'d>) -> Result<Self, Full> {
        let handle = stack.with(|i| (i.stack.add_udp_socket(), NoWake))?;

        Ok(Self { stack, handle })
    }

    /// Bind the socket.
    ///
    /// This method opens the socket and configures which packets it will
    /// send and receive. It is equivalent to `bind()` and/or `connect()` in the
    /// BSD / Linux socket API.
    ///
    /// # Local address
    ///
    /// - `None`: the socket sends/receives packets with any local address. It receives packets destined to unicast, multicast and broadcast addresses.
    /// - `Some(V4(UNSPECIFIED))`: same as `None`, but IPv4 packets only.
    /// - `Some(V6(UNSPECIFIED))`: same as `None`, but IPv6 packets only.
    /// - `Some(_)`: the socket sends/receives packets from/to the given local address only.
    ///   - If unicast, the stack checks the address is ours, else returns `Unaddressable`.
    ///   - Multicast and broadcast addresses are allowed, but then the socket can receive only, not send.
    ///
    /// # Local port
    ///
    /// - `0`: the stack allocates an unused ephemeral port. You can retrieve it with [`local_addr`](Self::local_addr).
    /// - non-zero: the given port is used.
    ///
    /// The socket only receives packets to that port and sends from that port. A UDP socket *must* be bound to at least a port, there's no way to make a UDP socket listen on all ports.
    ///
    /// # Remote address
    ///
    /// - `None`: the socket sends/receives packets with any remote address.
    /// - `Some(V4(UNSPECIFIED))`: same as `None`, but IPv4 packets only.
    /// - `Some(V6(UNSPECIFIED))`: same as `None`, but IPv6 packets only.
    /// - `Some(_)`: the socket sends/receives packets to/from the given remote address only. Multicast and broadcast addresses are allowed, but then the socket can send only, not receive.
    ///
    /// # Remote port
    ///
    /// - `0`: the socket sends/receives packets to/from any remote port.
    /// - non-zero: the socket sends/receives packets to/from the given port only.
    ///
    /// Overlapping bindings between sockets are allowed as long as they're
    /// not identical. For example you can bind a socket to `*:53` and another to
    /// `1.2.3.4:53`, but you can't bind two sockets to `*:53`. If a packet
    /// matches multiple sockets, the one with the most specific binding wins.
    /// Packets are not duplicated, only the winning socket will receive it.
    ///
    /// Multicast groups are not joined automatically, you must call [`Iface::join_multicast_group`](crate::iface::Iface::join_multicast_group) yourself.
    ///
    /// # Errors
    /// - `InvalidState`: if the socket is already bound (see
    ///   [is_open](#method.is_open)).
    /// - `InUse`: on an identical bind.
    /// - `NoFreePorts`: if the ephemeral range is exhausted.
    /// - `Unaddressable`: on an address family mismatch, if the local address is
    ///   not ours, or if no local address is available for the given
    ///   remote.
    pub fn bind(
        &mut self,
        local: impl Into<ListenSocketAddr>,
        remote: impl Into<ListenSocketAddr>,
    ) -> Result<(), BindError> {
        match self.with(|s| (s.bind(local, remote), NoWake)) {
            Ok(()) => Ok(()),
            Err(udp::BindError::InvalidState) => Err(BindError::InvalidState),
            Err(udp::BindError::InUse) => Err(BindError::InUse),
            Err(udp::BindError::NoFreePorts) => Err(BindError::NoFreePorts),
            Err(udp::BindError::Unaddressable) => Err(BindError::Unaddressable),
        }
    }

    /// Bind the socket to an interface, or unbind it with `None`.
    ///
    /// A socket bound to an interface only sends and receives packets on it:
    /// - Destinations must be on-link on that iface, or have a route through it.
    /// - Broadcast and multicast destinations go out on that iface only
    /// - Local addresses will be picked from that iface only.
    ///
    /// The socket must be closed. The binding is kept across
    /// [`close`](Self::close), so a socket stays bound to its interface when
    /// it is bound again.
    ///
    /// Two sockets with otherwise identical tuples may coexist if they are
    /// bound to different interfaces. On ingress, a socket bound to the
    /// arrival interface wins over an unbound one with an equal tuple.
    ///
    /// # Errors
    /// - `InvalidState`: if the socket is open.
    #[cfg(feature = "iface-bind")]
    pub fn bind_to_iface(&mut self, iface: Option<IfaceHandle>) -> Result<(), BindError> {
        match self.with(|s| (s.bind_to_iface(iface), NoWake)) {
            Ok(()) => Ok(()),
            Err(udp::BindError::InvalidState) => Err(BindError::InvalidState),
            Err(_) => unreachable!(),
        }
    }

    /// Return the interface the socket is bound to, or `None`.
    ///
    /// See [`bind_to_iface`](Self::bind_to_iface).
    #[cfg(feature = "iface-bind")]
    pub fn bound_iface(&self) -> Option<IfaceHandle> {
        self.with(|s| (s.bound_iface(), NoWake))
    }

    fn with<R>(&self, f: impl FnOnce(&mut udp::UdpSocket<'_, 'd>) -> (R, WakeRunner)) -> R {
        self.stack.with(|i| f(&mut i.stack.udp_socket(self.handle)))
    }

    /// Wait until the socket becomes readable.
    ///
    /// A socket is readable when a packet has been received, or when there are queued packets in
    /// the buffer.
    pub fn wait_recv_ready(&self) -> impl Future<Output = ()> + '_ {
        poll_fn(move |cx| self.poll_recv_ready(cx))
    }

    /// Wait until a datagram can be read.
    ///
    /// When no datagram is readable, this method will return `Poll::Pending` and
    /// register the current task to be notified when a datagram is received.
    ///
    /// When a datagram is received, this method will return `Poll::Ready`.
    pub fn poll_recv_ready(&self, cx: &mut Context<'_>) -> Poll<()> {
        self.with(|s| {
            (
                if s.can_recv() {
                    Poll::Ready(())
                } else {
                    // socket buffer is empty wait until at least one byte has arrived
                    s.register_recv_waker(cx.waker());
                    Poll::Pending
                },
                NoWake,
            )
        })
    }

    /// Dequeue a received datagram, copying the payload into the given slice, and
    /// return the number of octets copied along with its metadata.
    ///
    /// This method will wait until a datagram is received.
    ///
    /// See also [recv](#method.recv).
    ///
    /// # Errors
    /// - `InvalidState`: if the socket is not bound.
    /// - `Truncated`: if `buf` is smaller than the payload. The packet is
    ///   dropped.
    /// - `IcmpError`: with the `icmp-errors` feature, if an ICMP error is
    ///   pending. See [recv](#method.recv).
    pub fn recv_from<'s>(
        &'s self,
        buf: &'s mut [u8],
    ) -> impl Future<Output = Result<(usize, UdpMetadata), RecvError>> + 's {
        poll_fn(|cx| self.poll_recv_from(buf, cx))
    }

    /// Dequeue a received datagram, copying the payload into the given slice, and
    /// return the number of octets copied along with its metadata.
    ///
    /// This method will not wait for a datagram to be received.
    ///
    /// See also [try_recv](#method.try_recv).
    ///
    /// # Errors
    /// - `WouldBlock`: if no datagram is available.
    /// - `Other(InvalidState)`: if the socket is not bound.
    /// - `Other(Truncated)`: if `buf` is smaller than the payload. The packet is
    ///   dropped.
    /// - `Other(IcmpError)`: with the `icmp-errors` feature, if an ICMP error is
    ///   pending. See [recv](#method.recv).
    pub fn try_recv_from(&self, buf: &mut [u8]) -> Result<(usize, UdpMetadata), TryError<RecvError>> {
        self.with(|s| match s.recv_slice(buf) {
            Ok((n, meta)) => (Ok((n, meta)), Wake),
            Err(udp::RecvError::InvalidState) => (Err(TryError::Other(RecvError::InvalidState)), NoWake),
            #[cfg(feature = "icmp-errors")]
            Err(udp::RecvError::IcmpError { error, remote }) => {
                (Err(TryError::Other(RecvError::IcmpError { error, remote })), NoWake)
            }
            Err(udp::RecvError::Truncated) => (Err(TryError::Other(RecvError::Truncated)), Wake),
            Err(udp::RecvError::Exhausted) => (Err(TryError::WouldBlock), NoWake),
        })
    }

    /// Dequeue a received datagram, copying the payload into the given slice, and
    /// return the number of octets copied along with its metadata.
    ///
    /// When no datagram is available, this method will return `Poll::Pending` and
    /// register the current task to be notified when a datagram is received.
    ///
    /// When a datagram is received, this method will return `Poll::Ready` with the
    /// number of bytes received and the remote address.
    ///
    /// See also [recv](#method.recv).
    ///
    /// # Errors
    /// - `InvalidState`: if the socket is not bound.
    /// - `Truncated`: if `buf` is smaller than the payload. The packet is
    ///   dropped.
    /// - `IcmpError`: with the `icmp-errors` feature, if an ICMP error is
    ///   pending. See [recv](#method.recv).
    pub fn poll_recv_from(
        &self,
        buf: &mut [u8],
        cx: &mut Context<'_>,
    ) -> Poll<Result<(usize, UdpMetadata), RecvError>> {
        self.with(|s| match s.recv_slice(buf) {
            Ok((n, meta)) => (Poll::Ready(Ok((n, meta))), Wake),
            Err(udp::RecvError::InvalidState) => (Poll::Ready(Err(RecvError::InvalidState)), NoWake),
            #[cfg(feature = "icmp-errors")]
            Err(udp::RecvError::IcmpError { error, remote }) => {
                (Poll::Ready(Err(RecvError::IcmpError { error, remote })), NoWake)
            }
            Err(udp::RecvError::Truncated) => (Poll::Ready(Err(RecvError::Truncated)), Wake),
            // No data ready
            Err(udp::RecvError::Exhausted) => {
                s.register_recv_waker(cx.waker());
                (Poll::Pending, NoWake)
            }
        })
    }

    /// Dequeue a received datagram, as an owned packet ([`RecvPacket`]).
    ///
    /// This is zero-copy: the returned value is the buffer the datagram arrived in.
    ///
    /// This method will wait until a datagram is received.
    ///
    /// # Errors
    /// - `InvalidState`: if the socket is not bound.
    /// - `IcmpError`: with the `icmp-errors` feature, if an ICMP error is
    ///   pending. It is reported before any queued datagram, once, and taking
    ///   it clears it.
    pub async fn recv(&self) -> Result<RecvPacket, RecvError> {
        poll_fn(|cx| {
            self.with(|s| match s.recv() {
                Ok(packet) => (Poll::Ready(Ok(packet)), Wake),
                Err(udp::RecvError::InvalidState) => (Poll::Ready(Err(RecvError::InvalidState)), NoWake),
                #[cfg(feature = "icmp-errors")]
                Err(udp::RecvError::IcmpError { error, remote }) => {
                    (Poll::Ready(Err(RecvError::IcmpError { error, remote })), NoWake)
                }
                Err(udp::RecvError::Truncated) => unreachable!(),
                Err(udp::RecvError::Exhausted) => {
                    s.register_recv_waker(cx.waker());
                    (Poll::Pending, NoWake)
                }
            })
        })
        .await
    }

    /// Dequeue a received datagram, as an owned packet ([`RecvPacket`]).
    ///
    /// This is zero-copy: the returned value is the buffer the datagram arrived in.
    ///
    /// This method will not wait for a datagram to be received.
    ///
    /// # Errors
    /// - `WouldBlock`: if no datagram is available.
    /// - `Other(InvalidState)`: if the socket is not bound.
    /// - `Other(IcmpError)`: with the `icmp-errors` feature, if an ICMP error is
    ///   pending. It is reported before any queued datagram, once, and taking
    ///   it clears it.
    pub fn try_recv(&self) -> Result<RecvPacket, TryError<RecvError>> {
        self.with(|s| match s.recv() {
            Ok(packet) => (Ok(packet), Wake),
            Err(udp::RecvError::InvalidState) => (Err(TryError::Other(RecvError::InvalidState)), NoWake),
            #[cfg(feature = "icmp-errors")]
            Err(udp::RecvError::IcmpError { error, remote }) => {
                (Err(TryError::Other(RecvError::IcmpError { error, remote })), NoWake)
            }
            Err(udp::RecvError::Truncated) => unreachable!(),
            Err(udp::RecvError::Exhausted) => (Err(TryError::WouldBlock), NoWake),
        })
    }

    /// Receive a datagram with a zero-copy function.
    ///
    /// When no datagram is available, this method will return `Poll::Pending` and
    /// register the current task to be notified when a datagram is received.
    ///
    /// When a datagram is received, this method will call the provided function
    /// with a reference to the received bytes and the remote address and return
    /// `Poll::Ready` with the function's returned value.
    pub async fn recv_from_with<R>(&mut self, f: impl FnOnce(&[u8], UdpMetadata) -> R) -> Result<R, RecvError> {
        let packet = self.recv().await?;
        Ok(f(packet.payload(), packet.meta()))
    }

    /// Receive a datagram with a zero-copy function.
    ///
    /// This method will not wait for a datagram to be received.
    ///
    /// If no datagram is available, this method will return `Err(TryError::WouldBlock)`.
    pub fn try_recv_from_with<R>(&mut self, f: impl FnOnce(&[u8], UdpMetadata) -> R) -> Result<R, TryError<RecvError>> {
        let packet = self.try_recv()?;
        Ok(f(packet.payload(), packet.meta()))
    }

    /// Peek at the next received datagram without dequeueing it, copying the payload
    /// into the given slice.
    ///
    /// This method will wait until a datagram is received.
    ///
    /// # Errors
    /// - `InvalidState`: if the socket is not bound.
    /// - `Truncated`: if `buf` is smaller than the payload. No data is copied
    ///   and the packet stays in the queue.
    pub fn peek_from<'s>(
        &'s self,
        buf: &'s mut [u8],
    ) -> impl Future<Output = Result<(usize, UdpMetadata), RecvError>> + 's {
        poll_fn(|cx| self.poll_peek_from(buf, cx))
    }

    /// Peek at the next received datagram without dequeueing it, copying the payload
    /// into the given slice.
    ///
    /// This method will not wait for a datagram to be received.
    ///
    /// # Errors
    /// - `WouldBlock`: if no datagram is available.
    /// - `Other(InvalidState)`: if the socket is not bound.
    /// - `Other(Truncated)`: if `buf` is smaller than the payload. No data is copied
    ///   and the packet stays in the queue.
    pub fn try_peek_from(&self, buf: &mut [u8]) -> Result<(usize, UdpMetadata), TryError<RecvError>> {
        self.with(|s| {
            (
                match s.peek_slice(buf) {
                    Ok((n, meta)) => Ok((n, meta)),
                    Err(udp::RecvError::InvalidState) => Err(TryError::Other(RecvError::InvalidState)),
                    Err(udp::RecvError::Truncated) => Err(TryError::Other(RecvError::Truncated)),
                    Err(udp::RecvError::Exhausted) => Err(TryError::WouldBlock),
                    #[cfg(feature = "icmp-errors")]
                    Err(udp::RecvError::IcmpError { .. }) => unreachable!(),
                },
                NoWake,
            )
        })
    }

    /// Peek at the next received datagram without dequeueing it, copying the payload
    /// into the given slice.
    ///
    /// When no datagram is available, this method will return `Poll::Pending` and
    /// register the current task to be notified when a datagram is received.
    ///
    /// # Errors
    /// - `InvalidState`: if the socket is not bound.
    /// - `Truncated`: if `buf` is smaller than the payload. No data is copied
    ///   and the packet stays in the queue.
    pub fn poll_peek_from(
        &self,
        buf: &mut [u8],
        cx: &mut Context<'_>,
    ) -> Poll<Result<(usize, UdpMetadata), RecvError>> {
        self.with(|s| {
            (
                match s.peek_slice(buf) {
                    Ok((n, meta)) => Poll::Ready(Ok((n, meta))),
                    Err(udp::RecvError::InvalidState) => Poll::Ready(Err(RecvError::InvalidState)),
                    Err(udp::RecvError::Truncated) => Poll::Ready(Err(RecvError::Truncated)),
                    Err(udp::RecvError::Exhausted) => {
                        s.register_recv_waker(cx.waker());
                        Poll::Pending
                    }
                    #[cfg(feature = "icmp-errors")]
                    Err(udp::RecvError::IcmpError { .. }) => unreachable!(),
                },
                NoWake,
            )
        })
    }

    /// Peek at the next received datagram without dequeueing it, calling `f` with
    /// its payload and its metadata.
    ///
    /// This method will wait until a datagram is received.
    ///
    /// # Errors
    /// - `InvalidState`: if the socket is not bound.
    pub async fn peek_from_with<R>(&self, f: impl FnOnce(&[u8], UdpMetadata) -> R) -> Result<R, RecvError> {
        let mut f = Some(f);
        poll_fn(|cx| {
            self.with(|s| {
                (
                    match s.peek() {
                        Ok((payload, meta)) => Poll::Ready(Ok(unwrap!(f.take())(payload, meta))),
                        Err(udp::RecvError::InvalidState) => Poll::Ready(Err(RecvError::InvalidState)),
                        Err(udp::RecvError::Exhausted) => {
                            s.register_recv_waker(cx.waker());
                            Poll::Pending
                        }
                        Err(udp::RecvError::Truncated) => unreachable!(),
                        #[cfg(feature = "icmp-errors")]
                        Err(udp::RecvError::IcmpError { .. }) => unreachable!(),
                    },
                    NoWake,
                )
            })
        })
        .await
    }

    /// Peek at the next received datagram without dequeueing it, calling `f` with
    /// its payload and its metadata.
    ///
    /// This method will not wait for a datagram to be received.
    ///
    /// # Errors
    /// - `WouldBlock`: if no datagram is available.
    /// - `Other(InvalidState)`: if the socket is not bound.
    pub fn try_peek_from_with<R>(&self, f: impl FnOnce(&[u8], UdpMetadata) -> R) -> Result<R, TryError<RecvError>> {
        self.with(|s| {
            (
                match s.peek() {
                    Ok((payload, meta)) => Ok(f(payload, meta)),
                    Err(udp::RecvError::InvalidState) => Err(TryError::Other(RecvError::InvalidState)),
                    Err(udp::RecvError::Exhausted) => Err(TryError::WouldBlock),
                    Err(udp::RecvError::Truncated) => unreachable!(),
                    #[cfg(feature = "icmp-errors")]
                    Err(udp::RecvError::IcmpError { .. }) => unreachable!(),
                },
                NoWake,
            )
        })
    }

    /// Make one send attempt with `f`, and arrange to be polled again if it has to
    /// be retried.
    ///
    /// A datagram that went out wakes the runner: it may be parked on a neighbor
    /// resolution, or have left fragments behind. A refused one changes nothing the
    /// runner acts on, so it doesn't.
    fn poll_send<R>(
        &self,
        cx: &mut Context<'_>,
        f: impl FnOnce(&mut udp::UdpSocket<'_, 'd>) -> Result<R, udp::SendError>,
    ) -> Poll<Result<R, SendError>> {
        self.with(|s| match f(s) {
            Ok(r) => (Poll::Ready(Ok(r)), Wake),
            // xarxa wakes us when the device has room.
            Err(udp::SendError::DeviceBusy) => {
                s.register_send_waker(cx.waker());
                (Poll::Pending, NoWake)
            }
            // Nothing signals a freed buffer. Yield, and try again.
            Err(udp::SendError::NoBuffer) => {
                cx.waker().wake_by_ref();
                (Poll::Pending, NoWake)
            }
            Err(udp::SendError::BufferFull) => (Poll::Ready(Err(SendError::BufferFull)), NoWake),
            Err(udp::SendError::InvalidState) => (Poll::Ready(Err(SendError::InvalidState)), NoWake),
            Err(udp::SendError::Unaddressable) => (Poll::Ready(Err(SendError::Unaddressable)), NoWake),
        })
    }

    /// Make one send attempt with `f`, without waiting. Like
    /// [`poll_send`](Self::poll_send), only a datagram that went out wakes the runner.
    fn try_send_inner<R>(
        &self,
        f: impl FnOnce(&mut udp::UdpSocket<'_, 'd>) -> Result<R, udp::SendError>,
    ) -> Result<R, TryError<SendError>> {
        self.with(|s| match f(s) {
            Ok(r) => (Ok(r), Wake),
            Err(udp::SendError::DeviceBusy | udp::SendError::NoBuffer) => (Err(TryError::WouldBlock), NoWake),
            Err(udp::SendError::BufferFull) => (Err(TryError::Other(SendError::BufferFull)), NoWake),
            Err(udp::SendError::InvalidState) => (Err(TryError::Other(SendError::InvalidState)), NoWake),
            Err(udp::SendError::Unaddressable) => (Err(TryError::Other(SendError::Unaddressable)), NoWake),
        })
    }

    /// Send a datagram to the given remote address, copying the payload from a slice.
    ///
    /// This method will wait until the datagram has been sent.
    ///
    /// See [send_to_with](#method.send_to_with).
    ///
    /// # Errors
    /// - `InvalidState`: if the socket is not bound.
    /// - `Unaddressable`: if the destination address or port is still
    ///   unspecified after defaulting, the destination's address family does not
    ///   match the source address, no source address is available, or the source
    ///   address is not assigned to any interface.
    /// - `BufferFull`: if the payload cannot fit in a packet buffer.
    pub async fn send_to(&self, buf: &[u8], remote: impl Into<UdpMetadata>) -> Result<(), SendError> {
        let remote: UdpMetadata = remote.into();
        poll_fn(move |cx| self.poll_send_to(buf, remote, cx)).await
    }

    /// Send a datagram to the given remote address, copying the payload from a slice.
    ///
    /// This method will not wait for a packet buffer or device room to become free.
    ///
    /// See [send_to_with](#method.send_to_with).
    ///
    /// # Errors
    /// - `WouldBlock`: if every packet buffer is in use, or the interface the
    ///   datagram would go out of has no room for it right now.
    /// - `Other(InvalidState)`: if the socket is not bound.
    /// - `Other(Unaddressable)`: if the destination address or port is still
    ///   unspecified after defaulting, the destination's address family does not
    ///   match the source address, no source address is available, or the source
    ///   address is not assigned to any interface.
    /// - `Other(BufferFull)`: if the payload cannot fit in a packet buffer.
    pub fn try_send_to(&self, buf: &[u8], remote: impl Into<UdpMetadata>) -> Result<(), TryError<SendError>> {
        let remote: UdpMetadata = remote.into();
        self.try_send_inner(|s| s.send_slice(buf, remote))
    }

    /// Send a datagram to the given remote address, copying the payload from a slice.
    ///
    /// When the datagram has been sent, this method will return `Poll::Ready(Ok())`.
    ///
    /// When the datagram cannot be sent right now, this method will return `Poll::Pending`
    /// and register the current task to be notified when it can.
    ///
    /// See [send_to_with](#method.send_to_with).
    ///
    /// # Errors
    /// - `InvalidState`: if the socket is not bound.
    /// - `Unaddressable`: if the destination address or port is still
    ///   unspecified after defaulting, the destination's address family does not
    ///   match the source address, no source address is available, or the source
    ///   address is not assigned to any interface.
    /// - `BufferFull`: if the payload cannot fit in a packet buffer.
    pub fn poll_send_to(
        &self,
        buf: &[u8],
        remote: impl Into<UdpMetadata>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), SendError>> {
        let remote: UdpMetadata = remote.into();
        self.poll_send(cx, |s| s.send_slice(buf, remote))
    }

    /// Send a datagram, building the payload in place.
    ///
    /// The destination is `remote.remote_addr`, with unspecified parts defaulted from
    /// the socket's bound remote address. On a connected socket, sending to
    /// `SocketAddr::UNSPECIFIED` sends to the connected remote. An explicitly
    /// specified destination is honored even on a connected socket.
    ///
    /// The closure gets a `max_size`-byte slice inside a freshly allocated packet
    /// buffer, and returns how many bytes it wrote, along with a value that this
    /// method returns. The datagram is then sent immediately. If the destination's
    /// neighbor is unresolved, the packet is queued inside the stack and sent when
    /// resolution completes. This still counts as a successful send.
    ///
    /// This method will wait until a packet buffer is available before passing
    /// it to the closure.
    ///
    /// `remote.meta` is attached to the packet and handed to the driver with it: an id
    /// to tag the packet with, or a request to timestamp its transmission (see
    /// [`Stack::poll_tx_timestamp`](crate::Stack::poll_tx_timestamp)).
    ///
    /// # Errors
    /// - `InvalidState`: if the socket is not bound.
    /// - `Unaddressable`: if the destination address or port is still
    ///   unspecified after defaulting, the destination's address family does not
    ///   match the source address, no source address is available, or the source
    ///   address is not assigned to any interface.
    /// - `BufferFull`: if the payload cannot fit in a packet buffer.
    pub async fn send_to_with<R>(
        &mut self,
        max_size: usize,
        remote: impl Into<UdpMetadata> + Copy,
        f: impl FnOnce(&mut [u8]) -> (usize, R),
    ) -> Result<R, SendError> {
        let mut f = Some(f);
        poll_fn(move |cx| {
            self.poll_send(cx, |s| {
                let mut ret = None;
                s.send_with(max_size, remote.into(), |buf| {
                    let (size, r) = unwrap!(f.take())(buf);
                    ret = Some(r);
                    size
                })
                .map(|()| unwrap!(ret))
            })
        })
        .await
    }

    /// Send a datagram, building the payload in place.
    ///
    /// The destination is `remote.remote_addr`, with unspecified parts defaulted from
    /// the socket's bound remote address. On a connected socket, sending to
    /// `SocketAddr::UNSPECIFIED` sends to the connected remote. An explicitly
    /// specified destination is honored even on a connected socket.
    ///
    /// The closure gets a `size`-byte slice inside a freshly allocated packet
    /// buffer, fills it, and returns a value that this method returns. The datagram
    /// is then sent immediately. If the destination's neighbor is unresolved, the
    /// packet is queued inside the stack and sent when resolution completes. This
    /// still counts as a successful send.
    ///
    /// This method will not wait for a packet buffer to become free.
    ///
    /// `remote.meta` is attached to the packet and handed to the driver with it: an id
    /// to tag the packet with, or a request to timestamp its transmission (see
    /// [`Stack::poll_tx_timestamp`](crate::Stack::poll_tx_timestamp)).
    ///
    /// # Errors
    /// - `WouldBlock`: if every packet buffer is in use, or the interface the
    ///   datagram would go out of has no room for it right now.
    /// - `Other(InvalidState)`: if the socket is not bound.
    /// - `Other(Unaddressable)`: if the destination address or port is still
    ///   unspecified after defaulting, the destination's address family does not
    ///   match the source address, no source address is available, or the source
    ///   address is not assigned to any interface.
    /// - `Other(BufferFull)`: if the payload cannot fit in a packet buffer.
    pub fn try_send_to_with<R>(
        &mut self,
        size: usize,
        remote: impl Into<UdpMetadata>,
        f: impl FnOnce(&mut [u8]) -> R,
    ) -> Result<R, TryError<SendError>> {
        let remote: UdpMetadata = remote.into();
        self.try_send_inner(|s| {
            let mut ret = None;
            s.send_with(size, remote, |buf| {
                ret = Some(f(buf));
                size
            })
            .map(|()| unwrap!(ret))
        })
    }

    /// Return the bound local address.
    ///
    /// See [`bind`](Self::bind) for details on how UDP socket binding works.
    ///
    /// Returns `ListenSocketAddr::UNSPECIFIED` if not bound.
    pub fn local_addr(&self) -> ListenSocketAddr {
        self.with(|s| (s.local_addr(), NoWake))
    }

    /// Return the bound remote address.
    ///
    /// See [`bind`](Self::bind) for details on how UDP socket binding works.
    ///
    /// Returns `ListenSocketAddr::UNSPECIFIED` if not bound.
    pub fn remote_addr(&self) -> ListenSocketAddr {
        self.with(|s| (s.remote_addr(), NoWake))
    }

    /// Check whether the socket is open (bound to a port).
    pub fn is_open(&self) -> bool {
        self.with(|s| (s.is_open(), NoWake))
    }

    /// Close the socket, unbinding it and dropping any queued packets.
    pub fn close(&mut self) {
        self.with(|s| {
            let freed = s.can_recv();
            s.close();
            ((), wake_if(freed))
        })
    }

    /// Check whether the RX queue is not empty.
    pub fn can_recv(&self) -> bool {
        self.with(|s| (s.can_recv(), NoWake))
    }

    /// Return the time-to-live (IPv4) or hop limit (IPv6) value used in outgoing packets.
    ///
    /// See also the [set_hop_limit](#method.set_hop_limit) method.
    pub fn hop_limit(&self) -> Option<u8> {
        self.with(|s| (s.hop_limit(), NoWake))
    }

    /// Set the time-to-live (IPv4) or hop limit (IPv6) value used in outgoing packets.
    ///
    /// A socket without an explicitly set hop limit value uses the default [IANA
    /// recommended] value (64).
    ///
    /// # Errors
    /// - `InvalidHopLimit`: if the hop limit is `Some(0)`. A host must not send a
    ///   packet with a hop limit of zero ([RFC 1122 § 3.2.1.7]). The socket is
    ///   left unchanged.
    ///
    /// [IANA recommended]: https://www.iana.org/assignments/ip-parameters/ip-parameters.xhtml
    /// [RFC 1122 § 3.2.1.7]: https://tools.ietf.org/html/rfc1122#section-3.2.1.7
    pub fn set_hop_limit(&mut self, hop_limit: Option<u8>) -> Result<(), InvalidHopLimit> {
        self.with(|s| (s.set_hop_limit(hop_limit), NoWake))
    }
}

impl Drop for UdpSocket<'_> {
    fn drop(&mut self) {
        self.stack.with(|i| {
            let freed = i.stack.udp_socket(self.handle).can_recv();
            i.stack.remove_udp_socket(self.handle);
            ((), wake_if(freed))
        });
    }
}

impl core::fmt::Display for SendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidState => f.write_str("InvalidState"),
            Self::Unaddressable => f.write_str("Unaddressable"),
            Self::BufferFull => f.write_str("BufferFull"),
        }
    }
}
impl core::error::Error for SendError {}

impl core::fmt::Display for RecvError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidState => f.write_str("InvalidState"),
            Self::Truncated => f.write_str("Truncated"),
            #[cfg(feature = "icmp-errors")]
            Self::IcmpError { .. } => f.write_str("IcmpError"),
        }
    }
}
impl core::error::Error for RecvError {}

impl core::fmt::Display for BindError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidState => f.write_str("InvalidState"),
            Self::InUse => f.write_str("InUse"),
            Self::NoFreePorts => f.write_str("NoFreePorts"),
            Self::Unaddressable => f.write_str("Unaddressable"),
        }
    }
}
impl core::error::Error for BindError {}

// Keep `SocketAddr` in scope for the `UdpMetadata: From<SocketAddr>` docs links.
#[allow(unused_imports)]
use SocketAddr as _SocketAddr;

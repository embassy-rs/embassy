//! UDP sockets.

use core::future::{Future, poll_fn};
use core::task::{Context, Poll};

pub use xarxa::driver::PacketMeta;
#[cfg(feature = "iface-bind")]
pub use xarxa::iface::IfaceHandle;
use xarxa::udp::{self, UdpHandle};
pub use xarxa::udp::{RecvPacket, UdpMetadata};
use xarxa::wire::IpListenEndpoint;

use crate::wire::IpEndpoint;
use crate::{Full, Stack, TryError};

/// Error returned by [`UdpSocket::bind`].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BindError {
    /// The socket is already bound.
    InvalidState,
    /// Another socket holds an identical 4-tuple.
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
    /// Provided buffer was smaller than the received packet.
    Truncated,
    /// An ICMP error message quoting a packet this socket sent has arrived
    /// (reported once, taking it clears it).
    #[cfg(feature = "icmp-errors")]
    IcmpError {
        /// The kind of error.
        error: crate::IcmpError,
        /// The remote endpoint the erring packet was sent to.
        remote: IpEndpoint,
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
        let handle = stack.with(|i| i.stack.add_udp_socket())?;

        Ok(Self { stack, handle })
    }

    /// Bind the socket to a local endpoint.
    ///
    /// A port of 0 allocates an ephemeral port.
    pub fn bind<T>(&mut self, endpoint: T) -> Result<(), BindError>
    where
        T: Into<IpListenEndpoint>,
    {
        self.bind_to(endpoint, IpListenEndpoint::UNSPECIFIED)
    }

    /// Bind the socket to a local endpoint, and connect it to a remote one.
    ///
    /// Only datagrams matching the specified parts of `remote` are received, and
    /// unspecified parts of a send's destination default from it.
    pub fn bind_to<L, R>(&mut self, local: L, remote: R) -> Result<(), BindError>
    where
        L: Into<IpListenEndpoint>,
        R: Into<IpListenEndpoint>,
    {
        match self.with_mut(|s| s.bind(local, remote)) {
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
    /// - Destinations must be on-link on that interface, or have a route through it.
    /// - Broadcast and multicast destinations go out on that interface only.
    /// - Local addresses are picked from that interface only.
    ///
    /// The socket must be closed. The binding is kept across [`close`](Self::close),
    /// so a socket stays bound to its interface when it is bound again.
    ///
    /// Two sockets with otherwise identical tuples may coexist if they are bound
    /// to different interfaces. On ingress, a socket bound to the arrival
    /// interface wins over an unbound one with an equal tuple.
    ///
    /// Returns `Err(BindError::InvalidState)` if the socket is open.
    #[cfg(feature = "iface-bind")]
    pub fn bind_to_iface(&mut self, iface: Option<IfaceHandle>) -> Result<(), BindError> {
        match self.with_mut(|s| s.bind_to_iface(iface)) {
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
        self.with(|s| s.bound_iface())
    }

    fn with<R>(&self, f: impl FnOnce(&mut udp::UdpSocket<'_, 'd>) -> R) -> R {
        self.stack.with(|i| f(&mut i.stack.udp_socket(self.handle)))
    }

    fn with_mut<R>(&self, f: impl FnOnce(&mut udp::UdpSocket<'_, 'd>) -> R) -> R {
        self.stack.with_mut(|i| f(&mut i.stack.udp_socket(self.handle)))
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
        self.with_mut(|s| {
            if s.can_recv() {
                Poll::Ready(())
            } else {
                // socket buffer is empty wait until at least one byte has arrived
                s.register_recv_waker(cx.waker());
                Poll::Pending
            }
        })
    }

    /// Receive a datagram.
    ///
    /// This method will wait until a datagram is received.
    ///
    /// If the socket is not bound, this method will return `Err(RecvError::InvalidState)`.
    ///
    /// Returns the number of bytes received and the remote endpoint.
    pub fn recv_from<'s>(
        &'s self,
        buf: &'s mut [u8],
    ) -> impl Future<Output = Result<(usize, UdpMetadata), RecvError>> + 's {
        poll_fn(|cx| self.poll_recv_from(buf, cx))
    }

    /// Receive a datagram.
    ///
    /// This method will not wait for a datagram to be received.
    ///
    /// If no datagram is available, this method will return `Err(TryError::WouldBlock)`.
    ///
    /// Returns the number of bytes received and the remote endpoint.
    pub fn try_recv_from(&self, buf: &mut [u8]) -> Result<(usize, UdpMetadata), TryError<RecvError>> {
        self.with_mut(|s| match s.recv_slice(buf) {
            Ok((n, meta)) => Ok((n, meta)),
            Err(udp::RecvError::InvalidState) => Err(TryError::Other(RecvError::InvalidState)),
            #[cfg(feature = "icmp-errors")]
            Err(udp::RecvError::IcmpError { error, remote }) => {
                Err(TryError::Other(RecvError::IcmpError { error, remote }))
            }
            Err(udp::RecvError::Truncated) => Err(TryError::Other(RecvError::Truncated)),
            Err(udp::RecvError::Exhausted) => Err(TryError::WouldBlock),
        })
    }

    /// Receive a datagram.
    ///
    /// When no datagram is available, this method will return `Poll::Pending` and
    /// register the current task to be notified when a datagram is received.
    ///
    /// When a datagram is received, this method will return `Poll::Ready` with the
    /// number of bytes received and the remote endpoint.
    pub fn poll_recv_from(
        &self,
        buf: &mut [u8],
        cx: &mut Context<'_>,
    ) -> Poll<Result<(usize, UdpMetadata), RecvError>> {
        self.with_mut(|s| match s.recv_slice(buf) {
            Ok((n, meta)) => Poll::Ready(Ok((n, meta))),
            Err(udp::RecvError::InvalidState) => Poll::Ready(Err(RecvError::InvalidState)),
            #[cfg(feature = "icmp-errors")]
            Err(udp::RecvError::IcmpError { error, remote }) => {
                Poll::Ready(Err(RecvError::IcmpError { error, remote }))
            }
            Err(udp::RecvError::Truncated) => Poll::Ready(Err(RecvError::Truncated)),
            // No data ready
            Err(udp::RecvError::Exhausted) => {
                s.register_recv_waker(cx.waker());
                Poll::Pending
            }
        })
    }

    /// Receive a datagram, taking ownership of its packet buffer.
    ///
    /// This method will wait until a datagram is received.
    ///
    /// If the socket is not bound, this method will return `Err(RecvError::InvalidState)`.
    ///
    /// The returned packet derefs to the payload. Dropping it frees the buffer.
    pub async fn recv(&self) -> Result<RecvPacket, RecvError> {
        poll_fn(|cx| {
            self.with_mut(|s| match s.recv() {
                Ok(packet) => Poll::Ready(Ok(packet)),
                Err(udp::RecvError::InvalidState) => Poll::Ready(Err(RecvError::InvalidState)),
                #[cfg(feature = "icmp-errors")]
                Err(udp::RecvError::IcmpError { error, remote }) => {
                    Poll::Ready(Err(RecvError::IcmpError { error, remote }))
                }
                Err(udp::RecvError::Truncated) => unreachable!(),
                Err(udp::RecvError::Exhausted) => {
                    s.register_recv_waker(cx.waker());
                    Poll::Pending
                }
            })
        })
        .await
    }

    /// Receive a datagram, taking ownership of its packet buffer.
    ///
    /// This method will not wait for a datagram to be received.
    ///
    /// If no datagram is available, this method will return `Err(TryError::WouldBlock)`.
    pub fn try_recv(&self) -> Result<RecvPacket, TryError<RecvError>> {
        self.with_mut(|s| match s.recv() {
            Ok(packet) => Ok(packet),
            Err(udp::RecvError::InvalidState) => Err(TryError::Other(RecvError::InvalidState)),
            #[cfg(feature = "icmp-errors")]
            Err(udp::RecvError::IcmpError { error, remote }) => {
                Err(TryError::Other(RecvError::IcmpError { error, remote }))
            }
            Err(udp::RecvError::Truncated) => unreachable!(),
            Err(udp::RecvError::Exhausted) => Err(TryError::WouldBlock),
        })
    }

    /// Receive a datagram with a zero-copy function.
    ///
    /// When no datagram is available, this method will return `Poll::Pending` and
    /// register the current task to be notified when a datagram is received.
    ///
    /// When a datagram is received, this method will call the provided function
    /// with a reference to the received bytes and the remote endpoint and return
    /// `Poll::Ready` with the function's returned value.
    pub async fn recv_from_with<F, R>(&mut self, f: F) -> Result<R, RecvError>
    where
        F: FnOnce(&[u8], UdpMetadata) -> R,
    {
        let packet = self.recv().await?;
        Ok(f(packet.payload(), packet.meta()))
    }

    /// Receive a datagram with a zero-copy function.
    ///
    /// This method will not wait for a datagram to be received.
    ///
    /// If no datagram is available, this method will return `Err(TryError::WouldBlock)`.
    pub fn try_recv_from_with<F, R>(&mut self, f: F) -> Result<R, TryError<RecvError>>
    where
        F: FnOnce(&[u8], UdpMetadata) -> R,
    {
        let packet = self.try_recv()?;
        Ok(f(packet.payload(), packet.meta()))
    }

    /// Wait until the socket becomes writable.
    ///
    /// A socket becomes writable when the stack has a free packet buffer and the
    /// network device has room for a frame.
    pub fn wait_send_ready(&self) -> impl Future<Output = ()> + '_ {
        poll_fn(|cx| self.poll_send_ready(cx))
    }

    /// Wait until a datagram can be sent.
    ///
    /// When no datagram can be sent (the stack has no free packet buffer, or the
    /// network device has no room), this method will return `Poll::Pending` and
    /// register the current task to be notified when it can.
    ///
    /// When a datagram can be sent, this method will return `Poll::Ready`.
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
    fn map_send(r: Result<(), udp::SendError>) -> Poll<Result<(), SendError>> {
        match r {
            Ok(()) => Poll::Ready(Ok(())),
            Err(udp::SendError::NoBuffer) | Err(udp::SendError::DeviceBusy) => Poll::Pending,
            Err(udp::SendError::BufferFull) => Poll::Ready(Err(SendError::BufferFull)),
            Err(udp::SendError::InvalidState) => Poll::Ready(Err(SendError::InvalidState)),
            Err(udp::SendError::Unaddressable) => Poll::Ready(Err(SendError::Unaddressable)),
        }
    }

    /// Send a datagram to the specified remote endpoint.
    ///
    /// This method will wait until the datagram has been sent.
    ///
    /// If the datagram does not fit in a packet buffer, this method will return `Err(SendError::BufferFull)`
    ///
    /// When the remote endpoint is not reachable, this method will return `Err(SendError::Unaddressable)`
    pub async fn send_to<T>(&self, buf: &[u8], remote_endpoint: T) -> Result<(), SendError>
    where
        T: Into<UdpMetadata>,
    {
        let remote_endpoint: UdpMetadata = remote_endpoint.into();
        poll_fn(move |cx| self.poll_send_to(buf, remote_endpoint, cx)).await
    }

    /// Send a datagram to the specified remote endpoint.
    ///
    /// This method will not wait for a packet buffer or device room to become free.
    ///
    /// If the datagram cannot be sent right now, this method will return `Err(TryError::WouldBlock)`.
    ///
    /// If the datagram does not fit in a packet buffer, this method will return `Err(TryError::Other(SendError::BufferFull))`
    ///
    /// When the remote endpoint is not reachable, this method will return `Err(TryError::Other(SendError::Unaddressable))`
    pub fn try_send_to<T>(&self, buf: &[u8], remote_endpoint: T) -> Result<(), TryError<SendError>>
    where
        T: Into<UdpMetadata>,
    {
        let remote_endpoint: UdpMetadata = remote_endpoint.into();
        self.with_mut(|s| {
            let r = s.send_slice(buf, remote_endpoint);
            match Self::map_send(r) {
                Poll::Ready(r) => r.map_err(TryError::Other),
                Poll::Pending => Err(TryError::WouldBlock),
            }
        })
    }

    /// Send a datagram to the specified remote endpoint.
    ///
    /// When the datagram has been sent, this method will return `Poll::Ready(Ok())`.
    ///
    /// When the datagram cannot be sent right now, this method will return `Poll::Pending`
    /// and register the current task to be notified when it can.
    ///
    /// If the datagram does not fit in a packet buffer, this method will return `Poll::Ready(Err(SendError::BufferFull))`
    ///
    /// When the remote endpoint is not reachable, this method will return `Poll::Ready(Err(SendError::Unaddressable))`.
    pub fn poll_send_to<T>(&self, buf: &[u8], remote_endpoint: T, cx: &mut Context<'_>) -> Poll<Result<(), SendError>>
    where
        T: Into<UdpMetadata>,
    {
        let remote_endpoint: UdpMetadata = remote_endpoint.into();
        self.with_mut(|s| {
            let r = s.send_slice(buf, remote_endpoint);
            let r = Self::map_send(r);
            if r.is_pending() {
                s.register_send_waker(cx.waker());
            }
            r
        })
    }

    /// Send a datagram to the specified remote endpoint with a zero-copy function.
    ///
    /// This method will wait until a packet buffer is available before passing
    /// it to the closure. The closure returns the number of bytes written into
    /// the buffer.
    ///
    /// If `max_size` does not fit in a packet buffer, this method will return `Err(SendError::BufferFull)`
    ///
    /// When the remote endpoint is not reachable, this method will return `Err(SendError::Unaddressable)`
    pub async fn send_to_with<T, F, R>(&mut self, max_size: usize, remote_endpoint: T, f: F) -> Result<R, SendError>
    where
        T: Into<UdpMetadata> + Copy,
        F: FnOnce(&mut [u8]) -> (usize, R),
    {
        let mut f = Some(f);
        poll_fn(move |cx| {
            self.with_mut(|s| {
                let mut ret = None;
                let r = s.send_with(max_size, remote_endpoint.into(), |buf| {
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

    /// Send a datagram to the specified remote endpoint with a zero-copy function.
    ///
    /// This method will not wait for a packet buffer to become free.
    ///
    /// If the datagram cannot be sent right now, this method will return `Err(TryError::WouldBlock)`.
    ///
    /// If `size` does not fit in a packet buffer, this method will return `Err(TryError::Other(SendError::BufferFull))`
    ///
    /// When the remote endpoint is not reachable, this method will return `Err(TryError::Other(SendError::Unaddressable))`
    pub fn try_send_to_with<T, F, R>(&mut self, size: usize, remote_endpoint: T, f: F) -> Result<R, TryError<SendError>>
    where
        T: Into<UdpMetadata>,
        F: FnOnce(&mut [u8]) -> R,
    {
        let remote_endpoint: UdpMetadata = remote_endpoint.into();
        self.with_mut(|s| {
            let mut ret = None;
            let r = s.send_with(size, remote_endpoint, |buf| {
                ret = Some(f(buf));
                size
            });
            match Self::map_send(r) {
                Poll::Ready(Ok(())) => Ok(unwrap!(ret)),
                Poll::Ready(Err(e)) => Err(TryError::Other(e)),
                Poll::Pending => Err(TryError::WouldBlock),
            }
        })
    }

    /// Returns the local endpoint of the socket.
    pub fn endpoint(&self) -> IpListenEndpoint {
        self.with(|s| s.local_endpoint())
    }

    /// Returns the remote endpoint the socket is bound to, if any.
    pub fn remote_endpoint(&self) -> IpListenEndpoint {
        self.with(|s| s.remote_endpoint())
    }

    /// Returns whether the socket is open.
    pub fn is_open(&self) -> bool {
        self.with(|s| s.is_open())
    }

    /// Close the socket.
    pub fn close(&mut self) {
        self.with_mut(|s| s.close())
    }

    /// Returns whether the socket is ready to send data, i.e. it is bound and a packet buffer is free.
    pub fn may_send(&self) -> bool {
        self.with(|s| s.is_open())
    }

    /// Returns whether the socket is ready to receive data, i.e. it has received a packet that's now in the queue.
    pub fn may_recv(&self) -> bool {
        self.with(|s| s.can_recv())
    }

    /// Set the hop limit field in the IP header of sent packets.
    pub fn set_hop_limit(&mut self, hop_limit: Option<u8>) {
        self.with_mut(|s| s.set_hop_limit(hop_limit))
    }
}

impl Drop for UdpSocket<'_> {
    fn drop(&mut self) {
        self.stack.with_mut(|i| i.stack.remove_udp_socket(self.handle));
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

// Keep `IpEndpoint` in scope for the `UdpMetadata: From<IpEndpoint>` docs links.
#[allow(unused_imports)]
use IpEndpoint as _IpEndpoint;

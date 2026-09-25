//! TCP sockets.

use core::future::{Future, poll_fn};
use core::marker::PhantomData;
use core::mem;
use core::task::{Context, Poll};

use embassy_time::Duration;
use xarxa::error::{Full, InvalidHopLimit};
#[cfg(feature = "iface-bind")]
pub use xarxa::iface::IfaceHandle;
#[cfg(feature = "tcp-listener")]
pub use xarxa::tcp::AcceptToken;
pub use xarxa::tcp::State;
#[cfg(feature = "tcp-listener")]
use xarxa::tcp::TcpListenerHandle;
use xarxa::tcp::{self, TcpHandle};
use xarxa::wire::{ListenSocketAddr, SocketAddr};

use crate::time::{duration_from_xarxa, duration_to_xarxa};
use crate::{Stack, TryError};

/// Error returned by TcpSocket read/write functions.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// The connection was reset.
    ///
    /// This can happen on receiving a RST packet, or on timeout.
    ConnectionReset,
}

/// Error returned by [`TcpSocket::connect`]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ConnectError {
    /// The socket is already open.
    InvalidState,
    /// The remote port is zero, the remote address is unspecified, or there is
    /// no route to the remote host or no local address to reach it from.
    Unaddressable,
    /// No free port in the ephemeral range (only possible with tens of thousands
    /// of open sockets).
    NoFreePorts,
    /// Another TCP socket already holds the identical 4-tuple.
    InUse,
    /// The remote host rejected the connection with a RST packet.
    ConnectionReset,
    /// Connect timed out.
    TimedOut,
}

/// Error returned by [`TcpListener::listen`]
///
/// Requires the `tcp-listener` feature.
#[cfg(feature = "tcp-listener")]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ListenError {
    /// The listener is already listening on a different address, or
    /// `bind_to_iface` was called while it is open.
    InvalidState,
    /// The listen port is zero.
    Unaddressable,
    /// Another TCP listener is bound to the identical address.
    InUse,
}

/// Error returned by [`TcpListener::accept`] and [`TcpSocket::accept`]
///
/// Requires the `tcp-listener` feature.
#[cfg(feature = "tcp-listener")]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AcceptError {
    /// The listener is not listening, or the socket is still open, so it can't be
    /// used for a new connection.
    InvalidState,
    /// The remote reset the connection during the handshake.
    ConnectionReset,
}

/// A Transmission Control Protocol socket.
///
#[cfg_attr(
    not(feature = "tcp-listener"),
    doc = "A TCP socket represents a single connection (connecting or connected): its",
    doc = "4-tuple is fully set from the start, by [`connect`](Self::connect)."
)]
#[cfg_attr(
    feature = "tcp-listener",
    doc = "A TCP socket represents a single connection (connecting or connected): its",
    doc = "4-tuple is fully set from the start, by [`connect`](Self::connect) or by",
    doc = "[`accept`](Self::accept). Passive open lives in [`TcpListener`]."
)]
///
/// `'a` is the lifetime of the socket's buffers, `'d` the lifetime of the
/// stack.
pub struct TcpSocket<'a, 'd> {
    io: TcpIo<'d>,
    bufs: PhantomData<&'a mut [u8]>,
}

/// The reader half of a TCP socket.
pub struct TcpReader<'a, 'd> {
    io: TcpIo<'d>,
    bufs: PhantomData<&'a mut [u8]>,
}

/// The writer half of a TCP socket.
pub struct TcpWriter<'a, 'd> {
    io: TcpIo<'d>,
    bufs: PhantomData<&'a mut [u8]>,
}

impl<'a, 'd> TcpReader<'a, 'd> {
    /// Wait until the socket becomes readable.
    ///
    /// A socket becomes readable when the receive half of the full-duplex connection is open
    /// (see [`may_recv()`](TcpSocket::may_recv)), and there is some pending data in the receive buffer.
    ///
    /// This is the equivalent of [read](#method.read), without buffering any data.
    pub fn wait_read_ready(&self) -> impl Future<Output = ()> + '_ {
        poll_fn(move |cx| self.io.poll_read_ready(cx))
    }

    /// Read data from the socket.
    ///
    /// Returns how many bytes were read, or an error. If no data is available, it waits
    /// until there is at least one byte available.
    ///
    /// # Note
    /// A return value of Ok(0) means that we have read all data and the remote
    /// side has closed our receive half of the socket. The remote can no longer
    /// send bytes.
    ///
    /// The send half of the socket is still open. If you want to reconnect using
    /// the socket you split this reader off the send half needs to be closed using
    /// [`abort()`](TcpSocket::abort).
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.io.read(buf).await
    }

    /// Read data from the socket.
    ///
    /// This method will not wait for data to be received.
    ///
    /// If no data is available, this method will return `Err(TryError::WouldBlock)`.
    pub fn try_read(&mut self, buf: &mut [u8]) -> Result<usize, TryError<Error>> {
        self.io.try_read(buf)
    }

    /// Call `f` with the largest contiguous slice of octets in the receive buffer,
    /// and dequeue the amount of elements returned by `f`.
    ///
    /// If no data is available, it waits until there is at least one byte available.
    pub async fn read_with<R>(&mut self, f: impl FnOnce(&mut [u8]) -> (usize, R)) -> Result<R, Error> {
        self.io.read_with(f).await
    }

    /// Call `f` with the largest contiguous slice of octets in the receive buffer,
    /// and dequeue the amount of elements returned by `f`.
    ///
    /// This method will not wait for data to be received.
    ///
    /// If no data is available, this method will return `Err(TryError::WouldBlock)`.
    pub fn try_read_with<R>(&mut self, f: impl FnOnce(&mut [u8]) -> (usize, R)) -> Result<R, TryError<Error>> {
        self.io.try_read_with(f)
    }

    /// Peek at received data without removing it from the receive buffer, copying
    /// it into `buf`.
    ///
    /// If no data is available, it waits until there is at least one byte available.
    ///
    /// Like [`read`](Self::read), a return value of Ok(0) means that all data has
    /// been read and the remote side has closed our receive half of the socket.
    pub async fn peek(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.io.peek(buf).await
    }

    /// Peek at received data without removing it from the receive buffer, copying
    /// it into `buf`.
    ///
    /// This method will not wait for data to be received.
    ///
    /// If no data is available, this method will return `Err(TryError::WouldBlock)`.
    pub fn try_peek(&mut self, buf: &mut [u8]) -> Result<usize, TryError<Error>> {
        self.io.try_peek(buf)
    }

    /// Call `f` with the largest contiguous slice of octets in the receive buffer,
    /// without removing them from it.
    ///
    /// If no data is available, it waits until there is at least one byte available.
    pub async fn peek_with<R>(&mut self, f: impl FnOnce(&[u8]) -> R) -> Result<R, Error> {
        self.io.peek_with(f).await
    }

    /// Call `f` with the largest contiguous slice of octets in the receive buffer,
    /// without removing them from it.
    ///
    /// This method will not wait for data to be received.
    ///
    /// If no data is available, this method will return `Err(TryError::WouldBlock)`.
    pub fn try_peek_with<R>(&mut self, f: impl FnOnce(&[u8]) -> R) -> Result<R, TryError<Error>> {
        self.io.try_peek_with(f)
    }

    /// Return the maximum number of bytes inside the recv buffer.
    pub fn recv_capacity(&self) -> usize {
        self.io.recv_capacity()
    }

    /// Return the amount of octets queued in the receive buffer. This value can be larger than
    /// the slice read by the next `recv` or `peek` call because it includes all queued octets,
    /// and not only the octets that may be returned as a contiguous slice.
    ///
    /// Note that the Berkeley sockets interface does not have an equivalent of this API.
    pub fn recv_queue(&self) -> usize {
        self.io.recv_queue()
    }

    /// Return whether the receive half of the full-duplex connection is open.
    ///
    /// This function returns true if it's possible to receive data from the remote peer.
    /// It will return true while there is data in the receive buffer, and if there isn't,
    /// as long as the remote peer has not closed the connection.
    ///
    /// In terms of the TCP state machine, the socket must be in the `ESTABLISHED`,
    /// `FIN-WAIT-1`, or `FIN-WAIT-2` state, or have data in the receive buffer instead.
    pub fn may_recv(&self) -> bool {
        self.io.with(|s| s.may_recv())
    }

    /// Check whether the receive buffer is not empty.
    pub fn can_recv(&self) -> bool {
        self.io.with(|s| s.can_recv())
    }
}

impl<'a, 'd> TcpWriter<'a, 'd> {
    /// Wait until the socket becomes writable.
    ///
    /// A socket becomes writable when the transmit half of the full-duplex connection is open
    /// (see [`may_send()`](TcpSocket::may_send)), and the transmit buffer is not full.
    ///
    /// This is the equivalent of [write](#method.write), without sending any data.
    pub fn wait_write_ready(&self) -> impl Future<Output = ()> + '_ {
        poll_fn(move |cx| self.io.poll_write_ready(cx))
    }

    /// Write data to the socket.
    ///
    /// Returns how many bytes were written, or an error. If the socket is not ready to
    /// accept data, it waits until it is.
    pub fn write<'s>(&'s mut self, buf: &'s [u8]) -> impl Future<Output = Result<usize, Error>> + 's {
        self.io.write(buf)
    }

    /// Write data to the socket.
    ///
    /// This method will not wait for the buffer to become free.
    ///
    /// If the socket's send buffer is full, this method will return `Err(TryError::WouldBlock)`.
    pub fn try_write(&mut self, buf: &[u8]) -> Result<usize, TryError<Error>> {
        self.io.try_write(buf)
    }

    /// Flushes the written data to the socket.
    ///
    /// This waits until all data has been sent, and ACKed by the remote host. For a connection
    /// closed with [`abort()`](TcpSocket::abort) it will wait for the TCP RST packet to be sent.
    pub fn flush(&mut self) -> impl Future<Output = Result<(), Error>> + '_ {
        self.io.flush()
    }

    /// Try to flush the socket.
    ///
    /// This method will check if the socket is flushed, and if not, return `Err(TryError::WouldBlock)`.
    pub fn try_flush(&mut self) -> Result<(), TryError<Error>> {
        self.io.try_flush()
    }

    /// Call `f` with the largest contiguous slice of octets in the transmit buffer,
    /// and enqueue the amount of elements returned by `f`.
    ///
    /// If the socket is not ready to accept data, it waits until it is.
    pub async fn write_with<R>(&mut self, f: impl FnOnce(&mut [u8]) -> (usize, R)) -> Result<R, Error> {
        self.io.write_with(f).await
    }

    /// Call `f` with the largest contiguous slice of octets in the transmit buffer,
    /// and enqueue the amount of elements returned by `f`.
    ///
    /// This method will not wait for the buffer to become free.
    ///
    /// If the socket's send buffer is full, this method will return `Err(TryError::WouldBlock)`.
    pub fn try_write_with<R>(&mut self, f: impl FnOnce(&mut [u8]) -> (usize, R)) -> Result<R, TryError<Error>> {
        self.io.try_write_with(f)
    }

    /// Return the maximum number of bytes inside the transmit buffer.
    pub fn send_capacity(&self) -> usize {
        self.io.send_capacity()
    }

    /// Return the amount of octets queued in the transmit buffer.
    ///
    /// Note that the Berkeley sockets interface does not have an equivalent of this API.
    pub fn send_queue(&self) -> usize {
        self.io.send_queue()
    }

    /// Return whether the transmit half of the full-duplex connection is open.
    ///
    /// This function returns true if it's possible to send data and have it arrive
    /// to the remote peer. However, it does not make any guarantees about the state
    /// of the transmit buffer, and even if it returns true, [write](#method.write) may
    /// not be able to enqueue any octets.
    ///
    /// In terms of the TCP state machine, the socket must be in the `ESTABLISHED` or
    /// `CLOSE-WAIT` state.
    pub fn may_send(&self) -> bool {
        self.io.with(|s| s.may_send())
    }

    /// Check whether the transmit half of the full-duplex connection is open
    /// (see [may_send](#method.may_send)), and the transmit buffer is not full.
    pub fn can_send(&self) -> bool {
        self.io.with(|s| s.can_send())
    }

    /// Return whether the receive half of the full-duplex connection is open.
    ///
    /// This function returns true if it's possible for the corresponding [`TcpReader`]
    /// to receive data from the remote peer.
    /// It will return true while there is data in the receive buffer, and if there isn't,
    /// as long as the remote peer has not closed the connection.
    ///
    /// In terms of the TCP state machine, the socket must be in the `ESTABLISHED`,
    /// `FIN-WAIT-1`, or `FIN-WAIT-2` state, or have data in the receive buffer instead.
    pub fn may_recv(&self) -> bool {
        self.io.with(|s| s.may_recv())
    }
}

impl<'a, 'd> TcpSocket<'a, 'd> {
    /// Create a new TCP socket on the given stack, with the given buffers.
    ///
    /// # Errors
    /// - `Full`: if the stack has no room for another TCP socket. Only possible
    ///   without the `alloc` feature, where the limit is set by the
    ///   `tcp-socket-count-N` feature of `xarxa`.
    pub fn new(stack: Stack<'d>, rx_buffer: &'a mut [u8], tx_buffer: &'a mut [u8]) -> Result<Self, Full> {
        let handle = stack.with(|i| {
            let rx_buffer: &'d mut [u8] = unsafe { mem::transmute(rx_buffer) };
            let tx_buffer: &'d mut [u8] = unsafe { mem::transmute(tx_buffer) };
            i.stack.add_tcp_socket_with_bufs(rx_buffer, tx_buffer)
        })?;

        Ok(Self {
            io: TcpIo { stack, handle },
            bufs: PhantomData,
        })
    }

    /// Return the maximum number of bytes inside the recv buffer.
    pub fn recv_capacity(&self) -> usize {
        self.io.recv_capacity()
    }

    /// Return the maximum number of bytes inside the transmit buffer.
    pub fn send_capacity(&self) -> usize {
        self.io.send_capacity()
    }

    /// Return the amount of octets queued in the transmit buffer.
    ///
    /// Note that the Berkeley sockets interface does not have an equivalent of this API.
    pub fn send_queue(&self) -> usize {
        self.io.send_queue()
    }

    /// Return the amount of octets queued in the receive buffer. This value can be larger than
    /// the slice read by the next `recv` or `peek` call because it includes all queued octets,
    /// and not only the octets that may be returned as a contiguous slice.
    ///
    /// Note that the Berkeley sockets interface does not have an equivalent of this API.
    pub fn recv_queue(&self) -> usize {
        self.io.recv_queue()
    }

    /// Call `f` with the largest contiguous slice of octets in the transmit buffer,
    /// and enqueue the amount of elements returned by `f`.
    ///
    /// If the socket is not ready to accept data, it waits until it is.
    pub async fn write_with<R>(&mut self, f: impl FnOnce(&mut [u8]) -> (usize, R)) -> Result<R, Error> {
        self.io.write_with(f).await
    }

    /// Call `f` with the largest contiguous slice of octets in the transmit buffer,
    /// and enqueue the amount of elements returned by `f`.
    ///
    /// This method will not wait for the buffer to become free.
    ///
    /// If the socket's send buffer is full, this method will return `Err(TryError::WouldBlock)`.
    pub fn try_write_with<R>(&mut self, f: impl FnOnce(&mut [u8]) -> (usize, R)) -> Result<R, TryError<Error>> {
        self.io.try_write_with(f)
    }

    /// Call `f` with the largest contiguous slice of octets in the receive buffer,
    /// and dequeue the amount of elements returned by `f`.
    ///
    /// If no data is available, it waits until there is at least one byte available.
    pub async fn read_with<R>(&mut self, f: impl FnOnce(&mut [u8]) -> (usize, R)) -> Result<R, Error> {
        self.io.read_with(f).await
    }

    /// Call `f` with the largest contiguous slice of octets in the receive buffer,
    /// and dequeue the amount of elements returned by `f`.
    ///
    /// This method will not wait for data to be received.
    ///
    /// If no data is available, this method will return `Err(TryError::WouldBlock)`.
    pub fn try_read_with<R>(&mut self, f: impl FnOnce(&mut [u8]) -> (usize, R)) -> Result<R, TryError<Error>> {
        self.io.try_read_with(f)
    }

    /// Peek at received data without removing it from the receive buffer, copying
    /// it into `buf`.
    ///
    /// If no data is available, it waits until there is at least one byte available.
    ///
    /// Like [`read`](Self::read), a return value of Ok(0) means that all data has
    /// been read and the remote side has closed our receive half of the socket.
    pub async fn peek(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.io.peek(buf).await
    }

    /// Peek at received data without removing it from the receive buffer, copying
    /// it into `buf`.
    ///
    /// This method will not wait for data to be received.
    ///
    /// If no data is available, this method will return `Err(TryError::WouldBlock)`.
    pub fn try_peek(&mut self, buf: &mut [u8]) -> Result<usize, TryError<Error>> {
        self.io.try_peek(buf)
    }

    /// Call `f` with the largest contiguous slice of octets in the receive buffer,
    /// without removing them from it.
    ///
    /// If no data is available, it waits until there is at least one byte available.
    pub async fn peek_with<R>(&mut self, f: impl FnOnce(&[u8]) -> R) -> Result<R, Error> {
        self.io.peek_with(f).await
    }

    /// Call `f` with the largest contiguous slice of octets in the receive buffer,
    /// without removing them from it.
    ///
    /// This method will not wait for data to be received.
    ///
    /// If no data is available, this method will return `Err(TryError::WouldBlock)`.
    pub fn try_peek_with<R>(&mut self, f: impl FnOnce(&[u8]) -> R) -> Result<R, TryError<Error>> {
        self.io.try_peek_with(f)
    }

    /// Split the socket into reader and a writer halves.
    pub fn split(&mut self) -> (TcpReader<'_, 'd>, TcpWriter<'_, 'd>) {
        (
            TcpReader {
                io: self.io,
                bufs: PhantomData,
            },
            TcpWriter {
                io: self.io,
                bufs: PhantomData,
            },
        )
    }

    /// Bind the socket to an interface, or unbind it with `None`.
    ///
    /// A socket bound to an interface only sends and receives packets on it:
    /// - Destinations must be on-link on that iface, or have a route through it.
    /// - Broadcast and multicast destinations go out on that iface only
    /// - Local addresses will be picked from that iface only.
    ///
    /// The socket must be closed (see [`is_open`](Self::is_open)). The binding
    /// is kept across connections, so a reused socket stays bound to its interface.
    /// Accepting a connection into the socket overwrites it with the listener's binding.
    ///
    /// Two sockets with otherwise identical tuples may coexist if they are
    /// bound to different interfaces.
    ///
    /// # Errors
    /// - `InvalidState`: if the socket is open.
    #[cfg(feature = "iface-bind")]
    pub fn bind_to_iface(&mut self, iface: Option<IfaceHandle>) -> Result<(), ConnectError> {
        match self.io.with_mut(|s| s.bind_to_iface(iface)) {
            Ok(()) => Ok(()),
            Err(tcp::ConnectError::InvalidState) => Err(ConnectError::InvalidState),
            Err(_) => unreachable!(),
        }
    }

    /// Return the interface the socket is bound to, or `None`.
    ///
    /// See [`bind_to_iface`](Self::bind_to_iface).
    #[cfg(feature = "iface-bind")]
    pub fn bound_iface(&self) -> Option<IfaceHandle> {
        self.io.with(|s| s.bound_iface())
    }

    fn start_connect(&mut self, remote: impl Into<SocketAddr>) -> Result<(), ConnectError> {
        match self.io.with_mut(|s| s.connect(remote, ListenSocketAddr::UNSPECIFIED)) {
            Ok(()) => Ok(()),
            Err(tcp::ConnectError::InvalidState) => Err(ConnectError::InvalidState),
            Err(tcp::ConnectError::Unaddressable) => Err(ConnectError::Unaddressable),
            Err(tcp::ConnectError::NoFreePorts) => Err(ConnectError::NoFreePorts),
            Err(tcp::ConnectError::InUse) => Err(ConnectError::InUse),
        }
    }

    /// Connect to a given remote address, and wait until the connection is established.
    ///
    /// The local port is an ephemeral port (a free port in the 49152..=65535
    /// range, picked at a random starting point), and the local address is
    /// selected by the stack from the remote address.
    ///
    /// # Errors
    /// - `InvalidState`: if the socket is open.
    /// - `Unaddressable`: if the remote port is zero, the remote address is
    ///   unspecified, there is no route to the remote address, or the interface
    ///   the route goes out of has no address to send from.
    /// - `NoFreePorts`: if the ephemeral range is exhausted.
    /// - `InUse`: if another TCP socket already holds the identical 4-tuple.
    /// - `ConnectionReset`: if the connection is reset or aborted during the handshake.
    pub async fn connect(&mut self, remote: impl Into<SocketAddr>) -> Result<(), ConnectError> {
        self.start_connect(remote)?;

        poll_fn(|cx| {
            self.io.with_mut(|s| match s.state() {
                tcp::State::Closed | tcp::State::TimeWait => Poll::Ready(Err(ConnectError::ConnectionReset)),
                tcp::State::SynSent | tcp::State::SynReceived => {
                    s.register_send_waker(cx.waker());
                    Poll::Pending
                }
                _ => Poll::Ready(Ok(())),
            })
        })
        .await
    }

    /// Accept a connection attempt into this socket, and wait until its handshake completes.
    ///
    /// The token comes from [`TcpListener::accept`].
    ///
    /// The socket's interface binding (see `bind_to_iface`)
    /// is set to the listener's binding. Other configuration (hop limit,
    /// timeout, keep-alive, Nagle, ACK delay) is left unchanged.
    ///
    /// If the returned future is dropped before it completes, the connection attempt is reset.
    ///
    /// # Errors
    /// - `InvalidState`: if the socket is not closed.
    /// - `ConnectionReset`: if the remote resets the connection during the handshake.
    #[cfg(feature = "tcp-listener")]
    pub async fn accept(&mut self, token: AcceptToken) -> Result<(), AcceptError> {
        self.io.with_mut(|s| s.accept(token)).map_err(|e| match e {
            tcp::AcceptError::InvalidState => AcceptError::InvalidState,
        })?;

        poll_fn(|cx| {
            self.io.with_mut(|s| match s.state() {
                tcp::State::Closed | tcp::State::TimeWait => Poll::Ready(Err(AcceptError::ConnectionReset)),
                tcp::State::SynReceived => {
                    s.register_send_waker(cx.waker());
                    Poll::Pending
                }
                _ => Poll::Ready(Ok(())),
            })
        })
        .await
    }

    /// Connect to a remote host.
    ///
    /// This method will not wait for the connection to be established.
    ///
    /// If the socket is not already connecting, this method will initiate the connection
    /// and return `Err(TryError::WouldBlock)`. While the connection is being established,
    /// it will continue to return `Err(TryError::WouldBlock)`.
    ///
    /// Once the connection is successfully established and ready to send/receive data,
    /// this method will return `Ok(())`.
    pub fn try_connect(&mut self, remote: impl Into<SocketAddr>) -> Result<(), TryError<ConnectError>> {
        match self.state() {
            tcp::State::Closed | tcp::State::TimeWait => match self.start_connect(remote) {
                Ok(()) => Err(TryError::WouldBlock),
                Err(e) => Err(TryError::Other(e)),
            },
            tcp::State::SynSent | tcp::State::SynReceived => Err(TryError::WouldBlock),
            _ => Ok(()),
        }
    }

    /// Wait until the socket becomes readable.
    ///
    /// A socket becomes readable when the receive half of the full-duplex connection is open
    /// (see [may_recv](#method.may_recv)), and there is some pending data in the receive buffer.
    ///
    /// This is the equivalent of [read](#method.read), without buffering any data.
    pub fn wait_read_ready(&self) -> impl Future<Output = ()> + '_ {
        poll_fn(move |cx| self.io.poll_read_ready(cx))
    }

    /// Read data from the socket.
    ///
    /// Returns how many bytes were read, or an error. If no data is available, it waits
    /// until there is at least one byte available.
    ///
    /// A return value of Ok(0) means that the socket was closed and is longer
    /// able to receive any data.
    pub fn read<'s>(&'s mut self, buf: &'s mut [u8]) -> impl Future<Output = Result<usize, Error>> + 's {
        self.io.read(buf)
    }

    /// Read data from the socket.
    ///
    /// This method will not wait for data to be received.
    ///
    /// If no data is available, this method will return `Err(TryError::WouldBlock)`.
    pub fn try_read(&mut self, buf: &mut [u8]) -> Result<usize, TryError<Error>> {
        self.io.try_read(buf)
    }

    /// Wait until the socket becomes writable.
    ///
    /// A socket becomes writable when the transmit half of the full-duplex connection is open
    /// (see [may_send](#method.may_send)), and the transmit buffer is not full.
    ///
    /// This is the equivalent of [write](#method.write), without sending any data.
    pub fn wait_write_ready(&self) -> impl Future<Output = ()> + '_ {
        poll_fn(move |cx| self.io.poll_write_ready(cx))
    }

    /// Write data to the socket.
    ///
    /// Returns how many bytes were written, or an error. If the socket is not ready to
    /// accept data, it waits until it is.
    pub fn write<'s>(&'s mut self, buf: &'s [u8]) -> impl Future<Output = Result<usize, Error>> + 's {
        self.io.write(buf)
    }

    /// Write data to the socket.
    ///
    /// This method will not wait for the buffer to become free.
    ///
    /// If the socket's send buffer is full, this method will return `Err(TryError::WouldBlock)`.
    pub fn try_write(&mut self, buf: &[u8]) -> Result<usize, TryError<Error>> {
        self.io.try_write(buf)
    }

    /// Flushes the written data to the socket.
    ///
    /// This waits until all data has been sent, and ACKed by the remote host. For a connection
    /// closed with [`abort()`](TcpSocket::abort) it will wait for the TCP RST packet to be sent.
    pub fn flush(&mut self) -> impl Future<Output = Result<(), Error>> + '_ {
        self.io.flush()
    }

    /// Try to flush the socket.
    ///
    /// This method will check if the socket is flushed, and if not, return `Err(TryError::WouldBlock)`.
    pub fn try_flush(&mut self) -> Result<(), TryError<Error>> {
        self.io.try_flush()
    }

    /// Set the timeout duration.
    ///
    /// A socket with a timeout duration set will abort the connection if either of the following
    /// occurs:
    ///
    ///   * After a [connect](#method.connect) call, the remote peer does not respond within
    ///     the specified duration;
    ///   * After establishing a connection, there is data in the transmit buffer and the remote
    ///     peer exceeds the specified duration between any two packets it sends;
    ///   * After enabling [keep-alive](#method.set_keep_alive), the remote peer exceeds
    ///     the specified duration between any two packets it sends.
    pub fn set_timeout(&mut self, duration: Option<Duration>) {
        self.io.with_mut(|s| s.set_timeout(duration.map(duration_to_xarxa)))
    }

    /// Return the timeout duration.
    ///
    /// See also the [set_timeout](#method.set_timeout) method.
    pub fn timeout(&self) -> Option<Duration> {
        self.io.with(|s| s.timeout().map(duration_from_xarxa))
    }

    /// Set the ACK delay duration.
    ///
    /// By default, the ACK delay is set to 10ms.
    pub fn set_ack_delay(&mut self, duration: Option<Duration>) {
        self.io.with_mut(|s| s.set_ack_delay(duration.map(duration_to_xarxa)))
    }

    /// Return the ACK delay duration.
    ///
    /// See also the [set_ack_delay](#method.set_ack_delay) method.
    pub fn ack_delay(&self) -> Option<Duration> {
        self.io.with(|s| s.ack_delay().map(duration_from_xarxa))
    }

    /// Set the keep-alive interval.
    ///
    /// An idle socket with a keep-alive interval set will transmit a "keep-alive ACK" packet
    /// every time it receives no communication during that interval. As a result, three things
    /// may happen:
    ///
    ///   * The remote peer is fine and answers with an ACK packet.
    ///   * The remote peer has rebooted and answers with an RST packet.
    ///   * The remote peer has crashed and does not answer.
    ///
    /// The keep-alive functionality together with the timeout functionality allows to react
    /// to these error conditions.
    pub fn set_keep_alive(&mut self, interval: Option<Duration>) {
        self.io.with_mut(|s| s.set_keep_alive(interval.map(duration_to_xarxa)))
    }

    /// Return the keep-alive interval.
    ///
    /// See also the [set_keep_alive](#method.set_keep_alive) method.
    pub fn keep_alive(&self) -> Option<Duration> {
        self.io.with(|s| s.keep_alive().map(duration_from_xarxa))
    }

    /// Set the time-to-live (IPv4) or hop limit (IPv6) value used in outgoing packets.
    ///
    /// A socket without an explicitly set hop limit value uses the default [IANA recommended]
    /// value (64).
    ///
    /// # Errors
    /// - `InvalidHopLimit`: if the hop limit is `Some(0)`. A host must not send a
    ///   packet with a hop limit of zero ([RFC 1122 § 3.2.1.7]). The socket is
    ///   left unchanged.
    ///
    /// [IANA recommended]: https://www.iana.org/assignments/ip-parameters/ip-parameters.xhtml
    /// [RFC 1122 § 3.2.1.7]: https://tools.ietf.org/html/rfc1122#section-3.2.1.7
    pub fn set_hop_limit(&mut self, hop_limit: Option<u8>) -> Result<(), InvalidHopLimit> {
        self.io.with_mut(|s| s.set_hop_limit(hop_limit))
    }

    /// Return the time-to-live (IPv4) or hop limit (IPv6) value used in outgoing packets.
    ///
    /// See also the [set_hop_limit](#method.set_hop_limit) method
    pub fn hop_limit(&self) -> Option<u8> {
        self.io.with(|s| s.hop_limit())
    }

    /// Enable or disable Nagle's Algorithm.
    ///
    /// Also known as "tinygram prevention". By default, it is enabled.
    /// Disabling it is equivalent to Linux's TCP_NODELAY flag.
    ///
    /// When enabled, Nagle's Algorithm prevents sending segments smaller than MSS if
    /// there is data in flight (sent but not acknowledged). In other words, it ensures
    /// at most only one segment smaller than MSS is in flight at a time.
    ///
    /// It ensures better network utilization by preventing sending many very small packets,
    /// at the cost of increased latency in some situations, particularly when the remote peer
    /// has ACK delay enabled.
    pub fn set_nagle_enabled(&mut self, enabled: bool) {
        self.io.with_mut(|s| s.set_nagle_enabled(enabled))
    }

    /// Return whether Nagle's Algorithm is enabled.
    ///
    /// See also the [set_nagle_enabled](#method.set_nagle_enabled) method.
    pub fn nagle_enabled(&self) -> bool {
        self.io.with(|s| s.nagle_enabled())
    }

    /// Return the local address, or None if not connected.
    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.io.with(|s| s.local_addr())
    }

    /// Return the remote address, or None if not connected.
    pub fn remote_addr(&self) -> Option<SocketAddr> {
        self.io.with(|s| s.remote_addr())
    }

    /// Return the connection state, in terms of the TCP state machine.
    pub fn state(&self) -> State {
        self.io.with(|s| s.state())
    }

    /// Return whether the socket is open.
    ///
    /// This function returns true if the socket will process incoming or dispatch outgoing
    /// packets. Note that this does not mean that it is possible to send or receive data through
    /// the socket; for that, use [can_send](#method.can_send) or [can_recv](#method.can_recv).
    ///
    /// In terms of the TCP state machine, the socket must not be in the `CLOSED`
    /// or `TIME-WAIT` states.
    pub fn is_open(&self) -> bool {
        self.io.with(|s| s.is_open())
    }

    /// Return whether a connection is active.
    ///
    /// This function returns true if the socket is actively exchanging packets with
    /// a remote peer. Note that this does not mean that it is possible to send or receive
    /// data through the socket; for that, use [can_send](#method.can_send) or
    /// [can_recv](#method.can_recv).
    ///
    /// If a connection is established, [abort](#method.abort) will send a reset to
    /// the remote peer.
    ///
    /// In terms of the TCP state machine, the socket must not be in the `CLOSED`
    /// or `TIME-WAIT` state.
    pub fn is_active(&self) -> bool {
        self.io.with(|s| s.is_active())
    }

    /// Take the pending ICMP error, if one has been reported against this
    /// connection.
    #[cfg(feature = "icmp-errors")]
    pub fn take_icmp_error(&mut self) -> Option<crate::error::IcmpError> {
        self.io.with_mut(|s| s.take_icmp_error())
    }

    /// Close the transmit half of the full-duplex connection.
    ///
    /// Data that has been written to the socket and not yet sent (or not yet ACKed) will
    /// still be sent. The last segment of the pending to send data is sent with the FIN flag set.
    ///
    /// Note that there is no corresponding function for the receive half of the full-duplex
    /// connection; only the remote end can close it. If you no longer wish to receive any
    /// data and would like to reuse the socket right away, use [abort](#method.abort).
    pub fn close(&mut self) {
        self.io.with_mut(|s| s.close())
    }

    /// Aborts the connection, if any.
    ///
    /// This function instantly closes the socket. One reset packet will be sent to the remote
    /// peer.
    ///
    /// In terms of the TCP state machine, the socket may be in any state and is moved to
    /// the `CLOSED` state.
    ///
    /// Note that the TCP RST packet is not sent immediately - if the `TcpSocket` is dropped too soon
    /// the remote host may not know the connection has been closed.
    /// `abort()` callers should wait for a [`flush()`](TcpSocket::flush) call to complete before
    /// dropping or reusing the socket.
    pub fn abort(&mut self) {
        self.io.with_mut(|s| s.abort())
    }

    /// Return whether the transmit half of the full-duplex connection is open.
    ///
    /// This function returns true if it's possible to send data and have it arrive
    /// to the remote peer. However, it does not make any guarantees about the state
    /// of the transmit buffer, and even if it returns true, [write](#method.write) may
    /// not be able to enqueue any octets.
    ///
    /// In terms of the TCP state machine, the socket must be in the `ESTABLISHED` or
    /// `CLOSE-WAIT` state.
    pub fn may_send(&self) -> bool {
        self.io.with(|s| s.may_send())
    }

    /// Check whether the transmit half of the full-duplex connection is open
    /// (see [may_send](#method.may_send)), and the transmit buffer is not full.
    pub fn can_send(&self) -> bool {
        self.io.with(|s| s.can_send())
    }

    /// Return whether the receive half of the full-duplex connection is open.
    ///
    /// This function returns true if it's possible to receive data from the remote peer.
    /// It will return true while there is data in the receive buffer, and if there isn't,
    /// as long as the remote peer has not closed the connection.
    ///
    /// In terms of the TCP state machine, the socket must be in the `ESTABLISHED`,
    /// `FIN-WAIT-1`, or `FIN-WAIT-2` state, or have data in the receive buffer instead.
    pub fn may_recv(&self) -> bool {
        self.io.with(|s| s.may_recv())
    }

    /// Check whether the receive buffer is not empty.
    pub fn can_recv(&self) -> bool {
        self.io.with(|s| s.can_recv())
    }
}

impl Drop for TcpSocket<'_, '_> {
    fn drop(&mut self) {
        self.io.stack.with_mut(|i| i.stack.remove_tcp_socket(self.io.handle));
    }
}

// `'a` (the buffers) is covariant; `'d` (the stack) is invariant.
fn _assert_covariant<'a, 'b: 'a, 'd>(x: TcpSocket<'b, 'd>) -> TcpSocket<'a, 'd> {
    x
}
fn _assert_covariant_reader<'a, 'b: 'a, 'd>(x: TcpReader<'b, 'd>) -> TcpReader<'a, 'd> {
    x
}
fn _assert_covariant_writer<'a, 'b: 'a, 'd>(x: TcpWriter<'b, 'd>) -> TcpWriter<'a, 'd> {
    x
}

// =======================

/// A TCP listener.
///
/// Use a [`TcpListener`] to accept incoming TCP connections.
///
/// A listener can be bound to a port and optionally an address.
/// It receives all incoming connection attempts and queues them.
/// Calling [`accept`](TcpListener::accept) pops an attempt from the queue as
/// an [`AcceptToken`]. Pass it to [`TcpSocket::accept`] to set up a socket for it,
/// possibly in another task.
///
/// Connection attempts (SYN packets) are not answered (with a SYN|ACK packet)
/// until a socket accepts them. The queue depth is set by the
/// `tcp-listener-backlog-N` feature of `xarxa`.
#[cfg(feature = "tcp-listener")]
pub struct TcpListener<'d> {
    stack: Stack<'d>,
    handle: TcpListenerHandle,
}

#[cfg(feature = "tcp-listener")]
impl<'d> TcpListener<'d> {
    /// Create a new TCP listener on the given stack.
    ///
    /// The listener starts closed. Call [`listen`](Self::listen) to start listening.
    ///
    /// # Errors
    /// - `Full`: if the stack has no room for another listener. Only possible
    ///   without the `alloc` feature, where the limit is set by the
    ///   `tcp-listener-count-N` feature of `xarxa`.
    pub fn new(stack: Stack<'d>) -> Result<Self, Full> {
        let handle = stack.with(|i| i.stack.add_tcp_listener())?;

        Ok(Self { stack, handle })
    }

    fn with<R>(&self, f: impl FnOnce(&mut tcp::TcpListener<'_>) -> R) -> R {
        self.stack.with(|i| f(&mut i.stack.tcp_listener(self.handle)))
    }

    fn with_mut<R>(&self, f: impl FnOnce(&mut tcp::TcpListener<'_>) -> R) -> R {
        self.stack.with_mut(|i| f(&mut i.stack.tcp_listener(self.handle)))
    }

    /// Bind the listener to an interface, or unbind it with `None`.
    ///
    /// A listener bound to an interface only accepts connection attempts that arrive on that interface.
    ///
    /// When accepting a connection, the sockets inherit the binding.
    ///
    /// The listener must be closed. The binding is kept across
    /// [`close`](Self::close).
    ///
    /// Two listeners on the same address can coexist if they are bound to different interfaces,
    /// or if one is not bound. In the latter case, the bound listener "wins" for incoming
    /// connections coming from the bound interface.
    ///
    /// # Errors
    /// - `InvalidState`: if the listener is open.
    #[cfg(feature = "iface-bind")]
    pub fn bind_to_iface(&mut self, iface: Option<IfaceHandle>) -> Result<(), ListenError> {
        match self.with_mut(|l| l.bind_to_iface(iface)) {
            Ok(()) => Ok(()),
            Err(tcp::ListenError::InvalidState) => Err(ListenError::InvalidState),
            Err(_) => unreachable!(),
        }
    }

    /// Return the interface the listener is bound to, or `None`.
    ///
    /// See [`bind_to_iface`](Self::bind_to_iface).
    #[cfg(feature = "iface-bind")]
    pub fn bound_iface(&self) -> Option<IfaceHandle> {
        self.with(|l| l.bound_iface())
    }

    /// Start listening on the given local address.
    ///
    /// # Errors
    /// - `Unaddressable`: if the port is zero.
    /// - `InvalidState`: if the listener is already listening (unless it is
    ///   listening on this same address, which is a no-op).
    /// - `InUse`: if another listener is bound to an identical address.
    ///   Listeners on the same port with *different* specificity (one wildcard,
    ///   one per-version, one per-address) may coexist, and so may listeners on
    ///   identical addresses bound to different interfaces.
    pub fn listen(&mut self, local: impl Into<ListenSocketAddr>) -> Result<(), ListenError> {
        match self.with_mut(|l| l.listen(local)) {
            Ok(()) => Ok(()),
            Err(tcp::ListenError::InvalidState) => Err(ListenError::InvalidState),
            Err(tcp::ListenError::Unaddressable) => Err(ListenError::Unaddressable),
            Err(tcp::ListenError::InUse) => Err(ListenError::InUse),
        }
    }

    /// Stop listening, dropping all queued SYNs.
    ///
    /// The dropped SYNs are not reset. The clients' retransmissions are
    /// answered with an RST once the listener is gone.
    pub fn close(&mut self) {
        self.with_mut(|l| l.close())
    }

    /// Whether the listener is listening.
    pub fn is_open(&self) -> bool {
        self.with(|l| l.is_open())
    }

    /// Return the listened address. The address is the filter the listen scoped
    /// the listener to. A zero port means the listener is closed.
    pub fn local_addr(&self) -> ListenSocketAddr {
        self.with(|l| l.local_addr())
    }

    /// Whether a connection attempt is waiting to be [`accept`](Self::accept)ed.
    pub fn can_accept(&self) -> bool {
        self.with(|l| l.can_accept())
    }

    /// Wait until a connection attempt is waiting to be accepted.
    ///
    /// This is the equivalent of [accept](#method.accept), without accepting the connection.
    pub fn wait_accept_ready(&self) -> impl Future<Output = ()> + '_ {
        poll_fn(move |cx| {
            self.with_mut(|l| {
                if l.can_accept() {
                    Poll::Ready(())
                } else {
                    l.register_accept_waker(cx.waker());
                    Poll::Pending
                }
            })
        })
    }

    /// Accept a queued connection attempt, waiting if none is queued yet.
    ///
    /// Returns an [`AcceptToken`].
    ///
    /// Pass it to [`TcpSocket::accept`] to set up a socket for the incoming connection,
    /// possibly in another task.
    ///
    /// Dropping the token forgets the attempt. The client retransmits its SYN,
    /// which queues the attempt on the listener again.
    ///
    /// # Errors
    /// - `InvalidState`: if the listener is not listening.
    pub async fn accept(&mut self) -> Result<AcceptToken, AcceptError> {
        poll_fn(|cx| {
            self.with_mut(|l| {
                if !l.is_open() {
                    return Poll::Ready(Err(AcceptError::InvalidState));
                }
                match l.accept() {
                    Some(token) => Poll::Ready(Ok(token)),
                    None => {
                        l.register_accept_waker(cx.waker());
                        Poll::Pending
                    }
                }
            })
        })
        .await
    }
}

#[cfg(feature = "tcp-listener")]
impl Drop for TcpListener<'_> {
    fn drop(&mut self) {
        self.stack.with_mut(|i| i.stack.remove_tcp_listener(self.handle));
    }
}

// =======================

#[derive(Copy, Clone)]
struct TcpIo<'d> {
    stack: Stack<'d>,
    handle: TcpHandle,
}

impl<'d> TcpIo<'d> {
    fn with<R>(&self, f: impl FnOnce(&mut tcp::TcpSocket<'_, 'd>) -> R) -> R {
        self.stack.with(|i| f(&mut i.stack.tcp_socket(self.handle)))
    }

    fn with_mut<R>(&self, f: impl FnOnce(&mut tcp::TcpSocket<'_, 'd>) -> R) -> R {
        self.stack.with_mut(|i| f(&mut i.stack.tcp_socket(self.handle)))
    }

    fn poll_read_ready(&self, cx: &mut Context<'_>) -> Poll<()> {
        self.with_mut(|s| {
            if s.can_recv() {
                Poll::Ready(())
            } else {
                s.register_recv_waker(cx.waker());
                Poll::Pending
            }
        })
    }

    fn try_read(&mut self, buf: &mut [u8]) -> Result<usize, TryError<Error>> {
        self.with_mut(|s| match s.recv_slice(buf) {
            // Reading into empty buffer
            Ok(0) if buf.is_empty() => Ok(0),
            // No data ready
            Ok(0) => Err(TryError::WouldBlock),
            // Data ready!
            Ok(n) => Ok(n),
            // EOF
            Err(tcp::RecvError::Finished) => Ok(0),
            // Connection reset.
            Err(tcp::RecvError::InvalidState) => Err(TryError::Other(Error::ConnectionReset)),
        })
    }

    fn read<'s>(&'s mut self, buf: &'s mut [u8]) -> impl Future<Output = Result<usize, Error>> + 's {
        poll_fn(|cx| {
            // CAUTION: xarxa semantics around EOF are different to what you'd expect
            // from posix-like IO, so we have to tweak things here.
            self.with_mut(|s| match s.recv_slice(buf) {
                // Reading into empty buffer
                Ok(0) if buf.is_empty() => {
                    // embedded_io_async::Read's contract is to not block if buf is empty. While
                    // this function is not a direct implementor of the trait method, we still don't
                    // want our future to never resolve.
                    Poll::Ready(Ok(0))
                }
                // No data ready
                Ok(0) => {
                    s.register_recv_waker(cx.waker());
                    Poll::Pending
                }
                // Data ready!
                Ok(n) => Poll::Ready(Ok(n)),
                // EOF
                Err(tcp::RecvError::Finished) => Poll::Ready(Ok(0)),
                // Connection reset. TODO: this can also be timeouts etc, investigate.
                Err(tcp::RecvError::InvalidState) => Poll::Ready(Err(Error::ConnectionReset)),
            })
        })
    }

    fn poll_write_ready(&self, cx: &mut Context<'_>) -> Poll<()> {
        self.with_mut(|s| {
            if s.can_send() {
                Poll::Ready(())
            } else {
                s.register_send_waker(cx.waker());
                Poll::Pending
            }
        })
    }

    fn try_write(&mut self, buf: &[u8]) -> Result<usize, TryError<Error>> {
        self.with_mut(|s| match s.send_slice(buf) {
            // Writing an empty buffer
            Ok(0) if buf.is_empty() => Ok(0),
            // Not ready to send (no space in the tx buffer)
            Ok(0) => Err(TryError::WouldBlock),
            // Some data sent
            Ok(n) => Ok(n),
            // Connection reset.
            Err(tcp::SendError::InvalidState) => Err(TryError::Other(Error::ConnectionReset)),
        })
    }

    fn write<'s>(&'s mut self, buf: &'s [u8]) -> impl Future<Output = Result<usize, Error>> + 's {
        poll_fn(|cx| {
            self.with_mut(|s| match s.send_slice(buf) {
                // Writing an empty buffer
                Ok(0) if buf.is_empty() => Poll::Ready(Ok(0)),
                // Not ready to send (no space in the tx buffer)
                Ok(0) => {
                    s.register_send_waker(cx.waker());
                    Poll::Pending
                }
                // Some data sent
                Ok(n) => Poll::Ready(Ok(n)),
                // Connection reset. TODO: this can also be timeouts etc, investigate.
                Err(tcp::SendError::InvalidState) => Poll::Ready(Err(Error::ConnectionReset)),
            })
        })
    }

    fn try_write_with<R>(&mut self, f: impl FnOnce(&mut [u8]) -> (usize, R)) -> Result<R, TryError<Error>> {
        self.with_mut(|s| {
            if !s.can_send() {
                if s.may_send() {
                    Err(TryError::WouldBlock)
                } else {
                    Err(TryError::Other(Error::ConnectionReset))
                }
            } else {
                match s.send(f) {
                    Err(tcp::SendError::InvalidState) => Err(TryError::Other(Error::ConnectionReset)),
                    Ok(r) => Ok(r),
                }
            }
        })
    }

    async fn write_with<R>(&mut self, f: impl FnOnce(&mut [u8]) -> (usize, R)) -> Result<R, Error> {
        let mut f = Some(f);
        poll_fn(move |cx| {
            self.with_mut(|s| {
                if !s.can_send() {
                    if s.may_send() {
                        // socket buffer is full wait until it has atleast one byte free
                        s.register_send_waker(cx.waker());
                        Poll::Pending
                    } else {
                        // if we can't transmit because the transmit half of the duplex connection is closed then return an error
                        Poll::Ready(Err(Error::ConnectionReset))
                    }
                } else {
                    Poll::Ready(match s.send(unwrap!(f.take())) {
                        // Connection reset. TODO: this can also be timeouts etc, investigate.
                        Err(tcp::SendError::InvalidState) => Err(Error::ConnectionReset),
                        Ok(r) => Ok(r),
                    })
                }
            })
        })
        .await
    }

    fn try_read_with<R>(&mut self, f: impl FnOnce(&mut [u8]) -> (usize, R)) -> Result<R, TryError<Error>> {
        self.with_mut(|s| {
            if !s.can_recv() {
                if s.may_recv() {
                    Err(TryError::WouldBlock)
                } else {
                    Err(TryError::Other(Error::ConnectionReset))
                }
            } else {
                match s.recv(f) {
                    Err(tcp::RecvError::Finished) | Err(tcp::RecvError::InvalidState) => {
                        Err(TryError::Other(Error::ConnectionReset))
                    }
                    Ok(r) => Ok(r),
                }
            }
        })
    }

    async fn read_with<R>(&mut self, f: impl FnOnce(&mut [u8]) -> (usize, R)) -> Result<R, Error> {
        let mut f = Some(f);
        poll_fn(move |cx| {
            self.with_mut(|s| {
                if !s.can_recv() {
                    if s.may_recv() {
                        // socket buffer is empty wait until it has atleast one byte has arrived
                        s.register_recv_waker(cx.waker());
                        Poll::Pending
                    } else {
                        // if we can't receive because the receive half of the duplex connection is closed then return an error
                        Poll::Ready(Err(Error::ConnectionReset))
                    }
                } else {
                    Poll::Ready(match s.recv(unwrap!(f.take())) {
                        // Connection reset. TODO: this can also be timeouts etc, investigate.
                        Err(tcp::RecvError::Finished) | Err(tcp::RecvError::InvalidState) => {
                            Err(Error::ConnectionReset)
                        }
                        Ok(r) => Ok(r),
                    })
                }
            })
        })
        .await
    }

    /// Map the receive-half state to a peek result: `Ok(true)` if data can be
    /// peeked, `Ok(false)` if none has arrived yet, `Err(Ok(()))` on EOF.
    fn peek_state(s: &mut tcp::TcpSocket<'_, 'd>) -> Result<bool, Result<(), Error>> {
        match s.peek(0).map(|_| ()) {
            Err(tcp::RecvError::Finished) => Err(Ok(())),
            Err(tcp::RecvError::InvalidState) => Err(Err(Error::ConnectionReset)),
            Ok(()) => Ok(s.can_recv()),
        }
    }

    fn try_peek(&mut self, buf: &mut [u8]) -> Result<usize, TryError<Error>> {
        self.with_mut(|s| match Self::peek_state(s) {
            Err(Ok(())) => Ok(0),
            Err(Err(e)) => Err(TryError::Other(e)),
            Ok(_) if buf.is_empty() => Ok(0),
            Ok(false) => Err(TryError::WouldBlock),
            Ok(true) => Ok(unwrap!(s.peek_slice(buf))),
        })
    }

    fn peek<'s>(&'s mut self, buf: &'s mut [u8]) -> impl Future<Output = Result<usize, Error>> + 's {
        poll_fn(|cx| {
            self.with_mut(|s| match Self::peek_state(s) {
                Err(Ok(())) => Poll::Ready(Ok(0)),
                Err(Err(e)) => Poll::Ready(Err(e)),
                // Don't block if buf is empty, like `read`.
                Ok(_) if buf.is_empty() => Poll::Ready(Ok(0)),
                Ok(false) => {
                    s.register_recv_waker(cx.waker());
                    Poll::Pending
                }
                Ok(true) => Poll::Ready(Ok(unwrap!(s.peek_slice(buf)))),
            })
        })
    }

    fn try_peek_with<R>(&mut self, f: impl FnOnce(&[u8]) -> R) -> Result<R, TryError<Error>> {
        self.with_mut(|s| match Self::peek_state(s) {
            Err(_) => Err(TryError::Other(Error::ConnectionReset)),
            Ok(false) => Err(TryError::WouldBlock),
            Ok(true) => Ok(f(unwrap!(s.peek(usize::MAX)))),
        })
    }

    async fn peek_with<R>(&mut self, f: impl FnOnce(&[u8]) -> R) -> Result<R, Error> {
        let mut f = Some(f);
        poll_fn(move |cx| {
            self.with_mut(|s| match Self::peek_state(s) {
                Err(_) => Poll::Ready(Err(Error::ConnectionReset)),
                Ok(false) => {
                    s.register_recv_waker(cx.waker());
                    Poll::Pending
                }
                Ok(true) => Poll::Ready(Ok(unwrap!(f.take())(unwrap!(s.peek(usize::MAX))))),
            })
        })
        .await
    }

    fn try_flush(&mut self) -> Result<(), TryError<Error>> {
        self.with_mut(|s| {
            let data_pending = (s.send_queue() > 0) && s.state() != tcp::State::Closed;
            let fin_pending = matches!(
                s.state(),
                tcp::State::FinWait1 | tcp::State::Closing | tcp::State::LastAck
            );
            let rst_pending = s.state() == tcp::State::Closed && s.remote_addr().is_some();

            if data_pending || fin_pending || rst_pending {
                Err(TryError::WouldBlock)
            } else {
                Ok(())
            }
        })
    }

    fn flush(&mut self) -> impl Future<Output = Result<(), Error>> + '_ {
        poll_fn(|cx| {
            self.with_mut(|s| {
                let data_pending = (s.send_queue() > 0) && s.state() != tcp::State::Closed;
                let fin_pending = matches!(
                    s.state(),
                    tcp::State::FinWait1 | tcp::State::Closing | tcp::State::LastAck
                );
                let rst_pending = s.state() == tcp::State::Closed && s.remote_addr().is_some();

                // If there are outstanding send operations, register for wake up and wait
                // xarxa issues wake-ups when octets are dequeued from the send buffer
                if data_pending || fin_pending || rst_pending {
                    s.register_send_waker(cx.waker());
                    Poll::Pending
                // No outstanding sends, socket is flushed
                } else {
                    Poll::Ready(Ok(()))
                }
            })
        })
    }

    fn recv_capacity(&self) -> usize {
        self.with(|s| s.recv_capacity())
    }

    fn send_capacity(&self) -> usize {
        self.with(|s| s.send_capacity())
    }

    fn send_queue(&self) -> usize {
        self.with(|s| s.send_queue())
    }

    fn recv_queue(&self) -> usize {
        self.with(|s| s.recv_queue())
    }
}

mod embedded_io_impls {
    use super::*;

    impl core::fmt::Display for ConnectError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str("ConnectError")
        }
    }
    impl core::error::Error for ConnectError {}

    impl embedded_io_async::Error for ConnectError {
        fn kind(&self) -> embedded_io_async::ErrorKind {
            match self {
                ConnectError::ConnectionReset => embedded_io_async::ErrorKind::ConnectionReset,
                ConnectError::TimedOut => embedded_io_async::ErrorKind::TimedOut,
                ConnectError::Unaddressable | ConnectError::NoFreePorts => embedded_io_async::ErrorKind::NotConnected,
                ConnectError::InvalidState | ConnectError::InUse => embedded_io_async::ErrorKind::Other,
            }
        }
    }

    impl core::fmt::Display for Error {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                Self::ConnectionReset => f.write_str("ConnectionReset"),
            }
        }
    }
    impl core::error::Error for Error {}

    impl embedded_io_async::Error for Error {
        fn kind(&self) -> embedded_io_async::ErrorKind {
            match self {
                Error::ConnectionReset => embedded_io_async::ErrorKind::ConnectionReset,
            }
        }
    }

    impl embedded_io_async::ErrorType for TcpSocket<'_, '_> {
        type Error = Error;
    }

    impl embedded_io_async::Read for TcpSocket<'_, '_> {
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            self.io.read(buf).await
        }
    }

    impl embedded_io_async::ReadReady for TcpSocket<'_, '_> {
        fn read_ready(&mut self) -> Result<bool, Self::Error> {
            Ok(self.io.with(|s| s.can_recv() || !s.may_recv()))
        }
    }

    impl embedded_io_async::Write for TcpSocket<'_, '_> {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            self.io.write(buf).await
        }

        async fn flush(&mut self) -> Result<(), Self::Error> {
            self.io.flush().await
        }
    }

    impl embedded_io_async::WriteReady for TcpSocket<'_, '_> {
        fn write_ready(&mut self) -> Result<bool, Self::Error> {
            Ok(self.io.with(|s| s.can_send()))
        }
    }

    impl embedded_io_async::ErrorType for TcpReader<'_, '_> {
        type Error = Error;
    }

    impl embedded_io_async::Read for TcpReader<'_, '_> {
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            self.io.read(buf).await
        }
    }

    impl embedded_io_async::ReadReady for TcpReader<'_, '_> {
        fn read_ready(&mut self) -> Result<bool, Self::Error> {
            Ok(self.io.with(|s| s.can_recv() || !s.may_recv()))
        }
    }

    impl embedded_io_async::ErrorType for TcpWriter<'_, '_> {
        type Error = Error;
    }

    impl embedded_io_async::Write for TcpWriter<'_, '_> {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            self.io.write(buf).await
        }

        async fn flush(&mut self) -> Result<(), Self::Error> {
            self.io.flush().await
        }
    }

    impl embedded_io_async::WriteReady for TcpWriter<'_, '_> {
        fn write_ready(&mut self) -> Result<bool, Self::Error> {
            Ok(self.io.with(|s| s.can_send()))
        }
    }
}

#[cfg(feature = "tcp-listener")]
impl core::fmt::Display for AcceptError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidState => f.write_str("InvalidState"),
            Self::ConnectionReset => f.write_str("ConnectionReset"),
        }
    }
}
#[cfg(feature = "tcp-listener")]
impl core::error::Error for AcceptError {}

#[cfg(feature = "tcp-listener")]
impl core::fmt::Display for ListenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidState => f.write_str("InvalidState"),
            Self::Unaddressable => f.write_str("Unaddressable"),
            Self::InUse => f.write_str("InUse"),
        }
    }
}
#[cfg(feature = "tcp-listener")]
impl core::error::Error for ListenError {}

/// TCP client compatible with `embedded-nal-async` traits.
#[cfg(feature = "embedded-nal")]
pub mod client {
    use core::cell::{Cell, UnsafeCell};
    use core::mem::MaybeUninit;
    use core::net::IpAddr;
    use core::ptr::NonNull;

    use super::*;

    /// TCP client connection pool compatible with `embedded-nal-async` traits.
    ///
    /// The pool is capable of managing up to N concurrent connections with tx and rx buffers according to TX_SZ and RX_SZ.
    pub struct TcpClient<'d, const N: usize, const TX_SZ: usize = 1024, const RX_SZ: usize = 1024> {
        stack: Stack<'d>,
        state: &'d TcpClientState<N, TX_SZ, RX_SZ>,
        socket_timeout: Option<Duration>,
    }

    impl<'d, const N: usize, const TX_SZ: usize, const RX_SZ: usize> TcpClient<'d, N, TX_SZ, RX_SZ> {
        /// Create a new `TcpClient`.
        pub fn new(stack: Stack<'d>, state: &'d TcpClientState<N, TX_SZ, RX_SZ>) -> Self {
            Self {
                stack,
                state,
                socket_timeout: None,
            }
        }

        /// Set the timeout for each socket created by this `TcpClient`.
        ///
        /// If the timeout is set, the socket will be closed if no data is received for the
        /// specified duration.
        pub fn set_timeout(&mut self, timeout: Option<Duration>) {
            self.socket_timeout = timeout;
        }
    }

    impl<'d, const N: usize, const TX_SZ: usize, const RX_SZ: usize> embedded_nal_async::TcpConnect
        for TcpClient<'d, N, TX_SZ, RX_SZ>
    {
        type Error = Error;
        type Connection<'m>
            = TcpConnection<'m, 'd, N, TX_SZ, RX_SZ>
        where
            Self: 'm;

        async fn connect<'a>(&'a self, remote: core::net::SocketAddr) -> Result<Self::Connection<'a>, Self::Error> {
            let addr: crate::wire::IpAddr = match remote.ip() {
                #[cfg(feature = "ipv4")]
                IpAddr::V4(addr) => crate::wire::IpAddr::V4(addr),
                #[cfg(not(feature = "ipv4"))]
                IpAddr::V4(_) => panic!("ipv4 support not enabled"),
                #[cfg(feature = "ipv6")]
                IpAddr::V6(addr) => crate::wire::IpAddr::V6(addr),
                #[cfg(not(feature = "ipv6"))]
                IpAddr::V6(_) => panic!("ipv6 support not enabled"),
            };
            let remote_addr = (addr, remote.port());
            let mut socket = TcpConnection::new(self.stack, self.state)?;
            socket.socket.set_timeout(self.socket_timeout);
            socket
                .socket
                .connect(remote_addr)
                .await
                .map_err(|_| Error::ConnectionReset)?;
            Ok(socket)
        }
    }

    /// Opened TCP connection in a [`TcpClient`].
    ///
    /// `'a` is the lifetime of the borrow of the [`TcpClient`] (whose pool the
    /// connection's buffers come from), `'d` the lifetime of the stack.
    pub struct TcpConnection<'a, 'd, const N: usize, const TX_SZ: usize, const RX_SZ: usize> {
        socket: TcpSocket<'a, 'd>,
        state: &'a TcpClientState<N, TX_SZ, RX_SZ>,
        bufs: NonNull<([u8; TX_SZ], [u8; RX_SZ])>,
    }

    impl<'a, 'd, const N: usize, const TX_SZ: usize, const RX_SZ: usize> TcpConnection<'a, 'd, N, TX_SZ, RX_SZ> {
        fn new(stack: Stack<'d>, state: &'a TcpClientState<N, TX_SZ, RX_SZ>) -> Result<Self, Error> {
            let mut bufs = state.pool.alloc().ok_or(Error::ConnectionReset)?;
            let socket = unsafe { TcpSocket::new(stack, &mut bufs.as_mut().1, &mut bufs.as_mut().0) };
            let socket = match socket {
                Ok(socket) => socket,
                Err(Full) => {
                    unsafe { state.pool.free(bufs) };
                    return Err(Error::ConnectionReset);
                }
            };
            Ok(Self { socket, state, bufs })
        }
    }

    impl<const N: usize, const TX_SZ: usize, const RX_SZ: usize> Drop for TcpConnection<'_, '_, N, TX_SZ, RX_SZ> {
        fn drop(&mut self) {
            unsafe {
                self.socket.close();
                self.state.pool.free(self.bufs);
            }
        }
    }

    impl<const N: usize, const TX_SZ: usize, const RX_SZ: usize> embedded_io_async::ErrorType
        for TcpConnection<'_, '_, N, TX_SZ, RX_SZ>
    {
        type Error = Error;
    }

    impl<const N: usize, const TX_SZ: usize, const RX_SZ: usize> embedded_io_async::Read
        for TcpConnection<'_, '_, N, TX_SZ, RX_SZ>
    {
        async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
            self.socket.read(buf).await
        }
    }

    impl<const N: usize, const TX_SZ: usize, const RX_SZ: usize> embedded_io_async::Write
        for TcpConnection<'_, '_, N, TX_SZ, RX_SZ>
    {
        async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            self.socket.write(buf).await
        }

        async fn flush(&mut self) -> Result<(), Self::Error> {
            self.socket.flush().await
        }
    }

    /// State for TcpClient
    pub struct TcpClientState<const N: usize, const TX_SZ: usize, const RX_SZ: usize> {
        pool: Pool<([u8; TX_SZ], [u8; RX_SZ]), N>,
    }

    impl<const N: usize, const TX_SZ: usize, const RX_SZ: usize> TcpClientState<N, TX_SZ, RX_SZ> {
        /// Create a new `TcpClientState`.
        pub const fn new() -> Self {
            Self { pool: Pool::new() }
        }
    }

    struct Pool<T, const N: usize> {
        used: [Cell<bool>; N],
        data: [UnsafeCell<MaybeUninit<T>>; N],
    }

    impl<T, const N: usize> Pool<T, N> {
        const VALUE: Cell<bool> = Cell::new(false);
        const UNINIT: UnsafeCell<MaybeUninit<T>> = UnsafeCell::new(MaybeUninit::uninit());

        const fn new() -> Self {
            Self {
                used: [Self::VALUE; N],
                data: [Self::UNINIT; N],
            }
        }
    }

    impl<T, const N: usize> Pool<T, N> {
        fn alloc(&self) -> Option<NonNull<T>> {
            for n in 0..N {
                // this can't race because Pool is not Sync.
                if !self.used[n].get() {
                    self.used[n].set(true);
                    let p = self.data[n].get() as *mut T;
                    return Some(unsafe { NonNull::new_unchecked(p) });
                }
            }
            None
        }

        /// safety: p must be a pointer obtained from self.alloc that hasn't been freed yet.
        unsafe fn free(&self, p: NonNull<T>) {
            let origin = self.data.as_ptr() as *mut T;
            let n = p.as_ptr().offset_from(origin);
            assert!(n >= 0);
            assert!((n as usize) < N);
            self.used[n as usize].set(false);
        }
    }
}

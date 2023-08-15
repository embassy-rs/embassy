//! UDP sockets.

use core::cell::RefCell;
use core::future::poll_fn;
use core::mem;
use core::task::{Context, Poll};

use embassy_net_driver::Driver;
use smoltcp::iface::{Interface, SocketHandle};
use smoltcp::socket::udp;
pub use smoltcp::socket::udp::PacketMetadata;
use smoltcp::wire::{IpEndpoint, IpListenEndpoint};

use crate::{SocketStack, Stack};

/// Error returned by [`UdpSocket::bind`].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BindError {
    /// The socket was already open.
    InvalidState,
    /// No route to host.
    NoRoute,
}

/// Error returned by [`UdpSocket::recv_from`] and [`UdpSocket::send_to`].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// No route to host.
    NoRoute,
}

/// An UDP socket.
pub struct UdpSocket<'a> {
    stack: &'a RefCell<SocketStack>,
    handle: SocketHandle,
}

impl<'a> UdpSocket<'a> {
    /// Create a new UDP socket using the provided stack and buffers.
    pub fn new<D: Driver>(
        stack: &'a Stack<D>,
        rx_meta: &'a mut [PacketMetadata],
        rx_buffer: &'a mut [u8],
        tx_meta: &'a mut [PacketMetadata],
        tx_buffer: &'a mut [u8],
    ) -> Self {
        let s = &mut *stack.socket.borrow_mut();

        let rx_meta: &'static mut [PacketMetadata] = unsafe { mem::transmute(rx_meta) };
        let rx_buffer: &'static mut [u8] = unsafe { mem::transmute(rx_buffer) };
        let tx_meta: &'static mut [PacketMetadata] = unsafe { mem::transmute(tx_meta) };
        let tx_buffer: &'static mut [u8] = unsafe { mem::transmute(tx_buffer) };
        let handle = s.sockets.add(udp::Socket::new(
            udp::PacketBuffer::new(rx_meta, rx_buffer),
            udp::PacketBuffer::new(tx_meta, tx_buffer),
        ));

        Self {
            stack: &stack.socket,
            handle,
        }
    }

    /// Bind the socket to a local endpoint.
    pub fn bind<T>(&mut self, endpoint: T) -> Result<(), BindError>
    where
        T: Into<IpListenEndpoint>,
    {
        let mut endpoint = endpoint.into();

        if endpoint.port == 0 {
            // If user didn't specify port allocate a dynamic port.
            endpoint.port = self.stack.borrow_mut().get_local_port();
        }

        match self.with_mut(|s, _| s.bind(endpoint)) {
            Ok(()) => Ok(()),
            Err(udp::BindError::InvalidState) => Err(BindError::InvalidState),
            Err(udp::BindError::Unaddressable) => Err(BindError::NoRoute),
        }
    }

    fn with<R>(&self, f: impl FnOnce(&udp::Socket, &Interface) -> R) -> R {
        let s = &*self.stack.borrow();
        let socket = s.sockets.get::<udp::Socket>(self.handle);
        f(socket, &s.iface)
    }

    fn with_mut<R>(&self, f: impl FnOnce(&mut udp::Socket, &mut Interface) -> R) -> R {
        let s = &mut *self.stack.borrow_mut();
        let socket = s.sockets.get_mut::<udp::Socket>(self.handle);
        let res = f(socket, &mut s.iface);
        s.waker.wake();
        res
    }

    /// Receive a datagram.
    ///
    /// This method will wait until a datagram is received.
    ///
    /// Returns the number of bytes received and the remote endpoint.
    pub async fn recv_from(&self, buf: &mut [u8]) -> Result<(usize, IpEndpoint), Error> {
        poll_fn(move |cx| self.poll_recv_from(buf, cx)).await
    }

    /// Receive a datagram.
    ///
    /// When no datagram is available, this method will return `Poll::Pending` and
    /// register the current task to be notified when a datagram is received.
    ///
    /// When a datagram is received, this method will return `Poll::Ready` with the
    /// number of bytes received and the remote endpoint.
    pub fn poll_recv_from(&self, buf: &mut [u8], cx: &mut Context<'_>) -> Poll<Result<(usize, IpEndpoint), Error>> {
        self.with_mut(|s, _| match s.recv_slice(buf) {
            Ok((n, meta)) => Poll::Ready(Ok((n, meta.endpoint))),
            // No data ready
            Err(udp::RecvError::Exhausted) => {
                s.register_recv_waker(cx.waker());
                Poll::Pending
            }
        })
    }

    /// Send a datagram to the specified remote endpoint.
    ///
    /// This method will wait until the datagram has been sent.
    ///
    /// When the remote endpoint is not reachable, this method will return `Err(Error::NoRoute)`
    pub async fn send_to<T>(&self, buf: &[u8], remote_endpoint: T) -> Result<(), Error>
    where
        T: Into<IpEndpoint>,
    {
        let remote_endpoint: IpEndpoint = remote_endpoint.into();
        poll_fn(move |cx| self.poll_send_to(buf, remote_endpoint, cx)).await
    }

    /// Send a datagram to the specified remote endpoint.
    ///
    /// When the datagram has been sent, this method will return `Poll::Ready(Ok())`.
    ///
    /// When the socket's send buffer is full, this method will return `Poll::Pending`
    /// and register the current task to be notified when the buffer has space available.
    ///
    /// When the remote endpoint is not reachable, this method will return `Poll::Ready(Err(Error::NoRoute))`.
    pub fn poll_send_to<T>(&self, buf: &[u8], remote_endpoint: T, cx: &mut Context<'_>) -> Poll<Result<(), Error>>
    where
        T: Into<IpEndpoint>,
    {
        self.with_mut(|s, _| match s.send_slice(buf, remote_endpoint) {
            // Entire datagram has been sent
            Ok(()) => Poll::Ready(Ok(())),
            Err(udp::SendError::BufferFull) => {
                s.register_send_waker(cx.waker());
                Poll::Pending
            }
            Err(udp::SendError::Unaddressable) => Poll::Ready(Err(Error::NoRoute)),
        })
    }

    /// Returns the local endpoint of the socket.
    pub fn endpoint(&self) -> IpListenEndpoint {
        self.with(|s, _| s.endpoint())
    }

    /// Returns whether the socket is open.

    pub fn is_open(&self) -> bool {
        self.with(|s, _| s.is_open())
    }

    /// Close the socket.
    pub fn close(&mut self) {
        self.with_mut(|s, _| s.close())
    }

    /// Returns whether the socket is ready to send data, i.e. it has enough buffer space to hold a packet.
    pub fn may_send(&self) -> bool {
        self.with(|s, _| s.can_send())
    }

    /// Returns whether the socket is ready to receive data, i.e. it has received a packet that's now in the buffer.
    pub fn may_recv(&self) -> bool {
        self.with(|s, _| s.can_recv())
    }

    /// Return the maximum number packets the socket can receive.
    pub fn packet_recv_capacity(&self) -> usize {
        self.with(|s, _| s.packet_recv_capacity())
    }

    /// Return the maximum number packets the socket can receive.
    pub fn packet_send_capacity(&self) -> usize {
        self.with(|s, _| s.packet_send_capacity())
    }

    /// Return the maximum number of bytes inside the recv buffer.
    pub fn payload_recv_capacity(&self) -> usize {
        self.with(|s, _| s.payload_recv_capacity())
    }

    /// Return the maximum number of bytes inside the transmit buffer.
    pub fn payload_send_capacity(&self) -> usize {
        self.with(|s, _| s.payload_send_capacity())
    }
}

impl Drop for UdpSocket<'_> {
    fn drop(&mut self) {
        self.stack.borrow_mut().sockets.remove(self.handle);
    }
}

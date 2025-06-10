//! UDP sockets.

use core::future::{poll_fn, Future};
use core::mem;
use core::task::{Context, Poll};

use smoltcp::iface::{Interface, SocketHandle};
use smoltcp::socket::udp;
pub use smoltcp::socket::udp::{PacketMetadata, UdpMetadata};
use smoltcp::wire::IpListenEndpoint;

use crate::Stack;

/// Error returned by [`UdpSocket::bind`].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BindError {
    /// The socket was already open.
    InvalidState,
    /// No route to host.
    NoRoute,
}

/// Error returned by [`UdpSocket::send_to`].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SendError {
    /// No route to host.
    NoRoute,
    /// Socket not bound to an outgoing port.
    SocketNotBound,
    /// There is not enough transmit buffer capacity to ever send this packet.
    PacketTooLarge,
}

/// Error returned by [`UdpSocket::recv_from`].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RecvError {
    /// Provided buffer was smaller than the received packet.
    Truncated,
}

/// An UDP socket.
pub struct UdpSocket<'a> {
    stack: Stack<'a>,
    handle: SocketHandle,
}

impl<'a> UdpSocket<'a> {
    /// Create a new UDP socket using the provided stack and buffers.
    pub fn new(
        stack: Stack<'a>,
        rx_meta: &'a mut [PacketMetadata],
        rx_buffer: &'a mut [u8],
        tx_meta: &'a mut [PacketMetadata],
        tx_buffer: &'a mut [u8],
    ) -> Self {
        let handle = stack.with_mut(|i| {
            let rx_meta: &'static mut [PacketMetadata] = unsafe { mem::transmute(rx_meta) };
            let rx_buffer: &'static mut [u8] = unsafe { mem::transmute(rx_buffer) };
            let tx_meta: &'static mut [PacketMetadata] = unsafe { mem::transmute(tx_meta) };
            let tx_buffer: &'static mut [u8] = unsafe { mem::transmute(tx_buffer) };
            i.sockets.add(udp::Socket::new(
                udp::PacketBuffer::new(rx_meta, rx_buffer),
                udp::PacketBuffer::new(tx_meta, tx_buffer),
            ))
        });

        Self { stack, handle }
    }

    /// Bind the socket to a local endpoint.
    pub fn bind<T>(&mut self, endpoint: T) -> Result<(), BindError>
    where
        T: Into<IpListenEndpoint>,
    {
        let mut endpoint = endpoint.into();

        if endpoint.port == 0 {
            // If user didn't specify port allocate a dynamic port.
            endpoint.port = self.stack.with_mut(|i| i.get_local_port());
        }

        match self.with_mut(|s, _| s.bind(endpoint)) {
            Ok(()) => Ok(()),
            Err(udp::BindError::InvalidState) => Err(BindError::InvalidState),
            Err(udp::BindError::Unaddressable) => Err(BindError::NoRoute),
        }
    }

    fn with<R>(&self, f: impl FnOnce(&udp::Socket, &Interface) -> R) -> R {
        self.stack.with(|i| {
            let socket = i.sockets.get::<udp::Socket>(self.handle);
            f(socket, &i.iface)
        })
    }

    fn with_mut<R>(&self, f: impl FnOnce(&mut udp::Socket, &mut Interface) -> R) -> R {
        self.stack.with_mut(|i| {
            let socket = i.sockets.get_mut::<udp::Socket>(self.handle);
            let res = f(socket, &mut i.iface);
            i.waker.wake();
            res
        })
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
        self.with_mut(|s, _| {
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
    /// Returns the number of bytes received and the remote endpoint.
    pub fn recv_from<'s>(
        &'s self,
        buf: &'s mut [u8],
    ) -> impl Future<Output = Result<(usize, UdpMetadata), RecvError>> + 's {
        poll_fn(|cx| self.poll_recv_from(buf, cx))
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
        self.with_mut(|s, _| match s.recv_slice(buf) {
            Ok((n, meta)) => Poll::Ready(Ok((n, meta))),
            // No data ready
            Err(udp::RecvError::Truncated) => Poll::Ready(Err(RecvError::Truncated)),
            Err(udp::RecvError::Exhausted) => {
                s.register_recv_waker(cx.waker());
                Poll::Pending
            }
        })
    }

    /// Receive a datagram with a zero-copy function.
    ///
    /// When no datagram is available, this method will return `Poll::Pending` and
    /// register the current task to be notified when a datagram is received.
    ///
    /// When a datagram is received, this method will call the provided function
    /// with the number of bytes received and the remote endpoint and return
    /// `Poll::Ready` with the function's returned value.
    pub async fn recv_from_with<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&[u8], UdpMetadata) -> R,
    {
        let mut f = Some(f);
        poll_fn(move |cx| {
            self.with_mut(|s, _| {
                match s.recv() {
                    Ok((buffer, endpoint)) => Poll::Ready(unwrap!(f.take())(buffer, endpoint)),
                    Err(udp::RecvError::Truncated) => unreachable!(),
                    Err(udp::RecvError::Exhausted) => {
                        // socket buffer is empty wait until at least one byte has arrived
                        s.register_recv_waker(cx.waker());
                        Poll::Pending
                    }
                }
            })
        })
        .await
    }

    /// Wait until the socket becomes writable.
    ///
    /// A socket becomes writable when there is space in the buffer, from initial memory or after
    /// dispatching datagrams on a full buffer.
    pub fn wait_send_ready(&self) -> impl Future<Output = ()> + '_ {
        poll_fn(|cx| self.poll_send_ready(cx))
    }

    /// Wait until a datagram can be sent.
    ///
    /// When no datagram can be sent (i.e. the buffer is full), this method will return
    /// `Poll::Pending` and register the current task to be notified when
    /// space is freed in the buffer after a datagram has been dispatched.
    ///
    /// When a datagram can be sent, this method will return `Poll::Ready`.
    pub fn poll_send_ready(&self, cx: &mut Context<'_>) -> Poll<()> {
        self.with_mut(|s, _| {
            if s.can_send() {
                Poll::Ready(())
            } else {
                // socket buffer is full wait until a datagram has been dispatched
                s.register_send_waker(cx.waker());
                Poll::Pending
            }
        })
    }

    /// Send a datagram to the specified remote endpoint.
    ///
    /// This method will wait until the datagram has been sent.
    ///
    /// If the socket's send buffer is too small to fit `buf`, this method will return `Err(SendError::PacketTooLarge)`
    ///
    /// When the remote endpoint is not reachable, this method will return `Err(SendError::NoRoute)`
    pub async fn send_to<T>(&self, buf: &[u8], remote_endpoint: T) -> Result<(), SendError>
    where
        T: Into<UdpMetadata>,
    {
        let remote_endpoint: UdpMetadata = remote_endpoint.into();
        poll_fn(move |cx| self.poll_send_to(buf, remote_endpoint, cx)).await
    }

    /// Send a datagram to the specified remote endpoint.
    ///
    /// When the datagram has been sent, this method will return `Poll::Ready(Ok())`.
    ///
    /// When the socket's send buffer is full, this method will return `Poll::Pending`
    /// and register the current task to be notified when the buffer has space available.
    ///
    /// If the socket's send buffer is too small to fit `buf`, this method will return `Poll::Ready(Err(SendError::PacketTooLarge))`
    ///
    /// When the remote endpoint is not reachable, this method will return `Poll::Ready(Err(Error::NoRoute))`.
    pub fn poll_send_to<T>(&self, buf: &[u8], remote_endpoint: T, cx: &mut Context<'_>) -> Poll<Result<(), SendError>>
    where
        T: Into<UdpMetadata>,
    {
        // Don't need to wake waker in `with_mut` if the buffer will never fit the udp tx_buffer.
        let send_capacity_too_small = self.with(|s, _| s.payload_send_capacity() < buf.len());
        if send_capacity_too_small {
            return Poll::Ready(Err(SendError::PacketTooLarge));
        }

        self.with_mut(|s, _| match s.send_slice(buf, remote_endpoint) {
            // Entire datagram has been sent
            Ok(()) => Poll::Ready(Ok(())),
            Err(udp::SendError::BufferFull) => {
                s.register_send_waker(cx.waker());
                Poll::Pending
            }
            Err(udp::SendError::Unaddressable) => {
                // If no sender/outgoing port is specified, there is not really "no route"
                if s.endpoint().port == 0 {
                    Poll::Ready(Err(SendError::SocketNotBound))
                } else {
                    Poll::Ready(Err(SendError::NoRoute))
                }
            }
        })
    }

    /// Send a datagram to the specified remote endpoint with a zero-copy function.
    ///
    /// This method will wait until the buffer can fit the requested size before
    /// calling the function to fill its contents.
    ///
    /// If the socket's send buffer is too small to fit `size`, this method will return `Err(SendError::PacketTooLarge)`
    ///
    /// When the remote endpoint is not reachable, this method will return `Err(SendError::NoRoute)`
    pub async fn send_to_with<T, F, R>(&mut self, size: usize, remote_endpoint: T, f: F) -> Result<R, SendError>
    where
        T: Into<UdpMetadata> + Copy,
        F: FnOnce(&mut [u8]) -> R,
    {
        // Don't need to wake waker in `with_mut` if the buffer will never fit the udp tx_buffer.
        let send_capacity_too_small = self.with(|s, _| s.payload_send_capacity() < size);
        if send_capacity_too_small {
            return Err(SendError::PacketTooLarge);
        }

        let mut f = Some(f);
        poll_fn(move |cx| {
            self.with_mut(|s, _| {
                match s.send(size, remote_endpoint) {
                    Ok(buffer) => Poll::Ready(Ok(unwrap!(f.take())(buffer))),
                    Err(udp::SendError::BufferFull) => {
                        s.register_send_waker(cx.waker());
                        Poll::Pending
                    }
                    Err(udp::SendError::Unaddressable) => {
                        // If no sender/outgoing port is specified, there is not really "no route"
                        if s.endpoint().port == 0 {
                            Poll::Ready(Err(SendError::SocketNotBound))
                        } else {
                            Poll::Ready(Err(SendError::NoRoute))
                        }
                    }
                }
            })
        })
        .await
    }

    /// Flush the socket.
    ///
    /// This method will wait until the socket is flushed.
    pub fn flush(&mut self) -> impl Future<Output = ()> + '_ {
        poll_fn(|cx| {
            self.with_mut(|s, _| {
                if s.send_queue() == 0 {
                    Poll::Ready(())
                } else {
                    s.register_send_waker(cx.waker());
                    Poll::Pending
                }
            })
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

    /// Set the hop limit field in the IP header of sent packets.
    pub fn set_hop_limit(&mut self, hop_limit: Option<u8>) {
        self.with_mut(|s, _| s.set_hop_limit(hop_limit))
    }
}

impl Drop for UdpSocket<'_> {
    fn drop(&mut self) {
        self.stack.with_mut(|i| i.sockets.remove(self.handle));
    }
}

fn _assert_covariant<'a, 'b: 'a>(x: UdpSocket<'b>) -> UdpSocket<'a> {
    x
}

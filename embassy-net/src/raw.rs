//! Raw sockets.

use core::future::poll_fn;
use core::mem;
use core::task::{Context, Poll};

use embassy_net_driver::Driver;
use smoltcp::iface::{Interface, SocketHandle};
use smoltcp::socket::raw;
pub use smoltcp::socket::raw::PacketMetadata;
use smoltcp::wire::{IpProtocol, IpVersion};

use crate::Stack;

/// Error returned by [`RawSocket::recv`] and [`RawSocket::send`].
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RecvError {
    /// Provided buffer was smaller than the received packet.
    Truncated,
}

/// An Raw socket.
pub struct RawSocket<'a> {
    stack: Stack<'a>,
    handle: SocketHandle,
}

impl<'a> RawSocket<'a> {
    /// Create a new Raw socket using the provided stack and buffers.
    pub fn new<D: Driver>(
        stack: Stack<'a>,
        ip_version: IpVersion,
        ip_protocol: IpProtocol,
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
            i.sockets.add(raw::Socket::new(
                ip_version,
                ip_protocol,
                raw::PacketBuffer::new(rx_meta, rx_buffer),
                raw::PacketBuffer::new(tx_meta, tx_buffer),
            ))
        });

        Self { stack, handle }
    }

    fn with_mut<R>(&self, f: impl FnOnce(&mut raw::Socket, &mut Interface) -> R) -> R {
        self.stack.with_mut(|i| {
            let socket = i.sockets.get_mut::<raw::Socket>(self.handle);
            let res = f(socket, &mut i.iface);
            i.waker.wake();
            res
        })
    }

    /// Receive a datagram.
    ///
    /// This method will wait until a datagram is received.
    pub async fn recv(&self, buf: &mut [u8]) -> Result<usize, RecvError> {
        poll_fn(move |cx| self.poll_recv(buf, cx)).await
    }

    /// Receive a datagram.
    ///
    /// When no datagram is available, this method will return `Poll::Pending` and
    /// register the current task to be notified when a datagram is received.
    pub fn poll_recv(&self, buf: &mut [u8], cx: &mut Context<'_>) -> Poll<Result<usize, RecvError>> {
        self.with_mut(|s, _| match s.recv_slice(buf) {
            Ok(n) => Poll::Ready(Ok(n)),
            // No data ready
            Err(raw::RecvError::Truncated) => Poll::Ready(Err(RecvError::Truncated)),
            Err(raw::RecvError::Exhausted) => {
                s.register_recv_waker(cx.waker());
                Poll::Pending
            }
        })
    }

    /// Send a datagram.
    ///
    /// This method will wait until the datagram has been sent.`
    pub async fn send(&self, buf: &[u8]) {
        poll_fn(move |cx| self.poll_send(buf, cx)).await
    }

    /// Send a datagram.
    ///
    /// When the datagram has been sent, this method will return `Poll::Ready(Ok())`.
    ///
    /// When the socket's send buffer is full, this method will return `Poll::Pending`
    /// and register the current task to be notified when the buffer has space available.
    pub fn poll_send(&self, buf: &[u8], cx: &mut Context<'_>) -> Poll<()> {
        self.with_mut(|s, _| match s.send_slice(buf) {
            // Entire datagram has been sent
            Ok(()) => Poll::Ready(()),
            Err(raw::SendError::BufferFull) => {
                s.register_send_waker(cx.waker());
                Poll::Pending
            }
        })
    }
}

impl Drop for RawSocket<'_> {
    fn drop(&mut self) {
        self.stack.with_mut(|i| i.sockets.remove(self.handle));
    }
}

fn _assert_covariant<'a, 'b: 'a>(x: RawSocket<'b>) -> RawSocket<'a> {
    x
}

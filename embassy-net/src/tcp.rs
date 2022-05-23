use core::cell::UnsafeCell;
use core::future::Future;
use core::mem;
use core::task::Poll;
use futures::future::poll_fn;
use smoltcp::iface::{Interface, SocketHandle};
use smoltcp::socket::tcp;
use smoltcp::time::Duration;
use smoltcp::wire::IpEndpoint;
use smoltcp::wire::IpListenEndpoint;

use crate::stack::SocketStack;
use crate::Device;

use super::stack::Stack;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    ConnectionReset,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ConnectError {
    /// The socket is already connected or listening.
    InvalidState,
    /// The remote host rejected the connection with a RST packet.
    ConnectionReset,
    /// Connect timed out.
    TimedOut,
    /// No route to host.
    NoRoute,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AcceptError {
    /// The socket is already connected or listening.
    InvalidState,
    /// Invalid listen port
    InvalidPort,
    /// The remote host rejected the connection with a RST packet.
    ConnectionReset,
}

pub struct TcpSocket<'a> {
    io: TcpIo<'a>,
}

pub struct TcpReader<'a> {
    io: TcpIo<'a>,
}

pub struct TcpWriter<'a> {
    io: TcpIo<'a>,
}

impl<'a> TcpSocket<'a> {
    pub fn new<D: Device>(
        stack: &'a Stack<D>,
        rx_buffer: &'a mut [u8],
        tx_buffer: &'a mut [u8],
    ) -> Self {
        // safety: not accessed reentrantly.
        let s = unsafe { &mut *stack.socket.get() };
        let rx_buffer: &'static mut [u8] = unsafe { mem::transmute(rx_buffer) };
        let tx_buffer: &'static mut [u8] = unsafe { mem::transmute(tx_buffer) };
        let handle = s.sockets.add(tcp::Socket::new(
            tcp::SocketBuffer::new(rx_buffer),
            tcp::SocketBuffer::new(tx_buffer),
        ));

        Self {
            io: TcpIo {
                stack: &stack.socket,
                handle,
            },
        }
    }

    pub fn split(&mut self) -> (TcpReader<'_>, TcpWriter<'_>) {
        (TcpReader { io: self.io }, TcpWriter { io: self.io })
    }

    pub async fn connect<T>(&mut self, remote_endpoint: T) -> Result<(), ConnectError>
    where
        T: Into<IpEndpoint>,
    {
        // safety: not accessed reentrantly.
        let local_port = unsafe { &mut *self.io.stack.get() }.get_local_port();

        // safety: not accessed reentrantly.
        match unsafe {
            self.io
                .with_mut(|s, i| s.connect(i, remote_endpoint, local_port))
        } {
            Ok(()) => {}
            Err(tcp::ConnectError::InvalidState) => return Err(ConnectError::InvalidState),
            Err(tcp::ConnectError::Unaddressable) => return Err(ConnectError::NoRoute),
        }

        futures::future::poll_fn(|cx| unsafe {
            self.io.with_mut(|s, _| match s.state() {
                tcp::State::Closed | tcp::State::TimeWait => {
                    Poll::Ready(Err(ConnectError::ConnectionReset))
                }
                tcp::State::Listen => unreachable!(),
                tcp::State::SynSent | tcp::State::SynReceived => {
                    s.register_send_waker(cx.waker());
                    Poll::Pending
                }
                _ => Poll::Ready(Ok(())),
            })
        })
        .await
    }

    pub async fn accept<T>(&mut self, local_endpoint: T) -> Result<(), AcceptError>
    where
        T: Into<IpListenEndpoint>,
    {
        // safety: not accessed reentrantly.
        match unsafe { self.io.with_mut(|s, _| s.listen(local_endpoint)) } {
            Ok(()) => {}
            Err(tcp::ListenError::InvalidState) => return Err(AcceptError::InvalidState),
            Err(tcp::ListenError::Unaddressable) => return Err(AcceptError::InvalidPort),
        }

        futures::future::poll_fn(|cx| unsafe {
            self.io.with_mut(|s, _| match s.state() {
                tcp::State::Listen | tcp::State::SynSent | tcp::State::SynReceived => {
                    s.register_send_waker(cx.waker());
                    Poll::Pending
                }
                _ => Poll::Ready(Ok(())),
            })
        })
        .await
    }

    pub fn set_timeout(&mut self, duration: Option<Duration>) {
        unsafe { self.io.with_mut(|s, _| s.set_timeout(duration)) }
    }

    pub fn set_keep_alive(&mut self, interval: Option<Duration>) {
        unsafe { self.io.with_mut(|s, _| s.set_keep_alive(interval)) }
    }

    pub fn set_hop_limit(&mut self, hop_limit: Option<u8>) {
        unsafe { self.io.with_mut(|s, _| s.set_hop_limit(hop_limit)) }
    }

    pub fn local_endpoint(&self) -> Option<IpEndpoint> {
        unsafe { self.io.with(|s, _| s.local_endpoint()) }
    }

    pub fn remote_endpoint(&self) -> Option<IpEndpoint> {
        unsafe { self.io.with(|s, _| s.remote_endpoint()) }
    }

    pub fn state(&self) -> tcp::State {
        unsafe { self.io.with(|s, _| s.state()) }
    }

    pub fn close(&mut self) {
        unsafe { self.io.with_mut(|s, _| s.close()) }
    }

    pub fn abort(&mut self) {
        unsafe { self.io.with_mut(|s, _| s.abort()) }
    }

    pub fn may_send(&self) -> bool {
        unsafe { self.io.with(|s, _| s.may_send()) }
    }

    pub fn may_recv(&self) -> bool {
        unsafe { self.io.with(|s, _| s.may_recv()) }
    }
}

impl<'a> Drop for TcpSocket<'a> {
    fn drop(&mut self) {
        // safety: not accessed reentrantly.
        let s = unsafe { &mut *self.io.stack.get() };
        s.sockets.remove(self.io.handle);
    }
}

// =======================

#[derive(Copy, Clone)]
pub struct TcpIo<'a> {
    stack: &'a UnsafeCell<SocketStack>,
    handle: SocketHandle,
}

impl<'d> TcpIo<'d> {
    /// SAFETY: must not call reentrantly.
    unsafe fn with<R>(&self, f: impl FnOnce(&tcp::Socket, &Interface) -> R) -> R {
        let s = &*self.stack.get();
        let socket = s.sockets.get::<tcp::Socket>(self.handle);
        f(socket, &s.iface)
    }

    /// SAFETY: must not call reentrantly.
    unsafe fn with_mut<R>(&mut self, f: impl FnOnce(&mut tcp::Socket, &mut Interface) -> R) -> R {
        let s = &mut *self.stack.get();
        let socket = s.sockets.get_mut::<tcp::Socket>(self.handle);
        let res = f(socket, &mut s.iface);
        s.waker.wake();
        res
    }

    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        poll_fn(move |cx| unsafe {
            // CAUTION: smoltcp semantics around EOF are different to what you'd expect
            // from posix-like IO, so we have to tweak things here.
            self.with_mut(|s, _| match s.recv_slice(buf) {
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
        .await
    }

    async fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        poll_fn(move |cx| unsafe {
            self.with_mut(|s, _| match s.send_slice(buf) {
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
        .await
    }

    async fn flush(&mut self) -> Result<(), Error> {
        poll_fn(move |_| {
            Poll::Ready(Ok(())) // TODO: Is there a better implementation for this?
        })
        .await
    }
}

impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}

impl<'d> embedded_io::Io for TcpSocket<'d> {
    type Error = Error;
}

impl<'d> embedded_io::asynch::Read for TcpSocket<'d> {
    type ReadFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
    where
        Self: 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
        self.io.read(buf)
    }
}

impl<'d> embedded_io::asynch::Write for TcpSocket<'d> {
    type WriteFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
    where
        Self: 'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
        self.io.write(buf)
    }

    type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;

    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
        self.io.flush()
    }
}

impl<'d> embedded_io::Io for TcpReader<'d> {
    type Error = Error;
}

impl<'d> embedded_io::asynch::Read for TcpReader<'d> {
    type ReadFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
    where
        Self: 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
        self.io.read(buf)
    }
}

impl<'d> embedded_io::Io for TcpWriter<'d> {
    type Error = Error;
}

impl<'d> embedded_io::asynch::Write for TcpWriter<'d> {
    type WriteFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
    where
        Self: 'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
        self.io.write(buf)
    }

    type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;

    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
        self.io.flush()
    }
}

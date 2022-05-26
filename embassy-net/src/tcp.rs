use core::future::Future;
use core::marker::PhantomData;
use core::mem;
use core::task::Poll;
use futures::future::poll_fn;
use smoltcp::iface::{Context as SmolContext, SocketHandle};
use smoltcp::socket::TcpSocket as SyncTcpSocket;
use smoltcp::socket::{TcpSocketBuffer, TcpState};
use smoltcp::time::Duration;
use smoltcp::wire::IpEndpoint;

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
    handle: SocketHandle,
    ghost: PhantomData<&'a mut [u8]>,
}

impl<'a> Unpin for TcpSocket<'a> {}

pub struct TcpReader<'a> {
    handle: SocketHandle,
    ghost: PhantomData<&'a mut [u8]>,
}

impl<'a> Unpin for TcpReader<'a> {}

pub struct TcpWriter<'a> {
    handle: SocketHandle,
    ghost: PhantomData<&'a mut [u8]>,
}

impl<'a> Unpin for TcpWriter<'a> {}

impl<'a> TcpSocket<'a> {
    pub fn new(rx_buffer: &'a mut [u8], tx_buffer: &'a mut [u8]) -> Self {
        let handle = Stack::with(|stack| {
            let rx_buffer: &'static mut [u8] = unsafe { mem::transmute(rx_buffer) };
            let tx_buffer: &'static mut [u8] = unsafe { mem::transmute(tx_buffer) };
            stack.iface.add_socket(SyncTcpSocket::new(
                TcpSocketBuffer::new(rx_buffer),
                TcpSocketBuffer::new(tx_buffer),
            ))
        });

        Self {
            handle,
            ghost: PhantomData,
        }
    }

    pub fn split(&mut self) -> (TcpReader<'_>, TcpWriter<'_>) {
        (
            TcpReader {
                handle: self.handle,
                ghost: PhantomData,
            },
            TcpWriter {
                handle: self.handle,
                ghost: PhantomData,
            },
        )
    }

    pub async fn connect<T>(&mut self, remote_endpoint: T) -> Result<(), ConnectError>
    where
        T: Into<IpEndpoint>,
    {
        let local_port = Stack::with(|stack| stack.get_local_port());
        match with_socket(self.handle, |s, cx| {
            s.connect(cx, remote_endpoint, local_port)
        }) {
            Ok(()) => {}
            Err(smoltcp::Error::Illegal) => return Err(ConnectError::InvalidState),
            Err(smoltcp::Error::Unaddressable) => return Err(ConnectError::NoRoute),
            // smoltcp returns no errors other than the above.
            Err(_) => unreachable!(),
        }

        futures::future::poll_fn(|cx| {
            with_socket(self.handle, |s, _| match s.state() {
                TcpState::Closed | TcpState::TimeWait => {
                    Poll::Ready(Err(ConnectError::ConnectionReset))
                }
                TcpState::Listen => unreachable!(),
                TcpState::SynSent | TcpState::SynReceived => {
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
        T: Into<IpEndpoint>,
    {
        match with_socket(self.handle, |s, _| s.listen(local_endpoint)) {
            Ok(()) => {}
            Err(smoltcp::Error::Illegal) => return Err(AcceptError::InvalidState),
            Err(smoltcp::Error::Unaddressable) => return Err(AcceptError::InvalidPort),
            // smoltcp returns no errors other than the above.
            Err(_) => unreachable!(),
        }

        futures::future::poll_fn(|cx| {
            with_socket(self.handle, |s, _| match s.state() {
                TcpState::Listen | TcpState::SynSent | TcpState::SynReceived => {
                    s.register_send_waker(cx.waker());
                    Poll::Pending
                }
                _ => Poll::Ready(Ok(())),
            })
        })
        .await
    }

    pub fn set_timeout(&mut self, duration: Option<Duration>) {
        with_socket(self.handle, |s, _| s.set_timeout(duration))
    }

    pub fn set_keep_alive(&mut self, interval: Option<Duration>) {
        with_socket(self.handle, |s, _| s.set_keep_alive(interval))
    }

    pub fn set_hop_limit(&mut self, hop_limit: Option<u8>) {
        with_socket(self.handle, |s, _| s.set_hop_limit(hop_limit))
    }

    pub fn local_endpoint(&self) -> IpEndpoint {
        with_socket(self.handle, |s, _| s.local_endpoint())
    }

    pub fn remote_endpoint(&self) -> IpEndpoint {
        with_socket(self.handle, |s, _| s.remote_endpoint())
    }

    pub fn state(&self) -> TcpState {
        with_socket(self.handle, |s, _| s.state())
    }

    pub fn close(&mut self) {
        with_socket(self.handle, |s, _| s.close())
    }

    pub fn abort(&mut self) {
        with_socket(self.handle, |s, _| s.abort())
    }

    pub fn may_send(&self) -> bool {
        with_socket(self.handle, |s, _| s.may_send())
    }

    pub fn may_recv(&self) -> bool {
        with_socket(self.handle, |s, _| s.may_recv())
    }
}

fn with_socket<R>(
    handle: SocketHandle,
    f: impl FnOnce(&mut SyncTcpSocket, &mut SmolContext) -> R,
) -> R {
    Stack::with(|stack| {
        let res = {
            let (s, cx) = stack.iface.get_socket_and_context::<SyncTcpSocket>(handle);
            f(s, cx)
        };
        stack.wake();
        res
    })
}

impl<'a> Drop for TcpSocket<'a> {
    fn drop(&mut self) {
        Stack::with(|stack| {
            stack.iface.remove_socket(self.handle);
        })
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
        poll_fn(move |cx| {
            // CAUTION: smoltcp semantics around EOF are different to what you'd expect
            // from posix-like IO, so we have to tweak things here.
            with_socket(self.handle, |s, _| match s.recv_slice(buf) {
                // No data ready
                Ok(0) => {
                    s.register_recv_waker(cx.waker());
                    Poll::Pending
                }
                // Data ready!
                Ok(n) => Poll::Ready(Ok(n)),
                // EOF
                Err(smoltcp::Error::Finished) => Poll::Ready(Ok(0)),
                // Connection reset. TODO: this can also be timeouts etc, investigate.
                Err(smoltcp::Error::Illegal) => Poll::Ready(Err(Error::ConnectionReset)),
                // smoltcp returns no errors other than the above.
                Err(_) => unreachable!(),
            })
        })
    }
}

impl<'d> embedded_io::asynch::Write for TcpSocket<'d> {
    type WriteFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
    where
        Self: 'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
        poll_fn(move |cx| {
            with_socket(self.handle, |s, _| match s.send_slice(buf) {
                // Not ready to send (no space in the tx buffer)
                Ok(0) => {
                    s.register_send_waker(cx.waker());
                    Poll::Pending
                }
                // Some data sent
                Ok(n) => Poll::Ready(Ok(n)),
                // Connection reset. TODO: this can also be timeouts etc, investigate.
                Err(smoltcp::Error::Illegal) => Poll::Ready(Err(Error::ConnectionReset)),
                // smoltcp returns no errors other than the above.
                Err(_) => unreachable!(),
            })
        })
    }

    type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;

    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
        poll_fn(move |_| {
            Poll::Ready(Ok(())) // TODO: Is there a better implementation for this?
        })
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
        poll_fn(move |cx| {
            // CAUTION: smoltcp semantics around EOF are different to what you'd expect
            // from posix-like IO, so we have to tweak things here.
            with_socket(self.handle, |s, _| match s.recv_slice(buf) {
                // No data ready
                Ok(0) => {
                    s.register_recv_waker(cx.waker());
                    Poll::Pending
                }
                // Data ready!
                Ok(n) => Poll::Ready(Ok(n)),
                // EOF
                Err(smoltcp::Error::Finished) => Poll::Ready(Ok(0)),
                // Connection reset. TODO: this can also be timeouts etc, investigate.
                Err(smoltcp::Error::Illegal) => Poll::Ready(Err(Error::ConnectionReset)),
                // smoltcp returns no errors other than the above.
                Err(_) => unreachable!(),
            })
        })
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
        poll_fn(move |cx| {
            with_socket(self.handle, |s, _| match s.send_slice(buf) {
                // Not ready to send (no space in the tx buffer)
                Ok(0) => {
                    s.register_send_waker(cx.waker());
                    Poll::Pending
                }
                // Some data sent
                Ok(n) => Poll::Ready(Ok(n)),
                // Connection reset. TODO: this can also be timeouts etc, investigate.
                Err(smoltcp::Error::Illegal) => Poll::Ready(Err(Error::ConnectionReset)),
                // smoltcp returns no errors other than the above.
                Err(_) => unreachable!(),
            })
        })
    }

    type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>>
    where
        Self: 'a;

    fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
        poll_fn(move |_| {
            Poll::Ready(Ok(())) // TODO: Is there a better implementation for this?
        })
    }
}

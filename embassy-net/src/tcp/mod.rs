use core::marker::PhantomData;
use core::mem;
use core::task::Poll;
use smoltcp::iface::{Context as SmolContext, SocketHandle};
use smoltcp::socket::TcpSocket as SyncTcpSocket;
use smoltcp::socket::{TcpSocketBuffer, TcpState};
use smoltcp::time::Duration;
use smoltcp::wire::IpEndpoint;

#[cfg(feature = "nightly")]
mod io_impl;

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

    pub async fn connect<T>(&mut self, remote_endpoint: T) -> Result<(), ConnectError>
    where
        T: Into<IpEndpoint>,
    {
        let local_port = Stack::with(|stack| stack.get_local_port());
        match self.with(|s, cx| s.connect(cx, remote_endpoint, local_port)) {
            Ok(()) => {}
            Err(smoltcp::Error::Illegal) => return Err(ConnectError::InvalidState),
            Err(smoltcp::Error::Unaddressable) => return Err(ConnectError::NoRoute),
            // smoltcp returns no errors other than the above.
            Err(_) => unreachable!(),
        }

        futures::future::poll_fn(|cx| {
            self.with(|s, _| match s.state() {
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
        match self.with(|s, _| s.listen(local_endpoint)) {
            Ok(()) => {}
            Err(smoltcp::Error::Illegal) => return Err(AcceptError::InvalidState),
            Err(smoltcp::Error::Unaddressable) => return Err(AcceptError::InvalidPort),
            // smoltcp returns no errors other than the above.
            Err(_) => unreachable!(),
        }

        futures::future::poll_fn(|cx| {
            self.with(|s, _| match s.state() {
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
        self.with(|s, _| s.set_timeout(duration))
    }

    pub fn set_keep_alive(&mut self, interval: Option<Duration>) {
        self.with(|s, _| s.set_keep_alive(interval))
    }

    pub fn set_hop_limit(&mut self, hop_limit: Option<u8>) {
        self.with(|s, _| s.set_hop_limit(hop_limit))
    }

    pub fn local_endpoint(&self) -> IpEndpoint {
        self.with(|s, _| s.local_endpoint())
    }

    pub fn remote_endpoint(&self) -> IpEndpoint {
        self.with(|s, _| s.remote_endpoint())
    }

    pub fn state(&self) -> TcpState {
        self.with(|s, _| s.state())
    }

    pub fn close(&mut self) {
        self.with(|s, _| s.close())
    }

    pub fn abort(&mut self) {
        self.with(|s, _| s.abort())
    }

    pub fn may_send(&self) -> bool {
        self.with(|s, _| s.may_send())
    }

    pub fn may_recv(&self) -> bool {
        self.with(|s, _| s.may_recv())
    }

    fn with<R>(&self, f: impl FnOnce(&mut SyncTcpSocket, &mut SmolContext) -> R) -> R {
        Stack::with(|stack| {
            let res = {
                let (s, cx) = stack
                    .iface
                    .get_socket_and_context::<SyncTcpSocket>(self.handle);
                f(s, cx)
            };
            stack.wake();
            res
        })
    }
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

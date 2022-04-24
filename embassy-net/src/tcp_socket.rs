use core::marker::PhantomData;
use core::mem;
use core::pin::Pin;
use core::task::{Context, Poll};
use embassy::io;
use embassy::io::{AsyncBufRead, AsyncWrite};
use smoltcp::iface::{Context as SmolContext, SocketHandle};
use smoltcp::socket::TcpSocket as SyncTcpSocket;
use smoltcp::socket::{TcpSocketBuffer, TcpState};
use smoltcp::time::Duration;
use smoltcp::wire::IpEndpoint;

use super::stack::Stack;
use crate::{Error, Result};

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

    pub async fn connect<T>(&mut self, remote_endpoint: T) -> Result<()>
    where
        T: Into<IpEndpoint>,
    {
        let local_port = Stack::with(|stack| stack.get_local_port());
        self.with(|s, cx| s.connect(cx, remote_endpoint, local_port))?;

        futures::future::poll_fn(|cx| {
            self.with(|s, _| match s.state() {
                TcpState::Closed | TcpState::TimeWait => Poll::Ready(Err(Error::Unaddressable)),
                TcpState::Listen => Poll::Ready(Err(Error::Illegal)),
                TcpState::SynSent | TcpState::SynReceived => {
                    s.register_send_waker(cx.waker());
                    Poll::Pending
                }
                _ => Poll::Ready(Ok(())),
            })
        })
        .await
    }

    pub async fn accept<T>(&mut self, local_endpoint: T) -> Result<()>
    where
        T: Into<IpEndpoint>,
    {
        self.with(|s, _| s.listen(local_endpoint))?;

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

fn to_ioerr(_err: Error) -> io::Error {
    // todo
    io::Error::Other
}

impl<'a> Drop for TcpSocket<'a> {
    fn drop(&mut self) {
        Stack::with(|stack| {
            stack.iface.remove_socket(self.handle);
        })
    }
}

impl<'a> AsyncBufRead for TcpSocket<'a> {
    fn poll_fill_buf<'z>(
        self: Pin<&'z mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<&'z [u8]>> {
        self.with(|s, _| match s.peek(1 << 30) {
            // No data ready
            Ok(buf) if buf.is_empty() => {
                s.register_recv_waker(cx.waker());
                Poll::Pending
            }
            // Data ready!
            Ok(buf) => {
                // Safety:
                // - User can't touch the inner TcpSocket directly at all.
                // - The socket itself won't touch these bytes until consume() is called, which
                //   requires the user to release this borrow.
                let buf: &'z [u8] = unsafe { core::mem::transmute(&*buf) };
                Poll::Ready(Ok(buf))
            }
            // EOF
            Err(Error::Finished) => Poll::Ready(Ok(&[][..])),
            // Error
            Err(e) => Poll::Ready(Err(to_ioerr(e))),
        })
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        if amt == 0 {
            // smoltcp's recv returns Finished if we're at EOF,
            // even if we're "reading" 0 bytes.
            return;
        }
        self.with(|s, _| s.recv(|_| (amt, ()))).unwrap()
    }
}

impl<'a> AsyncWrite for TcpSocket<'a> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.with(|s, _| match s.send_slice(buf) {
            // Not ready to send (no space in the tx buffer)
            Ok(0) => {
                s.register_send_waker(cx.waker());
                Poll::Pending
            }
            // Some data sent
            Ok(n) => Poll::Ready(Ok(n)),
            // Error
            Err(e) => Poll::Ready(Err(to_ioerr(e))),
        })
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(())) // TODO: Is there a better implementation for this?
    }
}

use core::future::Future;
use core::task::Poll;
use futures::future::poll_fn;

use super::{Error, TcpSocket};

impl<'d> embedded_io::asynch::Read for TcpSocket<'d> {
    type ReadFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
    where
        Self: 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
        poll_fn(move |cx| {
            // CAUTION: smoltcp semantics around EOF are different to what you'd expect
            // from posix-like IO, so we have to tweak things here.
            self.with(|s, _| match s.recv_slice(buf) {
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
            self.with(|s, _| match s.send_slice(buf) {
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

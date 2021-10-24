use super::super::error::{Error, Result};
use super::super::traits::AsyncBufRead;

use core::cmp::min;
use core::mem;
use core::pin::Pin;
use futures::future::Future;
use futures::ready;
use futures::task::{Context, Poll};

/// Future for the [`read_exact`](super::AsyncBufReadExt::read_exact) method.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct ReadExact<'a, R: ?Sized> {
    reader: &'a mut R,
    buf: &'a mut [u8],
}

impl<R: ?Sized + Unpin> Unpin for ReadExact<'_, R> {}

impl<'a, R: AsyncBufRead + ?Sized + Unpin> ReadExact<'a, R> {
    pub(super) fn new(reader: &'a mut R, buf: &'a mut [u8]) -> Self {
        ReadExact { reader, buf }
    }
}

impl<R: AsyncBufRead + ?Sized + Unpin> Future for ReadExact<'_, R> {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        while !this.buf.is_empty() {
            let buf = ready!(Pin::new(&mut this.reader).poll_fill_buf(cx))?;
            if buf.is_empty() {
                return Poll::Ready(Err(Error::UnexpectedEof));
            }

            let n = min(this.buf.len(), buf.len());
            this.buf[..n].copy_from_slice(&buf[..n]);
            Pin::new(&mut this.reader).consume(n);
            {
                let (_, rest) = mem::take(&mut this.buf).split_at_mut(n);
                this.buf = rest;
            }
        }
        Poll::Ready(Ok(()))
    }
}

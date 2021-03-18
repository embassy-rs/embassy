use super::super::error::Result;
use super::super::traits::AsyncBufRead;

use core::cmp::min;

use core::pin::Pin;
use futures::future::Future;
use futures::ready;
use futures::task::{Context, Poll};

/// Future for the [`read_exact`](super::AsyncBufReadExt::read_exact) method.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Read<'a, R: ?Sized> {
    reader: &'a mut R,
    buf: &'a mut [u8],
}

impl<R: ?Sized + Unpin> Unpin for Read<'_, R> {}

impl<'a, R: AsyncBufRead + ?Sized + Unpin> Read<'a, R> {
    pub(super) fn new(reader: &'a mut R, buf: &'a mut [u8]) -> Self {
        Read { reader, buf }
    }
}

impl<R: AsyncBufRead + ?Sized + Unpin> Future for Read<'_, R> {
    type Output = Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        let buf = ready!(Pin::new(&mut this.reader).poll_fill_buf(cx))?;

        let n = min(this.buf.len(), buf.len());
        this.buf[..n].copy_from_slice(&buf[..n]);
        Pin::new(&mut this.reader).consume(n);
        Poll::Ready(Ok(n))
    }
}

use core::pin::Pin;
use futures::future::Future;
use futures::ready;
use futures::task::{Context, Poll};

use super::super::error::Result;
use super::super::traits::AsyncWrite;

/// Future for the [`write_all`](super::AsyncWriteExt::write_all) method.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Write<'a, W: ?Sized> {
    writer: &'a mut W,
    buf: &'a [u8],
}

impl<W: ?Sized + Unpin> Unpin for Write<'_, W> {}

impl<'a, W: AsyncWrite + ?Sized + Unpin> Write<'a, W> {
    pub(super) fn new(writer: &'a mut W, buf: &'a [u8]) -> Self {
        Write { writer, buf }
    }
}

impl<W: AsyncWrite + ?Sized + Unpin> Future for Write<'_, W> {
    type Output = Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<usize>> {
        let this = &mut *self;
        let n = ready!(Pin::new(&mut this.writer).poll_write(cx, this.buf))?;
        Poll::Ready(Ok(n))
    }
}

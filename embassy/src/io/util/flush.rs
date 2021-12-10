use core::pin::Pin;
use futures::future::Future;
use futures::ready;
use futures::task::{Context, Poll};

use super::super::error::Result;
use super::super::traits::AsyncWrite;

/// Future for the [`flush`](super::AsyncWriteExt::flush) method.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Flush<'a, W: ?Sized> {
    writer: &'a mut W,
}

impl<W: ?Sized + Unpin> Unpin for Flush<'_, W> {}

impl<'a, W: AsyncWrite + ?Sized + Unpin> Flush<'a, W> {
    pub(super) fn new(writer: &'a mut W) -> Self {
        Flush { writer }
    }
}

impl<W: AsyncWrite + ?Sized + Unpin> Future for Flush<'_, W> {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        let this = &mut *self;
        let _ = ready!(Pin::new(&mut this.writer).poll_flush(cx))?;
        Poll::Ready(Ok(()))
    }
}

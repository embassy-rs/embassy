use core::pin::Pin;
use futures::future::Future;
use futures::ready;
use futures::task::{Context, Poll};

use super::super::error::Result;
use super::super::traits::AsyncWrite;

/// Future for the [`write_all`](super::AsyncWriteExt::write_all) method.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct WriteByte<'a, W: ?Sized> {
    writer: &'a mut W,
    byte: u8,
}

impl<W: ?Sized + Unpin> Unpin for WriteByte<'_, W> {}

impl<'a, W: AsyncWrite + ?Sized + Unpin> WriteByte<'a, W> {
    pub(super) fn new(writer: &'a mut W, byte: u8) -> Self {
        WriteByte { writer, byte }
    }
}

impl<W: AsyncWrite + ?Sized + Unpin> Future for WriteByte<'_, W> {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        let this = &mut *self;
        let buf = [this.byte; 1];
        let n = ready!(Pin::new(&mut this.writer).poll_write(cx, &buf))?;
        if n == 0 {
            panic!();
        }
        assert!(n == 1);

        Poll::Ready(Ok(()))
    }
}

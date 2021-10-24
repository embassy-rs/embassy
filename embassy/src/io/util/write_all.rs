use core::mem;
use core::pin::Pin;
use futures::future::Future;
use futures::ready;
use futures::task::{Context, Poll};

use super::super::error::Result;
use super::super::traits::AsyncWrite;

/// Future for the [`write_all`](super::AsyncWriteExt::write_all) method.
#[derive(Debug)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct WriteAll<'a, W: ?Sized> {
    writer: &'a mut W,
    buf: &'a [u8],
}

impl<W: ?Sized + Unpin> Unpin for WriteAll<'_, W> {}

impl<'a, W: AsyncWrite + ?Sized + Unpin> WriteAll<'a, W> {
    pub(super) fn new(writer: &'a mut W, buf: &'a [u8]) -> Self {
        WriteAll { writer, buf }
    }
}

impl<W: AsyncWrite + ?Sized + Unpin> Future for WriteAll<'_, W> {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        let this = &mut *self;
        while !this.buf.is_empty() {
            let n = ready!(Pin::new(&mut this.writer).poll_write(cx, this.buf))?;
            {
                let (_, rest) = mem::take(&mut this.buf).split_at(n);
                this.buf = rest;
            }
            if n == 0 {
                panic!();
            }
        }

        Poll::Ready(Ok(()))
    }
}

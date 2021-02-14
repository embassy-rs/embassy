use core::pin::Pin;
use futures::future::Future;
use futures::ready;
use futures::task::{Context, Poll};

use super::super::error::{Error, Result};
use super::super::traits::AsyncBufRead;

pub struct ReadByte<'a, R: ?Sized> {
    reader: &'a mut R,
}

impl<R: ?Sized + Unpin> Unpin for ReadByte<'_, R> {}

impl<'a, R: AsyncBufRead + ?Sized + Unpin> ReadByte<'a, R> {
    pub(super) fn new(reader: &'a mut R) -> Self {
        Self { reader }
    }
}

impl<'a, R: AsyncBufRead + ?Sized + Unpin> Future for ReadByte<'a, R> {
    type Output = Result<u8>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self { reader } = &mut *self;
        let mut reader = Pin::new(reader);
        let rbuf = ready!(reader.as_mut().poll_fill_buf(cx))?;
        if rbuf.is_empty() {
            return Poll::Ready(Err(Error::UnexpectedEof));
        }

        let r = rbuf[0];
        reader.as_mut().consume(1);
        Poll::Ready(Ok(r))
    }
}

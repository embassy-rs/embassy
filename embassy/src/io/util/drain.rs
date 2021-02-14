use core::pin::Pin;
use futures::future::Future;
use futures::task::{Context, Poll};

use super::super::error::Result;
use super::super::traits::AsyncBufRead;

pub struct Drain<'a, R: ?Sized> {
    reader: &'a mut R,
}

impl<R: ?Sized + Unpin> Unpin for Drain<'_, R> {}

impl<'a, R: AsyncBufRead + ?Sized + Unpin> Drain<'a, R> {
    pub(super) fn new(reader: &'a mut R) -> Self {
        Self { reader }
    }
}

impl<'a, R: AsyncBufRead + ?Sized + Unpin> Future for Drain<'a, R> {
    type Output = Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self { reader } = &mut *self;
        let mut reader = Pin::new(reader);

        let mut n = 0;

        loop {
            match reader.as_mut().poll_fill_buf(cx) {
                Poll::Pending => return Poll::Ready(Ok(n)),
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Ready(Ok(buf)) => {
                    let len = buf.len();
                    n += len;
                    reader.as_mut().consume(len);
                }
            }
        }
    }
}

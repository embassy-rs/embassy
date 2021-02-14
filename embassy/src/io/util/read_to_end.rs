use core::cmp::min;
use core::pin::Pin;
use futures::future::Future;
use futures::ready;
use futures::task::{Context, Poll};

use super::super::error::{Error, Result};
use super::super::traits::AsyncBufRead;

pub struct ReadToEnd<'a, R: ?Sized> {
    reader: &'a mut R,
    buf: &'a mut [u8],
    n: usize,
}

impl<R: ?Sized + Unpin> Unpin for ReadToEnd<'_, R> {}

impl<'a, R: AsyncBufRead + ?Sized + Unpin> ReadToEnd<'a, R> {
    pub(super) fn new(reader: &'a mut R, buf: &'a mut [u8]) -> Self {
        Self { reader, buf, n: 0 }
    }
}

impl<'a, R: AsyncBufRead + ?Sized + Unpin> Future for ReadToEnd<'a, R> {
    type Output = Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self { reader, buf, n } = &mut *self;
        let mut reader = Pin::new(reader);
        loop {
            let rbuf = ready!(reader.as_mut().poll_fill_buf(cx))?;
            if rbuf.is_empty() {
                return Poll::Ready(Ok(*n));
            }

            if *n == buf.len() {
                return Poll::Ready(Err(Error::Truncated));
            }

            // truncate data if it doesn't fit in buf
            let p = min(rbuf.len(), buf.len() - *n);
            buf[*n..*n + p].copy_from_slice(&rbuf[..p]);
            *n += p;

            reader.as_mut().consume(p);
        }
    }
}

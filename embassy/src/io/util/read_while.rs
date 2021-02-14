use core::cmp::min;
use core::pin::Pin;
use futures::future::Future;
use futures::ready;
use futures::task::{Context, Poll};

use super::super::error::{Error, Result};
use super::super::traits::AsyncBufRead;

pub struct ReadWhile<'a, R: ?Sized, F> {
    reader: &'a mut R,
    buf: &'a mut [u8],
    n: usize,
    f: F,
}

impl<R: ?Sized + Unpin, F> Unpin for ReadWhile<'_, R, F> {}

impl<'a, R: AsyncBufRead + ?Sized + Unpin, F: Fn(u8) -> bool> ReadWhile<'a, R, F> {
    pub(super) fn new(reader: &'a mut R, f: F, buf: &'a mut [u8]) -> Self {
        Self {
            reader,
            f,
            buf,
            n: 0,
        }
    }
}

impl<'a, R: AsyncBufRead + ?Sized + Unpin, F: Fn(u8) -> bool> Future for ReadWhile<'a, R, F> {
    type Output = Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self { reader, f, buf, n } = &mut *self;
        let mut reader = Pin::new(reader);
        loop {
            let rbuf = ready!(reader.as_mut().poll_fill_buf(cx))?;
            if rbuf.is_empty() {
                return Poll::Ready(Err(Error::UnexpectedEof));
            }

            let (p, done) = match rbuf.iter().position(|&b| !f(b)) {
                Some(p) => (p, true),
                None => (rbuf.len(), false),
            };

            // truncate data if it doesn't fit in buf
            let p2 = min(p, buf.len() - *n);
            buf[*n..*n + p2].copy_from_slice(&rbuf[..p2]);
            *n += p2;

            // consume it all, even if it doesn't fit.
            // Otherwise we can deadlock because we never read to the ending char
            reader.as_mut().consume(p);

            if done {
                return Poll::Ready(Ok(*n));
            }
        }
    }
}

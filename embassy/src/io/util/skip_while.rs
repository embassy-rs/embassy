use core::iter::Iterator;
use core::pin::Pin;
use futures::future::Future;
use futures::ready;
use futures::task::{Context, Poll};

use super::super::error::{Error, Result};
use super::super::traits::AsyncBufRead;

pub struct SkipWhile<'a, R: ?Sized, F> {
    reader: &'a mut R,
    f: F,
}

impl<R: ?Sized + Unpin, F> Unpin for SkipWhile<'_, R, F> {}

impl<'a, R: AsyncBufRead + ?Sized + Unpin, F: Fn(u8) -> bool> SkipWhile<'a, R, F> {
    pub(super) fn new(reader: &'a mut R, f: F) -> Self {
        Self { reader, f }
    }
}

impl<'a, R: AsyncBufRead + ?Sized + Unpin, F: Fn(u8) -> bool> Future for SkipWhile<'a, R, F> {
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self { reader, f } = &mut *self;
        let mut reader = Pin::new(reader);
        loop {
            let buf = ready!(reader.as_mut().poll_fill_buf(cx))?;
            if buf.is_empty() {
                return Poll::Ready(Err(Error::UnexpectedEof));
            }

            let (p, done) = match buf.iter().position(|b| !f(*b)) {
                Some(p) => (p, true),
                None => (buf.len(), false),
            };
            reader.as_mut().consume(p);
            if done {
                return Poll::Ready(Ok(()));
            }
        }
    }
}

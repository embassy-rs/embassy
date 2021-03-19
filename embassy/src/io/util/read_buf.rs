use super::super::error::Result;
use super::super::traits::AsyncBufRead;

use core::pin::Pin;
use futures::future::Future;
use futures::ready;
use futures::task::{Context, Poll};

pub struct ReadBuf<'a, R: ?Sized> {
    reader: Option<&'a mut R>,
}

impl<R: ?Sized + Unpin> Unpin for ReadBuf<'_, R> {}

impl<'a, R: AsyncBufRead + ?Sized + Unpin> ReadBuf<'a, R> {
    pub(super) fn new(reader: &'a mut R) -> Self {
        ReadBuf {
            reader: Some(reader),
        }
    }
}

impl<'a, R: AsyncBufRead + ?Sized + Unpin> Future for ReadBuf<'a, R> {
    type Output = Result<&'a [u8]>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;

        let buf = ready!(Pin::new(this.reader.as_mut().unwrap()).poll_fill_buf(cx))?;
        let buf: &'a [u8] = unsafe { core::mem::transmute(buf) };
        this.reader = None;
        Poll::Ready(Ok(buf))
    }
}

use alloc::rc::Rc;
use core::cell::UnsafeCell;
use core::pin::Pin;
use futures::task::{Context, Poll};

use super::super::error::Result;
use super::super::traits::{AsyncBufRead, AsyncWrite};

/// The readable half of an object returned from `AsyncBufRead::split`.
#[derive(Debug)]
pub struct ReadHalf<T> {
    handle: Rc<UnsafeCell<T>>,
}

/// The writable half of an object returned from `AsyncBufRead::split`.
#[derive(Debug)]
pub struct WriteHalf<T> {
    handle: Rc<UnsafeCell<T>>,
}

impl<T: AsyncBufRead + Unpin> AsyncBufRead for ReadHalf<T> {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<&[u8]>> {
        Pin::new(unsafe { &mut *self.handle.get() }).poll_fill_buf(cx)
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        Pin::new(unsafe { &mut *self.handle.get() }).consume(amt)
    }
}

impl<T: AsyncWrite + Unpin> AsyncWrite for WriteHalf<T> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        Pin::new(unsafe { &mut *self.handle.get() }).poll_write(cx, buf)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(unsafe { &mut *self.handle.get() }).poll_flush(cx)
    }
}

pub fn split<T: AsyncBufRead + AsyncWrite>(t: T) -> (ReadHalf<T>, WriteHalf<T>) {
    let c = Rc::new(UnsafeCell::new(t));
    (ReadHalf { handle: c.clone() }, WriteHalf { handle: c })
}

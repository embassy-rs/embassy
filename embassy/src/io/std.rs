use core::pin::Pin;
use core::task::{Context, Poll};
use futures::io as std_io;

use super::{AsyncBufRead, AsyncWrite, Result};

pub struct FromStdIo<T>(T);

impl<T> FromStdIo<T> {
    pub fn new(inner: T) -> Self {
        Self(inner)
    }
}

impl<T: std_io::AsyncBufRead> AsyncBufRead for FromStdIo<T> {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<&[u8]>> {
        let Self(inner) = unsafe { self.get_unchecked_mut() };
        unsafe { Pin::new_unchecked(inner) }
            .poll_fill_buf(cx)
            .map_err(|e| e.into())
    }
    fn consume(self: Pin<&mut Self>, amt: usize) {
        let Self(inner) = unsafe { self.get_unchecked_mut() };
        unsafe { Pin::new_unchecked(inner) }.consume(amt)
    }
}

impl<T: std_io::AsyncWrite> AsyncWrite for FromStdIo<T> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        let Self(inner) = unsafe { self.get_unchecked_mut() };
        unsafe { Pin::new_unchecked(inner) }
            .poll_write(cx, buf)
            .map_err(|e| e.into())
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        let Self(inner) = unsafe { self.get_unchecked_mut() };
        unsafe { Pin::new_unchecked(inner) }
            .poll_flush(cx)
            .map_err(|e| e.into())
    }
}

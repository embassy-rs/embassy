use core::ops::DerefMut;
use core::pin::Pin;
use core::task::{Context, Poll};

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

use super::error::Result;

/// Read bytes asynchronously.
///
/// This trait is analogous to the `std::io::BufRead` trait, but integrates
/// with the asynchronous task system. In particular, the `poll_fill_buf`
/// method, unlike `BufRead::fill_buf`, will automatically queue the current task
/// for wakeup and return if data is not yet available, rather than blocking
/// the calling thread.
pub trait AsyncBufRead {
    /// Attempt to return the contents of the internal buffer, filling it with more data
    /// from the inner reader if it is empty.
    ///
    /// On success, returns `Poll::Ready(Ok(buf))`.
    ///
    /// If no data is available for reading, the method returns
    /// `Poll::Pending` and arranges for the current task (via
    /// `cx.waker().wake_by_ref()`) to receive a notification when the object becomes
    /// readable or is closed.
    ///
    /// This function is a lower-level call. It needs to be paired with the
    /// [`consume`] method to function properly. When calling this
    /// method, none of the contents will be "read" in the sense that later
    /// calling [`poll_fill_buf`] may return the same contents. As such, [`consume`] must
    /// be called with the number of bytes that are consumed from this buffer to
    /// ensure that the bytes are never returned twice.
    ///
    /// [`poll_fill_buf`]: AsyncBufRead::poll_fill_buf
    /// [`consume`]: AsyncBufRead::consume
    ///
    /// An empty buffer returned indicates that the stream has reached EOF.
    ///
    /// # Implementation
    ///
    /// This function may not return errors of kind `WouldBlock` or
    /// `Interrupted`.  Implementations must convert `WouldBlock` into
    /// `Poll::Pending` and either internally retry or convert
    /// `Interrupted` into another error kind.
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<&[u8]>>;

    /// Tells this buffer that `amt` bytes have been consumed from the buffer,
    /// so they should no longer be returned in calls to [`poll_fill_buf`].
    ///
    /// This function is a lower-level call. It needs to be paired with the
    /// [`poll_fill_buf`] method to function properly. This function does
    /// not perform any I/O, it simply informs this object that some amount of
    /// its buffer, returned from [`poll_fill_buf`], has been consumed and should
    /// no longer be returned. As such, this function may do odd things if
    /// [`poll_fill_buf`] isn't called before calling it.
    ///
    /// The `amt` must be `<=` the number of bytes in the buffer returned by
    /// [`poll_fill_buf`].
    ///
    /// [`poll_fill_buf`]: AsyncBufRead::poll_fill_buf
    fn consume(self: Pin<&mut Self>, amt: usize);
}

/// Write bytes asynchronously.
///
/// This trait is analogous to the `core::io::Write` trait, but integrates
/// with the asynchronous task system. In particular, the `poll_write`
/// method, unlike `Write::write`, will automatically queue the current task
/// for wakeup and return if the writer cannot take more data, rather than blocking
/// the calling thread.
pub trait AsyncWrite {
    /// Attempt to write bytes from `buf` into the object.
    ///
    /// On success, returns `Poll::Ready(Ok(num_bytes_written))`.
    ///
    /// If the object is not ready for writing, the method returns
    /// `Poll::Pending` and arranges for the current task (via
    /// `cx.waker().wake_by_ref()`) to receive a notification when the object becomes
    /// writable or is closed.
    ///
    /// # Implementation
    ///
    /// This function may not return errors of kind `WouldBlock` or
    /// `Interrupted`.  Implementations must convert `WouldBlock` into
    /// `Poll::Pending` and either internally retry or convert
    /// `Interrupted` into another error kind.
    ///
    /// `poll_write` must try to make progress by flushing the underlying object if
    /// that is the only way the underlying object can become writable again.
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>>;

    /// Attempt to flush the object, ensuring that any buffered data reach their destination.
    ///
    /// On success, returns Poll::Ready(Ok(())).
    ///
    /// If flushing cannot immediately complete, this method returns [Poll::Pending] and arranges for the
    /// current task (via cx.waker()) to receive a notification when the object can make progress
    /// towards flushing.
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>>;
}

macro_rules! defer_async_read {
    () => {
        fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<&[u8]>> {
            Pin::new(&mut **self.get_mut()).poll_fill_buf(cx)
        }

        fn consume(mut self: Pin<&mut Self>, amt: usize) {
            Pin::new(&mut **self).consume(amt)
        }
    };
}

#[cfg(feature = "alloc")]
impl<T: ?Sized + AsyncBufRead + Unpin> AsyncBufRead for Box<T> {
    defer_async_read!();
}

impl<T: ?Sized + AsyncBufRead + Unpin> AsyncBufRead for &mut T {
    defer_async_read!();
}

impl<P> AsyncBufRead for Pin<P>
where
    P: DerefMut + Unpin,
    P::Target: AsyncBufRead,
{
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<&[u8]>> {
        self.get_mut().as_mut().poll_fill_buf(cx)
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        self.get_mut().as_mut().consume(amt)
    }
}

macro_rules! deref_async_write {
    () => {
        fn poll_write(
            mut self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize>> {
            Pin::new(&mut **self).poll_write(cx, buf)
        }

        fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
            Pin::new(&mut **self).poll_flush(cx)
        }
    };
}

#[cfg(feature = "alloc")]
impl<T: ?Sized + AsyncWrite + Unpin> AsyncWrite for Box<T> {
    deref_async_write!();
}

impl<T: ?Sized + AsyncWrite + Unpin> AsyncWrite for &mut T {
    deref_async_write!();
}

impl<P> AsyncWrite for Pin<P>
where
    P: DerefMut + Unpin,
    P::Target: AsyncWrite,
{
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        self.get_mut().as_mut().poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.get_mut().as_mut().poll_flush(cx)
    }
}

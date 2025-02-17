//! Async byte stream pipe.

use core::cell::{RefCell, UnsafeCell};
use core::convert::Infallible;
use core::future::Future;
use core::ops::Range;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::blocking_mutex::raw::RawMutex;
use crate::blocking_mutex::Mutex;
use crate::ring_buffer::RingBuffer;
use crate::waitqueue::WakerRegistration;

/// Write-only access to a [`Pipe`].
pub struct Writer<'p, M, const N: usize>
where
    M: RawMutex,
{
    pipe: &'p Pipe<M, N>,
}

impl<'p, M, const N: usize> Clone for Writer<'p, M, N>
where
    M: RawMutex,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<'p, M, const N: usize> Copy for Writer<'p, M, N> where M: RawMutex {}

impl<'p, M, const N: usize> Writer<'p, M, N>
where
    M: RawMutex,
{
    /// Write some bytes to the pipe.
    ///
    /// See [`Pipe::write()`]
    pub fn write<'a>(&'a self, buf: &'a [u8]) -> WriteFuture<'a, M, N> {
        self.pipe.write(buf)
    }

    /// Attempt to immediately write some bytes to the pipe.
    ///
    /// See [`Pipe::try_write()`]
    pub fn try_write(&self, buf: &[u8]) -> Result<usize, TryWriteError> {
        self.pipe.try_write(buf)
    }
}

/// Future returned by [`Pipe::write`] and  [`Writer::write`].
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct WriteFuture<'p, M, const N: usize>
where
    M: RawMutex,
{
    pipe: &'p Pipe<M, N>,
    buf: &'p [u8],
}

impl<'p, M, const N: usize> Future for WriteFuture<'p, M, N>
where
    M: RawMutex,
{
    type Output = usize;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.pipe.try_write_with_context(Some(cx), self.buf) {
            Ok(n) => Poll::Ready(n),
            Err(TryWriteError::Full) => Poll::Pending,
        }
    }
}

impl<'p, M, const N: usize> Unpin for WriteFuture<'p, M, N> where M: RawMutex {}

/// Read-only access to a [`Pipe`].
pub struct Reader<'p, M, const N: usize>
where
    M: RawMutex,
{
    pipe: &'p Pipe<M, N>,
}

impl<'p, M, const N: usize> Reader<'p, M, N>
where
    M: RawMutex,
{
    /// Read some bytes from the pipe.
    ///
    /// See [`Pipe::read()`]
    pub fn read<'a>(&'a self, buf: &'a mut [u8]) -> ReadFuture<'a, M, N> {
        self.pipe.read(buf)
    }

    /// Attempt to immediately read some bytes from the pipe.
    ///
    /// See [`Pipe::try_read()`]
    pub fn try_read(&self, buf: &mut [u8]) -> Result<usize, TryReadError> {
        self.pipe.try_read(buf)
    }

    /// Return the contents of the internal buffer, filling it with more data from the inner reader if it is empty.
    ///
    /// If no bytes are currently available to read, this function waits until at least one byte is available.
    ///
    /// If the reader is at end-of-file (EOF), an empty slice is returned.
    pub fn fill_buf(&mut self) -> FillBufFuture<'_, M, N> {
        FillBufFuture { pipe: Some(self.pipe) }
    }

    /// Try returning contents of the internal buffer.
    ///
    /// If no bytes are currently available to read, this function returns `Err(TryReadError::Empty)`.
    ///
    /// If the reader is at end-of-file (EOF), an empty slice is returned.
    pub fn try_fill_buf(&mut self) -> Result<&[u8], TryReadError> {
        unsafe { self.pipe.try_fill_buf_with_context(None) }
    }

    /// Tell this buffer that `amt` bytes have been consumed from the buffer, so they should no longer be returned in calls to `fill_buf`.
    pub fn consume(&mut self, amt: usize) {
        self.pipe.consume(amt)
    }
}

/// Future returned by [`Pipe::read`] and  [`Reader::read`].
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct ReadFuture<'p, M, const N: usize>
where
    M: RawMutex,
{
    pipe: &'p Pipe<M, N>,
    buf: &'p mut [u8],
}

impl<'p, M, const N: usize> Future for ReadFuture<'p, M, N>
where
    M: RawMutex,
{
    type Output = usize;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.pipe.try_read_with_context(Some(cx), self.buf) {
            Ok(n) => Poll::Ready(n),
            Err(TryReadError::Empty) => Poll::Pending,
        }
    }
}

impl<'p, M, const N: usize> Unpin for ReadFuture<'p, M, N> where M: RawMutex {}

/// Future returned by [`Pipe::fill_buf`] and  [`Reader::fill_buf`].
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct FillBufFuture<'p, M, const N: usize>
where
    M: RawMutex,
{
    pipe: Option<&'p Pipe<M, N>>,
}

impl<'p, M, const N: usize> Future for FillBufFuture<'p, M, N>
where
    M: RawMutex,
{
    type Output = &'p [u8];

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let pipe = self.pipe.take().unwrap();
        match unsafe { pipe.try_fill_buf_with_context(Some(cx)) } {
            Ok(buf) => Poll::Ready(buf),
            Err(TryReadError::Empty) => {
                self.pipe = Some(pipe);
                Poll::Pending
            }
        }
    }
}

impl<'p, M, const N: usize> Unpin for FillBufFuture<'p, M, N> where M: RawMutex {}

/// Error returned by [`try_read`](Pipe::try_read).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TryReadError {
    /// No data could be read from the pipe because it is currently
    /// empty, and reading would require blocking.
    Empty,
}

/// Error returned by [`try_write`](Pipe::try_write).
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TryWriteError {
    /// No data could be written to the pipe because it is
    /// currently full, and writing would require blocking.
    Full,
}

struct PipeState<const N: usize> {
    buffer: RingBuffer<N>,
    read_waker: WakerRegistration,
    write_waker: WakerRegistration,
}

#[repr(transparent)]
struct Buffer<const N: usize>(UnsafeCell<[u8; N]>);

impl<const N: usize> Buffer<N> {
    unsafe fn get<'a>(&self, r: Range<usize>) -> &'a [u8] {
        let p = self.0.get() as *const u8;
        core::slice::from_raw_parts(p.add(r.start), r.end - r.start)
    }

    unsafe fn get_mut<'a>(&self, r: Range<usize>) -> &'a mut [u8] {
        let p = self.0.get() as *mut u8;
        core::slice::from_raw_parts_mut(p.add(r.start), r.end - r.start)
    }
}

unsafe impl<const N: usize> Send for Buffer<N> {}
unsafe impl<const N: usize> Sync for Buffer<N> {}

/// A bounded byte-oriented pipe for communicating between asynchronous tasks
/// with backpressure.
///
/// The pipe will buffer up to the provided number of bytes. Once the
/// buffer is full, attempts to `write` new bytes will wait until buffer space is freed up.
///
/// All data written will become available in the same order as it was written.
pub struct Pipe<M, const N: usize>
where
    M: RawMutex,
{
    buf: Buffer<N>,
    inner: Mutex<M, RefCell<PipeState<N>>>,
}

impl<M, const N: usize> Pipe<M, N>
where
    M: RawMutex,
{
    /// Establish a new bounded pipe. For example, to create one with a NoopMutex:
    ///
    /// ```
    /// use embassy_sync::pipe::Pipe;
    /// use embassy_sync::blocking_mutex::raw::NoopRawMutex;
    ///
    /// // Declare a bounded pipe, with a buffer of 256 bytes.
    /// let mut pipe = Pipe::<NoopRawMutex, 256>::new();
    /// ```
    pub const fn new() -> Self {
        Self {
            buf: Buffer(UnsafeCell::new([0; N])),
            inner: Mutex::new(RefCell::new(PipeState {
                buffer: RingBuffer::new(),
                read_waker: WakerRegistration::new(),
                write_waker: WakerRegistration::new(),
            })),
        }
    }

    fn lock<R>(&self, f: impl FnOnce(&mut PipeState<N>) -> R) -> R {
        self.inner.lock(|rc| f(&mut *rc.borrow_mut()))
    }

    fn try_read_with_context(&self, cx: Option<&mut Context<'_>>, buf: &mut [u8]) -> Result<usize, TryReadError> {
        self.inner.lock(|rc: &RefCell<PipeState<N>>| {
            let s = &mut *rc.borrow_mut();

            if s.buffer.is_full() {
                s.write_waker.wake();
            }

            let available = unsafe { self.buf.get(s.buffer.pop_buf()) };
            if available.is_empty() {
                if let Some(cx) = cx {
                    s.read_waker.register(cx.waker());
                }
                return Err(TryReadError::Empty);
            }

            let n = available.len().min(buf.len());
            buf[..n].copy_from_slice(&available[..n]);
            s.buffer.pop(n);
            Ok(n)
        })
    }

    // safety: While the returned slice is alive,
    // no `read` or `consume` methods in the pipe must be called.
    unsafe fn try_fill_buf_with_context(&self, cx: Option<&mut Context<'_>>) -> Result<&[u8], TryReadError> {
        self.inner.lock(|rc: &RefCell<PipeState<N>>| {
            let s = &mut *rc.borrow_mut();

            if s.buffer.is_full() {
                s.write_waker.wake();
            }

            let available = unsafe { self.buf.get(s.buffer.pop_buf()) };
            if available.is_empty() {
                if let Some(cx) = cx {
                    s.read_waker.register(cx.waker());
                }
                return Err(TryReadError::Empty);
            }

            Ok(available)
        })
    }

    fn consume(&self, amt: usize) {
        self.inner.lock(|rc: &RefCell<PipeState<N>>| {
            let s = &mut *rc.borrow_mut();
            let available = s.buffer.pop_buf();
            assert!(amt <= available.len());
            s.buffer.pop(amt);
        })
    }

    fn try_write_with_context(&self, cx: Option<&mut Context<'_>>, buf: &[u8]) -> Result<usize, TryWriteError> {
        self.inner.lock(|rc: &RefCell<PipeState<N>>| {
            let s = &mut *rc.borrow_mut();

            if s.buffer.is_empty() {
                s.read_waker.wake();
            }

            let available = unsafe { self.buf.get_mut(s.buffer.push_buf()) };
            if available.is_empty() {
                if let Some(cx) = cx {
                    s.write_waker.register(cx.waker());
                }
                return Err(TryWriteError::Full);
            }

            let n = available.len().min(buf.len());
            available[..n].copy_from_slice(&buf[..n]);
            s.buffer.push(n);
            Ok(n)
        })
    }

    /// Split this pipe into a BufRead-capable reader and a writer.
    ///
    /// The reader and writer borrow the current pipe mutably, so it is not
    /// possible to use it directly while they exist. This is needed because
    /// implementing `BufRead` requires there is a single reader.
    ///
    /// The writer is cloneable, the reader is not.
    pub fn split(&mut self) -> (Reader<'_, M, N>, Writer<'_, M, N>) {
        (Reader { pipe: self }, Writer { pipe: self })
    }

    /// Write some bytes to the pipe.
    ///
    /// This method writes a nonzero amount of bytes from `buf` into the pipe, and
    /// returns the amount of bytes written.
    ///
    /// If it is not possible to write a nonzero amount of bytes because the pipe's buffer is full,
    /// this method will wait until it isn't. See [`try_write`](Self::try_write) for a variant that
    /// returns an error instead of waiting.
    ///
    /// It is not guaranteed that all bytes in the buffer are written, even if there's enough
    /// free space in the pipe buffer for all. In other words, it is possible for `write` to return
    /// without writing all of `buf` (returning a number less than `buf.len()`) and still leave
    /// free space in the pipe buffer. You should always `write` in a loop, or use helpers like
    /// `write_all` from the `embedded-io` crate.
    pub fn write<'a>(&'a self, buf: &'a [u8]) -> WriteFuture<'a, M, N> {
        WriteFuture { pipe: self, buf }
    }

    /// Write all bytes to the pipe.
    ///
    /// This method writes all bytes from `buf` into the pipe
    pub async fn write_all(&self, mut buf: &[u8]) {
        while !buf.is_empty() {
            let n = self.write(buf).await;
            buf = &buf[n..];
        }
    }

    /// Attempt to immediately write some bytes to the pipe.
    ///
    /// This method will either write a nonzero amount of bytes to the pipe immediately,
    /// or return an error if the pipe is empty. See [`write`](Self::write) for a variant
    /// that waits instead of returning an error.
    pub fn try_write(&self, buf: &[u8]) -> Result<usize, TryWriteError> {
        self.try_write_with_context(None, buf)
    }

    /// Read some bytes from the pipe.
    ///
    /// This method reads a nonzero amount of bytes from the pipe into `buf` and
    /// returns the amount of bytes read.
    ///
    /// If it is not possible to read a nonzero amount of bytes because the pipe's buffer is empty,
    /// this method will wait until it isn't. See [`try_read`](Self::try_read) for a variant that
    /// returns an error instead of waiting.
    ///
    /// It is not guaranteed that all bytes in the buffer are read, even if there's enough
    /// space in `buf` for all. In other words, it is possible for `read` to return
    /// without filling `buf` (returning a number less than `buf.len()`) and still leave bytes
    /// in the pipe buffer. You should always `read` in a loop, or use helpers like
    /// `read_exact` from the `embedded-io` crate.
    pub fn read<'a>(&'a self, buf: &'a mut [u8]) -> ReadFuture<'a, M, N> {
        ReadFuture { pipe: self, buf }
    }

    /// Attempt to immediately read some bytes from the pipe.
    ///
    /// This method will either read a nonzero amount of bytes from the pipe immediately,
    /// or return an error if the pipe is empty. See [`read`](Self::read) for a variant
    /// that waits instead of returning an error.
    pub fn try_read(&self, buf: &mut [u8]) -> Result<usize, TryReadError> {
        self.try_read_with_context(None, buf)
    }

    /// Clear the data in the pipe's buffer.
    pub fn clear(&self) {
        self.inner.lock(|rc: &RefCell<PipeState<N>>| {
            let s = &mut *rc.borrow_mut();

            s.buffer.clear();
            s.write_waker.wake();
        })
    }

    /// Return whether the pipe is full (no free space in the buffer)
    pub fn is_full(&self) -> bool {
        self.len() == N
    }

    /// Return whether the pipe is empty (no data buffered)
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Total byte capacity.
    ///
    /// This is the same as the `N` generic param.
    pub fn capacity(&self) -> usize {
        N
    }

    /// Used byte capacity.
    pub fn len(&self) -> usize {
        self.lock(|c| c.buffer.len())
    }

    /// Free byte capacity.
    ///
    /// This is equivalent to `capacity() - len()`
    pub fn free_capacity(&self) -> usize {
        N - self.len()
    }
}

impl<M: RawMutex, const N: usize> embedded_io_async::ErrorType for Pipe<M, N> {
    type Error = Infallible;
}

impl<M: RawMutex, const N: usize> embedded_io_async::Read for Pipe<M, N> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(Pipe::read(self, buf).await)
    }
}

impl<M: RawMutex, const N: usize> embedded_io_async::Write for Pipe<M, N> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Ok(Pipe::write(self, buf).await)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<M: RawMutex, const N: usize> embedded_io_async::ErrorType for &Pipe<M, N> {
    type Error = Infallible;
}

impl<M: RawMutex, const N: usize> embedded_io_async::Read for &Pipe<M, N> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(Pipe::read(self, buf).await)
    }
}

impl<M: RawMutex, const N: usize> embedded_io_async::Write for &Pipe<M, N> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Ok(Pipe::write(self, buf).await)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<M: RawMutex, const N: usize> embedded_io_async::ErrorType for Reader<'_, M, N> {
    type Error = Infallible;
}

impl<M: RawMutex, const N: usize> embedded_io_async::Read for Reader<'_, M, N> {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(Reader::read(self, buf).await)
    }
}

impl<M: RawMutex, const N: usize> embedded_io_async::BufRead for Reader<'_, M, N> {
    async fn fill_buf(&mut self) -> Result<&[u8], Self::Error> {
        Ok(Reader::fill_buf(self).await)
    }

    fn consume(&mut self, amt: usize) {
        Reader::consume(self, amt)
    }
}

impl<M: RawMutex, const N: usize> embedded_io_async::ErrorType for Writer<'_, M, N> {
    type Error = Infallible;
}

impl<M: RawMutex, const N: usize> embedded_io_async::Write for Writer<'_, M, N> {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Ok(Writer::write(self, buf).await)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

//
// Type-erased variants
//

pub(crate) trait DynamicPipe {
    fn write<'a>(&'a self, buf: &'a [u8]) -> DynamicWriteFuture<'a>;
    fn read<'a>(&'a self, buf: &'a mut [u8]) -> DynamicReadFuture<'a>;

    fn try_read(&self, buf: &mut [u8]) -> Result<usize, TryReadError>;
    fn try_write(&self, buf: &[u8]) -> Result<usize, TryWriteError>;

    fn try_write_with_context(&self, cx: Option<&mut Context<'_>>, buf: &[u8]) -> Result<usize, TryWriteError>;
    fn try_read_with_context(&self, cx: Option<&mut Context<'_>>, buf: &mut [u8]) -> Result<usize, TryReadError>;

    fn consume(&self, amt: usize);
    unsafe fn try_fill_buf_with_context(&self, cx: Option<&mut Context<'_>>) -> Result<&[u8], TryReadError>;
}

impl<M, const N: usize> DynamicPipe for Pipe<M, N>
where
    M: RawMutex,
{
    fn consume(&self, amt: usize) {
        Pipe::consume(self, amt)
    }

    unsafe fn try_fill_buf_with_context(&self, cx: Option<&mut Context<'_>>) -> Result<&[u8], TryReadError> {
        Pipe::try_fill_buf_with_context(self, cx)
    }

    fn write<'a>(&'a self, buf: &'a [u8]) -> DynamicWriteFuture<'a> {
        Pipe::write(self, buf).into()
    }

    fn read<'a>(&'a self, buf: &'a mut [u8]) -> DynamicReadFuture<'a> {
        Pipe::read(self, buf).into()
    }

    fn try_read(&self, buf: &mut [u8]) -> Result<usize, TryReadError> {
        Pipe::try_read(self, buf)
    }

    fn try_write(&self, buf: &[u8]) -> Result<usize, TryWriteError> {
        Pipe::try_write(self, buf)
    }

    fn try_write_with_context(&self, cx: Option<&mut Context<'_>>, buf: &[u8]) -> Result<usize, TryWriteError> {
        Pipe::try_write_with_context(self, cx, buf)
    }

    fn try_read_with_context(&self, cx: Option<&mut Context<'_>>, buf: &mut [u8]) -> Result<usize, TryReadError> {
        Pipe::try_read_with_context(self, cx, buf)
    }
}

/// Write-only access to a [`DynamicPipe`].
pub struct DynamicWriter<'p> {
    pipe: &'p dyn DynamicPipe,
}

impl<'p> Clone for DynamicWriter<'p> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'p> Copy for DynamicWriter<'p> {}

impl<'p> DynamicWriter<'p> {
    /// Write some bytes to the pipe.
    ///
    /// See [`Pipe::write()`]
    pub fn write<'a>(&'a self, buf: &'a [u8]) -> DynamicWriteFuture<'a> {
        self.pipe.write(buf)
    }

    /// Attempt to immediately write some bytes to the pipe.
    ///
    /// See [`Pipe::try_write()`]
    pub fn try_write(&self, buf: &[u8]) -> Result<usize, TryWriteError> {
        self.pipe.try_write(buf)
    }
}

impl<'p, M, const N: usize> From<Writer<'p, M, N>> for DynamicWriter<'p>
where
    M: RawMutex,
{
    fn from(value: Writer<'p, M, N>) -> Self {
        Self { pipe: value.pipe }
    }
}

/// Future returned by [`DynamicWriter::write`].
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct DynamicWriteFuture<'p> {
    pipe: &'p dyn DynamicPipe,
    buf: &'p [u8],
}

impl<'p> Future for DynamicWriteFuture<'p> {
    type Output = usize;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.pipe.try_write_with_context(Some(cx), self.buf) {
            Ok(n) => Poll::Ready(n),
            Err(TryWriteError::Full) => Poll::Pending,
        }
    }
}

impl<'p> Unpin for DynamicWriteFuture<'p> {}

impl<'p, M, const N: usize> From<WriteFuture<'p, M, N>> for DynamicWriteFuture<'p>
where
    M: RawMutex,
{
    fn from(value: WriteFuture<'p, M, N>) -> Self {
        Self {
            pipe: value.pipe,
            buf: value.buf,
        }
    }
}

/// Read-only access to a [`DynamicPipe`].
pub struct DynamicReader<'p> {
    pipe: &'p dyn DynamicPipe,
}

impl<'p> DynamicReader<'p> {
    /// Read some bytes from the pipe.
    ///
    /// See [`Pipe::read()`]
    pub fn read<'a>(&'a self, buf: &'a mut [u8]) -> DynamicReadFuture<'a> {
        self.pipe.read(buf)
    }

    /// Attempt to immediately read some bytes from the pipe.
    ///
    /// See [`Pipe::try_read()`]
    pub fn try_read(&self, buf: &mut [u8]) -> Result<usize, TryReadError> {
        self.pipe.try_read(buf)
    }

    /// Return the contents of the internal buffer, filling it with more data from the inner reader if it is empty.
    ///
    /// If no bytes are currently available to read, this function waits until at least one byte is available.
    ///
    /// If the reader is at end-of-file (EOF), an empty slice is returned.
    pub fn fill_buf(&mut self) -> DynamicFillBufFuture<'_> {
        DynamicFillBufFuture { pipe: Some(self.pipe) }
    }

    /// Try returning contents of the internal buffer.
    ///
    /// If no bytes are currently available to read, this function returns `Err(TryReadError::Empty)`.
    ///
    /// If the reader is at end-of-file (EOF), an empty slice is returned.
    pub fn try_fill_buf(&mut self) -> Result<&[u8], TryReadError> {
        unsafe { self.pipe.try_fill_buf_with_context(None) }
    }

    /// Tell this buffer that `amt` bytes have been consumed from the buffer, so they should no longer be returned in calls to `fill_buf`.
    pub fn consume(&mut self, amt: usize) {
        self.pipe.consume(amt)
    }
}

impl<'p, M, const N: usize> From<Reader<'p, M, N>> for DynamicReader<'p>
where
    M: RawMutex,
{
    fn from(value: Reader<'p, M, N>) -> Self {
        Self { pipe: value.pipe }
    }
}

/// Future returned by [`Pipe::read`] and  [`Reader::read`].
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct DynamicReadFuture<'p> {
    pipe: &'p dyn DynamicPipe,
    buf: &'p mut [u8],
}

impl<'p> Future for DynamicReadFuture<'p> {
    type Output = usize;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.pipe.try_read_with_context(Some(cx), self.buf) {
            Ok(n) => Poll::Ready(n),
            Err(TryReadError::Empty) => Poll::Pending,
        }
    }
}

impl<'p> Unpin for DynamicReadFuture<'p> {}

impl<'p, M, const N: usize> From<ReadFuture<'p, M, N>> for DynamicReadFuture<'p>
where
    M: RawMutex,
{
    fn from(value: ReadFuture<'p, M, N>) -> Self {
        Self {
            pipe: value.pipe,
            buf: value.buf,
        }
    }
}

/// Future returned by [`DynamicPipe::fill_buf`] and  [`DynamicReader::fill_buf`].
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct DynamicFillBufFuture<'p> {
    pipe: Option<&'p dyn DynamicPipe>,
}

impl<'p> Future for DynamicFillBufFuture<'p> {
    type Output = &'p [u8];

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let pipe = self.pipe.take().unwrap();
        match unsafe { pipe.try_fill_buf_with_context(Some(cx)) } {
            Ok(buf) => Poll::Ready(buf),
            Err(TryReadError::Empty) => {
                self.pipe = Some(pipe);
                Poll::Pending
            }
        }
    }
}

impl<'p> Unpin for DynamicFillBufFuture<'p> {}

impl<'p, M, const N: usize> From<FillBufFuture<'p, M, N>> for DynamicFillBufFuture<'p>
where
    M: RawMutex,
{
    fn from(value: FillBufFuture<'p, M, N>) -> Self {
        Self {
            pipe: value.pipe.map(|p| p as &dyn DynamicPipe),
        }
    }
}

#[cfg(test)]
mod tests {
    use futures_executor::ThreadPool;
    use futures_util::task::SpawnExt;
    use static_cell::StaticCell;

    use super::*;
    use crate::blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex};

    #[test]
    fn writing_once() {
        let c = Pipe::<NoopRawMutex, 3>::new();
        assert!(c.try_write(&[1]).is_ok());
        assert_eq!(c.free_capacity(), 2);
    }

    #[test]
    fn writing_when_full() {
        let c = Pipe::<NoopRawMutex, 3>::new();
        assert_eq!(c.try_write(&[42]), Ok(1));
        assert_eq!(c.try_write(&[43]), Ok(1));
        assert_eq!(c.try_write(&[44]), Ok(1));
        assert_eq!(c.try_write(&[45]), Err(TryWriteError::Full));
        assert_eq!(c.free_capacity(), 0);
    }

    #[test]
    fn receiving_once_with_one_send() {
        let c = Pipe::<NoopRawMutex, 3>::new();
        assert!(c.try_write(&[42]).is_ok());
        let mut buf = [0; 16];
        assert_eq!(c.try_read(&mut buf), Ok(1));
        assert_eq!(buf[0], 42);
        assert_eq!(c.free_capacity(), 3);
    }

    #[test]
    fn receiving_when_empty() {
        let c = Pipe::<NoopRawMutex, 3>::new();
        let mut buf = [0; 16];
        assert_eq!(c.try_read(&mut buf), Err(TryReadError::Empty));
        assert_eq!(c.free_capacity(), 3);
    }

    #[test]
    fn simple_send_and_receive() {
        let c = Pipe::<NoopRawMutex, 3>::new();
        assert!(c.try_write(&[42]).is_ok());
        let mut buf = [0; 16];
        assert_eq!(c.try_read(&mut buf), Ok(1));
        assert_eq!(buf[0], 42);
    }

    #[test]
    fn read_buf() {
        let mut c = Pipe::<NoopRawMutex, 3>::new();
        let (mut r, w) = c.split();
        assert!(w.try_write(&[42, 43]).is_ok());
        let buf = r.try_fill_buf().unwrap();
        assert_eq!(buf, &[42, 43]);
        let buf = r.try_fill_buf().unwrap();
        assert_eq!(buf, &[42, 43]);
        r.consume(1);
        let buf = r.try_fill_buf().unwrap();
        assert_eq!(buf, &[43]);
        r.consume(1);
        assert_eq!(r.try_fill_buf(), Err(TryReadError::Empty));
        assert_eq!(w.try_write(&[44, 45, 46]), Ok(1));
        assert_eq!(w.try_write(&[45, 46]), Ok(2));
        let buf = r.try_fill_buf().unwrap();
        assert_eq!(buf, &[44]); // only one byte due to wraparound.
        r.consume(1);
        let buf = r.try_fill_buf().unwrap();
        assert_eq!(buf, &[45, 46]);
        assert!(w.try_write(&[47]).is_ok());
        let buf = r.try_fill_buf().unwrap();
        assert_eq!(buf, &[45, 46, 47]);
        r.consume(3);
    }

    #[test]
    fn writer_is_cloneable() {
        let mut c = Pipe::<NoopRawMutex, 3>::new();
        let (_r, w) = c.split();
        let _ = w.clone();
    }

    #[test]
    fn dynamic_dispatch_pipe() {
        let mut c = Pipe::<NoopRawMutex, 3>::new();
        let (r, w) = c.split();
        let (mut r, w): (DynamicReader<'_>, DynamicWriter<'_>) = (r.into(), w.into());

        assert!(w.try_write(&[42, 43]).is_ok());
        let buf = r.try_fill_buf().unwrap();
        assert_eq!(buf, &[42, 43]);
        let buf = r.try_fill_buf().unwrap();
        assert_eq!(buf, &[42, 43]);
        r.consume(1);
        let buf = r.try_fill_buf().unwrap();
        assert_eq!(buf, &[43]);
        r.consume(1);
        assert_eq!(r.try_fill_buf(), Err(TryReadError::Empty));
        assert_eq!(w.try_write(&[44, 45, 46]), Ok(1));
        assert_eq!(w.try_write(&[45, 46]), Ok(2));
        let buf = r.try_fill_buf().unwrap();
        assert_eq!(buf, &[44]); // only one byte due to wraparound.
        r.consume(1);
        let buf = r.try_fill_buf().unwrap();
        assert_eq!(buf, &[45, 46]);
        assert!(w.try_write(&[47]).is_ok());
        let buf = r.try_fill_buf().unwrap();
        assert_eq!(buf, &[45, 46, 47]);
        r.consume(3);
    }

    #[futures_test::test]
    async fn receiver_receives_given_try_write_async() {
        let executor = ThreadPool::new().unwrap();

        static CHANNEL: StaticCell<Pipe<CriticalSectionRawMutex, 3>> = StaticCell::new();
        let c = &*CHANNEL.init(Pipe::new());
        let c2 = c;
        let f = async move {
            assert_eq!(c2.try_write(&[42]), Ok(1));
        };
        executor.spawn(f).unwrap();
        let mut buf = [0; 16];
        assert_eq!(c.read(&mut buf).await, 1);
        assert_eq!(buf[0], 42);
    }

    #[futures_test::test]
    async fn sender_send_completes_if_capacity() {
        let c = Pipe::<CriticalSectionRawMutex, 1>::new();
        c.write(&[42]).await;
        let mut buf = [0; 16];
        assert_eq!(c.read(&mut buf).await, 1);
        assert_eq!(buf[0], 42);
    }
}

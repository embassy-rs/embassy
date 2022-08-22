//! Async byte stream pipe.

use core::cell::RefCell;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::blocking_mutex::raw::RawMutex;
use crate::blocking_mutex::Mutex;
use crate::ring_buffer::RingBuffer;
use crate::waitqueue::WakerRegistration;

/// Write-only access to a [`Pipe`].
#[derive(Copy)]
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
        Writer { pipe: self.pipe }
    }
}

impl<'p, M, const N: usize> Writer<'p, M, N>
where
    M: RawMutex,
{
    /// Writes a value.
    ///
    /// See [`Pipe::write()`]
    pub fn write<'a>(&'a self, buf: &'a [u8]) -> WriteFuture<'a, M, N> {
        self.pipe.write(buf)
    }

    /// Attempt to immediately write a message.
    ///
    /// See [`Pipe::write()`]
    pub fn try_write(&self, buf: &[u8]) -> Result<usize, TryWriteError> {
        self.pipe.try_write(buf)
    }
}

/// Future returned by [`Pipe::write`] and  [`Writer::write`].
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
#[derive(Copy)]
pub struct Reader<'p, M, const N: usize>
where
    M: RawMutex,
{
    pipe: &'p Pipe<M, N>,
}

impl<'p, M, const N: usize> Clone for Reader<'p, M, N>
where
    M: RawMutex,
{
    fn clone(&self) -> Self {
        Reader { pipe: self.pipe }
    }
}

impl<'p, M, const N: usize> Reader<'p, M, N>
where
    M: RawMutex,
{
    /// Reads a value.
    ///
    /// See [`Pipe::read()`]
    pub fn read<'a>(&'a self, buf: &'a mut [u8]) -> ReadFuture<'a, M, N> {
        self.pipe.read(buf)
    }

    /// Attempt to immediately read a message.
    ///
    /// See [`Pipe::read()`]
    pub fn try_read(&self, buf: &mut [u8]) -> Result<usize, TryReadError> {
        self.pipe.try_read(buf)
    }
}

/// Future returned by [`Pipe::read`] and  [`Reader::read`].
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

impl<const N: usize> PipeState<N> {
    const fn new() -> Self {
        PipeState {
            buffer: RingBuffer::new(),
            read_waker: WakerRegistration::new(),
            write_waker: WakerRegistration::new(),
        }
    }

    fn clear(&mut self) {
        self.buffer.clear();
        self.write_waker.wake();
    }

    fn try_read(&mut self, buf: &mut [u8]) -> Result<usize, TryReadError> {
        self.try_read_with_context(None, buf)
    }

    fn try_read_with_context(&mut self, cx: Option<&mut Context<'_>>, buf: &mut [u8]) -> Result<usize, TryReadError> {
        if self.buffer.is_full() {
            self.write_waker.wake();
        }

        let available = self.buffer.pop_buf();
        if available.is_empty() {
            if let Some(cx) = cx {
                self.read_waker.register(cx.waker());
            }
            return Err(TryReadError::Empty);
        }

        let n = available.len().min(buf.len());
        buf[..n].copy_from_slice(&available[..n]);
        self.buffer.pop(n);
        Ok(n)
    }

    fn try_write(&mut self, buf: &[u8]) -> Result<usize, TryWriteError> {
        self.try_write_with_context(None, buf)
    }

    fn try_write_with_context(&mut self, cx: Option<&mut Context<'_>>, buf: &[u8]) -> Result<usize, TryWriteError> {
        if self.buffer.is_empty() {
            self.read_waker.wake();
        }

        let available = self.buffer.push_buf();
        if available.is_empty() {
            if let Some(cx) = cx {
                self.write_waker.register(cx.waker());
            }
            return Err(TryWriteError::Full);
        }

        let n = available.len().min(buf.len());
        available[..n].copy_from_slice(&buf[..n]);
        self.buffer.push(n);
        Ok(n)
    }
}

/// A bounded pipe for communicating between asynchronous tasks
/// with backpressure.
///
/// The pipe will buffer up to the provided number of messages.  Once the
/// buffer is full, attempts to `write` new messages will wait until a message is
/// read from the pipe.
///
/// All data written will become available in the same order as it was written.
pub struct Pipe<M, const N: usize>
where
    M: RawMutex,
{
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
            inner: Mutex::new(RefCell::new(PipeState::new())),
        }
    }

    fn lock<R>(&self, f: impl FnOnce(&mut PipeState<N>) -> R) -> R {
        self.inner.lock(|rc| f(&mut *rc.borrow_mut()))
    }

    fn try_read_with_context(&self, cx: Option<&mut Context<'_>>, buf: &mut [u8]) -> Result<usize, TryReadError> {
        self.lock(|c| c.try_read_with_context(cx, buf))
    }

    fn try_write_with_context(&self, cx: Option<&mut Context<'_>>, buf: &[u8]) -> Result<usize, TryWriteError> {
        self.lock(|c| c.try_write_with_context(cx, buf))
    }

    /// Get a writer for this pipe.
    pub fn writer(&self) -> Writer<'_, M, N> {
        Writer { pipe: self }
    }

    /// Get a reader for this pipe.
    pub fn reader(&self) -> Reader<'_, M, N> {
        Reader { pipe: self }
    }

    /// Write a value, waiting until there is capacity.
    ///
    /// Writeing completes when the value has been pushed to the pipe's queue.
    /// This doesn't mean the value has been read yet.
    pub fn write<'a>(&'a self, buf: &'a [u8]) -> WriteFuture<'a, M, N> {
        WriteFuture { pipe: self, buf }
    }

    /// Attempt to immediately write a message.
    ///
    /// This method differs from [`write`](Pipe::write) by returning immediately if the pipe's
    /// buffer is full, instead of waiting.
    ///
    /// # Errors
    ///
    /// If the pipe capacity has been reached, i.e., the pipe has `n`
    /// buffered values where `n` is the argument passed to [`Pipe`], then an
    /// error is returned.
    pub fn try_write(&self, buf: &[u8]) -> Result<usize, TryWriteError> {
        self.lock(|c| c.try_write(buf))
    }

    /// Receive the next value.
    ///
    /// If there are no messages in the pipe's buffer, this method will
    /// wait until a message is written.
    pub fn read<'a>(&'a self, buf: &'a mut [u8]) -> ReadFuture<'a, M, N> {
        ReadFuture { pipe: self, buf }
    }

    /// Attempt to immediately read a message.
    ///
    /// This method will either read a message from the pipe immediately or return an error
    /// if the pipe is empty.
    pub fn try_read(&self, buf: &mut [u8]) -> Result<usize, TryReadError> {
        self.lock(|c| c.try_read(buf))
    }

    /// Clear the data in the pipe's buffer.
    pub fn clear(&self) {
        self.lock(|c| c.clear())
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

#[cfg(feature = "nightly")]
mod io_impls {
    use core::convert::Infallible;

    use futures_util::FutureExt;

    use super::*;

    impl<M: RawMutex, const N: usize> embedded_io::Io for Pipe<M, N> {
        type Error = Infallible;
    }

    impl<M: RawMutex, const N: usize> embedded_io::asynch::Read for Pipe<M, N> {
        type ReadFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
        where
            Self: 'a;

        fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
            Pipe::read(self, buf).map(Ok)
        }
    }

    impl<M: RawMutex, const N: usize> embedded_io::asynch::Write for Pipe<M, N> {
        type WriteFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
        where
            Self: 'a;

        fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
            Pipe::write(self, buf).map(Ok)
        }

        type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>>
        where
            Self: 'a;

        fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
            futures_util::future::ready(Ok(()))
        }
    }

    impl<M: RawMutex, const N: usize> embedded_io::Io for &Pipe<M, N> {
        type Error = Infallible;
    }

    impl<M: RawMutex, const N: usize> embedded_io::asynch::Read for &Pipe<M, N> {
        type ReadFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
        where
            Self: 'a;

        fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
            Pipe::read(self, buf).map(Ok)
        }
    }

    impl<M: RawMutex, const N: usize> embedded_io::asynch::Write for &Pipe<M, N> {
        type WriteFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
        where
            Self: 'a;

        fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
            Pipe::write(self, buf).map(Ok)
        }

        type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>>
        where
            Self: 'a;

        fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
            futures_util::future::ready(Ok(()))
        }
    }

    impl<M: RawMutex, const N: usize> embedded_io::Io for Reader<'_, M, N> {
        type Error = Infallible;
    }

    impl<M: RawMutex, const N: usize> embedded_io::asynch::Read for Reader<'_, M, N> {
        type ReadFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
        where
            Self: 'a;

        fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a> {
            Reader::read(self, buf).map(Ok)
        }
    }

    impl<M: RawMutex, const N: usize> embedded_io::Io for Writer<'_, M, N> {
        type Error = Infallible;
    }

    impl<M: RawMutex, const N: usize> embedded_io::asynch::Write for Writer<'_, M, N> {
        type WriteFuture<'a> = impl Future<Output = Result<usize, Self::Error>>
        where
            Self: 'a;

        fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a> {
            Writer::write(self, buf).map(Ok)
        }

        type FlushFuture<'a> = impl Future<Output = Result<(), Self::Error>>
        where
            Self: 'a;

        fn flush<'a>(&'a mut self) -> Self::FlushFuture<'a> {
            futures_util::future::ready(Ok(()))
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

    fn capacity<const N: usize>(c: &PipeState<N>) -> usize {
        N - c.buffer.len()
    }

    #[test]
    fn writing_once() {
        let mut c = PipeState::<3>::new();
        assert!(c.try_write(&[1]).is_ok());
        assert_eq!(capacity(&c), 2);
    }

    #[test]
    fn writing_when_full() {
        let mut c = PipeState::<3>::new();
        assert_eq!(c.try_write(&[42]), Ok(1));
        assert_eq!(c.try_write(&[43]), Ok(1));
        assert_eq!(c.try_write(&[44]), Ok(1));
        assert_eq!(c.try_write(&[45]), Err(TryWriteError::Full));
        assert_eq!(capacity(&c), 0);
    }

    #[test]
    fn receiving_once_with_one_send() {
        let mut c = PipeState::<3>::new();
        assert!(c.try_write(&[42]).is_ok());
        let mut buf = [0; 16];
        assert_eq!(c.try_read(&mut buf), Ok(1));
        assert_eq!(buf[0], 42);
        assert_eq!(capacity(&c), 3);
    }

    #[test]
    fn receiving_when_empty() {
        let mut c = PipeState::<3>::new();
        let mut buf = [0; 16];
        assert_eq!(c.try_read(&mut buf), Err(TryReadError::Empty));
        assert_eq!(capacity(&c), 3);
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
    fn cloning() {
        let c = Pipe::<NoopRawMutex, 3>::new();
        let r1 = c.reader();
        let w1 = c.writer();

        let _ = r1.clone();
        let _ = w1.clone();
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

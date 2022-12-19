use core::slice;
use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

/// Atomic reusable ringbuffer
///
/// This ringbuffer implementation is designed to be stored in a `static`,
/// therefore all methods take `&self` and not `&mut self`.
///
/// It is "reusable": when created it has no backing buffer, you can give it
/// one with `init` and take it back with `deinit`, and init it again in the
/// future if needed. This is very non-idiomatic, but helps a lot when storing
/// it in a `static`.
///
/// One concurrent writer and one concurrent reader are supported, even at
/// different execution priorities (like main and irq).
pub struct RingBuffer {
    buf: AtomicPtr<u8>,
    len: AtomicUsize,
    start: AtomicUsize,
    end: AtomicUsize,
}

pub struct Reader<'a>(&'a RingBuffer);
pub struct Writer<'a>(&'a RingBuffer);

impl RingBuffer {
    /// Create a new empty ringbuffer.
    pub const fn new() -> Self {
        Self {
            buf: AtomicPtr::new(core::ptr::null_mut()),
            len: AtomicUsize::new(0),
            start: AtomicUsize::new(0),
            end: AtomicUsize::new(0),
        }
    }

    /// Initialize the ring buffer with a buffer.
    ///
    /// # Safety
    /// - The buffer (`buf .. buf+len`) must be valid memory until `deinit` is called.
    /// - Must not be called concurrently with any other methods.
    pub unsafe fn init(&self, buf: *mut u8, len: usize) {
        // Ordering: it's OK to use `Relaxed` because this is not called
        // concurrently with other methods.
        self.buf.store(buf, Ordering::Relaxed);
        self.len.store(len, Ordering::Relaxed);
        self.start.store(0, Ordering::Relaxed);
        self.end.store(0, Ordering::Relaxed);
    }

    /// Deinitialize the ringbuffer.
    ///
    /// After calling this, the ringbuffer becomes empty, as if it was
    /// just created with `new()`.
    ///
    /// # Safety
    /// - Must not be called concurrently with any other methods.
    pub unsafe fn deinit(&self) {
        // Ordering: it's OK to use `Relaxed` because this is not called
        // concurrently with other methods.
        self.len.store(0, Ordering::Relaxed);
        self.start.store(0, Ordering::Relaxed);
        self.end.store(0, Ordering::Relaxed);
    }

    /// Create a reader.
    ///
    /// # Safety
    ///
    /// Only one reader can exist at a time.
    pub unsafe fn reader(&self) -> Reader<'_> {
        Reader(self)
    }

    /// Create a writer.
    ///
    /// # Safety
    ///
    /// Only one writer can exist at a time.
    pub unsafe fn writer(&self) -> Writer<'_> {
        Writer(self)
    }

    pub fn is_full(&self) -> bool {
        let len = self.len.load(Ordering::Relaxed);
        let start = self.start.load(Ordering::Relaxed);
        let end = self.end.load(Ordering::Relaxed);

        len == 0 || self.wrap(end + 1) == start
    }

    pub fn is_empty(&self) -> bool {
        let start = self.start.load(Ordering::Relaxed);
        let end = self.end.load(Ordering::Relaxed);

        start == end
    }

    fn wrap(&self, n: usize) -> usize {
        let len = self.len.load(Ordering::Relaxed);

        assert!(n <= len);
        if n == len {
            0
        } else {
            n
        }
    }
}

impl<'a> Writer<'a> {
    /// Push data into the buffer in-place.
    ///
    /// The closure `f` is called with a free part of the buffer, it must write
    /// some data to it and return the amount of bytes written.
    pub fn push(&mut self, f: impl FnOnce(&mut [u8]) -> usize) -> usize {
        let (p, n) = self.push_buf();
        let buf = unsafe { slice::from_raw_parts_mut(p, n) };
        let n = f(buf);
        self.push_done(n);
        n
    }

    /// Push one data byte.
    ///
    /// Returns true if pushed succesfully.
    pub fn push_one(&mut self, val: u8) -> bool {
        let n = self.push(|f| match f {
            [] => 0,
            [x, ..] => {
                *x = val;
                1
            }
        });
        n != 0
    }

    /// Get a buffer where data can be pushed to.
    ///
    /// Write data to the start of the buffer, then call `push_done` with
    /// however many bytes you've pushed.
    ///
    /// The buffer is suitable to DMA to.
    ///
    /// If the ringbuf is full, size=0 will be returned.
    ///
    /// The buffer stays valid as long as no other `Writer` method is called
    /// and `init`/`deinit` aren't called on the ringbuf.
    pub fn push_buf(&mut self) -> (*mut u8, usize) {
        // Ordering: popping writes `start` last, so we read `start` first.
        // Read it with Acquire ordering, so that the next accesses can't be reordered up past it.
        let start = self.0.start.load(Ordering::Acquire);
        let buf = self.0.buf.load(Ordering::Relaxed);
        let len = self.0.len.load(Ordering::Relaxed);
        let end = self.0.end.load(Ordering::Relaxed);

        let n = if start <= end {
            len - end - (start == 0 && len != 0) as usize
        } else {
            start - end - 1
        };

        trace!("  ringbuf: push_buf {:?}..{:?}", end, end + n);
        (unsafe { buf.add(end) }, n)
    }

    pub fn push_done(&mut self, n: usize) {
        trace!("  ringbuf: push {:?}", n);
        let end = self.0.end.load(Ordering::Relaxed);

        // Ordering: write `end` last, with Release ordering.
        // The ordering ensures no preceding memory accesses (such as writing
        // the actual data in the buffer) can be reordered down past it, which
        // will guarantee the reader sees them after reading from `end`.
        self.0.end.store(self.0.wrap(end + n), Ordering::Release);
    }
}

impl<'a> Reader<'a> {
    /// Pop data from the buffer in-place.
    ///
    /// The closure `f` is called with the next data, it must process
    /// some data from it and return the amount of bytes processed.
    pub fn pop(&mut self, f: impl FnOnce(&[u8]) -> usize) -> usize {
        let (p, n) = self.pop_buf();
        let buf = unsafe { slice::from_raw_parts(p, n) };
        let n = f(buf);
        self.pop_done(n);
        n
    }

    /// Pop one data byte.
    ///
    /// Returns true if popped succesfully.
    pub fn pop_one(&mut self) -> Option<u8> {
        let mut res = None;
        self.pop(|f| match f {
            &[] => 0,
            &[x, ..] => {
                res = Some(x);
                1
            }
        });
        res
    }

    /// Get a buffer where data can be popped from.
    ///
    /// Read data from the start of the buffer, then call `pop_done` with
    /// however many bytes you've processed.
    ///
    /// The buffer is suitable to DMA from.
    ///
    /// If the ringbuf is empty, size=0 will be returned.
    ///
    /// The buffer stays valid as long as no other `Reader` method is called
    /// and `init`/`deinit` aren't called on the ringbuf.
    pub fn pop_buf(&mut self) -> (*mut u8, usize) {
        // Ordering: pushing writes `end` last, so we read `end` first.
        // Read it with Acquire ordering, so that the next accesses can't be reordered up past it.
        // This is needed to guarantee we "see" the data written by the writer.
        let end = self.0.end.load(Ordering::Acquire);
        let buf = self.0.buf.load(Ordering::Relaxed);
        let len = self.0.len.load(Ordering::Relaxed);
        let start = self.0.start.load(Ordering::Relaxed);

        let n = if end < start { len - start } else { end - start };

        trace!("  ringbuf: pop_buf {:?}..{:?}", start, start + n);
        (unsafe { buf.add(start) }, n)
    }

    pub fn pop_done(&mut self, n: usize) {
        trace!("  ringbuf: pop {:?}", n);

        let start = self.0.start.load(Ordering::Relaxed);

        // Ordering: write `start` last, with Release ordering.
        // The ordering ensures no preceding memory accesses (such as reading
        // the actual data) can be reordered down past it. This is necessary
        // because writing to `start` is effectively freeing the read part of the
        // buffer, which "gives permission" to the writer to write to it again.
        // Therefore, all buffer accesses must be completed before this.
        self.0.start.store(self.0.wrap(start + n), Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop() {
        let mut b = [0; 4];
        let rb = RingBuffer::new();
        unsafe {
            rb.init(b.as_mut_ptr(), 4);

            assert_eq!(rb.is_empty(), true);
            assert_eq!(rb.is_full(), false);

            rb.writer().push(|buf| {
                // If capacity is 4, we can fill it up to 3.
                assert_eq!(3, buf.len());
                buf[0] = 1;
                buf[1] = 2;
                buf[2] = 3;
                3
            });

            assert_eq!(rb.is_empty(), false);
            assert_eq!(rb.is_full(), true);

            rb.writer().push(|buf| {
                // If it's full, we can push 0 bytes.
                assert_eq!(0, buf.len());
                0
            });

            assert_eq!(rb.is_empty(), false);
            assert_eq!(rb.is_full(), true);

            rb.reader().pop(|buf| {
                assert_eq!(3, buf.len());
                assert_eq!(1, buf[0]);
                1
            });

            assert_eq!(rb.is_empty(), false);
            assert_eq!(rb.is_full(), false);

            rb.reader().pop(|buf| {
                assert_eq!(2, buf.len());
                0
            });

            assert_eq!(rb.is_empty(), false);
            assert_eq!(rb.is_full(), false);

            rb.reader().pop(|buf| {
                assert_eq!(2, buf.len());
                assert_eq!(2, buf[0]);
                assert_eq!(3, buf[1]);
                2
            });

            assert_eq!(rb.is_empty(), true);
            assert_eq!(rb.is_full(), false);

            rb.reader().pop(|buf| {
                assert_eq!(0, buf.len());
                0
            });

            rb.writer().push(|buf| {
                assert_eq!(1, buf.len());
                buf[0] = 10;
                1
            });

            rb.writer().push(|buf| {
                assert_eq!(2, buf.len());
                buf[0] = 11;
                buf[1] = 12;
                2
            });

            assert_eq!(rb.is_empty(), false);
            assert_eq!(rb.is_full(), true);
        }
    }

    #[test]
    fn zero_len() {
        let rb = RingBuffer::new();
        unsafe {
            assert_eq!(rb.is_empty(), true);
            assert_eq!(rb.is_full(), true);

            rb.writer().push(|buf| {
                assert_eq!(0, buf.len());
                0
            });

            rb.reader().pop(|buf| {
                assert_eq!(0, buf.len());
                0
            });
        }
    }
}

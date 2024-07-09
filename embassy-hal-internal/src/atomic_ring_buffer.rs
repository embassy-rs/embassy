//! Atomic reusable ringbuffer.
use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use core::{ptr, slice};

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
    #[doc(hidden)]
    pub buf: AtomicPtr<u8>,
    len: AtomicUsize,

    // start and end wrap at len*2, not at len.
    // This allows distinguishing "full" and "empty".
    // full is when start+len == end (modulo len*2)
    // empty is when start == end
    //
    // This avoids having to consider the ringbuffer "full" at len-1 instead of len.
    // The usual solution is adding a "full" flag, but that can't be made atomic
    #[doc(hidden)]
    pub start: AtomicUsize,
    #[doc(hidden)]
    pub end: AtomicUsize,
}

/// A type which can only read from a ring buffer.
pub struct Reader<'a>(&'a RingBuffer);

/// A type which can only write to a ring buffer.
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
        self.buf.store(ptr::null_mut(), Ordering::Relaxed);
        self.len.store(0, Ordering::Relaxed);
        self.start.store(0, Ordering::Relaxed);
        self.end.store(0, Ordering::Relaxed);
    }

    /// Create a reader.
    ///
    /// # Safety
    ///
    /// - Only one reader can exist at a time.
    /// - Ringbuffer must be initialized.
    pub unsafe fn reader(&self) -> Reader<'_> {
        Reader(self)
    }

    /// Try creating a reader, fails if not initialized.
    ///
    /// # Safety
    ///
    /// Only one reader can exist at a time.
    pub unsafe fn try_reader(&self) -> Option<Reader<'_>> {
        if self.buf.load(Ordering::Relaxed).is_null() {
            return None;
        }
        Some(Reader(self))
    }

    /// Create a writer.
    ///
    /// # Safety
    ///
    /// - Only one writer can exist at a time.
    /// - Ringbuffer must be initialized.
    pub unsafe fn writer(&self) -> Writer<'_> {
        Writer(self)
    }

    /// Try creating a writer, fails if not initialized.
    ///
    /// # Safety
    ///
    /// Only one writer can exist at a time.
    pub unsafe fn try_writer(&self) -> Option<Writer<'_>> {
        if self.buf.load(Ordering::Relaxed).is_null() {
            return None;
        }
        Some(Writer(self))
    }

    /// Return if buffer is available.
    pub fn is_available(&self) -> bool {
        !self.buf.load(Ordering::Relaxed).is_null() && self.len.load(Ordering::Relaxed) != 0
    }

    /// Return length of buffer.
    pub fn len(&self) -> usize {
        self.len.load(Ordering::Relaxed)
    }

    /// Check if buffer is full.
    pub fn is_full(&self) -> bool {
        let len = self.len.load(Ordering::Relaxed);
        let start = self.start.load(Ordering::Relaxed);
        let end = self.end.load(Ordering::Relaxed);

        self.wrap(start + len) == end
    }

    /// Check if buffer is empty.
    pub fn is_empty(&self) -> bool {
        let start = self.start.load(Ordering::Relaxed);
        let end = self.end.load(Ordering::Relaxed);

        start == end
    }

    fn wrap(&self, mut n: usize) -> usize {
        let len = self.len.load(Ordering::Relaxed);

        if n >= len * 2 {
            n -= len * 2
        }
        n
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
    /// Returns true if pushed successfully.
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
    /// Equivalent to [`Self::push_buf`] but returns a slice.
    pub fn push_slice(&mut self) -> &mut [u8] {
        let (data, len) = self.push_buf();
        unsafe { slice::from_raw_parts_mut(data, len) }
    }

    /// Get up to two buffers where data can be pushed to.
    ///
    /// Equivalent to [`Self::push_bufs`] but returns slices.
    pub fn push_slices(&mut self) -> [&mut [u8]; 2] {
        let [(d0, l0), (d1, l1)] = self.push_bufs();
        unsafe { [slice::from_raw_parts_mut(d0, l0), slice::from_raw_parts_mut(d1, l1)] }
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
        let mut start = self.0.start.load(Ordering::Acquire);
        let buf = self.0.buf.load(Ordering::Relaxed);
        let len = self.0.len.load(Ordering::Relaxed);
        let mut end = self.0.end.load(Ordering::Relaxed);

        let empty = start == end;

        if start >= len {
            start -= len
        }
        if end >= len {
            end -= len
        }

        if start == end && !empty {
            // full
            return (buf, 0);
        }
        let n = if start > end { start - end } else { len - end };

        trace!("  ringbuf: push_buf {:?}..{:?}", end, end + n);
        (unsafe { buf.add(end) }, n)
    }

    /// Get up to two buffers where data can be pushed to.
    ///
    /// Write data starting at the beginning of the first buffer, then call
    /// `push_done` with however many bytes you've pushed.
    ///
    /// The buffers are suitable to DMA to.
    ///
    /// If the ringbuf is full, both buffers will be zero length.
    /// If there is only area available, the second buffer will be zero length.
    ///
    /// The buffer stays valid as long as no other `Writer` method is called
    /// and `init`/`deinit` aren't called on the ringbuf.
    pub fn push_bufs(&mut self) -> [(*mut u8, usize); 2] {
        // Ordering: as per push_buf()
        let mut start = self.0.start.load(Ordering::Acquire);
        let buf = self.0.buf.load(Ordering::Relaxed);
        let len = self.0.len.load(Ordering::Relaxed);
        let mut end = self.0.end.load(Ordering::Relaxed);

        let empty = start == end;

        if start >= len {
            start -= len
        }
        if end >= len {
            end -= len
        }

        if start == end && !empty {
            // full
            return [(buf, 0), (buf, 0)];
        }
        let n0 = if start > end { start - end } else { len - end };
        let n1 = if start <= end { start } else { 0 };

        trace!("  ringbuf: push_bufs [{:?}..{:?}, {:?}..{:?}]", end, end + n0, 0, n1);
        [(unsafe { buf.add(end) }, n0), (buf, n1)]
    }

    /// Mark n bytes as written and advance the write index.
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
    /// Returns true if popped successfully.
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
    /// Equivalent to [`Self::pop_buf`] but returns a slice.
    pub fn pop_slice(&mut self) -> &mut [u8] {
        let (data, len) = self.pop_buf();
        unsafe { slice::from_raw_parts_mut(data, len) }
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
        let mut end = self.0.end.load(Ordering::Acquire);
        let buf = self.0.buf.load(Ordering::Relaxed);
        let len = self.0.len.load(Ordering::Relaxed);
        let mut start = self.0.start.load(Ordering::Relaxed);

        if start == end {
            return (buf, 0);
        }

        if start >= len {
            start -= len
        }
        if end >= len {
            end -= len
        }

        let n = if end > start { end - start } else { len - start };

        trace!("  ringbuf: pop_buf {:?}..{:?}", start, start + n);
        (unsafe { buf.add(start) }, n)
    }

    /// Mark n bytes as read and allow advance the read index.
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
                assert_eq!(4, buf.len());
                buf[0] = 1;
                buf[1] = 2;
                buf[2] = 3;
                buf[3] = 4;
                4
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
                assert_eq!(4, buf.len());
                assert_eq!(1, buf[0]);
                1
            });

            assert_eq!(rb.is_empty(), false);
            assert_eq!(rb.is_full(), false);

            rb.reader().pop(|buf| {
                assert_eq!(3, buf.len());
                0
            });

            assert_eq!(rb.is_empty(), false);
            assert_eq!(rb.is_full(), false);

            rb.reader().pop(|buf| {
                assert_eq!(3, buf.len());
                assert_eq!(2, buf[0]);
                assert_eq!(3, buf[1]);
                2
            });
            rb.reader().pop(|buf| {
                assert_eq!(1, buf.len());
                assert_eq!(4, buf[0]);
                1
            });

            assert_eq!(rb.is_empty(), true);
            assert_eq!(rb.is_full(), false);

            rb.reader().pop(|buf| {
                assert_eq!(0, buf.len());
                0
            });

            rb.writer().push(|buf| {
                assert_eq!(4, buf.len());
                buf[0] = 10;
                1
            });

            rb.writer().push(|buf| {
                assert_eq!(3, buf.len());
                buf[0] = 11;
                buf[1] = 12;
                2
            });

            assert_eq!(rb.is_empty(), false);
            assert_eq!(rb.is_full(), false);

            rb.writer().push(|buf| {
                assert_eq!(1, buf.len());
                buf[0] = 13;
                1
            });

            assert_eq!(rb.is_empty(), false);
            assert_eq!(rb.is_full(), true);
        }
    }

    #[test]
    fn zero_len() {
        let mut b = [0; 0];

        let rb = RingBuffer::new();
        unsafe {
            rb.init(b.as_mut_ptr(), b.len());

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

    #[test]
    fn push_slices() {
        let mut b = [0; 4];
        let rb = RingBuffer::new();
        unsafe {
            rb.init(b.as_mut_ptr(), 4);

            /* push 3 -> [1 2 3 x] */
            let mut w = rb.writer();
            let ps = w.push_slices();
            assert_eq!(4, ps[0].len());
            assert_eq!(0, ps[1].len());
            ps[0][0] = 1;
            ps[0][1] = 2;
            ps[0][2] = 3;
            w.push_done(3);
            drop(w);

            /* pop 2 -> [x x 3 x] */
            rb.reader().pop(|buf| {
                assert_eq!(3, buf.len());
                assert_eq!(1, buf[0]);
                assert_eq!(2, buf[1]);
                assert_eq!(3, buf[2]);
                2
            });

            /* push 3 -> [5 6 3 4] */
            let mut w = rb.writer();
            let ps = w.push_slices();
            assert_eq!(1, ps[0].len());
            assert_eq!(2, ps[1].len());
            ps[0][0] = 4;
            ps[1][0] = 5;
            ps[1][1] = 6;
            w.push_done(3);
            drop(w);

            /* buf is now full */
            let mut w = rb.writer();
            let ps = w.push_slices();
            assert_eq!(0, ps[0].len());
            assert_eq!(0, ps[1].len());

            /* pop 2 -> [5 6 x x] */
            rb.reader().pop(|buf| {
                assert_eq!(2, buf.len());
                assert_eq!(3, buf[0]);
                assert_eq!(4, buf[1]);
                2
            });

            /* should now have one push slice again */
            let mut w = rb.writer();
            let ps = w.push_slices();
            assert_eq!(2, ps[0].len());
            assert_eq!(0, ps[1].len());
            drop(w);

            /* pop 2 -> [x x x x] */
            rb.reader().pop(|buf| {
                assert_eq!(2, buf.len());
                assert_eq!(5, buf[0]);
                assert_eq!(6, buf[1]);
                2
            });

            /* should now have two push slices */
            let mut w = rb.writer();
            let ps = w.push_slices();
            assert_eq!(2, ps[0].len());
            assert_eq!(2, ps[1].len());
            drop(w);

            /* make sure we exercise all wrap around cases properly */
            for _ in 0..10 {
                /* should be empty, push 1 */
                let mut w = rb.writer();
                let ps = w.push_slices();
                assert_eq!(4, ps[0].len() + ps[1].len());
                w.push_done(1);
                drop(w);

                /* should have 1 element */
                let mut w = rb.writer();
                let ps = w.push_slices();
                assert_eq!(3, ps[0].len() + ps[1].len());
                drop(w);

                /* pop 1 */
                rb.reader().pop(|buf| {
                    assert_eq!(1, buf.len());
                    1
                });
            }
        }
    }
}

use core::slice;
use core::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};

pub struct RingBuffer {
    buf: AtomicPtr<u8>,
    len: AtomicUsize,
    start: AtomicUsize,
    end: AtomicUsize,
}

pub struct Reader<'a>(&'a RingBuffer);
pub struct Writer<'a>(&'a RingBuffer);

impl RingBuffer {
    pub const fn new() -> Self {
        Self {
            buf: AtomicPtr::new(core::ptr::null_mut()),
            len: AtomicUsize::new(0),
            start: AtomicUsize::new(0),
            end: AtomicUsize::new(0),
        }
    }

    /// # Safety
    /// - The buffer (`buf .. buf+len`) must be valid memory until `deinit` is called.
    /// - Must not be called concurrently with any other methods.
    pub unsafe fn init(&self, buf: *mut u8, len: usize) {
        self.buf.store(buf, Ordering::Relaxed);
        self.len.store(len, Ordering::Relaxed);
        self.start.store(0, Ordering::Relaxed);
        self.end.store(0, Ordering::Relaxed);
    }

    pub unsafe fn deinit(&self) {
        self.len.store(0, Ordering::Relaxed);
    }

    pub unsafe fn reader(&self) -> Reader<'_> {
        Reader(self)
    }

    pub unsafe fn writer(&self) -> Writer<'_> {
        Writer(self)
    }

    pub fn is_full(&self) -> bool {
        let start = self.start.load(Ordering::Relaxed);
        let end = self.end.load(Ordering::Relaxed);

        self.wrap(end + 1) == start
    }

    pub fn is_empty(&self) -> bool {
        let start = self.start.load(Ordering::Relaxed);
        let end = self.end.load(Ordering::Relaxed);

        start == end
    }

    pub fn wrap(&self, n: usize) -> usize {
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
    pub fn push(&self, f: impl FnOnce(&mut [u8]) -> usize) -> usize {
        let (p, n) = self.push_buf();
        let buf = unsafe { slice::from_raw_parts_mut(p, n) };
        let n = f(buf);
        self.push_done(n);
        n
    }

    pub fn push_one(&self, val: u8) -> bool {
        let n = self.push(|f| match f {
            [] => 0,
            [x, ..] => {
                *x = val;
                1
            }
        });
        n != 0
    }

    pub fn push_buf(&self) -> (*mut u8, usize) {
        let start = self.0.start.load(Ordering::Acquire);
        let buf = self.0.buf.load(Ordering::Relaxed);
        let len = self.0.len.load(Ordering::Relaxed);
        let end = self.0.end.load(Ordering::Relaxed);

        let n = if start <= end {
            len - end - (start == 0) as usize
        } else {
            start - end - 1
        };

        trace!("  ringbuf: push_buf {:?}..{:?}", end, end + n);
        (unsafe { buf.add(end) }, n)
    }

    pub fn push_done(&self, n: usize) {
        trace!("  ringbuf: push {:?}", n);
        let end = self.0.end.load(Ordering::Relaxed);
        self.0.end.store(self.0.wrap(end + n), Ordering::Release);
    }
}

impl<'a> Reader<'a> {
    pub fn pop(&self, f: impl FnOnce(&[u8]) -> usize) -> usize {
        let (p, n) = self.pop_buf();
        let buf = unsafe { slice::from_raw_parts(p, n) };
        let n = f(buf);
        self.pop_done(n);
        n
    }

    pub fn pop_one(&self) -> Option<u8> {
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

    pub fn pop_buf(&self) -> (*mut u8, usize) {
        let end = self.0.end.load(Ordering::Acquire);
        let buf = self.0.buf.load(Ordering::Relaxed);
        let len = self.0.len.load(Ordering::Relaxed);
        let start = self.0.start.load(Ordering::Relaxed);

        let n = if end < start { len - start } else { end - start };

        trace!("  ringbuf: pop_buf {:?}..{:?}", start, start + n);
        (unsafe { buf.add(start) }, n)
    }

    pub fn pop_done(&self, n: usize) {
        trace!("  ringbuf: pop {:?}", n);

        let start = self.0.start.load(Ordering::Relaxed);
        self.0.start.store(self.0.wrap(start + n), Ordering::Release);
    }
}

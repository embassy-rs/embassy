use core::ops::Range;

pub struct RingBuffer<const N: usize> {
    start: usize,
    end: usize,
    full: bool,
}

impl<const N: usize> RingBuffer<N> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            full: false,
        }
    }

    pub fn push_buf(&mut self) -> Range<usize> {
        if self.is_full() {
            trace!("  ringbuf: push_buf full");
            return 0..0;
        }

        let n = if self.start <= self.end {
            N - self.end
        } else {
            self.start - self.end
        };

        trace!("  ringbuf: push_buf {:?}..{:?}", self.end, self.end + n);
        self.end..self.end + n
    }

    pub fn push(&mut self, n: usize) {
        trace!("  ringbuf: push {:?}", n);
        if n == 0 {
            return;
        }

        self.end = self.wrap(self.end + n);
        self.full = self.start == self.end;
    }

    pub fn pop_buf(&mut self) -> Range<usize> {
        if self.is_empty() {
            trace!("  ringbuf: pop_buf empty");
            return 0..0;
        }

        let n = if self.end <= self.start {
            N - self.start
        } else {
            self.end - self.start
        };

        trace!("  ringbuf: pop_buf {:?}..{:?}", self.start, self.start + n);
        self.start..self.start + n
    }

    pub fn pop(&mut self, n: usize) {
        trace!("  ringbuf: pop {:?}", n);
        if n == 0 {
            return;
        }

        self.start = self.wrap(self.start + n);
        self.full = false;
    }

    pub fn is_full(&self) -> bool {
        self.full
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end && !self.full
    }

    #[allow(unused)]
    pub fn len(&self) -> usize {
        if self.is_empty() {
            0
        } else if self.start < self.end {
            self.end - self.start
        } else {
            N + self.end - self.start
        }
    }

    pub fn clear(&mut self) {
        self.start = 0;
        self.end = 0;
        self.full = false;
    }

    fn wrap(&self, n: usize) -> usize {
        assert!(n <= N);
        if n == N {
            0
        } else {
            n
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop() {
        let mut rb: RingBuffer<4> = RingBuffer::new();
        let buf = rb.push_buf();
        assert_eq!(0..4, buf);
        rb.push(4);

        let buf = rb.pop_buf();
        assert_eq!(0..4, buf);
        rb.pop(1);

        let buf = rb.pop_buf();
        assert_eq!(1..4, buf);
        rb.pop(1);

        let buf = rb.pop_buf();
        assert_eq!(2..4, buf);
        rb.pop(1);

        let buf = rb.pop_buf();
        assert_eq!(3..4, buf);
        rb.pop(1);

        let buf = rb.pop_buf();
        assert_eq!(0..0, buf);

        let buf = rb.push_buf();
        assert_eq!(0..4, buf);
    }
}

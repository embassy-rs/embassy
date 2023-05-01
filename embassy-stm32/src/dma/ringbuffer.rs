#![cfg_attr(gpdma, allow(unused))]

use core::ops::Range;
use core::sync::atomic::{compiler_fence, Ordering};

use super::word::Word;

/// A "read-only" ring-buffer to be used together with the DMA controller which
/// writes in a circular way, "uncontrolled" to the buffer.
///
/// A snapshot of the ring buffer state can be attained by setting the `ndtr` field
/// to the current register value. `ndtr` describes the current position of the DMA
/// write.
///
/// # Buffer layout
///
/// ```text
/// Without wraparound:                             With wraparound:
///
///  + buf                      +--- NDTR ---+       + buf        +---------- NDTR ----------+
///  |                          |            |       |            |                          |
///  v                          v            v       v            v                          v
/// +-----------------------------------------+     +-----------------------------------------+
/// |oooooooooooXXXXXXXXXXXXXXXXoooooooooooooo|     |XXXXXXXXXXXXXooooooooooooXXXXXXXXXXXXXXXX|
/// +-----------------------------------------+     +-----------------------------------------+
///  ^          ^               ^                    ^            ^           ^
///  |          |               |                    |            |           |
///  +- first --+               |                    +- end ------+           |
///  |                          |                    |                        |
///  +- end --------------------+                    +- first ----------------+
/// ```
pub struct DmaRingBuffer<'a, W: Word> {
    pub(crate) dma_buf: &'a mut [W],
    first: usize,
    pub ndtr: usize,
}

#[derive(Debug, PartialEq)]
pub struct OverrunError;

pub trait DmaCtrl {
    /// Get the NDTR register value, i.e. the space left in the underlying
    /// buffer until the dma writer wraps.
    fn ndtr(&self) -> usize;

    /// Get the transfer completed counter.
    /// This counter is incremented by the dma controller when NDTR is reloaded,
    /// i.e. when the writing wraps.
    fn get_complete_count(&self) -> usize;

    /// Reset the transfer completed counter to 0 and return the value just prior to the reset.
    fn reset_complete_count(&mut self) -> usize;
}

impl<'a, W: Word> DmaRingBuffer<'a, W> {
    pub fn new(dma_buf: &'a mut [W]) -> Self {
        let ndtr = dma_buf.len();
        Self {
            dma_buf,
            first: 0,
            ndtr,
        }
    }

    /// Reset the ring buffer to its initial state
    pub fn clear(&mut self, dma: &mut impl DmaCtrl) {
        self.first = 0;
        self.ndtr = self.dma_buf.len();
        dma.reset_complete_count();
    }

    /// The buffer end position
    fn end(&self) -> usize {
        self.dma_buf.len() - self.ndtr
    }

    /// Returns whether the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.first == self.end()
    }

    /// The current number of bytes in the buffer
    /// This may change at any time if dma is currently active
    pub fn len(&self) -> usize {
        // Read out a stable end (the dma periheral can change it at anytime)
        let end = self.end();
        if self.first <= end {
            // No wrap
            end - self.first
        } else {
            self.dma_buf.len() - self.first + end
        }
    }

    /// Read bytes from the ring buffer
    /// OverrunError is returned if the portion to be read was overwritten by the DMA controller.
    pub fn read(&mut self, dma: &mut impl DmaCtrl, buf: &mut [W]) -> Result<usize, OverrunError> {
        let end = self.end();

        compiler_fence(Ordering::SeqCst);

        if self.first == end {
            // The buffer is currently empty

            if dma.get_complete_count() > 0 {
                // The DMA has written such that the ring buffer wraps at least once
                self.ndtr = dma.ndtr();
                if self.end() > self.first || dma.get_complete_count() > 1 {
                    return Err(OverrunError);
                }
            }

            Ok(0)
        } else if self.first < end {
            // The available, unread portion in the ring buffer DOES NOT wrap

            if dma.get_complete_count() > 1 {
                return Err(OverrunError);
            }

            // Copy out the bytes from the dma buffer
            let len = self.copy_to(buf, self.first..end);

            compiler_fence(Ordering::SeqCst);

            match dma.get_complete_count() {
                0 => {
                    // The DMA writer has not wrapped before nor after the copy
                }
                1 => {
                    // The DMA writer has written such that the ring buffer now wraps
                    self.ndtr = dma.ndtr();
                    if self.end() > self.first || dma.get_complete_count() > 1 {
                        // The bytes that we have copied out have overflowed
                        // as the writer has now both wrapped and is currently writing
                        // within the region that we have just copied out
                        return Err(OverrunError);
                    }
                }
                _ => {
                    return Err(OverrunError);
                }
            }

            self.first = (self.first + len) % self.dma_buf.len();
            Ok(len)
        } else {
            // The available, unread portion in the ring buffer DOES wrap
            // The DMA writer has wrapped since we last read and is currently
            // writing (or the next byte added will be) in the beginning of the ring buffer.

            let complete_count = dma.get_complete_count();
            if complete_count > 1 {
                return Err(OverrunError);
            }

            // If the unread portion wraps then the writer must also have wrapped
            assert!(complete_count == 1);

            if self.first + buf.len() < self.dma_buf.len() {
                // The provided read buffer is not large enough to include all bytes from the tail of the dma buffer.

                // Copy out from the dma buffer
                let len = self.copy_to(buf, self.first..self.dma_buf.len());

                compiler_fence(Ordering::SeqCst);

                // We have now copied out the data from dma_buf
                // Make sure that the just read part was not overwritten during the copy
                self.ndtr = dma.ndtr();
                if self.end() > self.first || dma.get_complete_count() > 1 {
                    // The writer has entered the data that we have just read since we read out `end` in the beginning and until now.
                    return Err(OverrunError);
                }

                self.first = (self.first + len) % self.dma_buf.len();
                Ok(len)
            } else {
                // The provided read buffer is large enough to include all bytes from the tail of the dma buffer,
                // so the next read will not have any unread tail bytes in the ring buffer.

                // Copy out from the dma buffer
                let tail = self.copy_to(buf, self.first..self.dma_buf.len());
                let head = self.copy_to(&mut buf[tail..], 0..end);

                compiler_fence(Ordering::SeqCst);

                // We have now copied out the data from dma_buf
                // Reset complete counter and make sure that the just read part was not overwritten during the copy
                self.ndtr = dma.ndtr();
                let complete_count = dma.reset_complete_count();
                if self.end() > self.first || complete_count > 1 {
                    return Err(OverrunError);
                }

                self.first = head;
                Ok(tail + head)
            }
        }
    }

    /// Copy from the dma buffer at `data_range` into `buf`
    fn copy_to(&mut self, buf: &mut [W], data_range: Range<usize>) -> usize {
        // Limit the number of bytes that can be copied
        let length = usize::min(data_range.len(), buf.len());

        // Copy from dma buffer into read buffer
        // We need to do it like this instead of a simple copy_from_slice() because
        // reading from a part of memory that may be simultaneously written to is unsafe
        unsafe {
            let dma_buf = self.dma_buf.as_ptr();

            for i in 0..length {
                buf[i] = core::ptr::read_volatile(dma_buf.offset((data_range.start + i) as isize));
            }
        }

        length
    }
}

#[cfg(test)]
mod tests {
    use core::array;
    use core::cell::RefCell;

    use super::*;

    struct TestCtrl {
        next_ndtr: RefCell<Option<usize>>,
        complete_count: usize,
    }

    impl TestCtrl {
        pub const fn new() -> Self {
            Self {
                next_ndtr: RefCell::new(None),
                complete_count: 0,
            }
        }

        pub fn set_next_ndtr(&mut self, ndtr: usize) {
            self.next_ndtr.borrow_mut().replace(ndtr);
        }
    }

    impl DmaCtrl for TestCtrl {
        fn ndtr(&self) -> usize {
            self.next_ndtr.borrow_mut().unwrap()
        }

        fn get_complete_count(&self) -> usize {
            self.complete_count
        }

        fn reset_complete_count(&mut self) -> usize {
            let old = self.complete_count;
            self.complete_count = 0;
            old
        }
    }

    #[test]
    fn empty() {
        let mut dma_buf = [0u8; 16];
        let ringbuf = DmaRingBuffer::new(&mut dma_buf);

        assert!(ringbuf.is_empty());
        assert_eq!(0, ringbuf.len());
    }

    #[test]
    fn can_read() {
        let mut dma_buf: [u8; 16] = array::from_fn(|idx| idx as u8); // 0, 1, ..., 15
        let mut ctrl = TestCtrl::new();
        let mut ringbuf = DmaRingBuffer::new(&mut dma_buf);
        ringbuf.ndtr = 6;

        assert!(!ringbuf.is_empty());
        assert_eq!(10, ringbuf.len());

        let mut buf = [0; 2];
        assert_eq!(2, ringbuf.read(&mut ctrl, &mut buf).unwrap());
        assert_eq!([0, 1], buf);
        assert_eq!(8, ringbuf.len());

        let mut buf = [0; 2];
        assert_eq!(2, ringbuf.read(&mut ctrl, &mut buf).unwrap());
        assert_eq!([2, 3], buf);
        assert_eq!(6, ringbuf.len());

        let mut buf = [0; 8];
        assert_eq!(6, ringbuf.read(&mut ctrl, &mut buf).unwrap());
        assert_eq!([4, 5, 6, 7, 8, 9], buf[..6]);
        assert_eq!(0, ringbuf.len());

        let mut buf = [0; 2];
        assert_eq!(0, ringbuf.read(&mut ctrl, &mut buf).unwrap());
    }

    #[test]
    fn can_read_with_wrap() {
        let mut dma_buf: [u8; 16] = array::from_fn(|idx| idx as u8); // 0, 1, ..., 15
        let mut ctrl = TestCtrl::new();
        let mut ringbuf = DmaRingBuffer::new(&mut dma_buf);
        ringbuf.first = 12;
        ringbuf.ndtr = 10;

        // The dma controller has written 4 + 6 bytes and has reloaded NDTR
        ctrl.complete_count = 1;
        ctrl.set_next_ndtr(10);

        assert!(!ringbuf.is_empty());
        assert_eq!(6 + 4, ringbuf.len());

        let mut buf = [0; 2];
        assert_eq!(2, ringbuf.read(&mut ctrl, &mut buf).unwrap());
        assert_eq!([12, 13], buf);
        assert_eq!(6 + 2, ringbuf.len());

        let mut buf = [0; 4];
        assert_eq!(4, ringbuf.read(&mut ctrl, &mut buf).unwrap());
        assert_eq!([14, 15, 0, 1], buf);
        assert_eq!(4, ringbuf.len());
    }

    #[test]
    fn can_read_when_dma_writer_is_wrapped_and_read_does_not_wrap() {
        let mut dma_buf: [u8; 16] = array::from_fn(|idx| idx as u8); // 0, 1, ..., 15
        let mut ctrl = TestCtrl::new();
        let mut ringbuf = DmaRingBuffer::new(&mut dma_buf);
        ringbuf.first = 2;
        ringbuf.ndtr = 6;

        // The dma controller has written 6 + 2 bytes and has reloaded NDTR
        ctrl.complete_count = 1;
        ctrl.set_next_ndtr(14);

        let mut buf = [0; 2];
        assert_eq!(2, ringbuf.read(&mut ctrl, &mut buf).unwrap());
        assert_eq!([2, 3], buf);

        assert_eq!(1, ctrl.complete_count); // The interrupt flag IS NOT cleared
    }

    #[test]
    fn can_read_when_dma_writer_is_wrapped_and_read_wraps() {
        let mut dma_buf: [u8; 16] = array::from_fn(|idx| idx as u8); // 0, 1, ..., 15
        let mut ctrl = TestCtrl::new();
        let mut ringbuf = DmaRingBuffer::new(&mut dma_buf);
        ringbuf.first = 12;
        ringbuf.ndtr = 10;

        // The dma controller has written 6 + 2 bytes and has reloaded NDTR
        ctrl.complete_count = 1;
        ctrl.set_next_ndtr(14);

        let mut buf = [0; 10];
        assert_eq!(10, ringbuf.read(&mut ctrl, &mut buf).unwrap());
        assert_eq!([12, 13, 14, 15, 0, 1, 2, 3, 4, 5], buf);

        assert_eq!(0, ctrl.complete_count); // The interrupt flag IS cleared
    }

    #[test]
    fn cannot_read_when_dma_writer_wraps_with_same_ndtr() {
        let mut dma_buf = [0u8; 16];
        let mut ctrl = TestCtrl::new();
        let mut ringbuf = DmaRingBuffer::new(&mut dma_buf);
        ringbuf.first = 6;
        ringbuf.ndtr = 10;
        ctrl.set_next_ndtr(9);

        assert!(ringbuf.is_empty()); // The ring buffer thinks that it is empty

        // The dma controller has written exactly 16 bytes
        ctrl.complete_count = 1;

        let mut buf = [0; 2];
        assert_eq!(Err(OverrunError), ringbuf.read(&mut ctrl, &mut buf));

        assert_eq!(1, ctrl.complete_count); // The complete counter is not reset
    }

    #[test]
    fn cannot_read_when_dma_writer_overwrites_during_not_wrapping_read() {
        let mut dma_buf: [u8; 16] = array::from_fn(|idx| idx as u8); // 0, 1, ..., 15
        let mut ctrl = TestCtrl::new();
        let mut ringbuf = DmaRingBuffer::new(&mut dma_buf);
        ringbuf.first = 2;
        ringbuf.ndtr = 6;

        // The dma controller has written 6 + 3 bytes and has reloaded NDTR
        ctrl.complete_count = 1;
        ctrl.set_next_ndtr(13);

        let mut buf = [0; 2];
        assert_eq!(Err(OverrunError), ringbuf.read(&mut ctrl, &mut buf));

        assert_eq!(1, ctrl.complete_count); // The complete counter is not reset
    }

    #[test]
    fn cannot_read_when_dma_writer_overwrites_during_wrapping_read() {
        let mut dma_buf: [u8; 16] = array::from_fn(|idx| idx as u8); // 0, 1, ..., 15
        let mut ctrl = TestCtrl::new();
        let mut ringbuf = DmaRingBuffer::new(&mut dma_buf);
        ringbuf.first = 12;
        ringbuf.ndtr = 10;

        // The dma controller has written 6 + 13 bytes and has reloaded NDTR
        ctrl.complete_count = 1;
        ctrl.set_next_ndtr(3);

        let mut buf = [0; 2];
        assert_eq!(Err(OverrunError), ringbuf.read(&mut ctrl, &mut buf));

        assert_eq!(1, ctrl.complete_count); // The complete counter is not reset
    }
}

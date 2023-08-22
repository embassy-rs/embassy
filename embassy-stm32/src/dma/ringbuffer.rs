#![cfg_attr(gpdma, allow(unused))]

use core::future::poll_fn;
use core::ops::Range;
use core::sync::atomic::{compiler_fence, Ordering};
use core::task::{Poll, Waker};

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
///  +- start --+               |                    +- end ------+           |
///  |                          |                    |                        |
///  +- end --------------------+                    +- start ----------------+
/// ```
pub struct ReadableDmaRingBuffer<'a, W: Word> {
    pub(crate) dma_buf: &'a mut [W],
    start: usize,
}

#[derive(Debug, PartialEq)]
pub struct OverrunError;

pub trait DmaCtrl {
    /// Get the NDTR register value, i.e. the space left in the underlying
    /// buffer until the dma writer wraps.
    fn get_remaining_transfers(&self) -> usize;

    /// Get the transfer completed counter.
    /// This counter is incremented by the dma controller when NDTR is reloaded,
    /// i.e. when the writing wraps.
    fn get_complete_count(&self) -> usize;

    /// Reset the transfer completed counter to 0 and return the value just prior to the reset.
    fn reset_complete_count(&mut self) -> usize;

    /// Set the waker for a running poll_fn
    fn set_waker(&mut self, waker: &Waker);
}

impl<'a, W: Word> ReadableDmaRingBuffer<'a, W> {
    pub fn new(dma_buf: &'a mut [W]) -> Self {
        Self { dma_buf, start: 0 }
    }

    /// Reset the ring buffer to its initial state
    pub fn clear(&mut self, dma: &mut impl DmaCtrl) {
        self.start = 0;
        dma.reset_complete_count();
    }

    /// The capacity of the ringbuffer
    pub const fn cap(&self) -> usize {
        self.dma_buf.len()
    }

    /// The current position of the ringbuffer
    fn pos(&self, dma: &mut impl DmaCtrl) -> usize {
        self.cap() - dma.get_remaining_transfers()
    }

    /// Read an exact number of elements from the ringbuffer.
    ///
    /// Returns the remaining number of elements available for immediate reading.
    /// OverrunError is returned if the portion to be read was overwritten by the DMA controller.
    ///
    /// Async/Wake Behavior:
    /// The underlying DMA peripheral only can wake us when its buffer pointer has reached the halfway point,
    /// and when it wraps around. This means that when called with a buffer of length 'M', when this
    /// ring buffer was created with a buffer of size 'N':
    /// - If M equals N/2 or N/2 divides evenly into M, this function will return every N/2 elements read on the DMA source.
    /// - Otherwise, this function may need up to N/2 extra elements to arrive before returning.
    pub async fn read_exact(&mut self, dma: &mut impl DmaCtrl, buffer: &mut [W]) -> Result<usize, OverrunError> {
        let mut read_data = 0;
        let buffer_len = buffer.len();

        poll_fn(|cx| {
            dma.set_waker(cx.waker());

            compiler_fence(Ordering::SeqCst);

            match self.read(dma, &mut buffer[read_data..buffer_len]) {
                Ok((len, remaining)) => {
                    read_data += len;
                    if read_data == buffer_len {
                        Poll::Ready(Ok(remaining))
                    } else {
                        Poll::Pending
                    }
                }
                Err(e) => Poll::Ready(Err(e)),
            }
        })
        .await
    }

    /// Read elements from the ring buffer
    /// Return a tuple of the length read and the length remaining in the buffer
    /// If not all of the elements were read, then there will be some elements in the buffer remaining
    /// The length remaining is the capacity, ring_buf.len(), less the elements remaining after the read
    /// OverrunError is returned if the portion to be read was overwritten by the DMA controller.
    pub fn read(&mut self, dma: &mut impl DmaCtrl, buf: &mut [W]) -> Result<(usize, usize), OverrunError> {
        /*
            This algorithm is optimistic: we assume we haven't overrun more than a full buffer and then check
            after we've done our work to see we have. This is because on stm32, an interrupt is not guaranteed
            to fire in the same clock cycle that a register is read, so checking get_complete_count early does
            not yield relevant information.

            Therefore, the only variable we really need to know is ndtr. If the dma has overrun by more than a full
            buffer, we will do a bit more work than we have to, but algorithms should not be optimized for error
            conditions.

            After we've done our work, we confirm that we haven't overrun more than a full buffer, and also that
            the dma has not overrun within the data we could have copied. We check the data we could have copied
            rather than the data we actually copied because it costs nothing and confirms an error condition
            earlier.
        */
        let end = self.pos(dma);
        if self.start == end && dma.get_complete_count() == 0 {
            // No elements are available in the buffer
            Ok((0, self.cap()))
        } else if self.start < end {
            // The available, unread portion in the ring buffer DOES NOT wrap
            // Copy out the elements from the dma buffer
            let len = self.copy_to(buf, self.start..end);

            compiler_fence(Ordering::SeqCst);

            /*
                first, check if the dma has wrapped at all if it's after end
                or more than once if it's before start

                this is in a critical section to try to reduce mushy behavior.
                it's not ideal but it's the best we can do

                then, get the current position of of the dma write and check
                if it's inside data we could have copied
            */
            let (pos, complete_count) = critical_section::with(|_| (self.pos(dma), dma.get_complete_count()));
            if (pos >= self.start && pos < end) || (complete_count > 0 && pos >= end) || complete_count > 1 {
                Err(OverrunError)
            } else {
                self.start = (self.start + len) % self.cap();

                Ok((len, self.cap() - self.start))
            }
        } else if self.start + buf.len() < self.cap() {
            // The available, unread portion in the ring buffer DOES wrap
            // The DMA writer has wrapped since we last read and is currently
            // writing (or the next byte added will be) in the beginning of the ring buffer.

            // The provided read buffer is not large enough to include all elements from the tail of the dma buffer.

            // Copy out from the dma buffer
            let len = self.copy_to(buf, self.start..self.cap());

            compiler_fence(Ordering::SeqCst);

            /*
                first, check if the dma has wrapped around more than once

                then, get the current position of of the dma write and check
                if it's inside data we could have copied
            */
            let pos = self.pos(dma);
            if pos > self.start || pos < end || dma.get_complete_count() > 1 {
                Err(OverrunError)
            } else {
                self.start = (self.start + len) % self.cap();

                Ok((len, self.start + end))
            }
        } else {
            // The available, unread portion in the ring buffer DOES wrap
            // The DMA writer has wrapped since we last read and is currently
            // writing (or the next byte added will be) in the beginning of the ring buffer.

            // The provided read buffer is large enough to include all elements from the tail of the dma buffer,
            // so the next read will not have any unread tail elements in the ring buffer.

            // Copy out from the dma buffer
            let tail = self.copy_to(buf, self.start..self.cap());
            let head = self.copy_to(&mut buf[tail..], 0..end);

            compiler_fence(Ordering::SeqCst);

            /*
                first, check if the dma has wrapped around more than once

                then, get the current position of of the dma write and check
                if it's inside data we could have copied
            */
            let pos = self.pos(dma);
            if pos > self.start || pos < end || dma.reset_complete_count() > 1 {
                Err(OverrunError)
            } else {
                self.start = head;
                Ok((tail + head, self.cap() - self.start))
            }
        }
    }
    /// Copy from the dma buffer at `data_range` into `buf`
    fn copy_to(&mut self, buf: &mut [W], data_range: Range<usize>) -> usize {
        // Limit the number of elements that can be copied
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

pub struct WritableDmaRingBuffer<'a, W: Word> {
    pub(crate) dma_buf: &'a mut [W],
    end: usize,
}

impl<'a, W: Word> WritableDmaRingBuffer<'a, W> {
    pub fn new(dma_buf: &'a mut [W]) -> Self {
        Self { dma_buf, end: 0 }
    }

    /// Reset the ring buffer to its initial state
    pub fn clear(&mut self, dma: &mut impl DmaCtrl) {
        self.end = 0;
        dma.reset_complete_count();
    }

    /// The capacity of the ringbuffer
    pub const fn cap(&self) -> usize {
        self.dma_buf.len()
    }

    /// The current position of the ringbuffer
    fn pos(&self, dma: &mut impl DmaCtrl) -> usize {
        self.cap() - dma.get_remaining_transfers()
    }

    /// Write an exact number of elements to the ringbuffer.
    pub async fn write_exact(&mut self, dma: &mut impl DmaCtrl, buffer: &[W]) -> Result<usize, OverrunError> {
        let mut written_data = 0;
        let buffer_len = buffer.len();

        poll_fn(|cx| {
            dma.set_waker(cx.waker());

            compiler_fence(Ordering::SeqCst);

            match self.write(dma, &buffer[written_data..buffer_len]) {
                Ok((len, remaining)) => {
                    written_data += len;
                    if written_data == buffer_len {
                        Poll::Ready(Ok(remaining))
                    } else {
                        Poll::Pending
                    }
                }
                Err(e) => Poll::Ready(Err(e)),
            }
        })
        .await
    }

    /// Write elements from the ring buffer
    /// Return a tuple of the length written and the capacity remaining to be written in the buffer
    pub fn write(&mut self, dma: &mut impl DmaCtrl, buf: &[W]) -> Result<(usize, usize), OverrunError> {
        let start = self.pos(dma);
        if start > self.end {
            // The occupied portion in the ring buffer DOES wrap
            let len = self.copy_from(buf, self.end..start);

            compiler_fence(Ordering::SeqCst);

            // Confirm that the DMA is not inside data we could have written
            let (pos, complete_count) = critical_section::with(|_| (self.pos(dma), dma.get_complete_count()));
            if (pos >= self.end && pos < start) || (complete_count > 0 && pos >= start) || complete_count > 1 {
                Err(OverrunError)
            } else {
                self.end = (self.end + len) % self.cap();

                Ok((len, self.cap() - (start - self.end)))
            }
        } else if start == self.end && dma.get_complete_count() == 0 {
            Ok((0, 0))
        } else if start <= self.end && self.end + buf.len() < self.cap() {
            // The occupied portion in the ring buffer DOES NOT wrap
            // and copying elements into the buffer WILL NOT cause it to

            // Copy into the dma buffer
            let len = self.copy_from(buf, self.end..self.cap());

            compiler_fence(Ordering::SeqCst);

            // Confirm that the DMA is not inside data we could have written
            let pos = self.pos(dma);
            if pos > self.end || pos < start || dma.get_complete_count() > 1 {
                Err(OverrunError)
            } else {
                self.end = (self.end + len) % self.cap();

                Ok((len, self.cap() - (self.end - start)))
            }
        } else {
            // The occupied portion in the ring buffer DOES NOT wrap
            // and copying elements into the buffer WILL cause it to

            let tail = self.copy_from(buf, self.end..self.cap());
            let head = self.copy_from(&buf[tail..], 0..start);

            compiler_fence(Ordering::SeqCst);

            // Confirm that the DMA is not inside data we could have written
            let pos = self.pos(dma);
            if pos > self.end || pos < start || dma.reset_complete_count() > 1 {
                Err(OverrunError)
            } else {
                self.end = head;

                Ok((tail + head, self.cap() - (start - self.end)))
            }
        }
    }
    /// Copy into the dma buffer at `data_range` from `buf`
    fn copy_from(&mut self, buf: &[W], data_range: Range<usize>) -> usize {
        // Limit the number of elements that can be copied
        let length = usize::min(data_range.len(), buf.len());

        // Copy into dma buffer from read buffer
        // We need to do it like this instead of a simple copy_from_slice() because
        // reading from a part of memory that may be simultaneously written to is unsafe
        unsafe {
            let dma_buf = self.dma_buf.as_mut_ptr();

            for i in 0..length {
                core::ptr::write_volatile(dma_buf.offset((data_range.start + i) as isize), buf[i]);
            }
        }

        length
    }
}
#[cfg(test)]
mod tests {
    use core::array;
    use std::{cell, vec};

    use super::*;

    #[allow(dead_code)]
    #[derive(PartialEq, Debug)]
    enum TestCircularTransferRequest {
        GetCompleteCount(usize),
        ResetCompleteCount(usize),
        PositionRequest(usize),
    }

    struct TestCircularTransfer {
        len: usize,
        requests: cell::RefCell<vec::Vec<TestCircularTransferRequest>>,
    }

    impl DmaCtrl for TestCircularTransfer {
        fn get_remaining_transfers(&self) -> usize {
            match self.requests.borrow_mut().pop().unwrap() {
                TestCircularTransferRequest::PositionRequest(pos) => {
                    let len = self.len;

                    assert!(len >= pos);

                    len - pos
                }
                _ => unreachable!(),
            }
        }

        fn get_complete_count(&self) -> usize {
            match self.requests.borrow_mut().pop().unwrap() {
                TestCircularTransferRequest::GetCompleteCount(complete_count) => complete_count,
                _ => unreachable!(),
            }
        }

        fn reset_complete_count(&mut self) -> usize {
            match self.requests.get_mut().pop().unwrap() {
                TestCircularTransferRequest::ResetCompleteCount(complete_count) => complete_count,
                _ => unreachable!(),
            }
        }

        fn set_waker(&mut self, waker: &Waker) {}
    }

    impl TestCircularTransfer {
        pub fn new(len: usize) -> Self {
            Self {
                requests: cell::RefCell::new(vec![]),
                len,
            }
        }

        pub fn setup(&self, mut requests: vec::Vec<TestCircularTransferRequest>) {
            requests.reverse();
            self.requests.replace(requests);
        }
    }

    #[test]
    fn empty_and_read_not_started() {
        let mut dma_buf = [0u8; 16];
        let ringbuf = ReadableDmaRingBuffer::new(&mut dma_buf);

        assert_eq!(0, ringbuf.start);
    }

    #[test]
    fn can_read() {
        let mut dma = TestCircularTransfer::new(16);

        let mut dma_buf: [u8; 16] = array::from_fn(|idx| idx as u8); // 0, 1, ..., 15
        let mut ringbuf = ReadableDmaRingBuffer::new(&mut dma_buf);

        assert_eq!(0, ringbuf.start);
        assert_eq!(16, ringbuf.cap());

        dma.setup(vec![
            TestCircularTransferRequest::PositionRequest(8),
            TestCircularTransferRequest::PositionRequest(10),
            TestCircularTransferRequest::GetCompleteCount(0),
        ]);
        let mut buf = [0; 2];
        assert_eq!(2, ringbuf.read(&mut dma, &mut buf).unwrap().0);
        assert_eq!([0, 1], buf);
        assert_eq!(2, ringbuf.start);

        dma.setup(vec![
            TestCircularTransferRequest::PositionRequest(10),
            TestCircularTransferRequest::PositionRequest(12),
            TestCircularTransferRequest::GetCompleteCount(0),
        ]);
        let mut buf = [0; 2];
        assert_eq!(2, ringbuf.read(&mut dma, &mut buf).unwrap().0);
        assert_eq!([2, 3], buf);
        assert_eq!(4, ringbuf.start);

        dma.setup(vec![
            TestCircularTransferRequest::PositionRequest(12),
            TestCircularTransferRequest::PositionRequest(14),
            TestCircularTransferRequest::GetCompleteCount(0),
        ]);
        let mut buf = [0; 8];
        assert_eq!(8, ringbuf.read(&mut dma, &mut buf).unwrap().0);
        assert_eq!([4, 5, 6, 7, 8, 9], buf[..6]);
        assert_eq!(12, ringbuf.start);
    }

    #[test]
    fn can_read_with_wrap() {
        let mut dma = TestCircularTransfer::new(16);

        let mut dma_buf: [u8; 16] = array::from_fn(|idx| idx as u8); // 0, 1, ..., 15
        let mut ringbuf = ReadableDmaRingBuffer::new(&mut dma_buf);

        assert_eq!(0, ringbuf.start);
        assert_eq!(16, ringbuf.cap());

        /*
            Read to close to the end of the buffer
        */
        dma.setup(vec![
            TestCircularTransferRequest::PositionRequest(14),
            TestCircularTransferRequest::PositionRequest(16),
            TestCircularTransferRequest::GetCompleteCount(0),
        ]);
        let mut buf = [0; 14];
        assert_eq!(14, ringbuf.read(&mut dma, &mut buf).unwrap().0);
        assert_eq!(14, ringbuf.start);

        /*
            Now, read around the buffer
        */
        dma.setup(vec![
            TestCircularTransferRequest::PositionRequest(6),
            TestCircularTransferRequest::PositionRequest(8),
            TestCircularTransferRequest::ResetCompleteCount(1),
        ]);
        let mut buf = [0; 6];
        assert_eq!(6, ringbuf.read(&mut dma, &mut buf).unwrap().0);
        assert_eq!(4, ringbuf.start);
    }

    #[test]
    fn can_read_when_dma_writer_is_wrapped_and_read_does_not_wrap() {
        let mut dma = TestCircularTransfer::new(16);

        let mut dma_buf: [u8; 16] = array::from_fn(|idx| idx as u8); // 0, 1, ..., 15
        let mut ringbuf = ReadableDmaRingBuffer::new(&mut dma_buf);

        assert_eq!(0, ringbuf.start);
        assert_eq!(16, ringbuf.cap());

        /*
            Read to close to the end of the buffer
        */
        dma.setup(vec![
            TestCircularTransferRequest::PositionRequest(14),
            TestCircularTransferRequest::PositionRequest(16),
            TestCircularTransferRequest::GetCompleteCount(0),
        ]);
        let mut buf = [0; 14];
        assert_eq!(14, ringbuf.read(&mut dma, &mut buf).unwrap().0);
        assert_eq!(14, ringbuf.start);

        /*
            Now, read to the end of the buffer
        */
        dma.setup(vec![
            TestCircularTransferRequest::PositionRequest(6),
            TestCircularTransferRequest::PositionRequest(8),
            TestCircularTransferRequest::ResetCompleteCount(1),
        ]);
        let mut buf = [0; 2];
        assert_eq!(2, ringbuf.read(&mut dma, &mut buf).unwrap().0);
        assert_eq!(0, ringbuf.start);
    }

    #[test]
    fn can_read_when_dma_writer_wraps_once_with_same_ndtr() {
        let mut dma = TestCircularTransfer::new(16);

        let mut dma_buf: [u8; 16] = array::from_fn(|idx| idx as u8); // 0, 1, ..., 15
        let mut ringbuf = ReadableDmaRingBuffer::new(&mut dma_buf);

        assert_eq!(0, ringbuf.start);
        assert_eq!(16, ringbuf.cap());

        /*
            Read to about the middle of the buffer
        */
        dma.setup(vec![
            TestCircularTransferRequest::PositionRequest(6),
            TestCircularTransferRequest::PositionRequest(6),
            TestCircularTransferRequest::GetCompleteCount(0),
        ]);
        let mut buf = [0; 6];
        assert_eq!(6, ringbuf.read(&mut dma, &mut buf).unwrap().0);
        assert_eq!(6, ringbuf.start);

        /*
            Now, wrap the DMA controller around
        */
        dma.setup(vec![
            TestCircularTransferRequest::PositionRequest(6),
            TestCircularTransferRequest::GetCompleteCount(1),
            TestCircularTransferRequest::PositionRequest(6),
            TestCircularTransferRequest::GetCompleteCount(1),
        ]);
        let mut buf = [0; 6];
        assert_eq!(6, ringbuf.read(&mut dma, &mut buf).unwrap().0);
        assert_eq!(12, ringbuf.start);
    }

    #[test]
    fn cannot_read_when_dma_writer_overwrites_during_not_wrapping_read() {
        let mut dma = TestCircularTransfer::new(16);

        let mut dma_buf: [u8; 16] = array::from_fn(|idx| idx as u8); // 0, 1, ..., 15
        let mut ringbuf = ReadableDmaRingBuffer::new(&mut dma_buf);

        assert_eq!(0, ringbuf.start);
        assert_eq!(16, ringbuf.cap());

        /*
            Read a few bytes
        */
        dma.setup(vec![
            TestCircularTransferRequest::PositionRequest(2),
            TestCircularTransferRequest::PositionRequest(2),
            TestCircularTransferRequest::GetCompleteCount(0),
        ]);
        let mut buf = [0; 6];
        assert_eq!(2, ringbuf.read(&mut dma, &mut buf).unwrap().0);
        assert_eq!(2, ringbuf.start);

        /*
            Now, overtake the reader
        */
        dma.setup(vec![
            TestCircularTransferRequest::PositionRequest(4),
            TestCircularTransferRequest::PositionRequest(6),
            TestCircularTransferRequest::GetCompleteCount(1),
        ]);
        let mut buf = [0; 6];
        assert_eq!(OverrunError, ringbuf.read(&mut dma, &mut buf).unwrap_err());
    }

    #[test]
    fn cannot_read_when_dma_writer_overwrites_during_wrapping_read() {
        let mut dma = TestCircularTransfer::new(16);

        let mut dma_buf: [u8; 16] = array::from_fn(|idx| idx as u8); // 0, 1, ..., 15
        let mut ringbuf = ReadableDmaRingBuffer::new(&mut dma_buf);

        assert_eq!(0, ringbuf.start);
        assert_eq!(16, ringbuf.cap());

        /*
            Read to close to the end of the buffer
        */
        dma.setup(vec![
            TestCircularTransferRequest::PositionRequest(14),
            TestCircularTransferRequest::PositionRequest(16),
            TestCircularTransferRequest::GetCompleteCount(0),
        ]);
        let mut buf = [0; 14];
        assert_eq!(14, ringbuf.read(&mut dma, &mut buf).unwrap().0);
        assert_eq!(14, ringbuf.start);

        /*
            Now, overtake the reader
        */
        dma.setup(vec![
            TestCircularTransferRequest::PositionRequest(8),
            TestCircularTransferRequest::PositionRequest(10),
            TestCircularTransferRequest::ResetCompleteCount(2),
        ]);
        let mut buf = [0; 6];
        assert_eq!(OverrunError, ringbuf.read(&mut dma, &mut buf).unwrap_err());
    }
}

#![cfg_attr(gpdma, allow(unused))]

use core::future::poll_fn;
use core::task::{Poll, Waker};

use crate::dma::word::Word;

pub trait DmaCtrl {
    /// Get the NDTR register value, i.e. the space left in the underlying
    /// buffer until the dma writer wraps.
    fn get_remaining_transfers(&self) -> usize;

    /// Reset the transfer completed counter to 0 and return the value just prior to the reset.
    fn reset_complete_count(&mut self) -> usize;

    /// Set the waker for a running poll_fn
    fn set_waker(&mut self, waker: &Waker);
}

#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OverrunError;

#[derive(Debug, Clone, Copy, Default)]
struct DmaIndex {
    completion_count: usize,
    pos: usize,
}

fn pos(cap: usize, dma: &impl DmaCtrl) -> usize {
    cap - dma.get_remaining_transfers()
}

impl DmaIndex {
    fn reset(&mut self) {
        self.pos = 0;
        self.completion_count = 0;
    }

    fn as_index(&self, cap: usize, offset: usize) -> usize {
        (self.pos + offset) % cap
    }

    fn dma_sync(&mut self, cap: usize, dma: &mut impl DmaCtrl) {
        let fst_pos = pos(cap, dma);
        let fst_count = dma.reset_complete_count();
        let pos = pos(cap, dma);

        let wrap_count = if pos >= fst_pos {
            fst_count
        } else {
            fst_count + dma.reset_complete_count()
        };

        self.pos = pos;
        self.completion_count += wrap_count;
    }

    fn advance(&mut self, cap: usize, steps: usize) {
        let next = self.pos + steps;
        self.completion_count += next / cap;
        self.pos = next % cap;
    }

    fn normalize(lhs: &mut DmaIndex, rhs: &mut DmaIndex) {
        let min_count = lhs.completion_count.min(rhs.completion_count);
        lhs.completion_count -= min_count;
        rhs.completion_count -= min_count;
    }

    fn diff(&mut self, cap: usize, rhs: &mut DmaIndex) -> isize {
        Self::normalize(self, rhs);
        (self.completion_count * cap + self.pos) as isize - (rhs.completion_count * cap + rhs.pos) as isize
    }
}

pub struct ReadableDmaRingBuffer<'a, W: Word> {
    dma_buf: &'a mut [W],
    write_index: DmaIndex,
    read_index: DmaIndex,
}

impl<'a, W: Word> ReadableDmaRingBuffer<'a, W> {
    /// Construct an empty buffer.
    pub fn new(dma_buf: &'a mut [W]) -> Self {
        Self {
            dma_buf,
            write_index: Default::default(),
            read_index: Default::default(),
        }
    }

    /// Reset the ring buffer to its initial state
    pub fn clear(&mut self, dma: &mut impl DmaCtrl) {
        dma.reset_complete_count();
        self.write_index.reset();
        self.update_dma_index(dma);
        self.read_index = self.write_index;
    }

    /// The capacity of the ringbuffer
    pub const fn cap(&self) -> usize {
        self.dma_buf.len()
    }

    /// Read elements from the ring buffer
    /// Return a tuple of the length read and the length remaining in the buffer
    /// If not all of the elements were read, then there will be some elements in the buffer remaining
    /// The length remaining is the capacity, ring_buf.len(), less the elements remaining after the read
    /// OverrunError is returned if the portion to be read was overwritten by the DMA controller.
    pub fn read(&mut self, dma: &mut impl DmaCtrl, buf: &mut [W]) -> Result<(usize, usize), OverrunError> {
        let readable = self.margin(dma)?.min(buf.len());
        for i in 0..readable {
            buf[i] = self.read_buf(i);
        }
        let available = self.margin(dma)?;
        self.read_index.advance(self.cap(), readable);
        Ok((readable, available - readable))
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

    fn update_dma_index(&mut self, dma: &mut impl DmaCtrl) {
        self.write_index.dma_sync(self.cap(), dma)
    }

    fn read_buf(&self, offset: usize) -> W {
        unsafe {
            core::ptr::read_volatile(
                self.dma_buf
                    .as_ptr()
                    .offset(self.read_index.as_index(self.cap(), offset) as isize),
            )
        }
    }

    /// Returns available dma samples
    fn margin(&mut self, dma: &mut impl DmaCtrl) -> Result<usize, OverrunError> {
        self.update_dma_index(dma);

        let diff: usize = self
            .write_index
            .diff(self.cap(), &mut self.read_index)
            .try_into()
            .unwrap();

        if diff > self.cap() {
            Err(OverrunError)
        } else {
            Ok(diff)
        }
    }
}

pub struct WritableDmaRingBuffer<'a, W: Word> {
    dma_buf: &'a mut [W],
    read_index: DmaIndex,
    write_index: DmaIndex,
}

impl<'a, W: Word> WritableDmaRingBuffer<'a, W> {
    /// Construct a ringbuffer filled with the given buffer data.
    pub fn new(dma_buf: &'a mut [W]) -> Self {
        let len = dma_buf.len();
        Self {
            dma_buf,
            read_index: Default::default(),
            write_index: DmaIndex {
                completion_count: 0,
                pos: len,
            },
        }
    }

    /// Reset the ring buffer to its initial state. The buffer after the reset will be full.
    pub fn clear(&mut self, dma: &mut impl DmaCtrl) {
        dma.reset_complete_count();
        self.read_index.reset();
        self.update_dma_index(dma);
        self.write_index = self.read_index;
        self.write_index.advance(self.cap(), self.cap());
    }

    /// Get the capacity of the ringbuffer.
    pub const fn cap(&self) -> usize {
        self.dma_buf.len()
    }

    /// Append data to the ring buffer.
    /// Returns a tuple of the data written and the remaining write capacity in the buffer.
    pub fn write(&mut self, dma: &mut impl DmaCtrl, buf: &[W]) -> Result<(usize, usize), OverrunError> {
        let writable = self.margin(dma)?.min(buf.len());
        for i in 0..writable {
            self.write_buf(i, buf[i]);
        }
        let available = self.margin(dma)?;
        self.write_index.advance(self.cap(), writable);
        Ok((writable, available - writable))
    }

    /// Write elements directly to the buffer.
    pub fn write_immediate(&mut self, buf: &[W]) -> Result<(usize, usize), OverrunError> {
        for (i, data) in buf.iter().enumerate() {
            self.write_buf(i, *data)
        }
        let written = buf.len().min(self.cap());
        Ok((written, self.cap() - written))
    }

    /// Write an exact number of elements to the ringbuffer.
    pub async fn write_exact(&mut self, dma: &mut impl DmaCtrl, buffer: &[W]) -> Result<usize, OverrunError> {
        let mut written_data = 0;
        let buffer_len = buffer.len();

        poll_fn(|cx| {
            dma.set_waker(cx.waker());

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

    fn update_dma_index(&mut self, dma: &mut impl DmaCtrl) {
        self.read_index.dma_sync(self.cap(), dma);
    }

    fn write_buf(&mut self, offset: usize, value: W) {
        unsafe {
            core::ptr::write_volatile(
                self.dma_buf
                    .as_mut_ptr()
                    .offset(self.write_index.as_index(self.cap(), offset) as isize),
                value,
            )
        }
    }

    fn margin(&mut self, dma: &mut impl DmaCtrl) -> Result<usize, OverrunError> {
        self.update_dma_index(dma);

        let diff = self.write_index.diff(self.cap(), &mut self.read_index);

        if diff < 0 {
            Err(OverrunError)
        } else {
            Ok(self.cap().saturating_sub(diff as usize))
        }
    }
}

#[cfg(test)]
mod tests;

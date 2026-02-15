use core::future::poll_fn;
use core::sync::atomic::{Ordering, fence};
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
pub enum Error {
    Overrun,
    /// the newly read DMA positions don't make sense compared to the previous
    /// ones. This can usually only occur due to wrong Driver implementation, if
    /// the driver author (or the user using raw metapac code) directly resets
    /// the channel for instance.
    DmaUnsynced,
}

#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct DmaIndex {
    complete_count: usize,
    pos: usize,
}

impl DmaIndex {
    fn reset(&mut self) {
        self.pos = 0;
        self.complete_count = 0;
    }

    fn as_index(&self, cap: usize, offset: usize) -> usize {
        (self.pos + offset) % cap
    }

    fn dma_sync(&mut self, cap: usize, dma: &mut impl DmaCtrl) {
        // Important!
        // The ordering of the first two lines matters!
        // If changed, the code will detect a wrong +capacity
        // jump at wrap-around.
        let count_diff = dma.reset_complete_count();
        let pos = cap - dma.get_remaining_transfers();
        self.pos = if pos < self.pos && count_diff == 0 {
            cap - 1
        } else {
            pos
        };

        self.complete_count += count_diff;
    }

    fn advance(&mut self, cap: usize, steps: usize) {
        let next = self.pos + steps;
        self.complete_count += next / cap;
        self.pos = next % cap;
    }

    fn normalize(lhs: &mut DmaIndex, rhs: &mut DmaIndex) {
        let min_count = lhs.complete_count.min(rhs.complete_count);
        lhs.complete_count -= min_count;
        rhs.complete_count -= min_count;
    }

    fn diff(&self, cap: usize, rhs: &DmaIndex) -> isize {
        (self.complete_count * cap + self.pos) as isize - (rhs.complete_count * cap + rhs.pos) as isize
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

    /// Reset the ring buffer to its initial state.
    pub fn reset(&mut self, dma: &mut impl DmaCtrl) {
        dma.reset_complete_count();
        self.write_index.reset();
        self.write_index.dma_sync(self.cap(), dma);
        self.read_index = self.write_index;
    }

    /// Get the full ringbuffer capacity.
    pub const fn cap(&self) -> usize {
        self.dma_buf.len()
    }

    /// Get the available readable dma samples.
    pub fn len(&mut self, dma: &mut impl DmaCtrl) -> Result<usize, Error> {
        self.write_index.dma_sync(self.cap(), dma);
        DmaIndex::normalize(&mut self.write_index, &mut self.read_index);

        let diff = self.write_index.diff(self.cap(), &self.read_index);

        if diff < 0 {
            Err(Error::DmaUnsynced)
        } else if diff > self.cap() as isize {
            Err(Error::Overrun)
        } else {
            Ok(diff as usize)
        }
    }

    /// Read elements from the ring buffer.
    ///
    /// Return a tuple of the length read and the length remaining in the buffer
    /// If not all of the elements were read, then there will be some elements in the buffer remaining
    /// The length remaining is the capacity, ring_buf.len(), less the elements remaining after the read
    /// Error is returned if the portion to be read was overwritten by the DMA controller,
    /// in which case the rinbuffer will automatically reset itself.
    pub fn read(&mut self, dma: &mut impl DmaCtrl, buf: &mut [W]) -> Result<(usize, usize), Error> {
        self.read_raw(dma, buf).inspect_err(|_e| {
            self.reset(dma);
        })
    }

    /// Read an exact number of elements from the ringbuffer.
    ///
    /// Returns the remaining number of elements available for immediate reading.
    /// Error is returned if the portion to be read was overwritten by the DMA controller.
    ///
    /// Async/Wake Behavior:
    /// The underlying DMA peripheral only can wake us when its buffer pointer has reached the halfway point,
    /// and when it wraps around. This means that when called with a buffer of length 'M', when this
    /// ring buffer was created with a buffer of size 'N':
    /// - If M equals N/2 or N/2 divides evenly into M, this function will return every N/2 elements read on the DMA source.
    /// - Otherwise, this function may need up to N/2 extra elements to arrive before returning.
    pub async fn read_exact(&mut self, dma: &mut impl DmaCtrl, buffer: &mut [W]) -> Result<usize, Error> {
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

    fn read_raw(&mut self, dma: &mut impl DmaCtrl, buf: &mut [W]) -> Result<(usize, usize), Error> {
        fence(Ordering::Acquire);

        let readable = self.len(dma)?.min(buf.len());
        for i in 0..readable {
            buf[i] = self.read_buf(i);
        }
        let available = self.len(dma)?;
        self.read_index.advance(self.cap(), readable);
        Ok((readable, available - readable))
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
                complete_count: 0,
                pos: len,
            },
        }
    }

    /// Reset the ring buffer to its initial state. The buffer after the reset will be full.
    pub fn reset(&mut self, dma: &mut impl DmaCtrl) {
        dma.reset_complete_count();
        self.read_index.reset();
        self.read_index.dma_sync(self.cap(), dma);
        self.write_index = self.read_index;
        self.write_index.advance(self.cap(), self.cap());
    }

    /// Get the remaining writable dma samples.
    pub fn len(&mut self, dma: &mut impl DmaCtrl) -> Result<usize, Error> {
        self.read_index.dma_sync(self.cap(), dma);
        DmaIndex::normalize(&mut self.read_index, &mut self.write_index);

        let diff = self.write_index.diff(self.cap(), &self.read_index);

        if diff < 0 {
            Err(Error::Overrun)
        } else if diff > self.cap() as isize {
            Err(Error::DmaUnsynced)
        } else {
            Ok(self.cap().saturating_sub(diff as usize))
        }
    }

    /// Get the full ringbuffer capacity.
    pub const fn cap(&self) -> usize {
        self.dma_buf.len()
    }

    /// Append data to the ring buffer.
    /// Returns a tuple of the data written and the remaining write capacity in the buffer.
    /// Error is returned if the portion to be written was previously read by the DMA controller.
    /// In this case, the ringbuffer will automatically reset itself, giving a full buffer worth of
    /// leeway between the write index and the DMA.
    pub fn write(&mut self, dma: &mut impl DmaCtrl, buf: &[W]) -> Result<(usize, usize), Error> {
        self.write_raw(dma, buf).inspect_err(|_e| {
            self.reset(dma);
        })
    }

    /// Write elements directly to the buffer.
    ///
    /// Subsequent writes will overwrite the content of the buffer, so it is not useful to call this more than once.
    /// Data is aligned towards the end of the buffer.
    ///
    /// In case of success, returns the written length, and the empty space in front of the written block.
    /// Fails if the data to write exceeds the buffer capacity.
    pub fn write_immediate(&mut self, buf: &[W]) -> Result<(usize, usize), Error> {
        fence(Ordering::Release);

        if buf.len() > self.cap() {
            return Err(Error::Overrun);
        }

        let start = self.cap() - buf.len();
        for (i, data) in buf.iter().enumerate() {
            self.write_buf(start + i, *data)
        }
        let written = buf.len().min(self.cap());
        Ok((written, self.cap() - written))
    }

    /// Wait for any ring buffer write error.
    pub async fn wait_write_error(&mut self, dma: &mut impl DmaCtrl) -> Result<usize, Error> {
        poll_fn(|cx| {
            dma.set_waker(cx.waker());

            match self.len(dma) {
                Ok(_) => Poll::Pending,
                Err(e) => Poll::Ready(Err(e)),
            }
        })
        .await
    }

    /// Write an exact number of elements to the ringbuffer.
    ///
    /// Returns the remaining write capacity in the buffer.
    #[allow(dead_code)]
    pub async fn write_exact(&mut self, dma: &mut impl DmaCtrl, buffer: &[W]) -> Result<usize, Error> {
        let mut written_len = 0;
        let buffer_len = buffer.len();

        poll_fn(|cx| {
            dma.set_waker(cx.waker());

            match self.write(dma, &buffer[written_len..buffer_len]) {
                Ok((len, remaining)) => {
                    written_len += len;
                    if written_len == buffer_len {
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

    fn write_raw(&mut self, dma: &mut impl DmaCtrl, buf: &[W]) -> Result<(usize, usize), Error> {
        fence(Ordering::Release);

        let writable = self.len(dma)?.min(buf.len());
        for i in 0..writable {
            self.write_buf(i, buf[i]);
        }
        let available = self.len(dma)?;
        self.write_index.advance(self.cap(), writable);
        Ok((writable, available - writable))
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
}

#[cfg(test)]
mod tests;

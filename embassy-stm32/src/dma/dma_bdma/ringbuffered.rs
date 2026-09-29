use core::future::poll_fn;
use core::sync::atomic::{Ordering, fence};
use core::task::Waker;

use crate::dma::ringbuffer::{DmaCtrl, ReadableDmaRingBuffer, WritableDmaRingBuffer};
use crate::dma::word::Word;
use super::{Channel, Dir, Increment, Request, STATE};
use crate::dma::{RingBufferError, TransferOptions};
use crate::rcc::WakeGuard;

struct DmaCtrlImpl<'a>(Channel<'a>);

impl<'a> DmaCtrl for DmaCtrlImpl<'a> {
    fn get_remaining_transfers(&self) -> usize {
        self.0.get_remaining_transfers() as _
    }

    fn reset_complete_count(&mut self) -> usize {
        let state = &STATE[self.0.channel as usize];
        #[cfg(not(armv6m))]
        return state.complete_count.swap(0, Ordering::AcqRel);
        #[cfg(armv6m)]
        return critical_section::with(|_| {
            let x = state.complete_count.load(Ordering::Acquire);
            state.complete_count.store(0, Ordering::Release);
            x
        });
    }

    fn set_waker(&mut self, waker: &Waker) {
        STATE[self.0.channel as usize].waker.register(waker);
    }
}

/// Ringbuffer for receiving data using DMA circular mode.
pub struct ReadableRingBuffer<'a, W: Word> {
    channel: Channel<'a>,
    _wake_guard: WakeGuard,
    ringbuf: ReadableDmaRingBuffer<'a, W>,
}

impl<'a, W: Word> ReadableRingBuffer<'a, W> {
    /// Create a new empty ring buffer.
    pub unsafe fn new<PW: Word>(
        channel: Channel<'a>,
        _request: Request,
        peri_addr: *mut PW,
        buffer: &'a mut [W],
        mut options: TransferOptions,
    ) -> Self {
        let mut channel: Channel<'a> = channel.into();

        let buffer_ptr = buffer.as_mut_ptr();
        let len = buffer.len();
        let dir = Dir::PeripheralToMemory;

        options.half_transfer_ir = true;
        options.complete_transfer_ir = true;
        options.circular = true;

        channel.configure(
            _request,
            dir,
            peri_addr as *mut u32,
            buffer_ptr as *mut u32,
            len,
            Increment::Memory,
            W::size(),
            PW::size(),
            options,
        );

        DmaCtrlImpl(channel.reborrow()).reset_complete_count();

        Self {
            _wake_guard: channel.info().wake_guard(),
            channel,
            ringbuf: ReadableDmaRingBuffer::new(buffer),
        }
    }

    /// Start the ring buffer operation.
    ///
    /// You must call this after creating it for it to work.
    ///
    /// It starts the channel and makes it run, even if earlier it was suspended (paused).
    pub fn start(&mut self) {
        self.channel.enable_circular_mode();
        self.channel.start();
    }

    /// Set the frame alignment for the ring buffer.
    ///
    /// See [`ReadableDmaRingBuffer::set_alignment`] for details.
    pub fn set_alignment(&mut self, alignment: usize) {
        self.ringbuf.set_alignment(alignment);
    }

    /// Clear all data in the ring buffer.
    pub fn clear(&mut self) {
        self.ringbuf.reset(&mut DmaCtrlImpl(self.channel.reborrow()));
    }

    /// Read elements from the ring buffer
    /// Return a tuple of the length read and the length remaining in the buffer
    /// If not all of the elements were read, then there will be some elements in the buffer remaining
    /// The length remaining is the capacity, ring_buf.len(), less the elements remaining after the read
    /// Error is returned if the portion to be read was overwritten by the DMA controller.
    pub fn read(&mut self, buf: &mut [W]) -> Result<(usize, usize), RingBufferError> {
        Ok(self.ringbuf.read(&mut DmaCtrlImpl(self.channel.reborrow()), buf)?)
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
    pub async fn read_exact(&mut self, buffer: &mut [W]) -> Result<usize, RingBufferError> {
        Ok(self
            .ringbuf
            .read_exact(&mut DmaCtrlImpl(self.channel.reborrow()), buffer)
            .await?)
    }

    /// The current length of the ringbuffer
    pub fn len(&mut self) -> Result<usize, RingBufferError> {
        Ok(self.ringbuf.sync_len(&mut DmaCtrlImpl(self.channel.reborrow()))?)
    }

    /// Read the most recent elements from the ring buffer, discarding any older data.
    ///
    /// Returns the number of elements actually read into `buf`. Unlike [`read`](Self::read),
    /// this method **never returns an overrun error**. If the DMA has lapped the read pointer,
    /// old data is silently discarded and only the most recent samples are returned.
    ///
    /// This is ideal for use cases like ADC sampling where the consumer only cares about
    /// the latest values.
    pub fn read_latest(&mut self, buf: &mut [W]) -> usize {
        self.ringbuf.read_latest(&mut DmaCtrlImpl(self.channel.reborrow()), buf)
    }

    /// The capacity of the ringbuffer
    pub const fn capacity(&self) -> usize {
        self.ringbuf.cap()
    }

    /// Set a waker to be woken when at least one byte is received.
    pub fn set_waker(&mut self, waker: &Waker) {
        DmaCtrlImpl(self.channel.reborrow()).set_waker(waker);
    }

    /// Request the transfer to pause, keeping the existing configuration for this channel.
    /// To restart the transfer, call [`start`](Self::start) again.
    ///
    /// This doesn't immediately stop the transfer, you have to wait until [`is_running`](Self::is_running) returns false.
    pub fn request_pause(&mut self) {
        self.channel.request_pause()
    }

    /// Request the transfer to resume after having been paused.
    pub fn request_resume(&mut self) {
        self.channel.request_resume()
    }

    /// Return whether DMA is still running.
    ///
    /// If this returns `false`, it can be because either the transfer finished, or
    /// it was requested to stop early with [`request_reset`](Self::request_reset).
    pub fn is_running(&mut self) -> bool {
        self.channel.is_running()
    }

    /// Warning:
    /// This function is legacy and on GPDMA has no effect except waiting.
    ///
    /// Stop the DMA transfer and await until the buffer is full.
    ///
    /// This disables the DMA transfer's circular mode so that the transfer
    /// stops when the buffer is full.
    ///
    /// This is designed to be used with streaming input data such as the
    /// I2S/SAI or ADC.
    ///
    /// When using the UART, you probably want `request_reset()`.
    pub async fn disable_circular_and_wait(&mut self) {
        self.channel.disable_circular_mode();
        //wait until cr.susp reads as true
        poll_fn(|cx| {
            self.set_waker(cx.waker());
            self.channel.poll_stop()
        })
        .await
    }
}

impl<'a, W: Word> Drop for ReadableRingBuffer<'a, W> {
    fn drop(&mut self) {
        self.channel.request_reset();
        while self.is_running() {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);
    }
}

/// Ringbuffer for writing data using DMA circular mode.
pub struct WritableRingBuffer<'a, W: Word> {
    channel: Channel<'a>,
    _wake_guard: WakeGuard,
    ringbuf: WritableDmaRingBuffer<'a, W>,
}

impl<'a, W: Word> WritableRingBuffer<'a, W> {
    /// Create a new ring buffer filled with the given buffer data.
    pub unsafe fn new<PW: Word>(
        channel: Channel<'a>,
        _request: Request,
        peri_addr: *mut PW,
        buffer: &'a mut [W],
        mut options: TransferOptions,
    ) -> Self {
        let mut channel: Channel<'a> = channel.into();

        let len = buffer.len();
        let dir = Dir::MemoryToPeripheral;
        let buffer_ptr = buffer.as_mut_ptr();

        options.half_transfer_ir = true;
        options.complete_transfer_ir = true;
        options.circular = true;

        channel.configure(
            _request,
            dir,
            peri_addr as *mut u32,
            buffer_ptr as *mut u32,
            len,
            Increment::Memory,
            W::size(),
            PW::size(),
            options,
        );

        DmaCtrlImpl(channel.reborrow()).reset_complete_count();

        Self {
            _wake_guard: channel.info().wake_guard(),
            channel,
            ringbuf: WritableDmaRingBuffer::new(buffer),
        }
    }

    /// Start the ring buffer operation.
    ///
    /// You must call this after creating it for it to work.
    pub fn start(&mut self) {
        self.channel.enable_circular_mode();
        self.channel.start();
    }

    /// Clear all data in the ring buffer.
    pub fn clear(&mut self) {
        self.ringbuf.reset(&mut DmaCtrlImpl(self.channel.reborrow()));
    }

    /// Write elements directly to the raw buffer.
    /// This can be used to fill the buffer before starting the DMA transfer.
    pub fn write_immediate(&mut self, buf: &[W]) -> Result<(usize, usize), RingBufferError> {
        Ok(self.ringbuf.write_immediate(buf)?)
    }

    /// Write elements from the ring buffer
    /// Return a tuple of the length written and the length remaining in the buffer
    pub fn write(&mut self, buf: &[W]) -> Result<(usize, usize), RingBufferError> {
        Ok(self.ringbuf.write(&mut DmaCtrlImpl(self.channel.reborrow()), buf)?)
    }

    /// Write an exact number of elements to the ringbuffer.
    pub async fn write_exact(&mut self, buffer: &[W]) -> Result<usize, RingBufferError> {
        Ok(self
            .ringbuf
            .write_exact(&mut DmaCtrlImpl(self.channel.reborrow()), buffer)
            .await?)
    }

    /// Wait for any ring buffer write error.
    pub async fn wait_write_error(&mut self) -> Result<usize, RingBufferError> {
        Ok(self
            .ringbuf
            .wait_write_error(&mut DmaCtrlImpl(self.channel.reborrow()))
            .await?)
    }

    /// The free capacity of the ring buffer.
    pub fn len(&mut self) -> Result<usize, RingBufferError> {
        Ok(self.ringbuf.sync_len(&mut DmaCtrlImpl(self.channel.reborrow()))?)
    }

    /// The capacity of the ringbuffer
    pub const fn capacity(&self) -> usize {
        self.ringbuf.cap()
    }

    /// Return the current write position in the DMA buffer.
    ///
    /// See [`WritableDmaRingBuffer::write_pos`] for details.
    pub fn write_pos(&self) -> usize {
        self.ringbuf.write_pos()
    }

    /// Set a waker to be woken when at least one byte is received.
    pub fn set_waker(&mut self, waker: &Waker) {
        DmaCtrlImpl(self.channel.reborrow()).set_waker(waker);
    }

    /// Request the transfer to pause, keeping the existing configuration for this channel.
    /// To restart the transfer, call [`start`](Self::start) again.
    ///
    /// This doesn't immediately stop the transfer, you have to wait until [`is_running`](Self::is_running) returns false.
    pub fn request_pause(&mut self) {
        self.channel.request_pause()
    }

    /// Return whether DMA is still running.
    ///
    /// If this returns `false`, it can be because either the transfer finished, or
    /// it was requested to stop early with [`request_reset`](Self::request_reset).
    pub fn is_running(&mut self) -> bool {
        self.channel.is_running()
    }

    /// Warning:
    /// This function is legacy and on GPDMA has no effect except waiting.
    ///
    /// Stop the DMA transfer and await until the buffer is empty.
    ///
    /// This disables the DMA transfer's circular mode so that the transfer
    /// stops when all available data has been written.
    ///
    /// This is designed to be used with streaming output data such as the
    /// I2S/SAI or DAC.
    pub async fn disable_circular_and_wait(&mut self) {
        self.channel.disable_circular_mode();
        //wait until cr.susp reads as true
        poll_fn(|cx| {
            self.set_waker(cx.waker());
            self.channel.poll_stop()
        })
        .await
    }
}

impl<'a, W: Word> Drop for WritableRingBuffer<'a, W> {
    fn drop(&mut self) {
        self.channel.request_reset();
        while self.is_running() {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);
    }
}

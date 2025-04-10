//! GPDMA ring buffer implementation.
//!
//! FIXME: add request_pause functionality?
use core::future::poll_fn;
use core::sync::atomic::{fence, Ordering};
use core::task::Waker;

use embassy_hal_internal::Peri;

use super::{AnyChannel, TransferOptions, STATE};
use crate::dma::gpdma::linked_list::{LinearItem, RunMode, Table};
use crate::dma::ringbuffer::{DmaCtrl, Error, ReadableDmaRingBuffer, WritableDmaRingBuffer};
use crate::dma::word::Word;
use crate::dma::{Channel, Request};

struct DmaCtrlImpl<'a>(Peri<'a, AnyChannel>);

impl<'a> DmaCtrl for DmaCtrlImpl<'a> {
    fn get_remaining_transfers(&self) -> usize {
        let state = &STATE[self.0.id as usize];
        let current_remaining = self.0.get_remaining_transfers() as usize;

        let lli_count = state.lli_state.count.load(Ordering::Acquire);

        if lli_count > 0 {
            // In linked-list mode, the remaining transfers are the sum of the full lengths of LLIs that follow,
            // and the remaining transfers for the current LLI.
            let lli_index = state.lli_state.index.load(Ordering::Acquire);
            let single_transfer_count = state.lli_state.transfer_count.load(Ordering::Acquire) / lli_count;

            (lli_count - lli_index - 1) * single_transfer_count + current_remaining
        } else {
            // No linked-list mode.
            current_remaining
        }
    }

    fn reset_complete_count(&mut self) -> usize {
        let state = &STATE[self.0.id as usize];

        state.complete_count.swap(0, Ordering::AcqRel)
    }

    fn set_waker(&mut self, waker: &Waker) {
        STATE[self.0.id as usize].waker.register(waker);
    }
}

/// Ringbuffer for receiving data using GPDMA linked-list mode.
pub struct ReadableRingBuffer<'a, W: Word> {
    channel: Peri<'a, AnyChannel>,
    ringbuf: ReadableDmaRingBuffer<'a, W>,
    table: Table<2>,
}

impl<'a, W: Word> ReadableRingBuffer<'a, W> {
    /// Create a new ring buffer.
    ///
    /// Transfer options are applied to the individual linked list items.
    pub unsafe fn new(
        channel: Peri<'a, impl Channel>,
        request: Request,
        peri_addr: *mut W,
        buffer: &'a mut [W],
        _options: TransferOptions,
    ) -> Self {
        let channel: Peri<'a, AnyChannel> = channel.into();

        // Buffer halves should be the same length.
        let half_len = buffer.len() / 2;
        assert_eq!(half_len * 2, buffer.len());

        let items = [
            LinearItem::new_read(request, peri_addr, &mut buffer[..half_len]),
            LinearItem::new_read(request, peri_addr, &mut buffer[half_len..]),
        ];
        let table = Table::new(items);

        Self {
            channel,
            ringbuf: ReadableDmaRingBuffer::new(buffer),
            table,
        }
    }

    /// Start the ring buffer operation.
    ///
    /// You must call this after creating it for it to work.
    pub fn start(&mut self) {
        unsafe { self.channel.configure_linked_list(&self.table, Default::default()) };
        self.table.link(RunMode::Circular);
        self.channel.start();
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
    pub fn read(&mut self, buf: &mut [W]) -> Result<(usize, usize), Error> {
        self.ringbuf.read(&mut DmaCtrlImpl(self.channel.reborrow()), buf)
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
    pub async fn read_exact(&mut self, buffer: &mut [W]) -> Result<usize, Error> {
        self.ringbuf
            .read_exact(&mut DmaCtrlImpl(self.channel.reborrow()), buffer)
            .await
    }

    /// The current length of the ringbuffer
    pub fn len(&mut self) -> Result<usize, Error> {
        Ok(self.ringbuf.len(&mut DmaCtrlImpl(self.channel.reborrow()))?)
    }

    /// The capacity of the ringbuffer
    pub const fn capacity(&self) -> usize {
        self.ringbuf.cap()
    }

    /// Set a waker to be woken when at least one byte is received.
    pub fn set_waker(&mut self, waker: &Waker) {
        DmaCtrlImpl(self.channel.reborrow()).set_waker(waker);
    }

    /// Request the DMA to stop.
    /// The configuration for this channel will **not be preserved**. If you need to restart the transfer
    /// at a later point with the same configuration, see [`request_pause`](Self::request_pause) instead.
    ///
    /// This doesn't immediately stop the transfer, you have to wait until [`is_running`](Self::is_running) returns false.
    pub fn request_stop(&mut self) {
        self.channel.request_stop()
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
    /// it was requested to stop early with [`request_stop`](Self::request_stop).
    pub fn is_running(&mut self) -> bool {
        self.channel.is_running()
    }

    /// Stop the DMA transfer and await until the buffer is full.
    ///
    /// This disables the DMA transfer's circular mode so that the transfer
    /// stops when the buffer is full.
    ///
    /// This is designed to be used with streaming input data such as the
    /// I2S/SAI or ADC.
    ///
    /// When using the UART, you probably want `request_stop()`.
    pub async fn stop(&mut self) {
        // wait until cr.susp reads as true
        poll_fn(|cx| {
            self.set_waker(cx.waker());
            self.channel.poll_stop()
        })
        .await
    }
}

impl<'a, W: Word> Drop for ReadableRingBuffer<'a, W> {
    fn drop(&mut self) {
        self.request_stop();
        while self.is_running() {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);
    }
}

/// Ringbuffer for writing data using DMA circular mode.
pub struct WritableRingBuffer<'a, W: Word> {
    channel: Peri<'a, AnyChannel>,
    ringbuf: WritableDmaRingBuffer<'a, W>,
    table: Table<2>,
}

impl<'a, W: Word> WritableRingBuffer<'a, W> {
    /// Create a new ring buffer.
    pub unsafe fn new(
        channel: Peri<'a, impl Channel>,
        request: Request,
        peri_addr: *mut W,
        buffer: &'a mut [W],
        _options: TransferOptions,
    ) -> Self {
        let channel: Peri<'a, AnyChannel> = channel.into();

        // Buffer halves should be the same length.
        let half_len = buffer.len() / 2;
        assert_eq!(half_len * 2, buffer.len());

        let items = [
            LinearItem::new_write(request, &mut buffer[..half_len], peri_addr),
            LinearItem::new_write(request, &mut buffer[half_len..], peri_addr),
        ];
        let table = Table::new(items);

        let this = Self {
            channel,
            ringbuf: WritableDmaRingBuffer::new(buffer),
            table,
        };

        this
    }

    /// Start the ring buffer operation.
    ///
    /// You must call this after creating it for it to work.
    pub fn start(&mut self) {
        unsafe { self.channel.configure_linked_list(&self.table, Default::default()) };
        self.table.link(RunMode::Circular);
        self.channel.start();
    }

    /// Clear all data in the ring buffer.
    pub fn clear(&mut self) {
        self.ringbuf.reset(&mut DmaCtrlImpl(self.channel.reborrow()));
    }

    /// Write elements directly to the raw buffer.
    /// This can be used to fill the buffer before starting the DMA transfer.
    pub fn write_immediate(&mut self, buf: &[W]) -> Result<(usize, usize), Error> {
        self.ringbuf.write_immediate(buf)
    }

    /// Write elements from the ring buffer
    /// Return a tuple of the length written and the length remaining in the buffer
    pub fn write(&mut self, buf: &[W]) -> Result<(usize, usize), Error> {
        self.ringbuf.write(&mut DmaCtrlImpl(self.channel.reborrow()), buf)
    }

    /// Write an exact number of elements to the ringbuffer.
    pub async fn write_exact(&mut self, buffer: &[W]) -> Result<usize, Error> {
        // return self
        //     .ringbuf
        //     .write_exact(&mut DmaCtrlImpl(self.channel.reborrow()), buffer)
        //     .await;

        let mut remaining_cap = 0;
        let mut written_len = 0;

        while written_len < buffer.len() {
            (written_len, remaining_cap) = self
                .ringbuf
                .write_half(&mut DmaCtrlImpl(self.channel.reborrow()), buffer)
                .await?;
            // info!("Written: {}/{}", written_len, buffer.len());
        }

        Ok(remaining_cap)
    }

    /// Wait for any ring buffer write error.
    pub async fn wait_write_error(&mut self) -> Result<usize, Error> {
        self.ringbuf
            .wait_write_error(&mut DmaCtrlImpl(self.channel.reborrow()))
            .await
    }

    /// The current length of the ringbuffer
    pub fn len(&mut self) -> Result<usize, Error> {
        Ok(self.ringbuf.len(&mut DmaCtrlImpl(self.channel.reborrow()))?)
    }

    /// The capacity of the ringbuffer
    pub const fn capacity(&self) -> usize {
        self.ringbuf.cap()
    }

    /// Set a waker to be woken when at least one byte is received.
    pub fn set_waker(&mut self, waker: &Waker) {
        DmaCtrlImpl(self.channel.reborrow()).set_waker(waker);
    }

    /// Request the DMA to stop.
    /// The configuration for this channel will **not be preserved**. If you need to restart the transfer
    /// at a later point with the same configuration, see [`request_pause`](Self::request_pause) instead.
    ///
    /// This doesn't immediately stop the transfer, you have to wait until [`is_running`](Self::is_running) returns false.
    pub fn request_stop(&mut self) {
        self.channel.request_stop()
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
    /// it was requested to stop early with [`request_stop`](Self::request_stop).
    pub fn is_running(&mut self) -> bool {
        self.channel.is_running()
    }

    /// Stop the DMA transfer and await until the buffer is full.
    ///
    /// This disables the DMA transfer's circular mode so that the transfer
    /// stops when the buffer is full.
    ///
    /// This is designed to be used with streaming input data such as the
    /// I2S/SAI or ADC.
    ///
    /// When using the UART, you probably want `request_stop()`.
    pub async fn stop(&mut self) {
        // wait until cr.susp reads as true
        poll_fn(|cx| {
            self.set_waker(cx.waker());
            self.channel.poll_stop()
        })
        .await
    }
}

impl<'a, W: Word> Drop for WritableRingBuffer<'a, W> {
    fn drop(&mut self) {
        self.request_stop();
        while self.is_running() {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);
    }
}

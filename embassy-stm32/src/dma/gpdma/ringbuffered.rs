//! GPDMA ring buffer implementation.
//!
//! FIXME: Stop the DMA, if a user does not queue new transfers (chain of linked-list items ends automatically).
use core::future::poll_fn;
use core::mem::MaybeUninit;
use core::sync::atomic::{Ordering, fence};
use core::task::Waker;

use super::{Channel, STATE, TransferOptions};
use crate::_generated::ringbuffer_table;
use crate::dma::gpdma::linked_list::{LinearItem, LinkedListItem, RunMode, Table};
use crate::dma::ringbuffer::{DmaCtrl, ReadableDmaRingBuffer, WritableDmaRingBuffer};
use crate::dma::word::Word;
use crate::dma::{Dir, Request, RingBufferError};
use crate::rcc::WakeGuard;

/// DmaCtrl implementation for GPDMA linked-list ring buffers.
///
/// Uses a critical section in `reset_complete_count` to atomically snapshot both
/// `complete_count` and the hardware BNDT register. This prevents the DMA ISR from
/// updating `complete_count` or `lli_index` between the two reads, which would cause
/// `dma_sync` (which calls `reset_complete_count` then `get_remaining_transfers`) to
/// see an inconsistent position.
struct DmaCtrlImpl<'a> {
    channel: Channel<'a>,
    /// Remaining transfers cached by the last `reset_complete_count` call.
    cached_remaining: usize,
}

impl<'a> DmaCtrlImpl<'a> {
    fn new(channel: Channel<'a>) -> Self {
        Self {
            channel,
            cached_remaining: 0,
        }
    }

    /// Compute remaining transfers from hardware and LLI state.
    fn compute_remaining(&self, _cs: critical_section::CriticalSection) -> usize {
        let state = &STATE[self.channel.channel as usize];
        let lli_count = state.lli_state.count.load(Ordering::Relaxed);

        if lli_count > 0 {
            let lli_index = state.lli_state.index.load(Ordering::Relaxed);
            let single_transfer_count = state.lli_state.transfer_count.load(Ordering::Relaxed) / lli_count;
            let current_remaining = self.channel.get_remaining_transfers() as usize;

            // During LLI reload, BNDT can momentarily read as 0. In a critical section
            // the ISR can't run, so lli_index is consistent with BNDT. If BNDT is 0,
            // the LLI just completed. Treat it as 1 to avoid pos = cap which would
            // double-count with the pending complete_count increment.
            let current_remaining = current_remaining.max(1);

            (lli_count - lli_index - 1) * single_transfer_count + current_remaining
        } else {
            self.channel.get_remaining_transfers() as usize
        }
    }
}

impl<'a> DmaCtrl for DmaCtrlImpl<'a> {
    fn get_remaining_transfers(&self) -> usize {
        self.cached_remaining
    }

    fn reset_complete_count(&mut self) -> usize {
        let state = &STATE[self.channel.channel as usize];

        // Snapshot complete_count and BNDT atomically by disabling interrupts.
        // This prevents the DMA ISR from modifying complete_count or lli_index
        // between the two reads, eliminating the race that causes DmaUnsynced.
        critical_section::with(|cs| {
            let count = state.complete_count.swap(0, Ordering::AcqRel);
            self.cached_remaining = self.compute_remaining(cs);
            count
        })
    }

    fn set_waker(&mut self, waker: &Waker) {
        STATE[self.channel.channel as usize].waker.register(waker);
    }
}

/// Ringbuffer for receiving data using GPDMA linked-list mode.
pub struct ReadableRingBuffer<'a, W: Word> {
    channel: Channel<'a>,
    _wake_guard: WakeGuard,
    ringbuf: ReadableDmaRingBuffer<'a, W>,
}

impl<'a, W: Word> ReadableRingBuffer<'a, W> {
    /// Create a new empty ring buffer.
    ///
    /// You must call [`start`](Self::start) after creating ring buffer for it to work.
    ///
    /// Transfer options are applied to the individual linked list items.
    /// Half-transfer and transfer-complete IRQs are always enabled (same as BDMA ring
    /// buffers) so async `read_exact` / `write_exact` can wake at half-buffer boundaries.
    pub unsafe fn new<PW: Word>(
        mut channel: Channel<'a>,
        request: Request,
        peri_addr: *mut PW,
        buffer: &'a mut [W],
        mut options: TransferOptions,
    ) -> Self {
        options.half_transfer_ir = true;
        options.complete_transfer_ir = true;

        let table = ringbuffer_table(channel.channel).write(Table::<LinearItem, 1>::new_circular::<W, PW>(
            request,
            peri_addr,
            buffer,
            Dir::PeripheralToMemory,
            Default::default(),
        ));

        unsafe {
            channel.configure_linked_list_raw(
                table.base_address(),
                table.offset_address(0),
                1,
                table.transfer_count(),
                options,
                false,
            );
        }

        table.link(RunMode::Circular);

        DmaCtrlImpl::new(channel.reborrow()).reset_complete_count();

        Self {
            _wake_guard: channel.info().wake_guard(),
            channel,
            ringbuf: ReadableDmaRingBuffer::new(buffer),
        }
    }

    /// Create a ring buffer whose linked-list table is written into `table` storage, split into
    /// `blocks` equal circular items.
    ///
    /// Each item raises a transfer-complete interrupt, so the consumer's waker fires once per
    /// `buffer.len() / blocks` words instead of once per half buffer: the wake granularity is
    /// the block size, not the buffer size. The items must be equal-sized (the position
    /// tracking assumes uniform items); at most `K` blocks, each at most 65535 bytes.
    ///
    /// You must call [`start`](Self::start) after creating ring buffer for it to work.
    ///
    /// # Safety
    ///
    /// Like [`new`](Self::new), plus `table` must stay valid and unaliased for as long as the
    /// ring buffer runs (the channel fetches its linked list from it).
    pub unsafe fn new_blocked<PW: Word, const K: usize>(
        table: *mut MaybeUninit<Table<LinearItem, K>>,
        blocks: usize,
        mut channel: Channel<'a>,
        request: Request,
        peri_addr: *mut PW,
        buffer: &'a mut [W],
        mut options: TransferOptions,
    ) -> Self {
        assert!(blocks > 0 && blocks <= K);
        assert!(!buffer.is_empty() && buffer.len() % blocks == 0);

        options.half_transfer_ir = false;
        options.complete_transfer_ir = true;

        let block = buffer.len() / blocks;
        let table = (*table).write(Table::new(core::array::from_fn(|i| {
            if i < blocks {
                LinearItem::new_read::<W, PW>(
                    request,
                    peri_addr,
                    &mut buffer[i * block..(i + 1) * block],
                    Default::default(),
                )
            } else {
                LinearItem::default()
            }
        })));

        // Link only the used prefix, circularly. The unused items stay unlinked and empty,
        // so `transfer_count` still sums the whole ring.
        for i in 0..blocks - 1 {
            table.link_indices(i, i + 1);
        }
        table.link_indices(blocks - 1, 0);
        // "Preceding reads and writes cannot be moved past subsequent writes."
        fence(Ordering::SeqCst);

        channel.configure_linked_list_raw(
            table.base_address(),
            table.offset_address(0),
            blocks,
            table.transfer_count(),
            options,
            false,
        );

        DmaCtrlImpl::new(channel.reborrow()).reset_complete_count();

        Self {
            _wake_guard: channel.info().wake_guard(),
            channel,
            ringbuf: ReadableDmaRingBuffer::new(buffer),
        }
    }

    /// Start the ring buffer operation.
    ///
    /// You must call this after creating ring buffer for it to work.
    pub fn start(&mut self) {
        self.channel.request_resume(); // clear SUSP if previously paused
    }

    /// Set the frame alignment for the ring buffer.
    ///
    /// See [`ReadableDmaRingBuffer::set_alignment`] for details.
    pub fn set_alignment(&mut self, alignment: usize) {
        self.ringbuf.set_alignment(alignment);
    }

    /// Clear all data in the ring buffer.
    pub fn clear(&mut self) {
        self.ringbuf.reset(&mut DmaCtrlImpl::new(self.channel.reborrow()));
    }

    /// Read elements from the ring buffer
    /// Return a tuple of the length read and the length remaining in the buffer
    /// If not all of the elements were read, then there will be some elements in the buffer remaining
    /// The length remaining is the capacity, ring_buf.sync_len(), less the elements remaining after the read
    /// Error is returned if the portion to be read was overwritten by the DMA controller.
    pub fn read(&mut self, buf: &mut [W]) -> Result<(usize, usize), RingBufferError> {
        Ok(self.ringbuf.read(&mut DmaCtrlImpl::new(self.channel.reborrow()), buf)?)
    }

    /// The read position, in elements from the buffer's start (wrapping).
    ///
    /// For consumers that keep their own place in the stream, e.g. the frame slots of the CRC
    /// validity marks.
    pub fn read_pos(&self) -> usize {
        self.ringbuf.read_pos()
    }

    /// Snapshot the write position, in words from the buffer's start.
    ///
    /// Reads the channel's destination address register directly, without synchronizing with
    /// the ring: for interrupt handlers that record where the DMA was when a peripheral flagged
    /// an error.
    pub fn write_head(&self) -> usize {
        let dar = unsafe { self.channel.destination_register().read_volatile() } as usize;
        (dar.wrapping_sub(self.ringbuf.buf_addr()) / W::size().bytes()) % self.ringbuf.cap()
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
            .read_exact(&mut DmaCtrlImpl::new(self.channel.reborrow()), buffer)
            .await?)
    }

    /// The current length of the ringbuffer
    pub fn len(&mut self) -> Result<usize, RingBufferError> {
        Ok(self.ringbuf.sync_len(&mut DmaCtrlImpl::new(self.channel.reborrow()))?)
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
        self.ringbuf
            .read_latest(&mut DmaCtrlImpl::new(self.channel.reborrow()), buf)
    }

    /// The capacity of the ringbuffer
    pub const fn capacity(&self) -> usize {
        self.ringbuf.cap()
    }

    /// Set a waker to be woken when at least one byte is received.
    pub fn set_waker(&mut self, waker: &Waker) {
        DmaCtrlImpl::new(self.channel.reborrow()).set_waker(waker);
    }

    /// Stop the ring buffer operation.
    /// To resume the transfer, call [`start`](Self::start).
    ///
    /// This doesn't immediately stop the transfer, you have to wait until [`is_running`](Self::is_running) returns false.
    pub fn stop(&mut self) {
        self.channel.request_pause()
    }

    /// Return whether this transfer is still running.
    ///
    /// If this returns `false`, it can be because either the transfer finished, or
    /// it was requested to stop early with [`request_pause`](Self::request_pause).
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
    pub async fn disable_circular_and_wait(&mut self) {
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
        self.channel.request_reset();
        while self.is_running() {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);
    }
}

/// Ringbuffer for writing data using GPDMA linked-list mode.
pub struct WritableRingBuffer<'a, W: Word> {
    channel: Channel<'a>,
    _wake_guard: WakeGuard,
    ringbuf: WritableDmaRingBuffer<'a, W>,
}

impl<'a, W: Word> WritableRingBuffer<'a, W> {
    /// Create a new ring buffer filled with the given buffer data.
    ///
    /// You must call [`start`](Self::start) after creating ring buffer for it to work.
    ///
    /// Transfer options are applied to the individual linked list items.
    /// Half-transfer and transfer-complete IRQs are always enabled (same as BDMA ring
    /// buffers) so async `read_exact` / `write_exact` can wake at half-buffer boundaries.
    pub unsafe fn new<PW: Word>(
        mut channel: Channel<'a>,
        request: Request,
        peri_addr: *mut PW,
        buffer: &'a mut [W],
        mut options: TransferOptions,
    ) -> Self {
        options.half_transfer_ir = true;
        options.complete_transfer_ir = true;

        let table = ringbuffer_table(channel.channel).write(Table::<LinearItem, 1>::new_circular::<W, PW>(
            request,
            peri_addr,
            buffer,
            Dir::MemoryToPeripheral,
            Default::default(),
        ));

        unsafe {
            channel.configure_linked_list_raw(
                table.base_address(),
                table.offset_address(0),
                1,
                table.transfer_count(),
                options,
                false,
            );
        }

        table.link(RunMode::Circular);

        DmaCtrlImpl::new(channel.reborrow()).reset_complete_count();

        Self {
            _wake_guard: channel.info().wake_guard(),
            channel,
            ringbuf: WritableDmaRingBuffer::new(buffer),
        }
    }

    /// Start the ring buffer operation.
    ///
    /// You must call this after creating ring buffer for it to work.
    pub fn start(&mut self) {
        self.channel.request_resume(); // clear SUSP if previously paused
    }

    /// Clear all data in the ring buffer.
    pub fn clear(&mut self) {
        self.ringbuf.reset(&mut DmaCtrlImpl::new(self.channel.reborrow()));
    }

    /// Write elements from the ring buffer
    /// Return a tuple of the length written and the length remaining in the buffer
    pub fn write(&mut self, buf: &[W]) -> Result<(usize, usize), RingBufferError> {
        Ok(self
            .ringbuf
            .write(&mut DmaCtrlImpl::new(self.channel.reborrow()), buf)?)
    }

    /// Write an exact number of elements to the ringbuffer.
    pub async fn write_exact(&mut self, buffer: &[W]) -> Result<usize, RingBufferError> {
        Ok(self
            .ringbuf
            .write_exact(&mut DmaCtrlImpl::new(self.channel.reborrow()), buffer)
            .await?)
    }

    /// Wait for any ring buffer write error.
    pub async fn wait_write_error(&mut self) -> Result<usize, RingBufferError> {
        Ok(self
            .ringbuf
            .wait_write_error(&mut DmaCtrlImpl::new(self.channel.reborrow()))
            .await?)
    }

    /// The free capacity of the ring buffer.
    pub fn len(&mut self) -> Result<usize, RingBufferError> {
        Ok(self.ringbuf.sync_len(&mut DmaCtrlImpl::new(self.channel.reborrow()))?)
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
        DmaCtrlImpl::new(self.channel.reborrow()).set_waker(waker);
    }

    /// Stop the ring buffer operation.
    /// To resume the transfer, call [`start`](Self::start).
    ///
    /// This doesn't immediately stop the transfer, you have to wait until [`is_running`](Self::is_running) returns false.
    pub fn stop(&mut self) {
        self.channel.request_pause()
    }

    /// Return whether DMA is still running.
    ///
    /// If this returns `false`, it can be because either the transfer finished, or
    /// it was requested to stop early with [`request_stop`](Self::request_stop).
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
    /// When using the UART, you probably want `request_stop()`.
    pub async fn disable_circular_and_wait(&mut self) {
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
        self.channel.request_reset();
        while self.is_running() {}

        // "Subsequent reads and writes cannot be moved ahead of preceding reads."
        fence(Ordering::SeqCst);
    }
}

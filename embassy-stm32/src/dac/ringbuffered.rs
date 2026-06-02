//! Ring-buffered DAC driver for continuous waveform streaming.

use core::mem::ManuallyDrop;
use core::sync::atomic::{Ordering, compiler_fence};

use crate::dac::{ChannelEvent, Info, State};
use crate::dma::WritableRingBuffer;
use crate::dma::ringbuffer::Error;
use crate::dma::word::Word;

/// A DAC channel backed by a DMA ring buffer.
///
/// Allows continuous waveform streaming by writing new samples into the ring buffer
/// while DMA is simultaneously consuming and outputting them. DMA runs in circular
/// mode so output never stops between writes.
///
/// Obtain this from [`DacChannel::into_ring_buffered_8bit`] or
/// [`DacChannel::into_ring_buffered_12right`].
pub struct RingBufferedDacChannel<'d, W: Word> {
    ring_buf: ManuallyDrop<WritableRingBuffer<'d, W>>,
    info: &'static Info,
    state: &'static State,
    idx: usize,
}

impl<'d, W: Word> RingBufferedDacChannel<'d, W> {
    pub(super) fn new(
        ring_buf: WritableRingBuffer<'d, W>,
        info: &'static Info,
        state: &'static State,
        idx: usize,
    ) -> Self {
        Self {
            ring_buf: ManuallyDrop::new(ring_buf),
            info,
            state,
            idx,
        }
    }

    /// Start the DMA transfer.
    ///
    /// Call this after creating the ring buffer (and optionally pre-filling it with
    /// [`write_immediate`](Self::write_immediate)) to begin DAC output.
    pub fn start(&mut self) {
        self.ring_buf.start();
    }

    /// Write samples directly into the raw DMA buffer without checking DMA position.
    ///
    /// Useful for pre-filling the buffer before calling [`start`](Self::start). Writes
    /// at most `capacity` elements aligned to the end of the buffer.
    pub fn write_immediate(&mut self, buf: &[W]) -> Result<(usize, usize), Error> {
        self.ring_buf.write_immediate(buf)
    }

    /// Write samples into the ring buffer.
    ///
    /// Returns `(written, remaining_space)`. Returns [`Error::Overrun`] if the DMA
    /// consumed data faster than the CPU supplied it; the ring buffer resets itself
    /// automatically in that case.
    pub fn write(&mut self, buf: &[W]) -> Result<(usize, usize), Error> {
        self.ring_buf.write(buf)
    }

    /// Write an exact number of samples, waiting asynchronously until space is available.
    pub async fn write_exact(&mut self, buffer: &[W]) -> Result<usize, Error> {
        self.ring_buf.write_exact(buffer).await
    }

    /// Wait for a ring buffer write error (underrun).
    pub async fn wait_write_error(&mut self) -> Result<usize, Error> {
        self.ring_buf.wait_write_error().await
    }

    /// Return the ring buffer capacity in samples.
    pub fn capacity(&self) -> usize {
        self.ring_buf.capacity()
    }

    /// Return whether the DMA is currently running.
    pub fn is_running(&mut self) -> bool {
        self.ring_buf.is_running()
    }

    /// Request the DMA to stop, discarding the channel configuration.
    pub fn request_reset(&mut self) {
        self.ring_buf.request_reset();
    }

    /// Request the DMA to pause, preserving the channel configuration.
    ///
    /// Resume with [`start`](Self::start).
    pub fn request_pause(&mut self) {
        self.ring_buf.request_pause();
    }

    /// Stop the DMA transfer, waiting until all buffered samples have been output.
    pub async fn stop(&mut self) {
        self.ring_buf.stop().await;
    }
}

impl<W: Word> Drop for RingBufferedDacChannel<'_, W> {
    fn drop(&mut self) {
        // Disable the DAC channel and DMA requests before stopping the engine.
        self.info.regs.cr().modify(|w| {
            w.set_en(self.idx, false);
            w.set_dmaen(self.idx, false);
        });
        compiler_fence(Ordering::SeqCst);

        // Safety: ring_buf is ManuallyDrop; this is the only place it is dropped.
        unsafe { ManuallyDrop::drop(&mut self.ring_buf) };

        // Safe to touch RCC now that DMA has fully stopped.
        let count = self.state.adjust_channel_count(ChannelEvent::Disable);
        if count == 0 {
            self.info.rcc.disable();
        }
    }
}

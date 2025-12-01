//! RingBuffered PWM driver.

use core::mem::ManuallyDrop;
use core::task::Waker;

use super::low_level::Timer;
use super::{Channel, GeneralInstance4Channel};
use crate::dma::WritableRingBuffer;
use crate::dma::ringbuffer::Error;
use crate::dma::word::Word;

/// A PWM channel that uses a DMA ring buffer for continuous waveform generation.
///
/// This allows you to continuously update PWM duty cycles via DMA without blocking the CPU.
/// The ring buffer enables smooth, uninterrupted waveform generation by automatically cycling
/// through duty cycle values stored in memory.
///
/// You can write new duty cycle values to the ring buffer while it's running, enabling
/// dynamic waveform generation for applications like motor control, LED dimming, or audio output.
///
/// # Example
/// ```ignore
/// let mut channel = pwm.ch1().into_ring_buffered_channel(dma_ch, &mut buffer);
/// channel.start(); // Start DMA transfer
/// channel.write(&[100, 200, 300]).ok(); // Update duty cycles
/// ```
pub struct RingBufferedPwmChannel<'d, T: GeneralInstance4Channel, W: Word + Into<T::Word>> {
    timer: ManuallyDrop<Timer<'d, T>>,
    ring_buf: WritableRingBuffer<'d, W>,
    channel: Channel,
}

impl<'d, T: GeneralInstance4Channel, W: Word + Into<T::Word>> RingBufferedPwmChannel<'d, T, W> {
    pub(crate) fn new(
        timer: ManuallyDrop<Timer<'d, T>>,
        channel: Channel,
        ring_buf: WritableRingBuffer<'d, W>,
    ) -> Self {
        Self {
            timer,
            ring_buf,
            channel,
        }
    }

    /// Start the ring buffer operation.
    ///
    /// You must call this after creating it for it to work.
    pub fn start(&mut self) {
        self.ring_buf.start()
    }

    /// Clear all data in the ring buffer.
    pub fn clear(&mut self) {
        self.ring_buf.clear()
    }

    /// Write elements directly to the raw buffer. This can be used to fill the buffer before starting the DMA transfer.
    pub fn write_immediate(&mut self, buf: &[W]) -> Result<(usize, usize), Error> {
        self.ring_buf.write_immediate(buf)
    }

    /// Write elements from the ring buffer
    /// Return a tuple of the length written and the length remaining in the buffer
    pub fn write(&mut self, buf: &[W]) -> Result<(usize, usize), Error> {
        self.ring_buf.write(buf)
    }

    /// Write an exact number of elements to the ringbuffer.
    pub async fn write_exact(&mut self, buffer: &[W]) -> Result<usize, Error> {
        self.ring_buf.write_exact(buffer).await
    }

    /// Wait for any ring buffer write error.
    pub async fn wait_write_error(&mut self) -> Result<usize, Error> {
        self.ring_buf.wait_write_error().await
    }

    /// The current length of the ringbuffer
    pub fn len(&mut self) -> Result<usize, Error> {
        self.ring_buf.len()
    }

    /// The capacity of the ringbuffer
    pub const fn capacity(&self) -> usize {
        self.ring_buf.capacity()
    }

    /// Set a waker to be woken when at least one byte is send.
    pub fn set_waker(&mut self, waker: &Waker) {
        self.ring_buf.set_waker(waker)
    }

    /// Request the DMA to reset. The configuration for this channel will not be preserved.
    ///
    /// This doesn't immediately stop the transfer, you have to wait until is_running returns false.
    pub fn request_reset(&mut self) {
        self.ring_buf.request_reset()
    }

    /// Request the transfer to pause, keeping the existing configuration for this channel.
    /// To restart the transfer, call [`start`](Self::start) again.
    ///
    /// This doesn't immediately stop the transfer, you have to wait until is_running returns false.
    pub fn request_pause(&mut self) {
        self.ring_buf.request_pause()
    }

    /// Return whether DMA is still running.
    ///
    /// If this returns false, it can be because either the transfer finished, or it was requested to stop early with request_stop.
    pub fn is_running(&mut self) -> bool {
        self.ring_buf.is_running()
    }

    /// Stop the DMA transfer and await until the buffer is empty.
    ///
    /// This disables the DMA transfer's circular mode so that the transfer stops when all available data has been written.
    ///
    /// This is designed to be used with streaming output data such as the I2S/SAI or DAC.
    pub async fn stop(&mut self) {
        self.ring_buf.stop().await
    }

    /// Enable the given channel.
    pub fn enable(&mut self) {
        self.timer.enable_channel(self.channel, true);
    }

    /// Disable the given channel.
    pub fn disable(&mut self) {
        self.timer.enable_channel(self.channel, false);
    }

    /// Check whether given channel is enabled
    pub fn is_enabled(&self) -> bool {
        self.timer.get_channel_enable_state(self.channel)
    }

    /// Get max duty value.
    ///
    /// This value depends on the configured frequency and the timer's clock rate from RCC.
    pub fn max_duty_cycle(&self) -> u16 {
        let max: u32 = self.timer.get_max_compare_value().into();
        assert!(max < u16::MAX as u32);
        max as u16 + 1
    }

    /// Set the output polarity for a given channel.
    pub fn set_polarity(&mut self, polarity: super::low_level::OutputPolarity) {
        self.timer.set_output_polarity(self.channel, polarity);
    }

    /// Set the output compare mode for a given channel.
    pub fn set_output_compare_mode(&mut self, mode: super::low_level::OutputCompareMode) {
        self.timer.set_output_compare_mode(self.channel, mode);
    }
}

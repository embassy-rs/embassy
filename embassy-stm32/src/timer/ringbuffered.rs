//! RingBuffered PWM driver.

use core::mem::ManuallyDrop;

use super::low_level::Timer;
use super::{Channel, GeneralInstance4Channel, TimerChannel, TimerPin};
use crate::Peri;
use crate::dma::ringbuffer::WritableDmaRingBuffer;
use super::simple_pwm::SimplePwm;

pub struct RingBufferedPwmChannel<'d, T: GeneralInstance4Channel> {
    timer: ManuallyDrop<Timer<'d, T>>,
    ring_buf: WritableDmaRingBuffer<'d, u8>,
    channel: Channel,
}

/// A group of four [`SimplePwmChannel`]s, obtained from [`SimplePwm::split`].
pub struct RingBufferedPwmChannels<'d, T: GeneralInstance4Channel> {
    /// Channel 1
    pub ch1: RingBufferedPwmChannel<'d, T>,
    /// Channel 2
    pub ch2: RingBufferedPwmChannel<'d, T>,
    /// Channel 3
    pub ch3: RingBufferedPwmChannel<'d, T>,
    /// Channel 4
    pub ch4: RingBufferedPwmChannel<'d, T>,
}

/// Simple PWM driver.
pub struct RingBufferedPwm<'d, T: GeneralInstance4Channel> {
    inner: Timer<'d, T>,
}

impl<'d, T: GeneralInstance4Channel> SimplePwm<'d, T> {
    pub fn into_ring_buffered_channel<C: TimerChannel>(mut self, tx_dma: Peri<'_, impl super::Dma<T, C>>, dma_buf: &'d mut [u8]) -> RingBufferedPwmChannel<'d> {
        assert!(!dma_buf.is_empty() && dma_buf.len() <= 0xFFFF);
        let ring_buf = WritableDmaRingBuffer::new(dma_buf);
        let channel = C::CHANNEL;
        RingBufferedPwmChannel {
            timer: unsafe { self.inner.clone_unchecked() },
            channel,
            ring_buf
        }

        // let ring_buf = WriteableRingBuffer::new();
    }
}

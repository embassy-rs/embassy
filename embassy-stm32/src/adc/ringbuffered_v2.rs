use core::marker::PhantomData;
use core::mem;
use core::sync::atomic::{compiler_fence, Ordering};

use stm32_metapac::adc::vals::SampleTime;

use crate::adc::{Adc, AdcChannel, Instance, RxDma};
use crate::dma::{Priority, ReadableRingBuffer, TransferOptions};
use crate::pac::adc::vals;
use crate::{rcc, Peri};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OverrunError;

fn clear_interrupt_flags(r: crate::pac::adc::Adc) {
    r.sr().modify(|regs| {
        regs.set_eoc(false);
        regs.set_ovr(false);
    });
}

#[derive(PartialOrd, PartialEq, Debug, Clone, Copy)]
pub enum Sequence {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Eleven,
    Twelve,
    Thirteen,
    Fourteen,
    Fifteen,
    Sixteen,
}

impl From<Sequence> for u8 {
    fn from(s: Sequence) -> u8 {
        match s {
            Sequence::One => 0,
            Sequence::Two => 1,
            Sequence::Three => 2,
            Sequence::Four => 3,
            Sequence::Five => 4,
            Sequence::Six => 5,
            Sequence::Seven => 6,
            Sequence::Eight => 7,
            Sequence::Nine => 8,
            Sequence::Ten => 9,
            Sequence::Eleven => 10,
            Sequence::Twelve => 11,
            Sequence::Thirteen => 12,
            Sequence::Fourteen => 13,
            Sequence::Fifteen => 14,
            Sequence::Sixteen => 15,
        }
    }
}

impl From<u8> for Sequence {
    fn from(val: u8) -> Self {
        match val {
            0 => Sequence::One,
            1 => Sequence::Two,
            2 => Sequence::Three,
            3 => Sequence::Four,
            4 => Sequence::Five,
            5 => Sequence::Six,
            6 => Sequence::Seven,
            7 => Sequence::Eight,
            8 => Sequence::Nine,
            9 => Sequence::Ten,
            10 => Sequence::Eleven,
            11 => Sequence::Twelve,
            12 => Sequence::Thirteen,
            13 => Sequence::Fourteen,
            14 => Sequence::Fifteen,
            15 => Sequence::Sixteen,
            _ => panic!("Invalid sequence number"),
        }
    }
}

pub struct RingBufferedAdc<'d, T: Instance> {
    _phantom: PhantomData<T>,
    ring_buf: ReadableRingBuffer<'d, u16>,
}

impl<'d, T: Instance> Adc<'d, T> {
    /// Configures the ADC to use a DMA ring buffer for continuous data acquisition.
    ///
    /// The `dma_buf` should be large enough to prevent DMA buffer overrun.
    /// The length of the `dma_buf` should be a multiple of the ADC channel count.
    /// For example, if 3 channels are measured, its length can be 3 * 40 = 120 measurements.
    ///
    /// `read` method is used to read out measurements from the DMA ring buffer, and its buffer should be exactly half of the `dma_buf` length.
    /// It is critical to call `read` frequently to prevent DMA buffer overrun.
    ///
    /// [`read`]: #method.read
    pub fn into_ring_buffered(self, dma: Peri<'d, impl RxDma<T>>, dma_buf: &'d mut [u16]) -> RingBufferedAdc<'d, T> {
        assert!(!dma_buf.is_empty() && dma_buf.len() <= 0xFFFF);

        let opts: crate::dma::TransferOptions = TransferOptions {
            half_transfer_ir: true,
            priority: Priority::VeryHigh,
            ..Default::default()
        };

        // Safety: we forget the struct before this function returns.
        let rx_src = T::regs().dr().as_ptr() as *mut u16;
        let request = dma.request();

        let ring_buf = unsafe { ReadableRingBuffer::new(dma, request, rx_src, dma_buf, opts) };

        // Don't disable the clock
        mem::forget(self);

        RingBufferedAdc {
            _phantom: PhantomData,
            ring_buf,
        }
    }
}

impl<'d, T: Instance> RingBufferedAdc<'d, T> {
    fn is_on() -> bool {
        T::regs().cr2().read().adon()
    }

    fn stop_adc() {
        T::regs().cr2().modify(|reg| {
            reg.set_adon(false);
        });
    }

    fn start_adc() {
        T::regs().cr2().modify(|reg| {
            reg.set_adon(true);
        });
    }

    /// Sets the channel sample time
    ///
    /// ## SAFETY:
    /// - ADON == 0 i.e ADC must not be enabled when this is called.
    unsafe fn set_channel_sample_time(ch: u8, sample_time: SampleTime) {
        if ch <= 9 {
            T::regs().smpr2().modify(|reg| reg.set_smp(ch as _, sample_time));
        } else {
            T::regs().smpr1().modify(|reg| reg.set_smp((ch - 10) as _, sample_time));
        }
    }

    fn set_channels_sample_time(&mut self, ch: &[u8], sample_time: SampleTime) {
        let ch_iter = ch.iter();
        for idx in ch_iter {
            unsafe {
                Self::set_channel_sample_time(*idx, sample_time);
            }
        }
    }

    pub fn set_sample_sequence(
        &mut self,
        sequence: Sequence,
        channel: &mut impl AdcChannel<T>,
        sample_time: SampleTime,
    ) {
        let was_on = Self::is_on();
        if !was_on {
            Self::start_adc();
        }

        // Check the sequence is long enough
        T::regs().sqr1().modify(|r| {
            let prev: Sequence = r.l().into();
            if prev < sequence {
                let new_l: Sequence = sequence;
                trace!("Setting sequence length from {:?} to {:?}", prev as u8, new_l as u8);
                r.set_l(sequence.into())
            } else {
                r.set_l(prev.into())
            }
        });

        // Set this GPIO as an analog input.
        channel.setup();

        // Set the channel in the right sequence field.
        match sequence {
            Sequence::One => T::regs().sqr3().modify(|w| w.set_sq(0, channel.channel())),
            Sequence::Two => T::regs().sqr3().modify(|w| w.set_sq(1, channel.channel())),
            Sequence::Three => T::regs().sqr3().modify(|w| w.set_sq(2, channel.channel())),
            Sequence::Four => T::regs().sqr3().modify(|w| w.set_sq(3, channel.channel())),
            Sequence::Five => T::regs().sqr3().modify(|w| w.set_sq(4, channel.channel())),
            Sequence::Six => T::regs().sqr3().modify(|w| w.set_sq(5, channel.channel())),
            Sequence::Seven => T::regs().sqr2().modify(|w| w.set_sq(6, channel.channel())),
            Sequence::Eight => T::regs().sqr2().modify(|w| w.set_sq(7, channel.channel())),
            Sequence::Nine => T::regs().sqr2().modify(|w| w.set_sq(8, channel.channel())),
            Sequence::Ten => T::regs().sqr2().modify(|w| w.set_sq(9, channel.channel())),
            Sequence::Eleven => T::regs().sqr2().modify(|w| w.set_sq(10, channel.channel())),
            Sequence::Twelve => T::regs().sqr2().modify(|w| w.set_sq(11, channel.channel())),
            Sequence::Thirteen => T::regs().sqr1().modify(|w| w.set_sq(12, channel.channel())),
            Sequence::Fourteen => T::regs().sqr1().modify(|w| w.set_sq(13, channel.channel())),
            Sequence::Fifteen => T::regs().sqr1().modify(|w| w.set_sq(14, channel.channel())),
            Sequence::Sixteen => T::regs().sqr1().modify(|w| w.set_sq(15, channel.channel())),
        };

        if !was_on {
            Self::stop_adc();
        }

        self.set_channels_sample_time(&[channel.channel()], sample_time);

        Self::start_adc();
    }

    /// Turns on ADC if it is not already turned on and starts continuous DMA transfer.
    pub fn start(&mut self) -> Result<(), OverrunError> {
        self.setup_adc();
        self.ring_buf.clear();

        Ok(())
    }

    fn stop(&mut self, err: OverrunError) -> Result<usize, OverrunError> {
        self.teardown_adc();
        Err(err)
    }

    /// Stops DMA transfer.
    /// It does not turn off ADC.
    /// Calling `start` restarts continuous DMA transfer.
    ///
    /// [`start`]: #method.start
    pub fn teardown_adc(&mut self) {
        // Stop the DMA transfer
        self.ring_buf.request_pause();

        let r = T::regs();

        // Stop ADC
        r.cr2().modify(|reg| {
            // Stop ADC
            reg.set_swstart(false);
            // Stop DMA
            reg.set_dma(false);
        });

        r.cr1().modify(|w| {
            // Disable interrupt for end of conversion
            w.set_eocie(false);
            // Disable interrupt for overrun
            w.set_ovrie(false);
        });

        clear_interrupt_flags(r);

        compiler_fence(Ordering::SeqCst);
    }

    fn setup_adc(&mut self) {
        compiler_fence(Ordering::SeqCst);

        self.ring_buf.start();

        let r = T::regs();

        // Enable ADC
        let was_on = Self::is_on();
        if !was_on {
            r.cr2().modify(|reg| {
                reg.set_adon(false);
                reg.set_swstart(false);
            });
        }

        // Clear all interrupts
        r.sr().modify(|regs| {
            regs.set_eoc(false);
            regs.set_ovr(false);
            regs.set_strt(false);
        });

        r.cr1().modify(|w| {
            // Enable interrupt for end of conversion
            w.set_eocie(true);
            // Enable interrupt for overrun
            w.set_ovrie(true);
            // Scanning converisons of multiple channels
            w.set_scan(true);
            // Continuous conversion mode
            w.set_discen(false);
        });

        r.cr2().modify(|w| {
            // Enable DMA mode
            w.set_dma(true);
            // Enable continuous conversions
            w.set_cont(true);
            // DMA requests are issues as long as DMA=1 and data are converted.
            w.set_dds(vals::Dds::CONTINUOUS);
            // EOC flag is set at the end of each conversion.
            w.set_eocs(vals::Eocs::EACH_CONVERSION);
        });

        // Begin ADC conversions
        T::regs().cr2().modify(|reg| {
            reg.set_adon(true);
            reg.set_swstart(true);
        });

        super::blocking_delay_us(3);
    }

    /// Read bytes that are readily available in the ring buffer.
    /// If no bytes are currently available in the buffer the call waits until the some
    /// bytes are available (at least one byte and at most half the buffer size)
    ///
    /// Background receive is started if `start()` has not been previously called.
    ///
    /// Receive in the background is terminated if an error is returned.
    /// It must then manually be started again by calling `start()` or by re-calling `read()`.
    pub fn blocking_read<const N: usize>(&mut self, buf: &mut [u16; N]) -> Result<usize, OverrunError> {
        let r = T::regs();

        // Start background receive if it was not already started
        if !r.cr2().read().dma() {
            self.start()?;
        }

        // Clear overrun flag if set.
        if r.sr().read().ovr() {
            return self.stop(OverrunError);
        }

        loop {
            match self.ring_buf.read(buf) {
                Ok((0, _)) => {}
                Ok((len, _)) => {
                    return Ok(len);
                }
                Err(_) => {
                    return self.stop(OverrunError);
                }
            }
        }
    }

    /// Reads measurements from the DMA ring buffer.
    ///
    /// This method fills the provided `measurements` array with ADC readings from the DMA buffer.
    /// The length of the `measurements` array should be exactly half of the DMA buffer length. Because interrupts are only generated if half or full DMA transfer completes.
    ///
    /// Each call to `read` will populate the `measurements` array in the same order as the channels defined with `set_sample_sequence`.
    /// There will be many sequences worth of measurements in this array because it only returns if at least half of the DMA buffer is filled.
    /// For example if 3 channels are sampled `measurements` contain: `[sq0 sq1 sq3 sq0 sq1 sq3 sq0 sq1 sq3 sq0 sq1 sq3..]`.
    ///
    /// If an error is returned, it indicates a DMA overrun, and the process must be restarted by calling `start` or `read` again.
    ///
    /// By default, the ADC fills the DMA buffer as quickly as possible. To control the sample rate, call `teardown_adc` after each readout, and then start the DMA again at the desired interval.
    /// Note that even if using `teardown_adc` to control the sample rate, with each call to `read`, measurements equivalent to half the size of the DMA buffer are still collected.
    ///
    /// Example:
    /// ```rust,ignore
    /// const DMA_BUF_LEN: usize = 120;
    /// let adc_dma_buf = [0u16; DMA_BUF_LEN];
    /// let mut adc: RingBufferedAdc<embassy_stm32::peripherals::ADC1> = adc.into_ring_buffered(p.DMA2_CH0, adc_dma_buf);
    ///
    /// adc.set_sample_sequence(Sequence::One, &mut p.PA0, SampleTime::CYCLES112);
    /// adc.set_sample_sequence(Sequence::Two, &mut p.PA1, SampleTime::CYCLES112);
    /// adc.set_sample_sequence(Sequence::Three, &mut p.PA2, SampleTime::CYCLES112);
    ///
    /// let mut measurements = [0u16; DMA_BUF_LEN / 2];
    /// loop {
    ///     match adc.read(&mut measurements).await {
    ///         Ok(_) => {
    ///             defmt::info!("adc1: {}", measurements);
    ///             // Only needed to manually control sample rate.
    ///             adc.teardown_adc();
    ///         }
    ///         Err(e) => {
    ///             defmt::warn!("Error: {:?}", e);
    ///             // DMA overrun, next call to `read` restarts ADC.
    ///         }
    ///     }
    ///
    ///     // Manually control sample rate.
    ///     Timer::after_millis(100).await;
    /// }
    /// ```
    ///
    ///
    /// [`set_sample_sequence`]: #method.set_sample_sequence
    /// [`teardown_adc`]: #method.teardown_adc
    /// [`start`]: #method.start
    pub async fn read<const N: usize>(&mut self, measurements: &mut [u16; N]) -> Result<usize, OverrunError> {
        assert_eq!(
            self.ring_buf.capacity() / 2,
            N,
            "Buffer size must be half the size of the ring buffer"
        );

        let r = T::regs();

        // Start background receive if it was not already started
        if !r.cr2().read().dma() {
            self.start()?;
        }

        // Clear overrun flag if set.
        if r.sr().read().ovr() {
            return self.stop(OverrunError);
        }
        match self.ring_buf.read_exact(measurements).await {
            Ok(len) => Ok(len),
            Err(_) => self.stop(OverrunError),
        }
    }
}

impl<T: Instance> Drop for RingBufferedAdc<'_, T> {
    fn drop(&mut self) {
        self.teardown_adc();
        rcc::disable::<T>();
    }
}

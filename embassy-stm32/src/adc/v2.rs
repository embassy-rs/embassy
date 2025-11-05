use core::mem;
use core::sync::atomic::{Ordering, compiler_fence};

use super::blocking_delay_us;
use crate::adc::{Adc, AdcChannel, AnyAdcChannel, Instance, Resolution, RxDma, SampleTime, SealedAdcChannel};
use crate::pac::adc::vals;
use crate::peripherals::ADC1;
use crate::time::Hertz;
use crate::{Peri, rcc};

mod ringbuffered;
pub use ringbuffered::RingBufferedAdc;

fn clear_interrupt_flags(r: crate::pac::adc::Adc) {
    r.sr().modify(|regs| {
        regs.set_eoc(false);
        regs.set_ovr(false);
    });
}

/// Default VREF voltage used for sample conversion to millivolts.
pub const VREF_DEFAULT_MV: u32 = 3300;
/// VREF voltage used for factory calibration of VREFINTCAL register.
pub const VREF_CALIB_MV: u32 = 3300;

pub struct VrefInt;
impl AdcChannel<ADC1> for VrefInt {}
impl super::SealedAdcChannel<ADC1> for VrefInt {
    fn channel(&self) -> u8 {
        17
    }
}

impl VrefInt {
    /// Time needed for internal voltage reference to stabilize
    pub fn start_time_us() -> u32 {
        10
    }
}

pub struct Temperature;
impl AdcChannel<ADC1> for Temperature {}
impl super::SealedAdcChannel<ADC1> for Temperature {
    fn channel(&self) -> u8 {
        cfg_if::cfg_if! {
            if #[cfg(any(stm32f2, stm32f40x, stm32f41x))] {
                16
            } else {
                18
            }
        }
    }
}

impl Temperature {
    /// Time needed for temperature sensor readings to stabilize
    pub fn start_time_us() -> u32 {
        10
    }
}

pub struct Vbat;
impl AdcChannel<ADC1> for Vbat {}
impl super::SealedAdcChannel<ADC1> for Vbat {
    fn channel(&self) -> u8 {
        18
    }
}

enum Prescaler {
    Div2,
    Div4,
    Div6,
    Div8,
}

impl Prescaler {
    fn from_pclk2(freq: Hertz) -> Self {
        // Datasheet for F2 specifies min frequency 0.6 MHz, and max 30 MHz (with VDDA 2.4-3.6V).
        #[cfg(stm32f2)]
        const MAX_FREQUENCY: Hertz = Hertz(30_000_000);
        // Datasheet for both F4 and F7 specifies min frequency 0.6 MHz, typ freq. 30 MHz and max 36 MHz.
        #[cfg(not(stm32f2))]
        const MAX_FREQUENCY: Hertz = Hertz(36_000_000);
        let raw_div = freq.0 / MAX_FREQUENCY.0;
        match raw_div {
            0..=1 => Self::Div2,
            2..=3 => Self::Div4,
            4..=5 => Self::Div6,
            6..=7 => Self::Div8,
            _ => panic!("Selected PCLK2 frequency is too high for ADC with largest possible prescaler."),
        }
    }

    fn adcpre(&self) -> crate::pac::adccommon::vals::Adcpre {
        match self {
            Prescaler::Div2 => crate::pac::adccommon::vals::Adcpre::DIV2,
            Prescaler::Div4 => crate::pac::adccommon::vals::Adcpre::DIV4,
            Prescaler::Div6 => crate::pac::adccommon::vals::Adcpre::DIV6,
            Prescaler::Div8 => crate::pac::adccommon::vals::Adcpre::DIV8,
        }
    }
}

impl<'d, T> Adc<'d, T>
where
    T: Instance,
{
    pub fn new(adc: Peri<'d, T>) -> Self {
        rcc::enable_and_reset::<T>();

        let presc = Prescaler::from_pclk2(T::frequency());
        T::common_regs().ccr().modify(|w| w.set_adcpre(presc.adcpre()));
        T::regs().cr2().modify(|reg| {
            reg.set_adon(true);
        });

        blocking_delay_us(3);

        Self {
            adc,
            sample_time: SampleTime::from_bits(0),
        }
    }

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
    pub fn into_ring_buffered<'a>(
        self,
        dma: Peri<'d, impl RxDma<T>>,
        dma_buf: &'d mut [u16],
        sequence: impl ExactSizeIterator<Item = (&'a mut AnyAdcChannel<T>, SampleTime)>,
    ) -> RingBufferedAdc<'d, T> {
        assert!(!dma_buf.is_empty() && dma_buf.len() <= 0xFFFF);

        T::regs().cr2().modify(|reg| {
            reg.set_adon(true);
        });

        // Check the sequence is long enough
        T::regs().sqr1().modify(|r| {
            r.set_l((sequence.len() - 1).try_into().unwrap());
        });

        for (i, (channel, sample_time)) in sequence.enumerate() {
            // Set this GPIO as an analog input.
            channel.setup();

            // Set the channel in the right sequence field.
            T::regs().sqr3().modify(|w| w.set_sq(i, channel.channel()));

            Self::set_channel_sample_time(channel.channel(), sample_time);
        }

        compiler_fence(Ordering::SeqCst);

        let r = T::regs();

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

        // Don't disable the clock
        mem::forget(self);

        RingBufferedAdc::new(dma, dma_buf)
    }

    pub(super) fn start() {
        // Begin ADC conversions
        T::regs().cr2().modify(|reg| {
            reg.set_adon(true);
            reg.set_swstart(true);
        });
    }

    pub(super) fn stop() {
        // Stop ADC
        T::regs().cr2().modify(|reg| {
            // Stop ADC
            reg.set_swstart(false);
        });
    }

    pub fn set_sample_time(&mut self, sample_time: SampleTime) {
        self.sample_time = sample_time;
    }

    pub fn set_resolution(&mut self, resolution: Resolution) {
        T::regs().cr1().modify(|reg| reg.set_res(resolution.into()));
    }

    /// Enables internal voltage reference and returns [VrefInt], which can be used in
    /// [Adc::read_internal()] to perform conversion.
    pub fn enable_vrefint(&self) -> VrefInt {
        T::common_regs().ccr().modify(|reg| {
            reg.set_tsvrefe(true);
        });

        VrefInt {}
    }

    /// Enables internal temperature sensor and returns [Temperature], which can be used in
    /// [Adc::read_internal()] to perform conversion.
    ///
    /// On STM32F42 and STM32F43 this can not be used together with [Vbat]. If both are enabled,
    /// temperature sensor will return vbat value.
    pub fn enable_temperature(&self) -> Temperature {
        T::common_regs().ccr().modify(|reg| {
            reg.set_tsvrefe(true);
        });

        Temperature {}
    }

    /// Enables vbat input and returns [Vbat], which can be used in
    /// [Adc::read_internal()] to perform conversion.
    pub fn enable_vbat(&self) -> Vbat {
        T::common_regs().ccr().modify(|reg| {
            reg.set_vbate(true);
        });

        Vbat {}
    }

    /// Perform a single conversion.
    fn convert(&mut self) -> u16 {
        // clear end of conversion flag
        T::regs().sr().modify(|reg| {
            reg.set_eoc(false);
        });

        // Start conversion
        T::regs().cr2().modify(|reg| {
            reg.set_swstart(true);
        });

        while T::regs().sr().read().strt() == false {
            // spin //wait for actual start
        }
        while T::regs().sr().read().eoc() == false {
            // spin //wait for finish
        }

        T::regs().dr().read().0 as u16
    }

    pub fn blocking_read(&mut self, channel: &mut impl AdcChannel<T>) -> u16 {
        channel.setup();

        // Configure ADC
        let channel = channel.channel();

        // Select channel
        T::regs().sqr3().write(|reg| reg.set_sq(0, channel));

        // Configure channel
        Self::set_channel_sample_time(channel, self.sample_time);

        self.convert()
    }

    fn set_channel_sample_time(ch: u8, sample_time: SampleTime) {
        let sample_time = sample_time.into();
        if ch <= 9 {
            T::regs().smpr2().modify(|reg| reg.set_smp(ch as _, sample_time));
        } else {
            T::regs().smpr1().modify(|reg| reg.set_smp((ch - 10) as _, sample_time));
        }
    }

    pub(super) fn teardown_adc() {
        let r = T::regs();

        // Stop ADC
        r.cr2().modify(|reg| {
            // Stop ADC
            reg.set_swstart(false);
            // Stop ADC
            reg.set_adon(false);
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
}

impl<'d, T: Instance> Drop for Adc<'d, T> {
    fn drop(&mut self) {
        T::regs().cr2().modify(|reg| {
            reg.set_adon(false);
        });

        rcc::disable::<T>();
    }
}

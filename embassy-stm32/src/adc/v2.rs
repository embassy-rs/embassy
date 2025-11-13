use core::sync::atomic::{Ordering, compiler_fence};

use super::{ConversionMode, Temperature, Vbat, VrefInt, blocking_delay_us};
use crate::adc::{Adc, Instance, Resolution, SampleTime};
use crate::pac::adc::vals;
pub use crate::pac::adccommon::vals::Adcpre;
use crate::time::Hertz;
use crate::{Peri, rcc};

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

impl super::SealedSpecialConverter<super::VrefInt> for crate::peripherals::ADC1 {
    const CHANNEL: u8 = 17;
}

#[cfg(any(stm32f2, stm32f40x, stm32f41x))]
impl super::SealedSpecialConverter<super::Temperature> for crate::peripherals::ADC1 {
    const CHANNEL: u8 = 16;
}

#[cfg(not(any(stm32f2, stm32f40x, stm32f41x)))]
impl super::SealedSpecialConverter<super::Temperature> for crate::peripherals::ADC1 {
    const CHANNEL: u8 = 18;
}

impl super::SealedSpecialConverter<super::Vbat> for crate::peripherals::ADC1 {
    const CHANNEL: u8 = 18;
}

impl VrefInt {
    /// Time needed for internal voltage reference to stabilize
    pub fn start_time_us() -> u32 {
        10
    }
}

impl Temperature {
    /// Time needed for temperature sensor readings to stabilize
    pub fn start_time_us() -> u32 {
        10
    }
}

fn from_pclk2(freq: Hertz) -> Adcpre {
    // Datasheet for F2 specifies min frequency 0.6 MHz, and max 30 MHz (with VDDA 2.4-3.6V).
    #[cfg(stm32f2)]
    const MAX_FREQUENCY: Hertz = Hertz(30_000_000);
    // Datasheet for both F4 and F7 specifies min frequency 0.6 MHz, typ freq. 30 MHz and max 36 MHz.
    #[cfg(not(stm32f2))]
    const MAX_FREQUENCY: Hertz = Hertz(36_000_000);
    let raw_div = freq.0 / MAX_FREQUENCY.0;
    match raw_div {
        0..=1 => Adcpre::DIV2,
        2..=3 => Adcpre::DIV4,
        4..=5 => Adcpre::DIV6,
        6..=7 => Adcpre::DIV8,
        _ => panic!("Selected PCLK2 frequency is too high for ADC with largest possible prescaler."),
    }
}

/// ADC configuration
#[derive(Default)]
pub struct AdcConfig {
    resolution: Option<Resolution>,
}

impl<T: Instance> super::SealedAnyInstance for T {
    fn dr() -> *mut u16 {
        T::regs().dr().as_ptr() as *mut u16
    }

    fn enable() {}

    fn start() {
        // Begin ADC conversions
        T::regs().cr2().modify(|reg| {
            reg.set_adon(true);
            reg.set_swstart(true);
        });
    }

    fn stop() {
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

    fn convert() -> u16 {
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

    fn configure_dma(conversion_mode: ConversionMode) {
        match conversion_mode {
            ConversionMode::Repeated(_) => {
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
            }
        }
    }

    fn configure_sequence(sequence: impl ExactSizeIterator<Item = ((u8, bool), SampleTime)>) {
        T::regs().cr2().modify(|reg| {
            reg.set_adon(true);
        });

        // Check the sequence is long enough
        T::regs().sqr1().modify(|r| {
            r.set_l((sequence.len() - 1).try_into().unwrap());
        });

        for (i, ((ch, _), sample_time)) in sequence.enumerate() {
            // Set the channel in the right sequence field.
            T::regs().sqr3().modify(|w| w.set_sq(i, ch));

            let sample_time = sample_time.into();
            if ch <= 9 {
                T::regs().smpr2().modify(|reg| reg.set_smp(ch as _, sample_time));
            } else {
                T::regs().smpr1().modify(|reg| reg.set_smp((ch - 10) as _, sample_time));
            }
        }
    }
}

impl<'d, T> Adc<'d, T>
where
    T: Instance,
{
    pub fn new(adc: Peri<'d, T>) -> Self {
        Self::new_with_config(adc, Default::default())
    }

    pub fn new_with_config(adc: Peri<'d, T>, config: AdcConfig) -> Self {
        rcc::enable_and_reset::<T>();

        let presc = from_pclk2(T::frequency());
        T::common_regs().ccr().modify(|w| w.set_adcpre(presc));
        T::regs().cr2().modify(|reg| {
            reg.set_adon(true);
        });

        blocking_delay_us(3);

        if let Some(resolution) = config.resolution {
            T::regs().cr1().modify(|reg| reg.set_res(resolution.into()));
        }

        Self { adc }
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
}

impl<'d, T: Instance> Drop for Adc<'d, T> {
    fn drop(&mut self) {
        T::regs().cr2().modify(|reg| {
            reg.set_adon(false);
        });

        rcc::disable::<T>();
    }
}

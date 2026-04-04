use core::sync::atomic::{Ordering, compiler_fence};

use super::{AnyAdcChannel, ConversionMode, Temperature, Vbat, VrefInt, blocking_delay_us};
use crate::adc::{Adc, AdcRegs, DefaultInstance, Instance, Resolution, SampleTime, SealedInjectedAdcRegs};
use crate::pac::adc::vals;
pub use crate::pac::adccommon::vals::Adcpre;
use crate::time::Hertz;
use crate::{Peri, rcc};

mod injected;
pub use injected::InjectedAdc;

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

pub const NR_INJECTED_RANKS: usize = 4;

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
    let raw_div = rcc::raw_prescaler(freq.0, MAX_FREQUENCY.0);
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
    pub resolution: Option<Resolution>,
}

impl AdcRegs for crate::pac::adc::Adc {
    fn data(&self) -> *mut u16 {
        crate::pac::adc::Adc::dr(*self).as_ptr() as *mut u16
    }

    fn enable(&self) {
        self.cr2().modify(|reg| {
            reg.set_adon(true);
        });

        blocking_delay_us(3);
    }

    fn start(&self) {
        // Begin ADC conversions
        self.cr2().modify(|reg| {
            reg.set_swstart(true);
        });
    }

    fn stop(&self) {
        let r = self;

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

        clear_interrupt_flags(*r);

        compiler_fence(Ordering::SeqCst);
    }

    fn convert(&self) {
        // clear end of conversion flag
        self.sr().modify(|reg| {
            reg.set_eoc(false);
        });

        // Start conversion
        self.cr2().modify(|reg| {
            reg.set_swstart(true);
        });

        while self.sr().read().strt() == false {
            // spin //wait for actual start
        }
        while self.sr().read().eoc() == false {
            // spin //wait for finish
        }
    }

    fn configure_dma(&self, conversion_mode: ConversionMode) {
        let r = self;

        // Clear all status flags before configuring DMA.
        r.sr().modify(|regs| {
            regs.set_eoc(false);
            regs.set_ovr(false);
            regs.set_strt(false);
        });
        let is_repeated = matches!(conversion_mode, ConversionMode::Repeated(_));
        r.cr1().modify(|w| {
            // Enable end of conversion interrupt only in repeated mode.
            w.set_eocie(is_repeated);
            // Enable overrun interrupt only in repeated mode.
            w.set_ovrie(is_repeated);
            // Scanning conversions of multiple channels.
            w.set_scan(true);
            // Disable discontinuous mode.
            w.set_discen(false);
        });

        r.cr2().modify(|w| {
            // Enable DMA mode
            w.set_dma(true);
            w.set_dds(match conversion_mode {
                // SINGLE: DMA requests stop after the sequence completes (one-shot).
                ConversionMode::Singular => vals::Dds::SINGLE,
                // CONTINUOUS: DMA stays armed between reads; cont=false below limits
                // the ADC to one sequence per SWSTART.
                ConversionMode::ConfiguredSequence => vals::Dds::CONTINUOUS,
                // CONTINUOUS: DMA requests keep being issued as long as DMA=1 and data are converted.
                ConversionMode::Repeated(_) => vals::Dds::CONTINUOUS,
            });
            // EOC flag is set at the end of each conversion.
            w.set_eocs(vals::Eocs::EACH_CONVERSION);
        });

        match conversion_mode {
            ConversionMode::Singular | ConversionMode::ConfiguredSequence => {
                r.cr2().modify(|w| w.set_cont(false));
            }
            ConversionMode::Repeated(trigger) => match trigger {
                None => {
                    // continuous conversion
                    r.cr2().modify(|w| {
                        // Enable continuous conversions
                        w.set_cont(true);
                    });
                }
                Some((signal, edge)) => {
                    r.cr2().modify(|w| {
                        // Disable continuous conversions
                        w.set_cont(false);
                        // Trigger detection edge
                        w.set_exten(edge);
                        // Trigger channel
                        w.set_extsel(signal);
                    })
                }
            },
        }
    }

    fn configure_sequence(&self, sequence: impl ExactSizeIterator<Item = ((u8, bool), SampleTime)>) {
        // Check the sequence is long enough
        self.sqr1().modify(|r| {
            r.set_l((sequence.len() - 1).try_into().unwrap());
        });

        for (i, ((ch, _), sample_time)) in sequence.enumerate() {
            match i {
                0..=5 => self.sqr3().modify(|w| w.set_sq(i, ch)),
                6..=11 => self.sqr2().modify(|w| w.set_sq(i - 6, ch)),
                12..=15 => self.sqr1().modify(|w| w.set_sq(i - 12, ch)),
                _ => unreachable!(),
            }

            let sample_time = sample_time.into();
            if ch <= 9 {
                self.smpr2().modify(|reg| reg.set_smp(ch as _, sample_time));
            } else {
                self.smpr1().modify(|reg| reg.set_smp((ch - 10) as _, sample_time));
            }
        }
    }
}

impl SealedInjectedAdcRegs for crate::pac::adc::Adc {
    fn configure_injected_sequence(&self, sequence: impl ExactSizeIterator<Item = ((u8, bool), Self::SampleTime)>) {
        let len: u8 = sequence.len().try_into().unwrap();
        self.cr1().modify(|w| w.set_jauto(false));
        // Set injected sequence length
        self.jsqr().modify(|w| w.set_jl(len - 1));

        for (n, ((channel, _), sample_time)) in sequence.enumerate() {
            let sample_time = sample_time.clone().into();
            if channel <= 9 {
                self.smpr2().modify(|reg| reg.set_smp(channel as _, sample_time));
            } else {
                self.smpr1().modify(|reg| reg.set_smp((channel - 10) as _, sample_time));
            }

            // On adc_v2/F4, injected JSQ rank field placement depends on the
            // programmed sequence length (JL). ST's HAL uses:
            //   shift = 5 * ((rank + 3) - sequence_len)
            // with rank starting at 1.
            let idx = n + (4usize - len as usize);

            self.jsqr().modify(|w| w.set_jsq(idx, channel));
        }
    }

    fn configure_injected_trigger(&self, trigger: (u8, vals::Exten), interrupt: bool) {
        self.cr1().modify(|w| {
            w.set_scan(true);
            w.set_jdiscen(false);
            w.set_jeocie(interrupt);
        });
        self.cr2().modify(|w| {
            w.set_jextsel(trigger.0);
            w.set_jexten(trigger.1);
        });
    }

    fn start_injected(&self) {
        self.sr().modify(|w| {
            w.set_jeoc(false);
            w.set_jstrt(false);
        });

        // On STM32F4 adc_v2, externally-triggered injected conversions are armed
        // by JEXTEN and start on the next trigger event. JSWSTART is only valid
        // for pure software-triggered injected conversions.
        if self.cr2().read().jexten() == vals::Exten::DISABLED {
            self.cr2().modify(|w| w.set_jswstart(true));
        }
    }

    fn stop_injected(&self) {
        // No true "abort injected conversion" primitive on adc_v2.
        // Best practical stop: disable external injected triggering.
        self.cr2().modify(|w| w.set_jexten(vals::Exten::DISABLED));
        self.cr1().modify(|w| w.set_jeocie(false));
        self.sr().modify(|w| {
            w.set_jeoc(false);
            w.set_jstrt(false);
        });
    }

    fn read_injected(&self, data: &mut [u16]) {
        for (i, d) in data.iter_mut().enumerate() {
            *d = self.jdr(i).read().jdata();
        }

        // Clear JEOC and JSTRT
        self.sr().modify(|w| {
            w.set_jeoc(false);
            w.set_jstrt(false);
        });
    }
}

impl<'d, T: DefaultInstance> Adc<'d, T> {
    pub fn new(adc: Peri<'d, T>) -> Self {
        Self::new_with_config(adc, Default::default())
    }

    pub fn new_with_config(adc: Peri<'d, T>, config: AdcConfig) -> Self {
        rcc::enable_and_reset::<T>();

        let presc = from_pclk2(T::frequency());
        T::common_regs().ccr().modify(|w| w.set_adcpre(presc));
        T::regs().enable();

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
        T::regs().stop();

        rcc::disable::<T>();
    }
}

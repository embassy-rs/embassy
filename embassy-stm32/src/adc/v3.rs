use cfg_if::cfg_if;
#[cfg(adc_g0)]
use heapless::Vec;
#[cfg(adc_g0)]
use pac::adc::vals::Ckmode;
use pac::adc::vals::Dmacfg;
#[cfg(adc_v3)]
use pac::adc::vals::{OversamplingRatio, OversamplingShift, Rovsm, Trovs};
#[cfg(adc_g0)]
pub use pac::adc::vals::{Ovsr, Ovss, Presc};

#[allow(unused_imports)]
use super::SealedAdcChannel;
use super::{Adc, Averaging, Instance, Resolution, SampleTime, Temperature, Vbat, VrefInt, blocking_delay_us};
use crate::adc::ConversionMode;
use crate::{Peri, pac, rcc};

/// Default VREF voltage used for sample conversion to millivolts.
pub const VREF_DEFAULT_MV: u32 = 3300;
/// VREF voltage used for factory calibration of VREFINTCAL register.
pub const VREF_CALIB_MV: u32 = 3000;

#[cfg(adc_g0)]
/// The number of variants in Smpsel
// TODO: Use [#![feature(variant_count)]](https://github.com/rust-lang/rust/issues/73662) when stable
const SAMPLE_TIMES_CAPACITY: usize = 2;

#[cfg(adc_g0)]
impl<T: Instance> super::SealedSpecialConverter<super::VrefInt> for T {
    const CHANNEL: u8 = 13;
}
#[cfg(any(adc_h5, adc_h7rs))]
impl<T: Instance> super::SealedSpecialConverter<super::VrefInt> for T {
    const CHANNEL: u8 = 17;
}
#[cfg(adc_u0)]
impl<T: Instance> super::SealedSpecialConverter<super::VrefInt> for T {
    const CHANNEL: u8 = 12;
}
#[cfg(not(any(adc_g0, adc_h5, adc_h7rs, adc_u0)))]
impl<T: Instance> super::SealedSpecialConverter<super::VrefInt> for T {
    const CHANNEL: u8 = 0;
}

#[cfg(adc_g0)]
impl<T: Instance> super::SealedSpecialConverter<super::Temperature> for T {
    const CHANNEL: u8 = 12;
}
#[cfg(any(adc_h5, adc_h7rs))]
impl<T: Instance> super::SealedSpecialConverter<super::Temperature> for T {
    const CHANNEL: u8 = 16;
}
#[cfg(adc_u0)]
impl<T: Instance> super::SealedSpecialConverter<super::Temperature> for T {
    const CHANNEL: u8 = 11;
}
#[cfg(not(any(adc_g0, adc_h5, adc_h7rs, adc_u0)))]
impl<T: Instance> super::SealedSpecialConverter<super::Temperature> for T {
    const CHANNEL: u8 = 17;
}

#[cfg(adc_g0)]
impl<T: Instance> super::SealedSpecialConverter<super::Vbat> for T {
    const CHANNEL: u8 = 14;
}
#[cfg(any(adc_h5, adc_h7rs))]
impl<T: Instance> super::SealedSpecialConverter<super::Vbat> for T {
    const CHANNEL: u8 = 2;
}
#[cfg(adc_u0)]
impl<T: Instance> super::SealedSpecialConverter<super::Vbat> for T {
    const CHANNEL: u8 = 13;
}
#[cfg(not(any(adc_g0, adc_h5, adc_h7rs, adc_u0)))]
impl<T: Instance> super::SealedSpecialConverter<super::Vbat> for T {
    const CHANNEL: u8 = 18;
}

cfg_if! {
    if #[cfg(any(adc_h5, adc_h7rs))] {
        pub struct VddCore;
        impl<T: Instance> super::AdcChannel<T> for VddCore {}
        impl<T: Instance> super::SealedAdcChannel<T> for VddCore {
            fn channel(&self) -> u8 {
                6
            }
        }
    }
}

cfg_if! {
    if #[cfg(adc_u0)] {
        pub struct DacOut;
        impl<T: Instance> super::AdcChannel<T> for DacOut {}
        impl<T: Instance> super::SealedAdcChannel<T> for DacOut {
            fn channel(&self) -> u8 {
                19
            }
        }
    }
}

cfg_if! { if #[cfg(adc_g0)] {

/// Synchronous PCLK prescaler
pub enum CkModePclk {
    DIV1,
    DIV2,
    DIV4,
}

/// The analog clock is either the synchronous prescaled PCLK or
/// the asynchronous prescaled ADCCLK configured by the RCC mux.
/// The data sheet states the maximum analog clock frequency -
/// for STM32WL55CC it is 36 MHz.
pub enum Clock {
    Sync { div: CkModePclk },
    Async { div: Presc },
}

}}

#[cfg(adc_u0)]
type Ovss = u8;
#[cfg(adc_u0)]
type Ovsr = u8;
#[cfg(adc_v3)]
type Ovss = OversamplingShift;
#[cfg(adc_v3)]
type Ovsr = OversamplingRatio;

/// Adc configuration
#[derive(Default)]
pub struct AdcConfig {
    #[cfg(any(adc_u0, adc_g0, adc_v3))]
    pub oversampling_shift: Option<Ovss>,
    #[cfg(any(adc_u0, adc_g0, adc_v3))]
    pub oversampling_ratio: Option<Ovsr>,
    #[cfg(any(adc_u0, adc_g0))]
    pub oversampling_enable: Option<bool>,
    #[cfg(adc_v3)]
    pub oversampling_mode: Option<(Rovsm, Trovs, bool)>,
    #[cfg(adc_g0)]
    pub clock: Option<Clock>,
    pub resolution: Option<Resolution>,
    pub averaging: Option<Averaging>,
}

impl<T: Instance> super::SealedAnyInstance for T {
    fn dr() -> *mut u16 {
        T::regs().dr().as_ptr() as *mut u16
    }

    // Enable ADC only when it is not already running.
    fn enable() {
        // Make sure bits are off
        while T::regs().cr().read().addis() {
            // spin
        }

        if !T::regs().cr().read().aden() {
            // Enable ADC
            T::regs().isr().modify(|reg| {
                reg.set_adrdy(true);
            });
            T::regs().cr().modify(|reg| {
                reg.set_aden(true);
            });

            while !T::regs().isr().read().adrdy() {
                // spin
            }
        }
    }

    fn start() {
        T::regs().cr().modify(|reg| {
            reg.set_adstart(true);
        });
    }

    fn stop() {
        // Ensure conversions are finished.
        if T::regs().cr().read().adstart() && !T::regs().cr().read().addis() {
            T::regs().cr().modify(|reg| {
                reg.set_adstp(true);
            });
            while T::regs().cr().read().adstart() {}
        }

        // Reset configuration.
        #[cfg(not(any(adc_g0, adc_u0)))]
        T::regs().cfgr().modify(|reg| {
            reg.set_cont(false);
            reg.set_dmaen(false);
        });
        #[cfg(any(adc_g0, adc_u0))]
        T::regs().cfgr1().modify(|reg| {
            reg.set_cont(false);
            reg.set_dmaen(false);
        });
    }

    /// Perform a single conversion.
    fn convert() -> u16 {
        // Some models are affected by an erratum:
        // If we perform conversions slower than 1 kHz, the first read ADC value can be
        // corrupted, so we discard it and measure again.
        //
        // STM32L471xx: Section 2.7.3
        // STM32G4: Section 2.7.3
        #[cfg(any(rcc_l4, rcc_g4))]
        let len = 2;

        #[cfg(not(any(rcc_l4, rcc_g4)))]
        let len = 1;

        for _ in 0..len {
            T::regs().isr().modify(|reg| {
                reg.set_eos(true);
                reg.set_eoc(true);
            });

            // Start conversion
            T::regs().cr().modify(|reg| {
                reg.set_adstart(true);
            });

            while !T::regs().isr().read().eos() {
                // spin
            }
        }

        T::regs().dr().read().0 as u16
    }

    fn configure_dma(conversion_mode: ConversionMode) {
        // Set continuous mode with oneshot dma.
        // Clear overrun flag before starting transfer.
        T::regs().isr().modify(|reg| {
            reg.set_ovr(true);
        });

        #[cfg(not(any(adc_g0, adc_u0)))]
        let regs = T::regs().cfgr();

        #[cfg(any(adc_g0, adc_u0))]
        let regs = T::regs().cfgr1();

        regs.modify(|reg| {
            reg.set_discen(false);
            reg.set_cont(true);
            reg.set_dmacfg(match conversion_mode {
                ConversionMode::Singular => Dmacfg::ONE_SHOT,
                #[cfg(any(adc_v2, adc_g4, adc_v3, adc_g0, adc_u0))]
                ConversionMode::Repeated(_) => Dmacfg::CIRCULAR,
            });
            reg.set_dmaen(true);
        });
    }

    fn configure_sequence(sequence: impl ExactSizeIterator<Item = ((u8, bool), SampleTime)>) {
        // Set sequence length
        #[cfg(not(any(adc_g0, adc_u0)))]
        T::regs().sqr1().modify(|w| {
            w.set_l(sequence.len() as u8 - 1);
        });

        #[cfg(adc_g0)]
        {
            let mut sample_times = Vec::<SampleTime, SAMPLE_TIMES_CAPACITY>::new();

            T::regs().chselr().write(|chselr| {
                T::regs().smpr().write(|smpr| {
                    for ((channel, _), sample_time) in sequence {
                        chselr.set_chsel(channel.into(), true);
                        if let Some(i) = sample_times.iter().position(|&t| t == sample_time) {
                            smpr.set_smpsel(channel.into(), (i as u8).into());
                        } else {
                            smpr.set_sample_time(sample_times.len(), sample_time);
                            if let Err(_) = sample_times.push(sample_time) {
                                panic!(
                                    "Implementation is limited to {} unique sample times among all channels.",
                                    SAMPLE_TIMES_CAPACITY
                                );
                            }
                        }
                    }
                })
            });
        }
        #[cfg(not(adc_g0))]
        {
            #[cfg(adc_u0)]
            let mut channel_mask = 0;

            // Configure channels and ranks
            for (_i, ((channel, _), sample_time)) in sequence.enumerate() {
                // RM0492, RM0481, etc.
                // "This option bit must be set to 1 when ADCx_INP0 or ADCx_INN1 channel is selected."
                #[cfg(any(adc_h5, adc_h7rs))]
                if channel == 0 {
                    T::regs().or().modify(|reg| reg.set_op0(true));
                }

                // Configure channel
                cfg_if! {
                    if #[cfg(adc_u0)] {
                        // On G0 and U6 all channels use the same sampling time.
                        T::regs().smpr().modify(|reg| reg.set_smp1(sample_time.into()));
                    } else if #[cfg(any(adc_h5, adc_h7rs))] {
                        match channel {
                            0..=9 => T::regs().smpr1().modify(|w| w.set_smp(channel as usize % 10, sample_time.into())),
                            _ => T::regs().smpr2().modify(|w| w.set_smp(channel as usize % 10, sample_time.into())),
                        }
                    } else {
                        let sample_time = sample_time.into();
                        T::regs()
                            .smpr(channel as usize / 10)
                            .modify(|reg| reg.set_smp(channel as usize % 10, sample_time));
                    }
                }

                #[cfg(stm32h7)]
                {
                    use crate::pac::adc::vals::Pcsel;

                    T::regs().cfgr2().modify(|w| w.set_lshift(0));
                    T::regs()
                        .pcsel()
                        .write(|w| w.set_pcsel(channel.channel() as _, Pcsel::PRESELECTED));
                }

                // Each channel is sampled according to sequence
                #[cfg(not(any(adc_g0, adc_u0)))]
                match _i {
                    0..=3 => {
                        T::regs().sqr1().modify(|w| {
                            w.set_sq(_i, channel);
                        });
                    }
                    4..=8 => {
                        T::regs().sqr2().modify(|w| {
                            w.set_sq(_i - 4, channel);
                        });
                    }
                    9..=13 => {
                        T::regs().sqr3().modify(|w| {
                            w.set_sq(_i - 9, channel);
                        });
                    }
                    14..=15 => {
                        T::regs().sqr4().modify(|w| {
                            w.set_sq(_i - 14, channel);
                        });
                    }
                    _ => unreachable!(),
                }

                #[cfg(adc_u0)]
                {
                    channel_mask |= 1 << channel;
                }
            }

            // On G0 and U0 enabled channels are sampled from 0 to last channel.
            // It is possible to add up to 8 sequences if CHSELRMOD = 1.
            // However for supporting more than 8 channels alternative CHSELRMOD = 0 approach is used.
            #[cfg(adc_u0)]
            T::regs().chselr().modify(|reg| {
                reg.set_chsel(channel_mask);
            });
        }
    }
}

impl<'d, T: Instance> Adc<'d, T> {
    /// Enable the voltage regulator
    fn init_regulator() {
        rcc::enable_and_reset::<T>();
        T::regs().cr().modify(|reg| {
            #[cfg(not(any(adc_g0, adc_u0)))]
            reg.set_deeppwd(false);
            reg.set_advregen(true);
        });

        // If this is false then each ADC_CHSELR bit enables an input channel.
        // This is the reset value, so has no effect.
        #[cfg(any(adc_g0, adc_u0))]
        T::regs().cfgr1().modify(|reg| {
            reg.set_chselrmod(false);
        });

        blocking_delay_us(20);
    }

    /// Calibrate to remove conversion offset
    fn init_calibrate() {
        T::regs().cr().modify(|reg| {
            reg.set_adcal(true);
        });

        while T::regs().cr().read().adcal() {
            // spin
        }

        blocking_delay_us(1);
    }

    /// Initialize the ADC leaving any analog clock at reset value.
    /// For G0 and WL, this is the async clock without prescaler.
    pub fn new(adc: Peri<'d, T>) -> Self {
        Self::init_regulator();
        Self::init_calibrate();
        Self { adc }
    }

    pub fn new_with_config(adc: Peri<'d, T>, config: AdcConfig) -> Self {
        #[cfg(not(adc_g0))]
        let s = Self::new(adc);

        #[cfg(adc_g0)]
        let s = match config.clock {
            Some(clock) => Self::new_with_clock(adc, clock),
            None => Self::new(adc),
        };

        #[cfg(any(adc_g0, adc_u0, adc_v3))]
        if let Some(shift) = config.oversampling_shift {
            T::regs().cfgr2().modify(|reg| reg.set_ovss(shift));
        }

        #[cfg(any(adc_g0, adc_u0, adc_v3))]
        if let Some(ratio) = config.oversampling_ratio {
            T::regs().cfgr2().modify(|reg| reg.set_ovsr(ratio));
        }

        #[cfg(any(adc_g0, adc_u0))]
        if let Some(enable) = config.oversampling_enable {
            T::regs().cfgr2().modify(|reg| reg.set_ovse(enable));
        }

        #[cfg(adc_v3)]
        if let Some((mode, trig_mode, enable)) = config.oversampling_mode {
            T::regs().cfgr2().modify(|reg| reg.set_trovs(trig_mode));
            T::regs().cfgr2().modify(|reg| reg.set_rovsm(mode));
            T::regs().cfgr2().modify(|reg| reg.set_rovse(enable));
        }

        if let Some(resolution) = config.resolution {
            #[cfg(not(any(adc_g0, adc_u0)))]
            T::regs().cfgr().modify(|reg| reg.set_res(resolution.into()));
            #[cfg(any(adc_g0, adc_u0))]
            T::regs().cfgr1().modify(|reg| reg.set_res(resolution.into()));
        }

        if let Some(averaging) = config.averaging {
            let (enable, samples, right_shift) = match averaging {
                Averaging::Disabled => (false, 0, 0),
                Averaging::Samples2 => (true, 0, 1),
                Averaging::Samples4 => (true, 1, 2),
                Averaging::Samples8 => (true, 2, 3),
                Averaging::Samples16 => (true, 3, 4),
                Averaging::Samples32 => (true, 4, 5),
                Averaging::Samples64 => (true, 5, 6),
                Averaging::Samples128 => (true, 6, 7),
                Averaging::Samples256 => (true, 7, 8),
            };
            T::regs().cfgr2().modify(|reg| {
                #[cfg(not(any(adc_g0, adc_u0)))]
                reg.set_rovse(enable);
                #[cfg(any(adc_g0, adc_u0))]
                reg.set_ovse(enable);
                #[cfg(any(adc_h5, adc_h7rs))]
                reg.set_ovsr(samples.into());
                #[cfg(not(any(adc_h5, adc_h7rs)))]
                reg.set_ovsr(samples.into());
                reg.set_ovss(right_shift.into());
            })
        }

        s
    }

    #[cfg(adc_g0)]
    /// Initialize ADC with explicit clock for the analog ADC
    pub fn new_with_clock(adc: Peri<'d, T>, clock: Clock) -> Self {
        Self::init_regulator();

        #[cfg(any(stm32wl5x))]
        {
            // Reset value 0 is actually _No clock selected_ in the STM32WL5x reference manual
            let async_clock_available = pac::RCC.ccipr().read().adcsel() != pac::rcc::vals::Adcsel::_RESERVED_0;
            match clock {
                Clock::Async { div: _ } => {
                    assert!(async_clock_available);
                }
                Clock::Sync { div: _ } => {
                    if async_clock_available {
                        warn!("Not using configured ADC clock");
                    }
                }
            }
        }
        match clock {
            Clock::Async { div } => T::regs().ccr().modify(|reg| reg.set_presc(div)),
            Clock::Sync { div } => T::regs().cfgr2().modify(|reg| {
                reg.set_ckmode(match div {
                    CkModePclk::DIV1 => Ckmode::PCLK,
                    CkModePclk::DIV2 => Ckmode::PCLK_DIV2,
                    CkModePclk::DIV4 => Ckmode::PCLK_DIV4,
                })
            }),
        }

        Self::init_calibrate();

        Self { adc }
    }

    pub fn enable_vrefint(&self) -> VrefInt {
        #[cfg(not(any(adc_g0, adc_u0)))]
        T::common_regs().ccr().modify(|reg| {
            reg.set_vrefen(true);
        });
        #[cfg(any(adc_g0, adc_u0))]
        T::regs().ccr().modify(|reg| {
            reg.set_vrefen(true);
        });

        // "Table 24. Embedded internal voltage reference" states that it takes a maximum of 12 us
        // to stabilize the internal voltage reference.
        blocking_delay_us(15);

        VrefInt {}
    }

    pub fn enable_temperature(&self) -> Temperature {
        cfg_if! {
            if #[cfg(any(adc_g0, adc_u0))] {
                T::regs().ccr().modify(|reg| {
                    reg.set_tsen(true);
                });
            } else if #[cfg(any(adc_h5, adc_h7rs))] {
                T::common_regs().ccr().modify(|reg| {
                    reg.set_tsen(true);
                });
            } else {
                T::common_regs().ccr().modify(|reg| {
                    reg.set_ch17sel(true);
                });
            }
        }

        Temperature {}
    }

    pub fn enable_vbat(&self) -> Vbat {
        cfg_if! {
            if #[cfg(any(adc_g0, adc_u0))] {
                T::regs().ccr().modify(|reg| {
                    reg.set_vbaten(true);
                });
            } else if #[cfg(any(adc_h5, adc_h7rs))] {
                T::common_regs().ccr().modify(|reg| {
                    reg.set_vbaten(true);
                });
            } else {
                T::common_regs().ccr().modify(|reg| {
                    reg.set_ch18sel(true);
                });
            }
        }

        Vbat {}
    }

    /*
    /// Convert a raw sample from the `Temperature` to deg C
    pub fn to_degrees_centigrade(sample: u16) -> f32 {
        (130.0 - 30.0) / (VtempCal130::get().read() as f32 - VtempCal30::get().read() as f32)
            * (sample as f32 - VtempCal30::get().read() as f32)
            + 30.0
    }
     */
}

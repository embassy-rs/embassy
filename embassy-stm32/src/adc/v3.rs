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
use crate::adc::SealedAdcChannel;
use crate::adc::{
    Adc, AdcRegs, Averaging, ConversionMode, Instance, Resolution, SampleTime, Temperature, Vbat, VrefInt,
};
use crate::wait::block_for_us;
use crate::{Peri, pac, rcc};

/// Default VREF voltage used for sample conversion to millivolts.
pub const VREF_DEFAULT_MV: u32 = 3300;
#[cfg(any(adc_v3, adc_g0, adc_u0))]
/// VREF voltage used for factory calibration of VREFINTCAL register.
pub const VREF_CALIB_MV: u32 = 3000;
#[cfg(any(adc_h5, adc_h7rs))]
/// VREF voltage used for factory calibration of VREFINTCAL register.
pub const VREF_CALIB_MV: u32 = 3300;

#[cfg(adc_g0)]
/// The number of variants in Smpsel
// TODO: Use [#![feature(variant_count)]](https://github.com/rust-lang/rust/issues/73662) when stable
const SAMPLE_TIMES_CAPACITY: usize = 2;

#[cfg(adc_g0)]
impl<T: Instance> super::ConverterFor<super::VrefInt> for T {
    const CHANNEL: u8 = 13;
}
#[cfg(any(adc_h5, adc_h7rs))]
impl<T: Instance> super::ConverterFor<super::VrefInt> for T {
    const CHANNEL: u8 = 17;
}
#[cfg(adc_u0)]
impl<T: Instance> super::ConverterFor<super::VrefInt> for T {
    const CHANNEL: u8 = 12;
}
#[cfg(not(any(adc_g0, adc_h5, adc_h7rs, adc_u0)))]
impl<T: Instance> super::ConverterFor<super::VrefInt> for T {
    const CHANNEL: u8 = 0;
}

#[cfg(adc_g0)]
impl<T: Instance> super::ConverterFor<super::Temperature> for T {
    const CHANNEL: u8 = 12;
}
#[cfg(any(adc_h5, adc_h7rs))]
impl<T: Instance> super::ConverterFor<super::Temperature> for T {
    const CHANNEL: u8 = 16;
}
#[cfg(adc_u0)]
impl<T: Instance> super::ConverterFor<super::Temperature> for T {
    const CHANNEL: u8 = 11;
}
#[cfg(not(any(adc_g0, adc_h5, adc_h7rs, adc_u0)))]
impl<T: Instance> super::ConverterFor<super::Temperature> for T {
    const CHANNEL: u8 = 17;
}

#[cfg(adc_g0)]
impl<T: Instance> super::ConverterFor<super::Vbat> for T {
    const CHANNEL: u8 = 14;
}
#[cfg(any(adc_h5, adc_h7rs))]
impl<T: Instance> super::ConverterFor<super::Vbat> for T {
    const CHANNEL: u8 = 16;
}
#[cfg(adc_u0)]
impl<T: Instance> super::ConverterFor<super::Vbat> for T {
    const CHANNEL: u8 = 13;
}
#[cfg(not(any(adc_g0, adc_h5, adc_h7rs, adc_u0)))]
impl<T: Instance> super::ConverterFor<super::Vbat> for T {
    const CHANNEL: u8 = 18;
}

cfg_if! {
    if #[cfg(any(adc_h5, adc_h7rs))] {
        pub struct VddCore;
        impl<T: Instance> super::AdcChannel<T> for VddCore {}
        impl<T: Instance> super::SealedAdcChannel<T> for VddCore {
            fn channel(&self) -> u8 {
                17
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

impl super::AdcRegs for crate::pac::adc::Adc {
    #[cfg(any(rcc_l4, rcc_g4))]
    const HAS_ERRATA: bool = true;

    fn data(&self) -> *mut u16 {
        crate::pac::adc::Adc::dr(*self).as_ptr() as *mut u16
    }

    // Enable ADC only when it is not already running.
    fn enable(&self) {
        #[cfg(adc_u0)]
        if self.cfgr1().read().autoff() {
            // In AUTOFF mode the ADC wakes automatically when conversion starts,
            // so waiting for ADRDY here can stall instead of helping.
            return;
        }

        // Make sure bits are off
        while self.cr().read().addis() {
            // spin
        }

        if !self.cr().read().aden() {
            // Enable ADC
            self.isr().modify(|reg| {
                reg.set_adrdy(true);
            });
            self.cr().modify(|reg| {
                reg.set_aden(true);
            });

            while !self.isr().read().adrdy() {
                // spin
            }
        }
    }

    fn start(&self) {
        self.isr().modify(|reg| {
            reg.set_eos(true);
            reg.set_eoc(true);
        });

        self.cr().modify(|reg| {
            reg.set_adstart(true);
        });
    }

    fn stop(&self, _disable: bool) {
        // Ensure conversions are finished.
        if self.cr().read().adstart() && !self.cr().read().addis() {
            self.cr().modify(|reg| {
                reg.set_adstp(true);
            });
            while self.cr().read().adstart() {}
        }

        // Reset configuration.
        #[cfg(not(any(adc_g0, adc_u0)))]
        self.cfgr().modify(|reg| {
            reg.set_cont(false);
            reg.set_dmaen(false);
        });
        #[cfg(any(adc_g0, adc_u0))]
        self.cfgr1().modify(|reg| {
            reg.set_cont(false);
            reg.set_dmaen(false);
        });
    }

    /// Perform a single conversion.
    fn wait_done(&self) -> bool {
        self.isr().read().eos()
    }

    fn configure_dma(&self, conversion_mode: ConversionMode) {
        // Set continuous mode with oneshot dma.
        // Clear overrun flag before starting transfer.
        self.isr().modify(|reg| {
            reg.set_ovr(true);
        });

        #[cfg(not(any(adc_g0, adc_u0)))]
        let regs = self.cfgr();

        #[cfg(any(adc_g0, adc_u0))]
        let regs = self.cfgr1();

        regs.modify(|w| {
            w.set_discen(false);
            w.set_dmaen(!matches!(conversion_mode, ConversionMode::NoDma));
            w.set_cont(false);
            #[cfg(any(adc_v3, adc_g0, adc_u0))]
            w.set_cont(matches!(conversion_mode, ConversionMode::Repeated(None)));
            w.set_dmacfg(Dmacfg::Circular);

            #[cfg(any(adc_v2, adc_g4, adc_v3, adc_g0, adc_u0, adc_wba, adc_c0))]
            if let ConversionMode::Repeated(Some((signal, _edge))) = conversion_mode {
                #[cfg(adc_g0)]
                w.set_exten(_edge);
                w.set_extsel(signal.into());
            }
        });
    }

    fn configure_sequence(&self, sequence: impl ExactSizeIterator<Item = ((u8, bool), SampleTime)>) {
        #[cfg(adc_g0)]
        {
            let mut sample_times = Vec::<SampleTime, SAMPLE_TIMES_CAPACITY>::new();

            self.chselr().write(|chselr| {
                self.smpr().write(|smpr| {
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

        #[cfg(adc_u0)]
        {
            let mut channel_mask = 0;
            let mut sample_time: Self::SampleTime = SampleTime::Cycles15;

            // Configure channels and ranks
            for (_i, ((channel, _is_differential), _sample_time)) in sequence.enumerate() {
                assert!(
                    sample_time == _sample_time || _i == 0,
                    "U0 only supports one sample time for the sequence."
                );

                sample_time = _sample_time;
                channel_mask |= 1 << channel;
            }

            self.smpr().modify(|reg| reg.set_smp1(sample_time.into()));

            // On G0 and U0 enabled channels are sampled from 0 to last channel.
            // It is possible to add up to 8 sequences if CHSELRMOD = 1.
            // However for supporting more than 8 channels alternative CHSELRMOD = 0 approach is used.
            self.chselr().modify(|reg| {
                reg.set_chsel(channel_mask);
            });
        }

        #[cfg(not(any(adc_g0, adc_u0)))]
        {
            use crate::pac::adc::regs::{Sqr1, Sqr2, Sqr3, Sqr4};

            #[cfg(adc_h5)]
            let mut difsel = 0u32;

            let mut sqr1 = Sqr1::default();
            let mut sqr2 = Sqr2::default();
            let mut sqr3 = Sqr3::default();
            let mut sqr4 = Sqr4::default();

            cfg_if! {
                if #[cfg(any(adc_h5, adc_h7rs))] {
                    let mut smpr1 = self.smpr1().read();
                    let mut smpr2 = self.smpr2().read();
                } else {
                    let mut smpr1 = self.smpr(0).read();
                    let mut smpr2 = self.smpr(1).read();
                }
            }

            // Set sequence length
            sqr1.set_l(sequence.len() as u8 - 1);

            // Configure channels and ranks
            for (_i, ((channel, _is_differential), sample_time)) in sequence.enumerate() {
                // RM0492, RM0481, etc.
                // "This option bit must be set to 1 when ADCx_INP0 or ADCx_INN1 channel is selected."
                #[cfg(any(adc_h5, adc_h7rs))]
                if channel == 0 {
                    self.or().modify(|reg| reg.set_op0(true));
                }

                // Configure channel
                match channel {
                    0..=9 => smpr1.set_smp(channel as usize % 10, sample_time.into()),
                    _ => smpr2.set_smp(channel as usize % 10, sample_time.into()),
                }

                #[cfg(stm32h7)]
                {
                    use crate::pac::adc::vals::Pcsel;

                    self.cfgr2().modify(|w| w.set_lshift(0));
                    self.pcsel()
                        .write(|w| w.set_pcsel(channel.channel() as _, Pcsel::PRESELECTED));
                }

                // Each channel is sampled according to sequence
                match _i {
                    0..=3 => {
                        sqr1.set_sq(_i, channel);
                    }
                    4..=8 => {
                        sqr2.set_sq(_i - 4, channel);
                    }
                    9..=13 => {
                        sqr3.set_sq(_i - 9, channel);
                    }
                    14..=15 => {
                        sqr4.set_sq(_i - 14, channel);
                    }
                    _ => unreachable!(),
                }

                #[cfg(adc_h5)]
                {
                    difsel |= (_is_differential as u32) << channel;
                }
            }

            self.sqr1().write_value(sqr1);
            self.sqr2().write_value(sqr2);
            self.sqr3().write_value(sqr3);
            self.sqr4().write_value(sqr4);

            cfg_if! {
                if #[cfg(any(adc_h5, adc_h7rs))] {
                    self.smpr1().write_value(smpr1);
                    self.smpr2().write_value(smpr2);
                } else {
                    self.smpr(0).write_value(smpr1);
                    self.smpr(1).write_value(smpr2);
                }
            }

            #[cfg(adc_h5)]
            self.difsel().write(|w| w.set_difsel(difsel));
        }
    }
}

impl<'d, T: Instance<Regs = crate::pac::adc::Adc>> Adc<'d, T> {
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

        block_for_us(20);
    }

    /// Calibrate to remove conversion offset
    fn init_calibrate() {
        #[cfg(adc_u0)]
        let auto_off = T::regs().cfgr1().read().autoff();
        #[cfg(adc_u0)]
        T::regs().cfgr1().modify(|reg| {
            reg.set_autoff(false);
        });

        T::regs().cr().modify(|reg| {
            reg.set_adcal(true);
        });

        while T::regs().cr().read().adcal() {
            // spin
        }

        #[cfg(adc_u0)]
        T::regs().cfgr1().modify(|reg| {
            reg.set_autoff(auto_off);
        });

        block_for_us(1);
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
                    CkModePclk::DIV1 => Ckmode::Pclk,
                    CkModePclk::DIV2 => Ckmode::PclkDiv2,
                    CkModePclk::DIV4 => Ckmode::PclkDiv4,
                })
            }),
        }

        Self::init_calibrate();

        Self { adc }
    }

    /// Power down the ADC.
    ///
    /// This stops ADC operation and may reduce power consumption.
    /// A later read will enable it automatically.
    pub fn power_down(&mut self) {
        T::regs().stop(false);

        if T::regs().cr().read().aden() {
            T::regs().cr().modify(|reg| {
                reg.set_addis(true);
            });
            while T::regs().cr().read().aden() {}
        }
    }

    #[cfg(adc_u0)]
    pub fn enable_auto_off(&mut self) {
        T::regs().cfgr1().modify(|reg| {
            reg.set_autoff(true);
        });
    }

    #[cfg(adc_u0)]
    pub fn disable_auto_off(&mut self) {
        T::regs().cfgr1().modify(|reg| {
            reg.set_autoff(false);
        });
    }

    pub fn enable_vrefint(&mut self) -> VrefInt {
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
        block_for_us(15);

        VrefInt {}
    }

    pub fn enable_temperature(&mut self) -> Temperature {
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

    pub fn enable_vbat(&mut self) -> Vbat {
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

    pub fn disable_vbat(&mut self) {
        cfg_if! {
            if #[cfg(any(adc_g0, adc_u0))] {
                T::regs().ccr().modify(|reg| {
                    reg.set_vbaten(false);
                });
            } else if #[cfg(any(adc_h5, adc_h7rs))] {
                T::common_regs().ccr().modify(|reg| {
                    reg.set_vbaten(false);
                });
            } else {
                T::common_regs().ccr().modify(|reg| {
                    reg.set_ch18sel(false);
                });
            }
        }
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

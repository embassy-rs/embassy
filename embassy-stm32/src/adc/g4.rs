#[cfg(stm32g4)]
use pac::adc::regs::Difsel as DifselReg;
#[allow(unused)]
#[cfg(stm32g4)]
pub use pac::adc::vals::{Adcaldif, Adstp, Difsel, Dmacfg, Dmaen, Exten, Rovsm, Trovs};
#[allow(unused)]
#[cfg(stm32h7)]
use pac::adc::vals::{Adcaldif, Difsel, Exten};
pub use pac::adccommon::vals::{Dual, Presc};

use super::{Adc, AnyAdcChannel, ConversionMode, Resolution, SampleTime, blocking_delay_us};
use crate::adc::{AdcRegs, DefaultInstance, SealedInjectedAdcRegs};
use crate::pac::adc::regs::{Smpr, Smpr2, Sqr1, Sqr2, Sqr3, Sqr4};
use crate::time::Hertz;
use crate::{Peri, pac, rcc};

mod injected;
pub use injected::InjectedAdc;

/// Default VREF voltage used for sample conversion to millivolts.
pub const VREF_DEFAULT_MV: u32 = 3300;
/// VREF voltage used for factory calibration of VREFINTCAL register.
pub const VREF_CALIB_MV: u32 = 3000;

pub const NR_INJECTED_RANKS: usize = 4;

/// Max single ADC operation clock frequency
#[cfg(stm32g4)]
const MAX_ADC_CLK_FREQ: Hertz = Hertz::mhz(60);
#[cfg(stm32h7)]
const MAX_ADC_CLK_FREQ: Hertz = Hertz::mhz(50);

fn from_ker_ck(frequency: Hertz) -> Presc {
    let raw_prescaler = rcc::raw_prescaler(frequency.0, MAX_ADC_CLK_FREQ.0);
    match raw_prescaler {
        0 => Presc::DIV1,
        1 => Presc::DIV2,
        2..=3 => Presc::DIV4,
        4..=5 => Presc::DIV6,
        6..=7 => Presc::DIV8,
        8..=9 => Presc::DIV10,
        10..=11 => Presc::DIV12,
        _ => unimplemented!(),
    }
}

/// ADC configuration
#[derive(Default)]
pub struct AdcConfig {
    pub dual_mode: Option<Dual>,
    pub resolution: Option<Resolution>,
    #[cfg(stm32g4)]
    pub oversampling_shift: Option<u8>,
    #[cfg(stm32g4)]
    pub oversampling_ratio: Option<u8>,
    #[cfg(stm32g4)]
    pub oversampling_mode: Option<(Rovsm, Trovs, bool)>,
}

impl super::AdcRegs for crate::pac::adc::Adc {
    fn data(&self) -> *mut u16 {
        crate::pac::adc::Adc::dr(*self).as_ptr() as *mut u16
    }

    fn enable(&self) {
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
        self.cr().modify(|reg| {
            reg.set_adstart(true);
        });
    }

    fn stop(&self) {
        if self.cr().read().adstart() && !self.cr().read().addis() {
            self.cr().modify(|reg| {
                reg.set_adstp(Adstp::STOP);
            });
            // The software must poll ADSTART until the bit is reset before assuming the
            // ADC is completely stopped
            while self.cr().read().adstart() {}
        }

        // Disable dma control and continuous conversion, if enabled
        self.cfgr().modify(|reg| {
            reg.set_cont(false);
            reg.set_dmaen(Dmaen::DISABLE);
        });
    }

    fn convert(&self) {
        self.isr().modify(|reg| {
            reg.set_eos(true);
            reg.set_eoc(true);
        });

        // Start conversion
        self.cr().modify(|reg| {
            reg.set_adstart(true);
        });

        while !self.isr().read().eos() {
            // spin
        }
    }

    fn configure_dma(&self, conversion_mode: ConversionMode) {
        self.isr().modify(|reg| {
            reg.set_ovr(true);
        });

        self.cfgr().modify(|reg| {
            reg.set_discen(false); // Convert all channels for each trigger
            reg.set_dmacfg(match conversion_mode {
                ConversionMode::Singular => Dmacfg::ONE_SHOT,
                ConversionMode::ConfiguredSequence => Dmacfg::CIRCULAR,
                ConversionMode::Repeated(_) => Dmacfg::CIRCULAR,
            });
            reg.set_dmaen(Dmaen::ENABLE);
        });

        if matches!(conversion_mode, ConversionMode::ConfiguredSequence) {
            // One sequence per adstart, DMA stays armed for the next call.
            self.cfgr().modify(|reg| reg.set_cont(false));
        }

        if let ConversionMode::Repeated(trigger) = conversion_mode {
            match trigger {
                None => {
                    // continuous conversions
                    self.cfgr().modify(|reg| {
                        reg.set_cont(true);
                    });
                }
                Some((signal, edge)) => {
                    self.cfgr().modify(|r| {
                        r.set_cont(false); // New trigger is needed for each sample to be read
                    });

                    self.cfgr().modify(|r| {
                        r.set_extsel(signal);
                        r.set_exten(edge);
                    });

                    // Regular conversions uses DMA so no need to generate interrupt
                    self.ier().modify(|r| r.set_eosie(false));
                }
            }
        }
    }

    fn configure_sequence(&self, sequence: impl ExactSizeIterator<Item = ((u8, bool), SampleTime)>) {
        self.cr().modify(|w| w.set_aden(false));

        #[cfg(stm32g4)]
        let mut difsel = DifselReg::default();
        let mut smpr = Smpr::default();
        let mut smpr2 = Smpr2::default();
        let mut sqr1 = Sqr1::default();
        let mut sqr2 = Sqr2::default();
        let mut sqr3 = Sqr3::default();
        let mut sqr4 = Sqr4::default();

        // Set sequence length
        sqr1.set_l(sequence.len() as u8 - 1);

        // Configure channels and ranks
        for (_i, ((ch, is_differential), sample_time)) in sequence.enumerate() {
            let sample_time = sample_time.into();
            if ch <= 9 {
                smpr.set_smp(ch as _, sample_time);
            } else {
                smpr2.set_smp((ch - 10) as _, sample_time);
            }

            match _i {
                0..=3 => {
                    sqr1.set_sq(_i, ch);
                }
                4..=8 => {
                    sqr2.set_sq(_i - 4, ch);
                }
                9..=13 => {
                    sqr3.set_sq(_i - 9, ch);
                }
                14..=15 => {
                    sqr4.set_sq(_i - 14, ch);
                }
                _ => unreachable!(),
            }

            #[cfg(stm32g4)]
            {
                if ch < 18 {
                    difsel.set_difsel(
                        ch.into(),
                        if is_differential {
                            Difsel::DIFFERENTIAL
                        } else {
                            Difsel::SINGLE_ENDED
                        },
                    );
                }
            }
        }

        self.smpr().write_value(smpr);
        self.smpr2().write_value(smpr2);
        self.sqr1().write_value(sqr1);
        self.sqr2().write_value(sqr2);
        self.sqr3().write_value(sqr3);
        self.sqr4().write_value(sqr4);
        #[cfg(stm32g4)]
        self.difsel().write_value(difsel);
    }
}

impl SealedInjectedAdcRegs for crate::pac::adc::Adc {
    fn configure_injected_sequence(&self, sequence: impl ExactSizeIterator<Item = ((u8, bool), Self::SampleTime)>) {
        let len: u8 = sequence.len().try_into().unwrap();
        self.jsqr().modify(|w| w.set_jl(len - 1));

        for (n, ((channel, _), sample_time)) in sequence.enumerate() {
            let sample_time = sample_time.clone().into();
            if channel <= 9 {
                self.smpr().modify(|reg| reg.set_smp(channel as _, sample_time));
            } else {
                self.smpr2().modify(|reg| reg.set_smp((channel - 10) as _, sample_time));
            }

            let idx = match n {
                0..=3 => n,
                4..=8 => n - 4,
                9..=13 => n - 9,
                14..=15 => n - 14,
                _ => unreachable!(),
            };

            self.jsqr().modify(|w| w.set_jsq(idx, channel));
        }
    }

    fn configure_injected_trigger(&self, trigger: (u8, Exten), interrupt: bool) {
        self.cfgr().modify(|reg| reg.set_jdiscen(false));

        // Set external trigger for injected conversion sequence
        // Possible trigger values are seen in Table 167 in RM0440 Rev 9
        self.jsqr().modify(|r| {
            r.set_jextsel(trigger.0);
            r.set_jexten(trigger.1);
        });

        // Enable end of injected sequence interrupt
        self.ier().modify(|r| r.set_jeosie(interrupt));
    }

    fn start_injected(&self) {
        self.cr().modify(|reg| {
            reg.set_jadstart(true);
        });
    }

    fn stop_injected(&self) {
        if self.cr().read().adstart() && !self.cr().read().addis() {
            self.cr().modify(|reg| {
                reg.set_jadstp(Adstp::STOP);
            });
            // The software must poll JADSTART until the bit is reset before assuming the
            // ADC is completely stopped
            while self.cr().read().jadstart() {}
        }
    }

    fn read_injected(&self, data: &mut [u16]) {
        for (i, d) in data.iter_mut().enumerate() {
            *d = self.jdr(i).read().jdata();
        }

        // Clear JEOS by writing 1
        self.isr().modify(|r| r.set_jeos(true));
    }
}

impl<'d, T: DefaultInstance> Adc<'d, T> {
    /// Create a new ADC driver.
    pub fn new(adc: Peri<'d, T>, config: AdcConfig) -> Self {
        rcc::enable_and_reset::<T>();

        let prescaler = from_ker_ck(T::frequency());

        T::common_regs().ccr().modify(|w| w.set_presc(prescaler));

        let frequency = T::frequency() / prescaler;
        trace!("ADC frequency set to {}", frequency);

        if frequency > MAX_ADC_CLK_FREQ {
            panic!(
                "Maximal allowed frequency for the ADC is {} MHz and it varies with different packages, refer to ST docs for more information.",
                MAX_ADC_CLK_FREQ.0 / 1_000_000
            );
        }

        T::regs().cr().modify(|reg| {
            reg.set_deeppwd(false);
            reg.set_advregen(true);
        });

        blocking_delay_us(20);

        T::regs().difsel().modify(|w| {
            for n in 0..18 {
                w.set_difsel(n, Difsel::SINGLE_ENDED);
            }
        });

        T::regs().cr().modify(|w| {
            w.set_adcaldif(Adcaldif::SINGLE_ENDED);
        });

        T::regs().cr().modify(|w| w.set_adcal(true));

        while T::regs().cr().read().adcal() {}

        blocking_delay_us(20);

        T::regs().cr().modify(|w| {
            w.set_adcaldif(Adcaldif::DIFFERENTIAL);
        });

        T::regs().cr().modify(|w| w.set_adcal(true));

        while T::regs().cr().read().adcal() {}

        blocking_delay_us(20);

        T::regs().enable();

        // single conversion mode, software trigger
        T::regs().cfgr().modify(|w| {
            w.set_cont(false);
            w.set_exten(Exten::DISABLED);
        });

        if let Some(dual) = config.dual_mode {
            T::common_regs().ccr().modify(|reg| {
                reg.set_dual(dual);
            })
        }

        if let Some(resolution) = config.resolution {
            T::regs().cfgr().modify(|reg| reg.set_res(resolution.into()));
        }

        #[cfg(stm32g4)]
        if let Some(shift) = config.oversampling_shift {
            T::regs().cfgr2().modify(|reg| reg.set_ovss(shift));
        }

        #[cfg(stm32g4)]
        if let Some(ratio) = config.oversampling_ratio {
            T::regs().cfgr2().modify(|reg| reg.set_ovsr(ratio));
        }

        #[cfg(stm32g4)]
        if let Some((mode, trig_mode, enable)) = config.oversampling_mode {
            T::regs().cfgr2().modify(|reg| reg.set_trovs(trig_mode));
            T::regs().cfgr2().modify(|reg| reg.set_rovsm(mode));
            T::regs().cfgr2().modify(|reg| reg.set_rovse(enable));
        }

        Self { adc }
    }

    /// Enable reading the voltage reference internal channel.
    pub fn enable_vrefint(&self) -> super::VrefInt
    where
        T: super::SpecialConverter<super::VrefInt>,
    {
        T::common_regs().ccr().modify(|reg| {
            reg.set_vrefen(true);
        });

        super::VrefInt {}
    }

    /// Enable reading the temperature internal channel.
    pub fn enable_temperature(&self) -> super::Temperature
    where
        T: super::SpecialConverter<super::Temperature>,
    {
        T::common_regs().ccr().modify(|reg| {
            reg.set_vsenseen(true);
        });

        super::Temperature {}
    }

    /// Enable reading the vbat internal channel.
    pub fn enable_vbat(&self) -> super::Vbat
    where
        T: super::SpecialConverter<super::Vbat>,
    {
        T::common_regs().ccr().modify(|reg| {
            reg.set_vbaten(true);
        });

        super::Vbat {}
    }

    // Reads that are not implemented as INJECTED in "blocking_read"
    // #[cfg(stm32g4)]
    // pub fn enalble_injected_oversampling_mode(&mut self, enable: bool) {
    //     T::regs().cfgr2().modify(|reg| reg.set_jovse(enable));
    // }

    // #[cfg(stm32g4)]
    // pub fn enable_oversampling_regular_injected_mode(&mut self, enable: bool) {
    //     // the regularoversampling mode is forced to resumed mode (ROVSM bit ignored),
    //     T::regs().cfgr2().modify(|reg| reg.set_rovse(enable));
    //     T::regs().cfgr2().modify(|reg| reg.set_jovse(enable));
    // }
}

#[cfg(stm32g4)]
mod g4 {
    use crate::adc::{SealedSpecialConverter, Temperature, Vbat, VrefInt};

    impl SealedSpecialConverter<Temperature> for crate::peripherals::ADC1 {
        const CHANNEL: u8 = 16;
    }

    impl SealedSpecialConverter<VrefInt> for crate::peripherals::ADC1 {
        const CHANNEL: u8 = 18;
    }

    impl SealedSpecialConverter<Vbat> for crate::peripherals::ADC1 {
        const CHANNEL: u8 = 17;
    }

    #[cfg(peri_adc3_common)]
    impl SealedSpecialConverter<VrefInt> for crate::peripherals::ADC3 {
        const CHANNEL: u8 = 18;
    }

    #[cfg(peri_adc3_common)]
    impl SealedSpecialConverter<Vbat> for crate::peripherals::ADC3 {
        const CHANNEL: u8 = 17;
    }

    #[cfg(not(stm32g4x1))]
    impl SealedSpecialConverter<VrefInt> for crate::peripherals::ADC4 {
        const CHANNEL: u8 = 18;
    }

    #[cfg(not(stm32g4x1))]
    impl SealedSpecialConverter<Temperature> for crate::peripherals::ADC5 {
        const CHANNEL: u8 = 4;
    }

    #[cfg(not(stm32g4x1))]
    impl SealedSpecialConverter<VrefInt> for crate::peripherals::ADC5 {
        const CHANNEL: u8 = 18;
    }

    #[cfg(not(stm32g4x1))]
    impl SealedSpecialConverter<Vbat> for crate::peripherals::ADC5 {
        const CHANNEL: u8 = 17;
    }
}

// TODO this should look at each ADC individually and impl the correct channels
#[cfg(stm32h7)]
mod h7 {
    impl<T: Instance> SealedSpecialConverter<Temperature> for T {
        const CHANNEL: u8 = 18;
    }
    impl<T: Instance> SealedSpecialConverter<VrefInt> for T {
        const CHANNEL: u8 = 19;
    }
    impl<T: Instance> SealedSpecialConverter<Vbat> for T {
        // TODO this should be 14 for H7a/b/35
        const CHANNEL: u8 = 17;
    }
}

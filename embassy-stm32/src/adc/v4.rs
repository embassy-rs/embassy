#[cfg(not(stm32u3))]
use pac::adc::vals::Difsel;
#[cfg(not(any(stm32u5, stm32u3)))]
use pac::adc::vals::{Adcaldif, Boost};
#[allow(unused)]
use pac::adc::vals::{Adstp, Dmngt, Exten, Pcsel};
#[cfg(not(stm32u3))]
use pac::adccommon::vals::Presc;

#[cfg(any(stm32u5, stm32u3))]
use crate::adc::DefaultInstance;
use crate::adc::{
    Adc, AdcRegs, Averaging, ConversionMode, Instance, Resolution, SampleTime, Temperature, Vbat, VrefInt,
};
use crate::pac::adc::regs::{Sqr1, Sqr2, Sqr3, Sqr4};
use crate::time::Hertz;
use crate::wait::block_for_us;
use crate::{Peri, pac, rcc};

/// Default VREF voltage used for sample conversion to millivolts.
pub const VREF_DEFAULT_MV: u32 = 3300;
/// VREF voltage used for factory calibration of VREFINTCAL register.
pub const VREF_CALIB_MV: u32 = 3300;

/// Max single ADC operation clock frequency
#[cfg(stm32g4)]
const MAX_ADC_CLK_FREQ: Hertz = Hertz::mhz(60);
#[cfg(stm32h7)]
const MAX_ADC_CLK_FREQ: Hertz = Hertz::mhz(50);
#[cfg(stm32u5)]
const MAX_ADC_CLK_FREQ: Hertz = Hertz::mhz(55);
#[cfg(stm32u3)]
const MAX_ADC_CLK_FREQ: Hertz = Hertz::mhz(48);

#[cfg(stm32g4)]
impl<T: Instance> super::ConverterFor<super::VrefInt> for T {
    const CHANNEL: u8 = 18;
}
#[cfg(stm32g4)]
impl<T: Instance> super::ConverterFor<super::Temperature> for T {
    const CHANNEL: u8 = 16;
}

#[cfg(stm32h7)]
impl<T: Instance> super::ConverterFor<super::VrefInt> for T {
    const CHANNEL: u8 = 19;
}
#[cfg(stm32h7)]
impl<T: Instance> super::ConverterFor<super::Temperature> for T {
    const CHANNEL: u8 = 18;
}

// TODO this should be 14 for H7a/b/35
#[cfg(not(any(stm32u5, stm32u3)))]
impl<T: Instance> super::ConverterFor<super::Vbat> for T {
    const CHANNEL: u8 = 17;
}

#[cfg(any(stm32u5, stm32u3))]
impl<T: DefaultInstance> super::ConverterFor<super::VrefInt> for T {
    const CHANNEL: u8 = 0;
}
#[cfg(stm32u5)]
impl<T: DefaultInstance> super::ConverterFor<super::Temperature> for T {
    const CHANNEL: u8 = 19;
}
#[cfg(stm32u5)]
impl<T: DefaultInstance> super::ConverterFor<super::Vbat> for T {
    const CHANNEL: u8 = 18;
}

#[cfg(stm32u3)]
impl<T: DefaultInstance> super::ConverterFor<super::Vbat> for T {
    const CHANNEL: u8 = 16;
}

#[cfg(stm32u3)]
impl<T: DefaultInstance> super::ConverterFor<super::Temperature> for T {
    const CHANNEL: u8 = 17;
}

#[cfg(not(stm32u3))]
fn from_ker_ck(frequency: Hertz) -> Presc {
    let raw_prescaler = rcc::raw_prescaler(frequency.0, MAX_ADC_CLK_FREQ.0);
    match raw_prescaler {
        0 => Presc::Div1,
        1 => Presc::Div2,
        2..=3 => Presc::Div4,
        4..=5 => Presc::Div6,
        6..=7 => Presc::Div8,
        8..=9 => Presc::Div10,
        10..=11 => Presc::Div12,
        _ => unimplemented!(),
    }
}

/// Adc configuration
#[derive(Default)]
pub struct AdcConfig {
    pub resolution: Option<Resolution>,
    pub averaging: Option<Averaging>,
}

impl AdcRegs for crate::pac::adc::Adc {
    fn data(&self) -> *mut u16 {
        crate::pac::adc::Adc::dr(*self).as_ptr() as *mut u16
    }

    fn enable(&self) {
        if !self.cr().read().aden() {
            self.isr().write(|w| w.set_adrdy(true));
            self.cr().modify(|w| w.set_aden(true));
            while !self.isr().read().adrdy() {}
            self.isr().write(|w| w.set_adrdy(true));
        }
    }

    fn start(&self) {
        // Start conversion
        self.cr().modify(|reg| {
            reg.set_adstart(true);
        });
    }

    fn stop(&self, _disable: bool) {
        if self.cr().read().adstart() && !self.cr().read().addis() {
            self.cr().modify(|reg| {
                reg.set_adstp(Adstp::Stop);
            });
            while self.cr().read().adstart() {}
        }

        // Reset configuration.
        self.cfgr().modify(|reg| {
            reg.set_cont(false);
            reg.set_dmngt(Dmngt::from_bits(0));
        });
    }

    fn wait_done(&self) -> bool {
        self.isr().read().eos()
    }

    fn configure_dma(&self, conversion_mode: ConversionMode) {
        self.isr().modify(|reg| {
            reg.set_ovr(true);
        });

        self.cfgr().modify(|w| {
            w.set_cont(false);
            w.set_dmngt(match conversion_mode {
                ConversionMode::NoDma => Dmngt::from_bits(0),
                _ => Dmngt::DmaCircular,
            });
        });
    }

    fn configure_sequence(&self, sequence: impl ExactSizeIterator<Item = ((u8, bool), SampleTime)>) {
        let mut sqr1 = Sqr1::default();
        let mut sqr2 = Sqr2::default();
        let mut sqr3 = Sqr3::default();
        let mut sqr4 = Sqr4::default();

        let mut smpr1 = self.smpr(0).read();
        let mut smpr2 = self.smpr(1).read();

        // Set sequence length
        self.sqr1().modify(|w| {
            w.set_l(sequence.len() as u8 - 1);
        });

        // Configure channels and ranks
        for (i, ((channel, _), sample_time)) in sequence.enumerate() {
            let sample_time = sample_time.into();
            if channel <= 9 {
                smpr1.set_smp(channel as _, sample_time);
            } else {
                smpr2.set_smp((channel - 10) as _, sample_time);
            }

            #[cfg(any(stm32h7, stm32u5, stm32u3))]
            {
                self.cfgr2().modify(|w| w.set_lshift(0));
                self.pcsel().modify(|w| w.set_pcsel(channel as _, Pcsel::Preselected));
            }

            match i {
                0..=3 => {
                    sqr1.set_sq(i, channel);
                }
                4..=8 => {
                    sqr2.set_sq(i - 4, channel);
                }
                9..=13 => {
                    sqr3.set_sq(i - 9, channel);
                }
                14..=15 => {
                    sqr4.set_sq(i - 14, channel);
                }
                _ => unreachable!(),
            }
        }

        self.sqr1().write_value(sqr1);
        self.sqr2().write_value(sqr2);
        self.sqr3().write_value(sqr3);
        self.sqr4().write_value(sqr4);
        self.smpr(0).write_value(smpr1);
        self.smpr(1).write_value(smpr2);
    }
}

impl<'d, T: Instance<Regs = crate::pac::adc::Adc>> Adc<'d, T> {
    pub fn new_with_config(adc: Peri<'d, T>, config: AdcConfig) -> Self {
        let s = Self::new(adc);

        // Set the ADC resolution.
        if let Some(resolution) = config.resolution {
            T::regs().cfgr().modify(|reg| reg.set_res(resolution.into()));
        }

        // Set hardware averaging.
        if let Some(averaging) = config.averaging {
            let (enable, samples, right_shift) = match averaging {
                Averaging::Disabled => (false, 0, 0),
                Averaging::Samples2 => (true, 1, 1),
                Averaging::Samples4 => (true, 3, 2),
                Averaging::Samples8 => (true, 7, 3),
                Averaging::Samples16 => (true, 15, 4),
                Averaging::Samples32 => (true, 31, 5),
                Averaging::Samples64 => (true, 63, 6),
                Averaging::Samples128 => (true, 127, 7),
                Averaging::Samples256 => (true, 255, 8),
                Averaging::Samples512 => (true, 511, 9),
                Averaging::Samples1024 => (true, 1023, 10),
            };

            T::regs().cfgr2().modify(|reg| {
                reg.set_rovse(enable);
                reg.set_ovsr(samples);
                reg.set_ovss(right_shift);
            })
        }

        s
    }

    /// Create a new ADC driver.
    pub fn new(adc: Peri<'d, T>) -> Self {
        rcc::enable_and_reset::<T>();

        #[cfg(not(stm32u3))]
        let prescaler = from_ker_ck(T::frequency());
        #[cfg(not(stm32u3))]
        T::common_regs().ccr().modify(|w| w.set_presc(prescaler));
        #[cfg(not(stm32u3))]
        let frequency = T::frequency() / prescaler;

        #[cfg(stm32u3)]
        let frequency = T::frequency();

        info!("ADC frequency set to {}", frequency);

        if frequency > MAX_ADC_CLK_FREQ {
            panic!(
                "Maximal allowed frequency for the ADC is {} MHz and it varies with different packages, refer to ST docs for more information.",
                MAX_ADC_CLK_FREQ.0 / 1_000_000
            );
        }

        #[cfg(stm32h7)]
        {
            let boost = if frequency < Hertz::khz(6_250) {
                Boost::Lt625
            } else if frequency < Hertz::khz(12_500) {
                Boost::Lt125
            } else if frequency < Hertz::mhz(25) {
                Boost::Lt25
            } else {
                Boost::Lt50
            };
            T::regs().cr().modify(|w| w.set_boost(boost));
        }

        T::regs().cr().modify(|reg| {
            reg.set_deeppwd(false);
            reg.set_advregen(true);
        });

        block_for_us(10);

        #[cfg(not(stm32u3))]
        T::regs().difsel().modify(|w| {
            for n in 0..20 {
                w.set_difsel(n, Difsel::SingleEnded);
            }
        });

        #[cfg(not(stm32u3))]
        T::regs().cr().modify(|w| {
            #[cfg(not(adc_u5))]
            w.set_adcaldif(Adcaldif::SingleEnded);
            w.set_adcallin(true);
        });

        T::regs().cr().modify(|w| w.set_adcal(true));

        while T::regs().cr().read().adcal() {}

        block_for_us(1);

        T::regs().enable();

        // single conversion mode, software trigger
        T::regs().cfgr().modify(|w| {
            w.set_cont(false);
            w.set_exten(Exten::Disabled);
        });

        Self { adc }
    }

    /// Enable reading the voltage reference internal channel.
    pub fn enable_vrefint(&mut self) -> VrefInt {
        T::common_regs().ccr().modify(|reg| {
            reg.set_vrefen(true);
        });

        VrefInt {}
    }

    /// Enable reading the temperature internal channel.
    pub fn enable_temperature(&mut self) -> Temperature {
        T::common_regs().ccr().modify(|reg| {
            reg.set_vsenseen(true);
        });

        Temperature {}
    }

    /// Enable reading the vbat internal channel.
    pub fn enable_vbat(&mut self) -> Vbat {
        T::common_regs().ccr().modify(|reg| {
            reg.set_vbaten(true);
        });

        Vbat {}
    }
}

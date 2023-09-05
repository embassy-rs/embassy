use core::sync::atomic::{AtomicU8, Ordering};

use embedded_hal_02::blocking::delay::DelayUs;
#[allow(unused)]
use pac::adc::vals::{Adcaldif, Boost, Difsel, Exten, Pcsel};
use pac::adccommon::vals::Presc;
use paste::paste;

use super::{Adc, AdcPin, Instance, InternalChannel, Resolution, SampleTime};
use crate::time::Hertz;
use crate::{pac, Peripheral};

/// Default VREF voltage used for sample conversion to millivolts.
pub const VREF_DEFAULT_MV: u32 = 3300;
/// VREF voltage used for factory calibration of VREFINTCAL register.
pub const VREF_CALIB_MV: u32 = 3300;

/// Max single ADC operation clock frequency
#[cfg(stm32g4)]
const MAX_ADC_CLK_FREQ: Hertz = Hertz::mhz(60);
#[cfg(stm32h7)]
const MAX_ADC_CLK_FREQ: Hertz = Hertz::mhz(50);

#[cfg(stm32g4)]
const VREF_CHANNEL: u8 = 18;
#[cfg(stm32g4)]
const TEMP_CHANNEL: u8 = 16;

#[cfg(stm32h7)]
const VREF_CHANNEL: u8 = 19;
#[cfg(stm32h7)]
const TEMP_CHANNEL: u8 = 18;

// TODO this should be 14 for H7a/b/35
const VBAT_CHANNEL: u8 = 17;

// NOTE: Vrefint/Temperature/Vbat are not available on all ADCs, this currently cannot be modeled with stm32-data, so these are available from the software on all ADCs
pub struct VrefInt;
impl<T: Instance> InternalChannel<T> for VrefInt {}
impl<T: Instance> super::sealed::InternalChannel<T> for VrefInt {
    fn channel(&self) -> u8 {
        VREF_CHANNEL
    }
}

pub struct Temperature;
impl<T: Instance> InternalChannel<T> for Temperature {}
impl<T: Instance> super::sealed::InternalChannel<T> for Temperature {
    fn channel(&self) -> u8 {
        TEMP_CHANNEL
    }
}

pub struct Vbat;
impl<T: Instance> InternalChannel<T> for Vbat {}
impl<T: Instance> super::sealed::InternalChannel<T> for Vbat {
    fn channel(&self) -> u8 {
        VBAT_CHANNEL
    }
}

static ADC12_ENABLE_COUNTER: AtomicU8 = AtomicU8::new(0);
#[cfg(any(stm32g4x3, stm32g4x4))]
static ADC345_ENABLE_COUNTER: AtomicU8 = AtomicU8::new(0);

macro_rules! rcc_peripheral {
    ($adc_name:ident, $freqs:ident, $ahb:ident, $reg:ident $(, $counter:ident )? ) => {
        impl crate::rcc::sealed::RccPeripheral for crate::peripherals::$adc_name {
            fn frequency() -> crate::time::Hertz {
                critical_section::with(|_| {
                    match unsafe { crate::rcc::get_freqs() }.$freqs {
                        Some(ck) => ck,
                        None => panic!("Invalid ADC clock configuration, AdcClockSource was likely not properly configured.")
                    }
                })
            }

            fn enable() {
                critical_section::with(|_| {
                    paste!{crate::pac::RCC.[< $ahb enr >]().modify(|w| w.[< set_ $reg en >](true))}
                });
                $ ( $counter.fetch_add(1, Ordering::SeqCst); )?
            }

            fn disable() {
               $ ( if $counter.load(Ordering::SeqCst) == 1  )? {
                    critical_section::with(|_| {
                        paste!{crate::pac::RCC.[< $ahb enr >]().modify(|w| w.[< set_ $reg en >](false))}
                    })
               }
               $ ( $counter.fetch_sub(1, Ordering::SeqCst); )?
            }

            fn reset() {
               $ ( if $counter.load(Ordering::SeqCst) == 1 )? {
                    critical_section::with(|_| {
                        paste!{crate::pac::RCC.[< $ahb rstr >]().modify(|w| w.[< set_ $reg rst >](true))}
                        paste!{crate::pac::RCC.[< $ahb rstr >]().modify(|w| w.[< set_ $reg rst >](false))}
                    });
               }
            }
        }

        impl crate::rcc::RccPeripheral for crate::peripherals::$adc_name {}
    };
}

#[cfg(stm32g4)]
foreach_peripheral!(
    (adc, ADC1) => { rcc_peripheral!(ADC1, adc, ahb2, adc12, ADC12_ENABLE_COUNTER); };
    (adc, ADC2) => { rcc_peripheral!(ADC2, adc, ahb2, adc12, ADC12_ENABLE_COUNTER); };
);

#[cfg(stm32g4x1)]
foreach_peripheral!(
    (adc, ADC3) => { rcc_peripheral!(ADC3, adc34, ahb2, adc345); };
);

#[cfg(any(stm32g4x3, stm32g4x4))]
foreach_peripheral!(
    (adc, ADC3) => { rcc_peripheral!(ADC3, adc34, ahb2, adc345, ADC345_ENABLE_COUNTER); };
    (adc, ADC4) => { rcc_peripheral!(ADC4, adc34, ahb2, adc345, ADC345_ENABLE_COUNTER); };
    (adc, ADC5) => { rcc_peripheral!(ADC5, adc34, ahb2, adc345, ADC345_ENABLE_COUNTER); };
);

#[cfg(stm32h7)]
foreach_peripheral!(
    (adc, ADC1) => { rcc_peripheral!(ADC1, adc, ahb1, adc12, ADC12_ENABLE_COUNTER); };
    (adc, ADC2) => { rcc_peripheral!(ADC2, adc, ahb1, adc12, ADC12_ENABLE_COUNTER); };
    (adc, ADC3) => { rcc_peripheral!(ADC3, adc, ahb4, adc3); };
);

// NOTE (unused): The prescaler enum closely copies the hardware capabilities,
// but high prescaling doesn't make a lot of sense in the current implementation and is ommited.
#[allow(unused)]
enum Prescaler {
    NotDivided,
    DividedBy2,
    DividedBy4,
    DividedBy6,
    DividedBy8,
    DividedBy10,
    DividedBy12,
    DividedBy16,
    DividedBy32,
    DividedBy64,
    DividedBy128,
    DividedBy256,
}

impl Prescaler {
    fn from_ker_ck(frequency: Hertz) -> Self {
        let raw_prescaler = frequency.0 / MAX_ADC_CLK_FREQ.0;
        match raw_prescaler {
            0 => Self::NotDivided,
            1 => Self::DividedBy2,
            2..=3 => Self::DividedBy4,
            4..=5 => Self::DividedBy6,
            6..=7 => Self::DividedBy8,
            8..=9 => Self::DividedBy10,
            10..=11 => Self::DividedBy12,
            _ => unimplemented!(),
        }
    }

    fn divisor(&self) -> u32 {
        match self {
            Prescaler::NotDivided => 1,
            Prescaler::DividedBy2 => 2,
            Prescaler::DividedBy4 => 4,
            Prescaler::DividedBy6 => 6,
            Prescaler::DividedBy8 => 8,
            Prescaler::DividedBy10 => 10,
            Prescaler::DividedBy12 => 12,
            Prescaler::DividedBy16 => 16,
            Prescaler::DividedBy32 => 32,
            Prescaler::DividedBy64 => 64,
            Prescaler::DividedBy128 => 128,
            Prescaler::DividedBy256 => 256,
        }
    }

    fn presc(&self) -> Presc {
        match self {
            Prescaler::NotDivided => Presc::DIV1,
            Prescaler::DividedBy2 => Presc::DIV2,
            Prescaler::DividedBy4 => Presc::DIV4,
            Prescaler::DividedBy6 => Presc::DIV6,
            Prescaler::DividedBy8 => Presc::DIV8,
            Prescaler::DividedBy10 => Presc::DIV10,
            Prescaler::DividedBy12 => Presc::DIV12,
            Prescaler::DividedBy16 => Presc::DIV16,
            Prescaler::DividedBy32 => Presc::DIV32,
            Prescaler::DividedBy64 => Presc::DIV64,
            Prescaler::DividedBy128 => Presc::DIV128,
            Prescaler::DividedBy256 => Presc::DIV256,
        }
    }
}

impl<'d, T: Instance> Adc<'d, T> {
    pub fn new(adc: impl Peripheral<P = T> + 'd, delay: &mut impl DelayUs<u16>) -> Self {
        embassy_hal_internal::into_ref!(adc);
        T::enable();
        T::reset();

        let prescaler = Prescaler::from_ker_ck(T::frequency());

        T::common_regs().ccr().modify(|w| w.set_presc(prescaler.presc()));

        let frequency = Hertz(T::frequency().0 / prescaler.divisor());
        info!("ADC frequency set to {} Hz", frequency.0);

        if frequency > MAX_ADC_CLK_FREQ {
            panic!("Maximal allowed frequency for the ADC is {} MHz and it varies with different packages, refer to ST docs for more information.", MAX_ADC_CLK_FREQ.0 /  1_000_000 );
        }

        #[cfg(stm32h7)]
        {
            let boost = if frequency < Hertz::khz(6_250) {
                Boost::LT6_25
            } else if frequency < Hertz::khz(12_500) {
                Boost::LT12_5
            } else if frequency < Hertz::mhz(25) {
                Boost::LT25
            } else {
                Boost::LT50
            };
            T::regs().cr().modify(|w| w.set_boost(boost));
        }
        let mut s = Self {
            adc,
            sample_time: Default::default(),
        };
        s.power_up(delay);
        s.configure_differential_inputs();

        s.calibrate();
        delay.delay_us(1);

        s.enable();
        s.configure();

        s
    }

    fn power_up(&mut self, delay: &mut impl DelayUs<u16>) {
        T::regs().cr().modify(|reg| {
            reg.set_deeppwd(false);
            reg.set_advregen(true);
        });

        delay.delay_us(10);
    }

    fn configure_differential_inputs(&mut self) {
        T::regs().difsel().modify(|w| {
            for n in 0..20 {
                w.set_difsel(n, Difsel::SINGLEENDED);
            }
        });
    }

    fn calibrate(&mut self) {
        T::regs().cr().modify(|w| {
            w.set_adcaldif(Adcaldif::SINGLEENDED);
            w.set_adcallin(true);
        });

        T::regs().cr().modify(|w| w.set_adcal(true));

        while T::regs().cr().read().adcal() {}
    }

    fn enable(&mut self) {
        T::regs().isr().write(|w| w.set_adrdy(true));
        T::regs().cr().modify(|w| w.set_aden(true));
        while !T::regs().isr().read().adrdy() {}
        T::regs().isr().write(|w| w.set_adrdy(true));
    }

    fn configure(&mut self) {
        // single conversion mode, software trigger
        T::regs().cfgr().modify(|w| {
            w.set_cont(false);
            w.set_exten(Exten::DISABLED);
        });
    }

    pub fn enable_vrefint(&self) -> VrefInt {
        T::common_regs().ccr().modify(|reg| {
            reg.set_vrefen(true);
        });

        VrefInt {}
    }

    pub fn enable_temperature(&self) -> Temperature {
        T::common_regs().ccr().modify(|reg| {
            reg.set_vsenseen(true);
        });

        Temperature {}
    }

    pub fn enable_vbat(&self) -> Vbat {
        T::common_regs().ccr().modify(|reg| {
            reg.set_vbaten(true);
        });

        Vbat {}
    }

    pub fn set_sample_time(&mut self, sample_time: SampleTime) {
        self.sample_time = sample_time;
    }

    pub fn set_resolution(&mut self, resolution: Resolution) {
        T::regs().cfgr().modify(|reg| reg.set_res(resolution.into()));
    }

    /// Perform a single conversion.
    fn convert(&mut self) -> u16 {
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

        T::regs().dr().read().0 as u16
    }

    pub fn read<P>(&mut self, pin: &mut P) -> u16
    where
        P: AdcPin<T>,
        P: crate::gpio::sealed::Pin,
    {
        pin.set_as_analog();

        self.read_channel(pin.channel())
    }

    pub fn read_internal(&mut self, channel: &mut impl InternalChannel<T>) -> u16 {
        self.read_channel(channel.channel())
    }

    fn read_channel(&mut self, channel: u8) -> u16 {
        // Configure channel
        Self::set_channel_sample_time(channel, self.sample_time);

        #[cfg(stm32h7)]
        {
            T::regs().cfgr2().modify(|w| w.set_lshift(0));
            T::regs()
                .pcsel()
                .write(|w| w.set_pcsel(channel as _, Pcsel::PRESELECTED));
        }

        T::regs().sqr1().write(|reg| {
            reg.set_sq(0, channel);
            reg.set_l(0);
        });

        self.convert()
    }

    fn set_channel_sample_time(ch: u8, sample_time: SampleTime) {
        let sample_time = sample_time.into();
        if ch <= 9 {
            T::regs().smpr(0).modify(|reg| reg.set_smp(ch as _, sample_time));
        } else {
            T::regs().smpr(1).modify(|reg| reg.set_smp((ch - 10) as _, sample_time));
        }
    }
}

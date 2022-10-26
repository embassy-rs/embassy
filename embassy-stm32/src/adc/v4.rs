use atomic_polyfill::{AtomicU8, Ordering};
use embedded_hal_02::blocking::delay::DelayUs;
use pac::adc::vals::{Adcaldif, Boost, Difsel, Exten, Pcsel};
use pac::adccommon::vals::Presc;

use super::{Adc, AdcPin, Instance, InternalChannel, SingleChannel};
use crate::time::Hertz;
use crate::{pac, Peripheral};

/// Default VREF voltage used for sample conversion to millivolts.
pub const VREF_DEFAULT_MV: u32 = 3300;
/// VREF voltage used for factory calibration of VREFINTCAL register.
pub const VREF_CALIB_MV: u32 = 3300;

pub enum Resolution {
    SixteenBit,
    FourteenBit,
    TwelveBit,
    TenBit,
    EightBit,
}

impl Default for Resolution {
    fn default() -> Self {
        Self::SixteenBit
    }
}

impl Resolution {
    fn res(&self) -> pac::adc::vals::Res {
        match self {
            Resolution::SixteenBit => pac::adc::vals::Res::SIXTEENBIT,
            Resolution::FourteenBit => pac::adc::vals::Res::FOURTEENBITV,
            Resolution::TwelveBit => pac::adc::vals::Res::TWELVEBITV,
            Resolution::TenBit => pac::adc::vals::Res::TENBIT,
            Resolution::EightBit => pac::adc::vals::Res::EIGHTBIT,
        }
    }

    pub fn to_max_count(&self) -> u32 {
        match self {
            Resolution::SixteenBit => (1 << 16) - 1,
            Resolution::FourteenBit => (1 << 14) - 1,
            Resolution::TwelveBit => (1 << 12) - 1,
            Resolution::TenBit => (1 << 10) - 1,
            Resolution::EightBit => (1 << 8) - 1,
        }
    }
}

// NOTE: Vrefint/Temperature/Vbat are only available on ADC3 on H7, this currently cannot be modeled with stm32-data, so these are available from the software on all ADCs
pub struct VrefInt;
impl<T: Instance> InternalChannel<T> for VrefInt {}
impl<T: Instance> super::sealed::InternalChannel<T> for VrefInt {
    fn channel(&self) -> u8 {
        19
    }
}

pub struct Temperature;
impl<T: Instance> InternalChannel<T> for Temperature {}
impl<T: Instance> super::sealed::InternalChannel<T> for Temperature {
    fn channel(&self) -> u8 {
        18
    }
}

pub struct Vbat;
impl<T: Instance> InternalChannel<T> for Vbat {}
impl<T: Instance> super::sealed::InternalChannel<T> for Vbat {
    fn channel(&self) -> u8 {
        // TODO this should be 14 for H7a/b/35
        17
    }
}

static ADC12_ENABLE_COUNTER: AtomicU8 = AtomicU8::new(0);

#[cfg(stm32h7)]
foreach_peripheral!(
    (adc, ADC1) => {
        impl crate::rcc::sealed::RccPeripheral for crate::peripherals::ADC1 {
            fn frequency() -> crate::time::Hertz {
                critical_section::with(|_| unsafe {
                    match crate::rcc::get_freqs().adc {
                        Some(ck) => ck,
                        None => panic!("Invalid ADC clock configuration, AdcClockSource was likely not properly configured.")
                    }
                })
            }

            fn enable() {
                critical_section::with(|_| unsafe {
                    crate::pac::RCC.ahb1enr().modify(|w| w.set_adc12en(true))
                });
                ADC12_ENABLE_COUNTER.fetch_add(1, Ordering::SeqCst);
            }

            fn disable() {
                if ADC12_ENABLE_COUNTER.load(Ordering::SeqCst) == 1 {
                    critical_section::with(|_| unsafe {
                        crate::pac::RCC.ahb1enr().modify(|w| w.set_adc12en(false));
                    })
                }
                ADC12_ENABLE_COUNTER.fetch_sub(1, Ordering::SeqCst);
            }

            fn reset() {
                if ADC12_ENABLE_COUNTER.load(Ordering::SeqCst) == 1 {
                    critical_section::with(|_| unsafe {
                        crate::pac::RCC.ahb1rstr().modify(|w| w.set_adc12rst(true));
                        crate::pac::RCC.ahb1rstr().modify(|w| w.set_adc12rst(false));
                    });
                }
            }
        }

        impl crate::rcc::RccPeripheral for crate::peripherals::ADC1 {}
    };
    (adc, ADC2) => {
        impl crate::rcc::sealed::RccPeripheral for crate::peripherals::ADC2 {
            fn frequency() -> crate::time::Hertz {
                critical_section::with(|_| unsafe {
                    match crate::rcc::get_freqs().adc {
                        Some(ck) => ck,
                        None => panic!("Invalid ADC clock configuration, AdcClockSource was likely not properly configured.")
                    }
                })
            }

            fn enable() {
                critical_section::with(|_| unsafe {
                    crate::pac::RCC.ahb1enr().modify(|w| w.set_adc12en(true))
                });
                ADC12_ENABLE_COUNTER.fetch_add(1, Ordering::SeqCst);
            }

            fn disable() {
                if ADC12_ENABLE_COUNTER.load(Ordering::SeqCst) == 1 {
                    critical_section::with(|_| unsafe {
                        crate::pac::RCC.ahb1enr().modify(|w| w.set_adc12en(false));
                    })
                }
                ADC12_ENABLE_COUNTER.fetch_sub(1, Ordering::SeqCst);
            }

            fn reset() {
                if ADC12_ENABLE_COUNTER.load(Ordering::SeqCst) == 1 {
                    critical_section::with(|_| unsafe {
                        crate::pac::RCC.ahb1rstr().modify(|w| w.set_adc12rst(true));
                        crate::pac::RCC.ahb1rstr().modify(|w| w.set_adc12rst(false));
                    });
                }
            }
        }

        impl crate::rcc::RccPeripheral for crate::peripherals::ADC2 {}
    };
    (adc, ADC3) => {
        impl crate::rcc::sealed::RccPeripheral for crate::peripherals::ADC3 {
            fn frequency() -> crate::time::Hertz {
                critical_section::with(|_| unsafe {
                    match crate::rcc::get_freqs().adc {
                        Some(ck) => ck,
                        None => panic!("Invalid ADC clock configuration, AdcClockSource was likely not properly configured.")
                    }
                })
            }

            fn enable() {
                critical_section::with(|_| unsafe {
                    crate::pac::RCC.ahb4enr().modify(|w| w.set_adc3en(true))
                });
            }

            fn disable() {
                    critical_section::with(|_| unsafe {
                        crate::pac::RCC.ahb4enr().modify(|w| w.set_adc3en(false));
                    })
            }

            fn reset() {
                    critical_section::with(|_| unsafe {
                        crate::pac::RCC.ahb4rstr().modify(|w| w.set_adc3rst(true));
                        crate::pac::RCC.ahb4rstr().modify(|w| w.set_adc3rst(false));
                    });
            }
        }

        impl crate::rcc::RccPeripheral for crate::peripherals::ADC3 {}
    };
);

/// ADC sample time
///
/// The default setting is 2.5 ADC clock cycles.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SampleTime {
    /// 1.5 ADC clock cycles
    Cycles1_5,

    /// 2.5 ADC clock cycles
    Cycles2_5,

    /// 8.5 ADC clock cycles
    Cycles8_5,

    /// 16.5 ADC clock cycles
    Cycles16_5,

    /// 32.5 ADC clock cycles
    Cycles32_5,

    /// 64.5 ADC clock cycles
    Cycles64_5,

    /// 387.5 ADC clock cycles
    Cycles387_5,

    /// 810.5 ADC clock cycles
    Cycles810_5,
}

impl SampleTime {
    pub(crate) fn sample_time(&self) -> pac::adc::vals::Smp {
        match self {
            SampleTime::Cycles1_5 => pac::adc::vals::Smp::CYCLES1_5,
            SampleTime::Cycles2_5 => pac::adc::vals::Smp::CYCLES2_5,
            SampleTime::Cycles8_5 => pac::adc::vals::Smp::CYCLES8_5,
            SampleTime::Cycles16_5 => pac::adc::vals::Smp::CYCLES16_5,
            SampleTime::Cycles32_5 => pac::adc::vals::Smp::CYCLES32_5,
            SampleTime::Cycles64_5 => pac::adc::vals::Smp::CYCLES64_5,
            SampleTime::Cycles387_5 => pac::adc::vals::Smp::CYCLES387_5,
            SampleTime::Cycles810_5 => pac::adc::vals::Smp::CYCLES810_5,
        }
    }
}

impl Default for SampleTime {
    fn default() -> Self {
        Self::Cycles1_5
    }
}

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
        let raw_prescaler = frequency.0 / 50_000_000;
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

impl<'d, T: Instance + crate::rcc::RccPeripheral> Adc<'d, T> {
    pub fn new(adc: impl Peripheral<P = T> + 'd, delay: &mut impl DelayUs<u16>) -> Self {
        embassy_hal_common::into_ref!(adc);
        T::enable();
        T::reset();

        let prescaler = Prescaler::from_ker_ck(T::frequency());

        unsafe {
            T::common_regs().ccr().modify(|w| w.set_presc(prescaler.presc()));
        }

        let frequency = Hertz(T::frequency().0 / prescaler.divisor());
        info!("ADC frequency set to {} Hz", frequency.0);

        if frequency > Hertz::mhz(50) {
            panic!("Maximal allowed frequency for the ADC is 50 MHz and it varies with different packages, refer to ST docs for more information.");
        }
        let boost = if frequency < Hertz::khz(6_250) {
            Boost::LT6_25
        } else if frequency < Hertz::khz(12_500) {
            Boost::LT12_5
        } else if frequency < Hertz::mhz(25) {
            Boost::LT25
        } else {
            Boost::LT50
        };
        unsafe {
            T::regs().cr().modify(|w| w.set_boost(boost));
        }

        let mut s = Self { adc };
        s.power_up(delay);
        s.configure_differential_inputs();

        s.calibrate();
        delay.delay_us(1);

        s.enable();
        s.configure();

        s
    }

    fn power_up(&mut self, delay: &mut impl DelayUs<u16>) {
        unsafe {
            T::regs().cr().modify(|reg| {
                reg.set_deeppwd(false);
                reg.set_advregen(true);
            });
        }

        delay.delay_us(10);
    }

    fn configure_differential_inputs(&mut self) {
        unsafe {
            T::regs().difsel().modify(|w| {
                for n in 0..20 {
                    w.set_difsel(n, Difsel::SINGLEENDED);
                }
            })
        };
    }

    fn calibrate(&mut self) {
        unsafe {
            T::regs().cr().modify(|w| {
                w.set_adcaldif(Adcaldif::SINGLEENDED);
                w.set_adcallin(true);
            });

            T::regs().cr().modify(|w| w.set_adcal(true));

            while T::regs().cr().read().adcal() {}
        }
    }

    fn enable(&mut self) {
        unsafe {
            T::regs().isr().write(|w| w.set_adrdy(true));
            T::regs().cr().modify(|w| w.set_aden(true));
            while !T::regs().isr().read().adrdy() {}
            T::regs().isr().write(|w| w.set_adrdy(true));
        }
    }

    fn configure(&mut self) {
        // single conversion mode, software trigger
        unsafe {
            T::regs().cfgr().modify(|w| {
                w.set_cont(false);
                w.set_exten(Exten::DISABLED);
            })
        }
    }

    pub fn enable_vrefint(&self) -> VrefInt {
        unsafe {
            T::common_regs().ccr().modify(|reg| {
                reg.set_vrefen(true);
            });
        }

        VrefInt {}
    }

    pub fn enable_temperature(&self) -> Temperature {
        unsafe {
            T::common_regs().ccr().modify(|reg| {
                reg.set_vsenseen(true);
            });
        }

        Temperature {}
    }

    pub fn enable_vbat(&self) -> Vbat {
        unsafe {
            T::common_regs().ccr().modify(|reg| {
                reg.set_vbaten(true);
            });
        }

        Vbat {}
    }

    pub fn single_channel<'a, P>(
        &'a mut self,
        pin: &'a mut P,
        sample_time: SampleTime,
        resolution: Resolution,
    ) -> SingleChannel<'a, T>
    where
        P: AdcPin<T>,
        P: crate::gpio::sealed::Pin,
    {
        unsafe {
            pin.set_as_analog();

            self._single_channel(pin.channel(), sample_time, resolution)
        }
    }

    pub fn read<P>(&mut self, pin: &mut P, sample_time: SampleTime, resolution: Resolution) -> u16
    where
        P: AdcPin<T>,
        P: crate::gpio::sealed::Pin,
    {
        self.single_channel(pin, sample_time, resolution).read()
    }

    pub fn single_channel_internal<'a>(
        &'a mut self,
        channel: &'a mut impl InternalChannel<T>,
        sample_time: SampleTime,
        resolution: Resolution,
    ) -> SingleChannel<'a, T> {
        unsafe { self._single_channel(channel.channel(), sample_time, resolution) }
    }

    pub fn read_internal(
        &mut self,
        channel: &mut impl InternalChannel<T>,
        sample_time: SampleTime,
        resolution: Resolution,
    ) -> u16 {
        self.single_channel_internal(channel, sample_time, resolution).read()
    }

    unsafe fn _single_channel(
        &mut self,
        channel: u8,
        sample_time: SampleTime,
        resolution: Resolution,
    ) -> SingleChannel<'_, T> {
        // Configure ADC
        T::regs().cfgr().modify(|reg| reg.set_res(resolution.res()));

        // Configure channel
        Self::set_channel_sample_time(channel, sample_time);

        T::regs().cfgr2().modify(|w| w.set_lshift(0));
        T::regs()
            .pcsel()
            .write(|w| w.set_pcsel(channel as _, Pcsel::PRESELECTED));
        T::regs().sqr1().write(|reg| {
            reg.set_sq(0, channel);
            reg.set_l(0);
        });

        SingleChannel {
            adc: self.adc.reborrow(),
        }
    }

    unsafe fn set_channel_sample_time(ch: u8, sample_time: SampleTime) {
        if ch <= 9 {
            T::regs()
                .smpr(0)
                .modify(|reg| reg.set_smp(ch as _, sample_time.sample_time()));
        } else {
            T::regs()
                .smpr(1)
                .modify(|reg| reg.set_smp((ch - 10) as _, sample_time.sample_time()));
        }
    }
}

/// Perform a single conversion.
pub(super) unsafe fn convert(regs: crate::pac::adc::Adc) -> u16 {
    regs.isr().modify(|reg| {
        reg.set_eos(true);
        reg.set_eoc(true);
    });

    // Start conversion
    regs.cr().modify(|reg| {
        reg.set_adstart(true);
    });

    while !regs.isr().read().eos() {
        // spin
    }

    regs.dr().read().0 as u16
}

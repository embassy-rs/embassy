use core::marker::PhantomData;

use embassy_hal_common::into_ref;
use embedded_hal_02::blocking::delay::DelayUs;

use crate::adc::{AdcPin, Instance};
use crate::{pac, Peripheral};

pub const VDDA_CALIB_MV: u32 = 3300;
pub const VREF_INT: u32 = 1230;

fn enable() {
    critical_section::with(|_| unsafe {
        crate::pac::RCC.apb2enr().modify(|reg| reg.set_adcen(true));
    });
}

pub enum Resolution {
    TwelveBit,
    TenBit,
    EightBit,
    SixBit,
}

impl Default for Resolution {
    fn default() -> Self {
        Self::TwelveBit
    }
}

impl Resolution {
    fn res(&self) -> pac::adc::vals::Res {
        match self {
            Resolution::TwelveBit => pac::adc::vals::Res::TWELVEBIT,
            Resolution::TenBit => pac::adc::vals::Res::TENBIT,
            Resolution::EightBit => pac::adc::vals::Res::EIGHTBIT,
            Resolution::SixBit => pac::adc::vals::Res::SIXBIT,
        }
    }

    pub fn to_max_count(&self) -> u32 {
        match self {
            Resolution::TwelveBit => (1 << 12) - 1,
            Resolution::TenBit => (1 << 10) - 1,
            Resolution::EightBit => (1 << 8) - 1,
            Resolution::SixBit => (1 << 6) - 1,
        }
    }
}

pub struct Vbat;
impl<T: Instance> AdcPin<T> for Vbat {}
impl<T: Instance> super::sealed::AdcPin<T> for Vbat {
    fn channel(&self) -> u8 {
        18
    }
}

pub struct Vref;
impl<T: Instance> AdcPin<T> for Vref {}
impl<T: Instance> super::sealed::AdcPin<T> for Vref {
    fn channel(&self) -> u8 {
        17
    }
}

pub struct Temperature;
impl<T: Instance> AdcPin<T> for Temperature {}
impl<T: Instance> super::sealed::AdcPin<T> for Temperature {
    fn channel(&self) -> u8 {
        16
    }
}

mod sample_time {
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
    pub enum SampleTime {
        /// 1.5 ADC clock cycles
        Cycles1_5 = 0b000,

        /// 7.5 ADC clock cycles
        Cycles7_5 = 0b001,

        /// 13.5 ADC clock cycles
        Cycles13_5 = 0b010,

        /// 28.5 ADC clock cycles
        Cycles28_5 = 0b011,

        /// 41.5 ADC clock cycles
        Cycles41_5 = 0b100,

        /// 55.5 ADC clock cycles
        Cycles55_5 = 0b101,

        /// 71.5 ADC clock cycles
        Cycles71_5 = 0b110,

        /// 239.5 ADC clock cycles
        Cycles239_5 = 0b111,
    }

    impl SampleTime {
        pub(crate) fn sample_time(&self) -> crate::pac::adc::vals::Smp {
            match self {
                SampleTime::Cycles1_5 => crate::pac::adc::vals::Smp::CYCLES1_5,
                SampleTime::Cycles7_5 => crate::pac::adc::vals::Smp::CYCLES7_5,
                SampleTime::Cycles13_5 => crate::pac::adc::vals::Smp::CYCLES13_5,
                SampleTime::Cycles28_5 => crate::pac::adc::vals::Smp::CYCLES28_5,
                SampleTime::Cycles41_5 => crate::pac::adc::vals::Smp::CYCLES41_5,
                SampleTime::Cycles55_5 => crate::pac::adc::vals::Smp::CYCLES55_5,
                SampleTime::Cycles71_5 => crate::pac::adc::vals::Smp::CYCLES71_5,
                SampleTime::Cycles239_5 => crate::pac::adc::vals::Smp::CYCLES239_5,
            }
        }
    }

    impl Default for SampleTime {
        fn default() -> Self {
            Self::Cycles1_5
        }
    }
}

pub use sample_time::SampleTime;

pub struct Adc<'d, T: Instance> {
    sample_time: SampleTime,
    vref_mv: u32,
    resolution: Resolution,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Adc<'d, T> {
    pub fn new(_peri: impl Peripheral<P = T> + 'd, delay: &mut impl DelayUs<u32>) -> Self {
        into_ref!(_peri);
        enable();

        // Delay 1μs when using HSI14 as the ADC clock.
        //
        // Table 57. ADC characteristics
        // tstab = 14 * 1/fadc
        delay.delay_us(1);

        let s = Self {
            sample_time: Default::default(),
            vref_mv: VDDA_CALIB_MV,
            resolution: Resolution::default(),
            phantom: PhantomData,
        };
        s.calibrate();
        s
    }

    pub fn enable_vbat(&self, _delay: &mut impl DelayUs<u32>) -> Vbat {
        // SMP must be ≥ 56 ADC clock cycles when using HSI14.
        //
        // 6.3.20 Vbat monitoring characteristics
        // ts_vbat ≥ 4μs
        unsafe {
            T::regs().ccr().modify(|reg| reg.set_vbaten(true));
        }
        Vbat
    }

    pub fn enable_vref(&self, delay: &mut impl DelayUs<u32>) -> Vref {
        // Table 28. Embedded internal reference voltage
        // tstart = 10μs
        unsafe {
            T::regs().ccr().modify(|reg| reg.set_vrefen(true));
        }
        delay.delay_us(10);
        Vref
    }

    pub fn enable_temperature(&self, delay: &mut impl DelayUs<u32>) -> Temperature {
        // SMP must be ≥ 56 ADC clock cycles when using HSI14.
        //
        // 6.3.19 Temperature sensor characteristics
        // tstart ≤ 10μs
        // ts_temp ≥ 4μs
        unsafe {
            T::regs().ccr().modify(|reg| reg.set_tsen(true));
        }
        delay.delay_us(10);
        Temperature
    }

    fn calibrate(&self) {
        unsafe {
            // A.7.1 ADC calibration code example
            if T::regs().cr().read().aden() {
                T::regs().cr().modify(|reg| reg.set_addis(true));
            }
            while T::regs().cr().read().aden() {
                // spin
            }
            T::regs().cfgr1().modify(|reg| reg.set_dmaen(false));
            T::regs().cr().modify(|reg| reg.set_adcal(true));
            while T::regs().cr().read().adcal() {
                // spin
            }
        }
    }

    pub fn set_sample_time(&mut self, sample_time: SampleTime) {
        self.sample_time = sample_time;
    }

    pub fn set_vref_mv(&mut self, vref_mv: u32) {
        self.vref_mv = vref_mv;
    }

    pub fn set_resolution(&mut self, resolution: Resolution) {
        self.resolution = resolution;
    }

    pub fn to_millivolts(&self, sample: u16) -> u16 {
        ((u32::from(sample) * self.vref_mv) / self.resolution.to_max_count()) as u16
    }

    fn convert(&mut self) -> u16 {
        unsafe {
            T::regs().isr().modify(|reg| {
                reg.set_eoc(true);
                reg.set_eosmp(true);
            });

            // A.7.5 Single conversion sequence code example - Software trigger
            T::regs().cr().modify(|reg| reg.set_adstart(true));
            while !T::regs().isr().read().eoc() {
                // spin
            }

            T::regs().dr().read().0 as u16
        }
    }

    pub fn read(&mut self, pin: &mut impl AdcPin<T>) -> u16 {
        unsafe {
            // A.7.2 ADC enable sequence code example
            if T::regs().isr().read().adrdy() {
                T::regs().isr().modify(|reg| reg.set_adrdy(true));
            }
            T::regs().cr().modify(|reg| reg.set_aden(true));
            while !T::regs().isr().read().adrdy() {
                // ES0233, 2.4.3 ADEN bit cannot be set immediately after the ADC calibration
                // Workaround: When the ADC calibration is complete (ADCAL = 0), keep setting the
                // ADEN bit until the ADRDY flag goes high.
                T::regs().cr().modify(|reg| reg.set_aden(true));
            }

            T::regs().cfgr1().modify(|reg| reg.set_res(self.resolution.res()));
            Self::set_channel_sample_time(pin.channel(), self.sample_time);
            T::regs()
                .chselr()
                .write(|reg| reg.set_chselx(pin.channel() as usize, true));

            let value = self.convert();

            // A.7.3 ADC disable code example
            T::regs().cr().modify(|reg| reg.set_adstp(true));
            while T::regs().cr().read().adstp() {
                // spin
            }
            T::regs().cr().modify(|reg| reg.set_addis(true));
            while T::regs().cr().read().aden() {
                // spin
            }

            value
        }
    }

    unsafe fn set_channel_sample_time(_ch: u8, sample_time: SampleTime) {
        T::regs().smpr().modify(|reg| reg.set_smp(sample_time.sample_time()));
    }
}

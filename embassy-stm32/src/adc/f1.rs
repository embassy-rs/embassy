use crate::adc::{AdcPin, Instance};
use crate::rcc::get_freqs;
use crate::time::Hertz;
use core::marker::PhantomData;
use embassy::util::Unborrow;
use embassy_hal_common::unborrow;
use embedded_hal_02::blocking::delay::DelayUs;

pub const VDDA_CALIB_MV: u32 = 3300;
pub const ADC_MAX: u32 = (1 << 12) - 1;
// No calibration data for F103, voltage should be 1.2v
pub const VREF_INT: u32 = 1200;

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
    /// ADC sample time
    ///
    /// The default setting is 1.5 ADC clock cycles.
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
        pub(crate) fn sample_time(&self) -> crate::pac::adc::vals::SampleTime {
            match self {
                SampleTime::Cycles1_5 => crate::pac::adc::vals::SampleTime::CYCLES1_5,
                SampleTime::Cycles7_5 => crate::pac::adc::vals::SampleTime::CYCLES7_5,
                SampleTime::Cycles13_5 => crate::pac::adc::vals::SampleTime::CYCLES13_5,
                SampleTime::Cycles28_5 => crate::pac::adc::vals::SampleTime::CYCLES28_5,
                SampleTime::Cycles41_5 => crate::pac::adc::vals::SampleTime::CYCLES41_5,
                SampleTime::Cycles55_5 => crate::pac::adc::vals::SampleTime::CYCLES55_5,
                SampleTime::Cycles71_5 => crate::pac::adc::vals::SampleTime::CYCLES71_5,
                SampleTime::Cycles239_5 => crate::pac::adc::vals::SampleTime::CYCLES239_5,
            }
        }
    }

    impl Default for SampleTime {
        fn default() -> Self {
            Self::Cycles28_5
        }
    }
}

pub use sample_time::SampleTime;

pub struct Adc<'d, T: Instance> {
    sample_time: SampleTime,
    calibrated_vdda: u32,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Adc<'d, T> {
    pub fn new(_peri: impl Unborrow<Target = T> + 'd, delay: &mut impl DelayUs<u32>) -> Self {
        unborrow!(_peri);
        T::enable();
        T::reset();
        unsafe {
            T::regs().cr2().modify(|reg| reg.set_adon(true));
        }

        // 11.4: Before starting a calibration, the ADC must have been in power-on state (ADON bit = ‘1’)
        // for at least two ADC clock cycles
        delay.delay_us((1_000_000 * 2) / Self::freq().0 + 1);

        unsafe {
            // Reset calibration
            T::regs().cr2().modify(|reg| reg.set_rstcal(true));
            while T::regs().cr2().read().rstcal() {
                // spin
            }

            // Calibrate
            T::regs().cr2().modify(|reg| reg.set_cal(true));
            while T::regs().cr2().read().cal() {
                // spin
            }
        }

        // One cycle after calibration
        delay.delay_us((1_000_000) / Self::freq().0 + 1);

        Self {
            sample_time: Default::default(),
            calibrated_vdda: VDDA_CALIB_MV,
            phantom: PhantomData,
        }
    }

    fn freq() -> Hertz {
        unsafe { get_freqs() }.adc
    }

    pub fn sample_time_for_us(&self, us: u32) -> SampleTime {
        match us * Self::freq().0 / 1_000_000 {
            0..=1 => SampleTime::Cycles1_5,
            2..=7 => SampleTime::Cycles7_5,
            8..=13 => SampleTime::Cycles13_5,
            14..=28 => SampleTime::Cycles28_5,
            29..=41 => SampleTime::Cycles41_5,
            42..=55 => SampleTime::Cycles55_5,
            56..=71 => SampleTime::Cycles71_5,
            _ => SampleTime::Cycles239_5,
        }
    }

    pub fn enable_vref(&self, _delay: &mut impl DelayUs<u32>) -> Vref {
        unsafe {
            T::regs().cr2().modify(|reg| {
                reg.set_tsvrefe(true);
            })
        }
        Vref {}
    }

    pub fn enable_temperature(&self) -> Temperature {
        unsafe {
            T::regs().cr2().modify(|reg| {
                reg.set_tsvrefe(true);
            })
        }
        Temperature {}
    }

    /// Calculates the system VDDA by sampling the internal VREF channel and comparing
    /// to the expected value. If the chip's VDDA is not stable, run this before each ADC
    /// conversion.
    pub fn calibrate(&mut self, vref: &mut Vref) -> u32 {
        let old_sample_time = self.sample_time;
        self.sample_time = SampleTime::Cycles239_5;

        let vref_samp = self.read(vref);
        self.sample_time = old_sample_time;

        self.calibrated_vdda = (ADC_MAX * VREF_INT) / u32::from(vref_samp);
        self.calibrated_vdda
    }

    pub fn set_sample_time(&mut self, sample_time: SampleTime) {
        self.sample_time = sample_time;
    }

    /// Convert a measurement to millivolts
    pub fn to_millivolts(&self, sample: u16) -> u16 {
        ((u32::from(sample) * self.calibrated_vdda) / ADC_MAX) as u16
    }

    /// Perform a single conversion.
    fn convert(&mut self) -> u16 {
        unsafe {
            T::regs().cr2().modify(|reg| {
                reg.set_adon(true);
                reg.set_swstart(true);
            });
            while T::regs().cr2().read().swstart() {}
            while !T::regs().sr().read().eoc() {}

            T::regs().dr().read().0 as u16
        }
    }

    pub fn read(&mut self, pin: &mut impl AdcPin<T>) -> u16 {
        unsafe {
            Self::set_channel_sample_time(pin.channel(), self.sample_time);
            T::regs().cr1().modify(|reg| {
                reg.set_scan(false);
                reg.set_discen(false);
            });
            T::regs().sqr1().modify(|reg| reg.set_l(0));

            T::regs().cr2().modify(|reg| {
                reg.set_cont(false);
                reg.set_exttrig(true);
                reg.set_swstart(false);
                reg.set_extsel(crate::pac::adc::vals::Extsel::SWSTART);
            });
        }

        // Configure the channel to sample
        unsafe { T::regs().sqr3().write(|reg| reg.set_sq(0, pin.channel())) }
        self.convert()
    }

    unsafe fn set_channel_sample_time(ch: u8, sample_time: SampleTime) {
        if ch <= 9 {
            T::regs()
                .smpr2()
                .modify(|reg| reg.set_smp(ch as _, sample_time.sample_time()));
        } else {
            T::regs()
                .smpr1()
                .modify(|reg| reg.set_smp((ch - 10) as _, sample_time.sample_time()));
        }
    }
}

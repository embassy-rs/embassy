use crate::adc::{AdcPin, Instance};
use core::marker::PhantomData;
use embassy::util::Unborrow;
use embassy_hal_common::unborrow;
use embedded_hal_02::blocking::delay::DelayUs;

pub const VDDA_CALIB_MV: u32 = 3000;

#[cfg(not(rcc_f4))]
unsafe fn enable() {
    todo!()
}

#[cfg(rcc_f4)]
unsafe fn enable() {
    // TODO do not enable all adc clocks if not needed
    crate::pac::RCC.apb2enr().modify(|w| w.set_adc1en(true));
    crate::pac::RCC.apb2enr().modify(|w| w.set_adc2en(true));
    crate::pac::RCC.apb2enr().modify(|w| w.set_adc3en(true));
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
    fn res(&self) -> crate::pac::adc::vals::Res {
        match self {
            Resolution::TwelveBit => crate::pac::adc::vals::Res::TWELVEBIT,
            Resolution::TenBit => crate::pac::adc::vals::Res::TENBIT,
            Resolution::EightBit => crate::pac::adc::vals::Res::EIGHTBIT,
            Resolution::SixBit => crate::pac::adc::vals::Res::SIXBIT,
        }
    }

    fn to_max_count(&self) -> u32 {
        match self {
            Resolution::TwelveBit => (1 << 12) - 1,
            Resolution::TenBit => (1 << 10) - 1,
            Resolution::EightBit => (1 << 8) - 1,
            Resolution::SixBit => (1 << 6) - 1,
        }
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

pub struct Vbat;
impl<T: Instance> AdcPin<T> for Vbat {}
impl<T: Instance> super::sealed::AdcPin<T> for Vbat {
    fn channel(&self) -> u8 {
        18
    }
}

/// ADC sample time
///
/// The default setting is 3 ADC clock cycles.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SampleTime {
    Cycles3 = 0b000,
    Cycles15 = 0b001,
    Cycles28 = 0b010,
    Cycles56 = 0b011,
    Cycles85 = 0b100,
    Cycles112 = 0b101,
    Cycles144 = 0b110,
    Cycles480 = 0b111,
}

impl SampleTime {
    pub(crate) fn sample_time(&self) -> crate::pac::adc::vals::Smp {
        match self {
            SampleTime::Cycles3 => crate::pac::adc::vals::Smp::CYCLES3,
            SampleTime::Cycles15 => crate::pac::adc::vals::Smp::CYCLES15,
            SampleTime::Cycles28 => crate::pac::adc::vals::Smp::CYCLES28,
            SampleTime::Cycles56 => crate::pac::adc::vals::Smp::CYCLES56,
            SampleTime::Cycles85 => crate::pac::adc::vals::Smp::CYCLES84,
            SampleTime::Cycles112 => crate::pac::adc::vals::Smp::CYCLES112,
            SampleTime::Cycles144 => crate::pac::adc::vals::Smp::CYCLES144,
            SampleTime::Cycles480 => crate::pac::adc::vals::Smp::CYCLES480,
        }
    }
}

impl Default for SampleTime {
    fn default() -> Self {
        Self::Cycles3
    }
}

pub struct Adc<'d, T: Instance> {
    sample_time: SampleTime,
    calibrated_vdda: u32,
    resolution: Resolution,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T> Adc<'d, T>
where
    T: Instance,
{
    pub fn new(_peri: impl Unborrow<Target = T> + 'd, delay: &mut impl DelayUs<u32>) -> Self {
        unborrow!(_peri);
        unsafe {
            enable();
            // disable before config is set
            T::regs().cr2().modify(|reg| {
                reg.set_adon(crate::pac::adc::vals::Adon::DISABLED);
            });
        }

        delay.delay_us(20); // TODO?

        Self {
            sample_time: Default::default(),
            resolution: Resolution::default(),
            calibrated_vdda: VDDA_CALIB_MV,
            phantom: PhantomData,
        }
    }

    pub fn set_sample_time(&mut self, sample_time: SampleTime) {
        self.sample_time = sample_time;
    }

    pub fn set_resolution(&mut self, resolution: Resolution) {
        self.resolution = resolution;
    }

    /// Convert a measurement to millivolts
    pub fn to_millivolts(&self, sample: u16) -> u16 {
        ((u32::from(sample) * self.calibrated_vdda) / self.resolution.to_max_count()) as u16
    }

    /// Perform a single conversion.
    fn convert(&mut self) -> u16 {
        unsafe {
            // clear end of conversion flag
            T::regs().sr().modify(|reg| {
                reg.set_eoc(crate::pac::adc::vals::Eoc::NOTCOMPLETE);
            });

            // Start conversion
            T::regs().cr2().modify(|reg| {
                reg.set_swstart(true);
            });

            while T::regs().sr().read().strt() == crate::pac::adc::vals::Strt::NOTSTARTED {
                // spin //wait for actual start
            }
            while T::regs().sr().read().eoc() == crate::pac::adc::vals::Eoc::NOTCOMPLETE {
                // spin //wait for finish
            }

            T::regs().dr().read().0 as u16
        }
    }

    pub fn read<P>(&mut self, pin: &mut P) -> u16
    where
        P: AdcPin<T>,
        P: crate::gpio::sealed::Pin,
    {
        unsafe {
            // dissable ADC
            T::regs().cr2().modify(|reg| {
                reg.set_swstart(false);
            });
            T::regs().cr2().modify(|reg| {
                reg.set_adon(crate::pac::adc::vals::Adon::DISABLED);
            });

            pin.set_as_analog();

            // Configure ADC
            T::regs()
                .cr1()
                .modify(|reg| reg.set_res(self.resolution.res()));

            // Select channel
            T::regs().sqr3().write(|reg| reg.set_sq(0, pin.channel()));

            // Configure channel
            Self::set_channel_sample_time(pin.channel(), self.sample_time);

            // enable adc
            T::regs().cr2().modify(|reg| {
                reg.set_adon(crate::pac::adc::vals::Adon::ENABLED);
            });

            let val = self.convert();

            // dissable ADC
            T::regs().cr2().modify(|reg| {
                reg.set_swstart(false);
            });
            T::regs().cr2().modify(|reg| {
                reg.set_adon(crate::pac::adc::vals::Adon::DISABLED);
            });

            val
        }
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

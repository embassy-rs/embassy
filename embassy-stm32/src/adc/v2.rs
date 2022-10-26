use core::marker::PhantomData;

use embassy_hal_common::into_ref;
use embedded_hal_02::blocking::delay::DelayUs;

use super::InternalChannel;
use crate::adc::{AdcPin, Instance};
use crate::peripherals::ADC1;
use crate::time::Hertz;
use crate::Peripheral;

/// Default VREF voltage used for sample conversion to millivolts.
pub const VREF_DEFAULT_MV: u32 = 3300;
/// VREF voltage used for factory calibration of VREFINTCAL register.
pub const VREF_CALIB_MV: u32 = 3300;

/// ADC turn-on time
pub const ADC_POWERUP_TIME_US: u32 = 3;

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

    pub fn to_max_count(&self) -> u32 {
        match self {
            Resolution::TwelveBit => (1 << 12) - 1,
            Resolution::TenBit => (1 << 10) - 1,
            Resolution::EightBit => (1 << 8) - 1,
            Resolution::SixBit => (1 << 6) - 1,
        }
    }
}

pub struct VrefInt;
impl InternalChannel<ADC1> for VrefInt {}
impl super::sealed::InternalChannel<ADC1> for VrefInt {
    fn channel(&self) -> u8 {
        17
    }
}

impl VrefInt {
    /// Time needed for internal voltage reference to stabilize
    pub fn start_time_us() -> u32 {
        10
    }
}

pub struct Temperature;
impl InternalChannel<ADC1> for Temperature {}
impl super::sealed::InternalChannel<ADC1> for Temperature {
    fn channel(&self) -> u8 {
        cfg_if::cfg_if! {
            if #[cfg(any(stm32f40, stm32f41))] {
                16
            } else {
                18
            }
        }
    }
}

impl Temperature {
    /// Converts temperature sensor reading in millivolts to degrees celcius
    pub fn to_celcius(sample_mv: u16) -> f32 {
        // From 6.3.22 Temperature sensor characteristics
        const V25: i32 = 760; // mV
        const AVG_SLOPE: f32 = 2.5; // mV/C

        (sample_mv as i32 - V25) as f32 / AVG_SLOPE + 25.0
    }

    /// Time needed for temperature sensor readings to stabilize
    pub fn start_time_us() -> u32 {
        10
    }
}

pub struct Vbat;
impl InternalChannel<ADC1> for Vbat {}
impl super::sealed::InternalChannel<ADC1> for Vbat {
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

enum Prescaler {
    Div2,
    Div4,
    Div6,
    Div8,
}

impl Prescaler {
    fn from_pclk2(freq: Hertz) -> Self {
        // Datasheet for both F4 and F7 specifies min frequency 0.6 MHz, typ freq. 30 MHz and max 36 MHz.
        const MAX_FREQUENCY: Hertz = Hertz(36_000_000);
        let raw_div = freq.0 / MAX_FREQUENCY.0;
        match raw_div {
            0..=1 => Self::Div2,
            2..=3 => Self::Div4,
            4..=5 => Self::Div6,
            6..=7 => Self::Div8,
            _ => panic!("Selected PCLK2 frequency is too high for ADC with largest possible prescaler."),
        }
    }

    fn adcpre(&self) -> crate::pac::adccommon::vals::Adcpre {
        match self {
            Prescaler::Div2 => crate::pac::adccommon::vals::Adcpre::DIV2,
            Prescaler::Div4 => crate::pac::adccommon::vals::Adcpre::DIV4,
            Prescaler::Div6 => crate::pac::adccommon::vals::Adcpre::DIV6,
            Prescaler::Div8 => crate::pac::adccommon::vals::Adcpre::DIV8,
        }
    }
}

pub struct Adc<'d, T: Instance> {
    sample_time: SampleTime,
    vref_mv: u32,
    resolution: Resolution,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T> Adc<'d, T>
where
    T: Instance,
{
    pub fn new(_peri: impl Peripheral<P = T> + 'd, delay: &mut impl DelayUs<u32>) -> Self {
        into_ref!(_peri);
        T::enable();
        T::reset();

        let presc = Prescaler::from_pclk2(T::frequency());
        unsafe {
            T::common_regs().ccr().modify(|w| w.set_adcpre(presc.adcpre()));

            T::regs().cr2().modify(|reg| {
                reg.set_adon(crate::pac::adc::vals::Adon::ENABLED);
            });
        }

        delay.delay_us(ADC_POWERUP_TIME_US);

        Self {
            sample_time: Default::default(),
            resolution: Resolution::default(),
            vref_mv: VREF_DEFAULT_MV,
            phantom: PhantomData,
        }
    }

    pub fn set_sample_time(&mut self, sample_time: SampleTime) {
        self.sample_time = sample_time;
    }

    pub fn set_resolution(&mut self, resolution: Resolution) {
        self.resolution = resolution;
    }

    /// Set VREF value in millivolts. This value is used for [to_millivolts()] sample conversion.
    ///
    /// Use this if you have a known precise VREF (VDDA) pin reference voltage.
    pub fn set_vref_mv(&mut self, vref_mv: u32) {
        self.vref_mv = vref_mv;
    }

    /// Convert a measurement to millivolts
    pub fn to_millivolts(&self, sample: u16) -> u16 {
        ((u32::from(sample) * self.vref_mv) / self.resolution.to_max_count()) as u16
    }

    /// Enables internal voltage reference and returns [VrefInt], which can be used in
    /// [Adc::read_internal()] to perform conversion.
    pub fn enable_vrefint(&self) -> VrefInt {
        unsafe {
            T::common_regs().ccr().modify(|reg| {
                reg.set_tsvrefe(crate::pac::adccommon::vals::Tsvrefe::ENABLED);
            });
        }

        VrefInt {}
    }

    /// Enables internal temperature sensor and returns [Temperature], which can be used in
    /// [Adc::read_internal()] to perform conversion.
    ///
    /// On STM32F42 and STM32F43 this can not be used together with [Vbat]. If both are enabled,
    /// temperature sensor will return vbat value.
    pub fn enable_temperature(&self) -> Temperature {
        unsafe {
            T::common_regs().ccr().modify(|reg| {
                reg.set_tsvrefe(crate::pac::adccommon::vals::Tsvrefe::ENABLED);
            });
        }

        Temperature {}
    }

    /// Enables vbat input and returns [Vbat], which can be used in
    /// [Adc::read_internal()] to perform conversion.
    pub fn enable_vbat(&self) -> Vbat {
        unsafe {
            T::common_regs().ccr().modify(|reg| {
                reg.set_vbate(crate::pac::adccommon::vals::Vbate::ENABLED);
            });
        }

        Vbat {}
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
            pin.set_as_analog();

            self.read_channel(pin.channel())
        }
    }

    pub fn read_internal(&mut self, channel: &mut impl InternalChannel<T>) -> u16 {
        unsafe { self.read_channel(channel.channel()) }
    }

    unsafe fn read_channel(&mut self, channel: u8) -> u16 {
        // Configure ADC
        T::regs().cr1().modify(|reg| reg.set_res(self.resolution.res()));

        // Select channel
        T::regs().sqr3().write(|reg| reg.set_sq(0, channel));

        // Configure channel
        Self::set_channel_sample_time(channel, self.sample_time);

        let val = self.convert();

        val
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

impl<'d, T: Instance> Drop for Adc<'d, T> {
    fn drop(&mut self) {
        T::disable();
    }
}

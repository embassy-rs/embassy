use embassy_hal_internal::into_ref;
use embedded_hal_02::blocking::delay::DelayUs;

use super::InternalChannel;
use crate::adc::{Adc, AdcPin, Instance, Resolution, SampleTime};
use crate::peripherals::ADC1;
use crate::time::Hertz;
use crate::Peripheral;

/// Default VREF voltage used for sample conversion to millivolts.
pub const VREF_DEFAULT_MV: u32 = 3300;
/// VREF voltage used for factory calibration of VREFINTCAL register.
pub const VREF_CALIB_MV: u32 = 3300;

/// ADC turn-on time
pub const ADC_POWERUP_TIME_US: u32 = 3;

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

impl<'d, T> Adc<'d, T>
where
    T: Instance,
{
    pub fn new(adc: impl Peripheral<P = T> + 'd, delay: &mut impl DelayUs<u32>) -> Self {
        into_ref!(adc);
        T::enable();
        T::reset();

        let presc = Prescaler::from_pclk2(T::frequency());
        T::common_regs().ccr().modify(|w| w.set_adcpre(presc.adcpre()));
        T::regs().cr2().modify(|reg| {
            reg.set_adon(crate::pac::adc::vals::Adon::ENABLED);
        });

        delay.delay_us(ADC_POWERUP_TIME_US);

        Self {
            adc,
            sample_time: Default::default(),
        }
    }

    pub fn set_sample_time(&mut self, sample_time: SampleTime) {
        self.sample_time = sample_time;
    }

    pub fn set_resolution(&mut self, resolution: Resolution) {
        T::regs().cr1().modify(|reg| reg.set_res(resolution.into()));
    }

    /// Enables internal voltage reference and returns [VrefInt], which can be used in
    /// [Adc::read_internal()] to perform conversion.
    pub fn enable_vrefint(&self) -> VrefInt {
        T::common_regs().ccr().modify(|reg| {
            reg.set_tsvrefe(crate::pac::adccommon::vals::Tsvrefe::ENABLED);
        });

        VrefInt {}
    }

    /// Enables internal temperature sensor and returns [Temperature], which can be used in
    /// [Adc::read_internal()] to perform conversion.
    ///
    /// On STM32F42 and STM32F43 this can not be used together with [Vbat]. If both are enabled,
    /// temperature sensor will return vbat value.
    pub fn enable_temperature(&self) -> Temperature {
        T::common_regs().ccr().modify(|reg| {
            reg.set_tsvrefe(crate::pac::adccommon::vals::Tsvrefe::ENABLED);
        });

        Temperature {}
    }

    /// Enables vbat input and returns [Vbat], which can be used in
    /// [Adc::read_internal()] to perform conversion.
    pub fn enable_vbat(&self) -> Vbat {
        T::common_regs().ccr().modify(|reg| {
            reg.set_vbate(crate::pac::adccommon::vals::Vbate::ENABLED);
        });

        Vbat {}
    }

    /// Perform a single conversion.
    fn convert(&mut self) -> u16 {
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
        // Configure ADC

        // Select channel
        T::regs().sqr3().write(|reg| reg.set_sq(0, channel));

        // Configure channel
        Self::set_channel_sample_time(channel, self.sample_time);

        let val = self.convert();

        val
    }

    fn set_channel_sample_time(ch: u8, sample_time: SampleTime) {
        let sample_time = sample_time.into();
        if ch <= 9 {
            T::regs().smpr2().modify(|reg| reg.set_smp(ch as _, sample_time));
        } else {
            T::regs().smpr1().modify(|reg| reg.set_smp((ch - 10) as _, sample_time));
        }
    }
}

impl<'d, T: Instance> Drop for Adc<'d, T> {
    fn drop(&mut self) {
        T::disable();
    }
}

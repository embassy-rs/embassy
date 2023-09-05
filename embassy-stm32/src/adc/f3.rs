use embassy_hal_internal::into_ref;
use embedded_hal_02::blocking::delay::DelayUs;

use crate::adc::{Adc, AdcPin, Instance, SampleTime};
use crate::time::Hertz;
use crate::Peripheral;

pub const VDDA_CALIB_MV: u32 = 3300;
pub const ADC_MAX: u32 = (1 << 12) - 1;
// No calibration data for F103, voltage should be 1.2v
pub const VREF_INT: u32 = 1200;

pub struct Vref;
impl<T: Instance> AdcPin<T> for Vref {}
impl<T: Instance> super::sealed::AdcPin<T> for Vref {
    fn channel(&self) -> u8 {
        18
    }
}

pub struct Temperature;
impl<T: Instance> AdcPin<T> for Temperature {}
impl<T: Instance> super::sealed::AdcPin<T> for Temperature {
    fn channel(&self) -> u8 {
        16
    }
}

impl<'d, T: Instance> Adc<'d, T> {
    pub fn new(adc: impl Peripheral<P = T> + 'd, delay: &mut impl DelayUs<u32>) -> Self {
        use crate::pac::adc::vals;

        into_ref!(adc);

        T::enable();
        T::reset();

        // Enable the adc regulator
        T::regs().cr().modify(|w| w.set_advregen(vals::Advregen::INTERMEDIATE));
        T::regs().cr().modify(|w| w.set_advregen(vals::Advregen::ENABLED));

        // Wait for the regulator to stabilize
        delay.delay_us(10);

        assert!(!T::regs().cr().read().aden());

        // Begin calibration
        T::regs().cr().modify(|w| w.set_adcaldif(false));
        T::regs().cr().modify(|w| w.set_adcal(true));

        while T::regs().cr().read().adcal() {}

        // Enable the adc
        T::regs().cr().modify(|w| w.set_aden(true));

        // Wait until the adc is ready
        while !T::regs().isr().read().adrdy() {}

        Self {
            adc,
            sample_time: Default::default(),
        }
    }

    fn freq() -> Hertz {
        <T as crate::adc::sealed::Instance>::frequency()
    }

    pub fn sample_time_for_us(&self, us: u32) -> SampleTime {
        match us * Self::freq().0 / 1_000_000 {
            0..=1 => SampleTime::Cycles1_5,
            2..=4 => SampleTime::Cycles4_5,
            5..=7 => SampleTime::Cycles7_5,
            8..=19 => SampleTime::Cycles19_5,
            20..=61 => SampleTime::Cycles61_5,
            62..=181 => SampleTime::Cycles181_5,
            _ => SampleTime::Cycles601_5,
        }
    }

    pub fn enable_vref(&self, _delay: &mut impl DelayUs<u32>) -> Vref {
        T::common_regs().ccr().modify(|w| w.set_vrefen(true));

        Vref {}
    }

    pub fn enable_temperature(&self) -> Temperature {
        T::common_regs().ccr().modify(|w| w.set_tsen(true));

        Temperature {}
    }

    pub fn set_sample_time(&mut self, sample_time: SampleTime) {
        self.sample_time = sample_time;
    }

    /// Perform a single conversion.
    fn convert(&mut self) -> u16 {
        T::regs().isr().write(|_| {});
        T::regs().cr().modify(|w| w.set_adstart(true));

        while !T::regs().isr().read().eoc() && !T::regs().isr().read().eos() {}
        T::regs().isr().write(|_| {});

        T::regs().dr().read().0 as u16
    }

    pub fn read(&mut self, pin: &mut impl AdcPin<T>) -> u16 {
        // pin.set_as_analog();

        Self::set_channel_sample_time(pin.channel(), self.sample_time);

        // Configure the channel to sample
        T::regs().sqr3().write(|w| w.set_sq(0, pin.channel()));
        self.convert()
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

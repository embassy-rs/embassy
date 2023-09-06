use embassy_hal_internal::into_ref;
use embedded_hal_02::blocking::delay::DelayUs;

use crate::adc::{Adc, AdcPin, Instance, Resolution, SampleTime};
use crate::Peripheral;

/// Default VREF voltage used for sample conversion to millivolts.
pub const VREF_DEFAULT_MV: u32 = 3300;
/// VREF voltage used for factory calibration of VREFINTCAL register.
pub const VREF_CALIB_MV: u32 = 3000;

/// Sadly we cannot use `RccPeripheral::enable` since devices are quite inconsistent ADC clock
/// configuration.
fn enable() {
    critical_section::with(|_| {
        #[cfg(any(stm32h7, stm32wl))]
        crate::pac::RCC.apb2enr().modify(|w| w.set_adcen(true));
        #[cfg(stm32g0)]
        crate::pac::RCC.apbenr2().modify(|w| w.set_adcen(true));
        #[cfg(any(stm32l4, stm32l5, stm32wb))]
        crate::pac::RCC.ahb2enr().modify(|w| w.set_adcen(true));
    });
}

pub struct VrefInt;
impl<T: Instance> AdcPin<T> for VrefInt {}
impl<T: Instance> super::sealed::AdcPin<T> for VrefInt {
    fn channel(&self) -> u8 {
        #[cfg(not(adc_g0))]
        let val = 0;
        #[cfg(adc_g0)]
        let val = 13;
        val
    }
}

pub struct Temperature;
impl<T: Instance> AdcPin<T> for Temperature {}
impl<T: Instance> super::sealed::AdcPin<T> for Temperature {
    fn channel(&self) -> u8 {
        #[cfg(not(adc_g0))]
        let val = 17;
        #[cfg(adc_g0)]
        let val = 12;
        val
    }
}

pub struct Vbat;
impl<T: Instance> AdcPin<T> for Vbat {}
impl<T: Instance> super::sealed::AdcPin<T> for Vbat {
    fn channel(&self) -> u8 {
        #[cfg(not(adc_g0))]
        let val = 18;
        #[cfg(adc_g0)]
        let val = 14;
        val
    }
}

impl<'d, T: Instance> Adc<'d, T> {
    pub fn new(adc: impl Peripheral<P = T> + 'd, delay: &mut impl DelayUs<u32>) -> Self {
        into_ref!(adc);
        enable();
        T::regs().cr().modify(|reg| {
            #[cfg(not(adc_g0))]
            reg.set_deeppwd(false);
            reg.set_advregen(true);
        });

        #[cfg(adc_g0)]
        T::regs().cfgr1().modify(|reg| {
            reg.set_chselrmod(false);
        });

        delay.delay_us(20);

        T::regs().cr().modify(|reg| {
            reg.set_adcal(true);
        });

        while T::regs().cr().read().adcal() {
            // spin
        }

        delay.delay_us(1);

        Self {
            adc,
            sample_time: Default::default(),
        }
    }

    pub fn enable_vrefint(&self, delay: &mut impl DelayUs<u32>) -> VrefInt {
        #[cfg(not(adc_g0))]
        T::common_regs().ccr().modify(|reg| {
            reg.set_vrefen(true);
        });
        #[cfg(adc_g0)]
        T::regs().ccr().modify(|reg| {
            reg.set_vrefen(true);
        });

        // "Table 24. Embedded internal voltage reference" states that it takes a maximum of 12 us
        // to stabilize the internal voltage reference, we wait a little more.
        // TODO: delay 15us
        //cortex_m::asm::delay(20_000_000);
        delay.delay_us(15);

        VrefInt {}
    }

    pub fn enable_temperature(&self) -> Temperature {
        #[cfg(not(adc_g0))]
        T::common_regs().ccr().modify(|reg| {
            reg.set_ch17sel(true);
        });
        #[cfg(adc_g0)]
        T::regs().ccr().modify(|reg| {
            reg.set_tsen(true);
        });

        Temperature {}
    }

    pub fn enable_vbat(&self) -> Vbat {
        #[cfg(not(adc_g0))]
        T::common_regs().ccr().modify(|reg| {
            reg.set_ch18sel(true);
        });
        #[cfg(adc_g0)]
        T::regs().ccr().modify(|reg| {
            reg.set_vbaten(true);
        });

        Vbat {}
    }

    pub fn set_sample_time(&mut self, sample_time: SampleTime) {
        self.sample_time = sample_time;
    }

    pub fn set_resolution(&mut self, resolution: Resolution) {
        #[cfg(not(adc_g0))]
        T::regs().cfgr().modify(|reg| reg.set_res(resolution.into()));
        #[cfg(adc_g0)]
        T::regs().cfgr1().modify(|reg| reg.set_res(resolution.into()));
    }

    /*
    /// Convert a raw sample from the `Temperature` to deg C
    pub fn to_degrees_centigrade(sample: u16) -> f32 {
        (130.0 - 30.0) / (VtempCal130::get().read() as f32 - VtempCal30::get().read() as f32)
            * (sample as f32 - VtempCal30::get().read() as f32)
            + 30.0
    }
     */

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

    pub fn read(&mut self, pin: &mut impl AdcPin<T>) -> u16 {
        // Make sure bits are off
        while T::regs().cr().read().addis() {
            // spin
        }

        // Enable ADC
        T::regs().isr().modify(|reg| {
            reg.set_adrdy(true);
        });
        T::regs().cr().modify(|reg| {
            reg.set_aden(true);
        });

        while !T::regs().isr().read().adrdy() {
            // spin
        }

        // Configure channel
        Self::set_channel_sample_time(pin.channel(), self.sample_time);

        // Select channel
        #[cfg(not(adc_g0))]
        T::regs().sqr1().write(|reg| reg.set_sq(0, pin.channel()));
        #[cfg(adc_g0)]
        T::regs().chselr().write(|reg| reg.set_chsel(1 << pin.channel()));

        // Some models are affected by an erratum:
        // If we perform conversions slower than 1 kHz, the first read ADC value can be
        // corrupted, so we discard it and measure again.
        //
        // STM32L471xx: Section 2.7.3
        // STM32G4: Section 2.7.3
        #[cfg(any(rcc_l4, rcc_g4))]
        let _ = self.convert();

        let val = self.convert();

        T::regs().cr().modify(|reg| reg.set_addis(true));

        val
    }

    #[cfg(adc_g0)]
    fn set_channel_sample_time(_ch: u8, sample_time: SampleTime) {
        T::regs().smpr().modify(|reg| reg.set_smp1(sample_time.into()));
    }

    #[cfg(not(adc_g0))]
    fn set_channel_sample_time(ch: u8, sample_time: SampleTime) {
        let sample_time = sample_time.into();
        T::regs()
            .smpr(ch as usize / 10)
            .modify(|reg| reg.set_smp(ch as usize % 10, sample_time));
    }
}

use cfg_if::cfg_if;
use embassy_hal_internal::into_ref;

use crate::adc::{Adc, AdcPin, Instance, Resolution, SampleTime};
use crate::Peripheral;

/// Default VREF voltage used for sample conversion to millivolts.
pub const VREF_DEFAULT_MV: u32 = 3300;
/// VREF voltage used for factory calibration of VREFINTCAL register.
pub const VREF_CALIB_MV: u32 = 3000;

pub struct VrefInt;
impl<T: Instance> AdcPin<T> for VrefInt {}
impl<T: Instance> super::SealedAdcPin<T> for VrefInt {
    fn channel(&self) -> u8 {
        cfg_if! {
            if #[cfg(adc_g0)] {
                let val = 13;
            } else if #[cfg(adc_h5)] {
                let val = 17;
            } else {
                let val = 0;
            }
        }
        val
    }
}

pub struct Temperature;
impl<T: Instance> AdcPin<T> for Temperature {}
impl<T: Instance> super::SealedAdcPin<T> for Temperature {
    fn channel(&self) -> u8 {
        cfg_if! {
            if #[cfg(adc_g0)] {
                let val = 12;
            } else if #[cfg(adc_h5)] {
                let val = 16;
            } else {
                let val = 17;
            }
        }
        val
    }
}

pub struct Vbat;
impl<T: Instance> AdcPin<T> for Vbat {}
impl<T: Instance> super::SealedAdcPin<T> for Vbat {
    fn channel(&self) -> u8 {
        cfg_if! {
            if #[cfg(adc_g0)] {
                let val = 14;
            } else if #[cfg(adc_h5)] {
                let val = 2;
            } else {
                let val = 18;
            }
        }
        val
    }
}

cfg_if! {
    if #[cfg(adc_h5)] {
        pub struct VddCore;
        impl<T: Instance> AdcPin<T> for VddCore {}
        impl<T: Instance> super::SealedAdcPin<T> for VddCore {
            fn channel(&self) -> u8 {
                6
            }
        }
    }
}

impl<'d, T: Instance> Adc<'d, T> {
    pub fn new(adc: impl Peripheral<P = T> + 'd) -> Self {
        into_ref!(adc);
        T::enable_and_reset();
        T::regs().cr().modify(|reg| {
            #[cfg(not(adc_g0))]
            reg.set_deeppwd(false);
            reg.set_advregen(true);
        });

        #[cfg(adc_g0)]
        T::regs().cfgr1().modify(|reg| {
            reg.set_chselrmod(false);
        });

        #[cfg(time)]
        embassy_time::block_for(embassy_time::Duration::from_micros(20));
        #[cfg(not(time))]
        cortex_m::asm::delay(unsafe { crate::rcc::get_freqs() }.sys.unwrap().0 / 50_000);

        T::regs().cr().modify(|reg| {
            reg.set_adcal(true);
        });

        while T::regs().cr().read().adcal() {
            // spin
        }

        #[cfg(time)]
        embassy_time::block_for(embassy_time::Duration::from_micros(1));
        #[cfg(not(time))]
        cortex_m::asm::delay(unsafe { crate::rcc::get_freqs() }.sys.unwrap().0 / 1000_000);

        Self {
            adc,
            sample_time: SampleTime::from_bits(0),
        }
    }

    pub fn enable_vrefint(&self) -> VrefInt {
        #[cfg(not(adc_g0))]
        T::common_regs().ccr().modify(|reg| {
            reg.set_vrefen(true);
        });
        #[cfg(adc_g0)]
        T::regs().ccr().modify(|reg| {
            reg.set_vrefen(true);
        });

        // "Table 24. Embedded internal voltage reference" states that it takes a maximum of 12 us
        // to stabilize the internal voltage reference.
        #[cfg(time)]
        embassy_time::block_for(embassy_time::Duration::from_micros(20));
        #[cfg(not(time))]
        cortex_m::asm::delay(unsafe { crate::rcc::get_freqs() }.sys.unwrap().0 / 50_000);

        VrefInt {}
    }

    pub fn enable_temperature(&self) -> Temperature {
        cfg_if! {
            if #[cfg(adc_g0)] {
                T::regs().ccr().modify(|reg| {
                    reg.set_tsen(true);
                });
            } else if #[cfg(adc_h5)] {
                T::common_regs().ccr().modify(|reg| {
                    reg.set_tsen(true);
                });
            } else {
                T::common_regs().ccr().modify(|reg| {
                    reg.set_ch17sel(true);
                });
            }
        }

        Temperature {}
    }

    pub fn enable_vbat(&self) -> Vbat {
        cfg_if! {
            if #[cfg(adc_g0)] {
                T::regs().ccr().modify(|reg| {
                    reg.set_vbaten(true);
                });
            } else if #[cfg(adc_h5)] {
                T::common_regs().ccr().modify(|reg| {
                    reg.set_vbaten(true);
                });
            } else {
                T::common_regs().ccr().modify(|reg| {
                    reg.set_ch18sel(true);
                });
            }
        }

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

        // RM0492, RM0481, etc.
        // "This option bit must be set to 1 when ADCx_INP0 or ADCx_INN1 channel is selected."
        #[cfg(adc_h5)]
        if pin.channel() == 0 {
            T::regs().or().modify(|reg| reg.set_op0(true));
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

        // RM0492, RM0481, etc.
        // "This option bit must be set to 1 when ADCx_INP0 or ADCx_INN1 channel is selected."
        #[cfg(adc_h5)]
        if pin.channel() == 0 {
            T::regs().or().modify(|reg| reg.set_op0(false));
        }

        val
    }

    fn set_channel_sample_time(_ch: u8, sample_time: SampleTime) {
        cfg_if! {
            if #[cfg(adc_g0)] {
                T::regs().smpr().modify(|reg| reg.set_smp1(sample_time.into()));
            } else if #[cfg(adc_h5)] {
                match _ch {
                    0..=9 => T::regs().smpr1().modify(|w| w.set_smp(_ch as usize % 10, sample_time.into())),
                    _ => T::regs().smpr2().modify(|w| w.set_smp(_ch as usize % 10, sample_time.into())),
                }
            } else {
                let sample_time = sample_time.into();
                T::regs()
                    .smpr(_ch as usize / 10)
                    .modify(|reg| reg.set_smp(_ch as usize % 10, sample_time));
            }
        }
    }
}

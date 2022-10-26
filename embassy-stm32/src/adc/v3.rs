use embassy_hal_common::into_ref;
use embedded_hal_02::blocking::delay::DelayUs;

use crate::adc::{Adc, AdcPin, Instance, SingleChannel};
use crate::Peripheral;

/// Default VREF voltage used for sample conversion to millivolts.
pub const VREF_DEFAULT_MV: u32 = 3300;
/// VREF voltage used for factory calibration of VREFINTCAL register.
pub const VREF_CALIB_MV: u32 = 3000;

/// Sadly we cannot use `RccPeripheral::enable` since devices are quite inconsistent ADC clock
/// configuration.
fn enable() {
    critical_section::with(|_| unsafe {
        #[cfg(stm32h7)]
        crate::pac::RCC.apb2enr().modify(|w| w.set_adcen(true));
        #[cfg(stm32g0)]
        crate::pac::RCC.apbenr2().modify(|w| w.set_adcen(true));
        #[cfg(any(stm32l4, stm32l5, stm32wb))]
        crate::pac::RCC.ahb2enr().modify(|w| w.set_adcen(true));
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
impl<T: Instance> AdcPin<T> for VrefInt {}
impl<T: Instance> super::sealed::AdcPin<T> for VrefInt {
    fn channel(&self) -> u8 {
        #[cfg(not(stm32g0))]
        let val = 0;
        #[cfg(stm32g0)]
        let val = 13;
        val
    }
}

pub struct Temperature;
impl<T: Instance> AdcPin<T> for Temperature {}
impl<T: Instance> super::sealed::AdcPin<T> for Temperature {
    fn channel(&self) -> u8 {
        #[cfg(not(stm32g0))]
        let val = 17;
        #[cfg(stm32g0)]
        let val = 12;
        val
    }
}

pub struct Vbat;
impl<T: Instance> AdcPin<T> for Vbat {}
impl<T: Instance> super::sealed::AdcPin<T> for Vbat {
    fn channel(&self) -> u8 {
        #[cfg(not(stm32g0))]
        let val = 18;
        #[cfg(stm32g0)]
        let val = 14;
        val
    }
}

#[cfg(not(adc_g0))]
mod sample_time {
    /// ADC sample time
    ///
    /// The default setting is 2.5 ADC clock cycles.
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
    pub enum SampleTime {
        /// 2.5 ADC clock cycles
        Cycles2_5 = 0b000,

        /// 6.5 ADC clock cycles
        Cycles6_5 = 0b001,

        /// 12.5 ADC clock cycles
        Cycles12_5 = 0b010,

        /// 24.5 ADC clock cycles
        Cycles24_5 = 0b011,

        /// 47.5 ADC clock cycles
        Cycles47_5 = 0b100,

        /// 92.5 ADC clock cycles
        Cycles92_5 = 0b101,

        /// 247.5 ADC clock cycles
        Cycles247_5 = 0b110,

        /// 640.5 ADC clock cycles
        Cycles640_5 = 0b111,
    }

    impl SampleTime {
        pub(crate) fn sample_time(&self) -> crate::pac::adc::vals::SampleTime {
            match self {
                SampleTime::Cycles2_5 => crate::pac::adc::vals::SampleTime::CYCLES2_5,
                SampleTime::Cycles6_5 => crate::pac::adc::vals::SampleTime::CYCLES6_5,
                SampleTime::Cycles12_5 => crate::pac::adc::vals::SampleTime::CYCLES12_5,
                SampleTime::Cycles24_5 => crate::pac::adc::vals::SampleTime::CYCLES24_5,
                SampleTime::Cycles47_5 => crate::pac::adc::vals::SampleTime::CYCLES47_5,
                SampleTime::Cycles92_5 => crate::pac::adc::vals::SampleTime::CYCLES92_5,
                SampleTime::Cycles247_5 => crate::pac::adc::vals::SampleTime::CYCLES247_5,
                SampleTime::Cycles640_5 => crate::pac::adc::vals::SampleTime::CYCLES640_5,
            }
        }
    }

    impl Default for SampleTime {
        fn default() -> Self {
            Self::Cycles2_5
        }
    }
}

#[cfg(adc_g0)]
mod sample_time {
    /// ADC sample time
    ///
    /// The default setting is 1.5 ADC clock cycles.
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
    pub enum SampleTime {
        /// 1.5 ADC clock cycles
        Cycles1_5 = 0b000,

        /// 3.5 ADC clock cycles
        Cycles3_5 = 0b001,

        /// 7.5 ADC clock cycles
        Cycles7_5 = 0b010,

        /// 12.5 ADC clock cycles
        Cycles12_5 = 0b011,

        /// 19.5 ADC clock cycles
        Cycles19_5 = 0b100,

        /// 39.5 ADC clock cycles
        Cycles39_5 = 0b101,

        /// 79.5 ADC clock cycles
        Cycles79_5 = 0b110,

        /// 160.5 ADC clock cycles
        Cycles160_5 = 0b111,
    }

    impl SampleTime {
        pub(crate) fn sample_time(&self) -> crate::pac::adc::vals::SampleTime {
            match self {
                SampleTime::Cycles1_5 => crate::pac::adc::vals::SampleTime::CYCLES1_5,
                SampleTime::Cycles3_5 => crate::pac::adc::vals::SampleTime::CYCLES3_5,
                SampleTime::Cycles7_5 => crate::pac::adc::vals::SampleTime::CYCLES7_5,
                SampleTime::Cycles12_5 => crate::pac::adc::vals::SampleTime::CYCLES12_5,
                SampleTime::Cycles19_5 => crate::pac::adc::vals::SampleTime::CYCLES19_5,
                SampleTime::Cycles39_5 => crate::pac::adc::vals::SampleTime::CYCLES39_5,
                SampleTime::Cycles79_5 => crate::pac::adc::vals::SampleTime::CYCLES79_5,
                SampleTime::Cycles160_5 => crate::pac::adc::vals::SampleTime::CYCLES160_5,
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

impl<'d, T: Instance> Adc<'d, T> {
    pub fn new(adc: impl Peripheral<P = T> + 'd, delay: &mut impl DelayUs<u32>) -> Self {
        into_ref!(adc);
        enable();
        unsafe {
            T::regs().cr().modify(|reg| {
                #[cfg(not(adc_g0))]
                reg.set_deeppwd(false);
                reg.set_advregen(true);
            });

            #[cfg(adc_g0)]
            T::regs().cfgr1().modify(|reg| {
                reg.set_chselrmod(true);
            });
        }

        delay.delay_us(20);

        unsafe {
            T::regs().cr().modify(|reg| {
                reg.set_adcal(true);
            });

            while T::regs().cr().read().adcal() {
                // spin
            }
        }

        delay.delay_us(1);

        Self { adc }
    }

    pub fn enable_vrefint(&self, delay: &mut impl DelayUs<u32>) -> VrefInt {
        unsafe {
            T::common_regs().ccr().modify(|reg| {
                reg.set_vrefen(true);
            });
        }

        // "Table 24. Embedded internal voltage reference" states that it takes a maximum of 12 us
        // to stabilize the internal voltage reference, we wait a little more.
        // TODO: delay 15us
        //cortex_m::asm::delay(20_000_000);
        delay.delay_us(15);

        VrefInt {}
    }

    pub fn enable_temperature(&self) -> Temperature {
        unsafe {
            T::common_regs().ccr().modify(|reg| {
                reg.set_ch17sel(true);
            });
        }

        Temperature {}
    }

    pub fn enable_vbat(&self) -> Vbat {
        unsafe {
            T::common_regs().ccr().modify(|reg| {
                reg.set_ch18sel(true);
            });
        }

        Vbat {}
    }

    /*
    /// Convert a raw sample from the `Temperature` to deg C
    pub fn to_degrees_centigrade(sample: u16) -> f32 {
        (130.0 - 30.0) / (VtempCal130::get().read() as f32 - VtempCal30::get().read() as f32)
            * (sample as f32 - VtempCal30::get().read() as f32)
            + 30.0
    }
     */

    pub fn single_channel<'a>(
        &'a mut self,
        pin: &'a mut impl AdcPin<T>,
        sample_time: SampleTime,
        resolution: Resolution,
    ) -> SingleChannel<'a, T> {
        unsafe {
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

            // Configure ADC
            #[cfg(not(stm32g0))]
            T::regs().cfgr().modify(|reg| reg.set_res(resolution.res()));
            #[cfg(stm32g0)]
            T::regs().cfgr1().modify(|reg| reg.set_res(resolution.res()));

            // Configure channel
            Self::set_channel_sample_time(pin.channel(), sample_time);

            // Select channel
            #[cfg(not(stm32g0))]
            T::regs().sqr1().write(|reg| reg.set_sq(0, pin.channel()));
            #[cfg(stm32g0)]
            T::regs().chselr().write(|reg| reg.set_chsel(pin.channel() as u32));
        }

        SingleChannel {
            adc: self.adc.reborrow(),
        }
    }

    pub fn read(&mut self, pin: &mut impl AdcPin<T>, sample_time: SampleTime, resolution: Resolution) -> u16 {
        self.single_channel(pin, sample_time, resolution).read()
    }

    #[cfg(stm32g0)]
    unsafe fn set_channel_sample_time(_ch: u8, sample_time: SampleTime) {
        T::regs().smpr().modify(|reg| reg.set_smp1(sample_time.sample_time()));
    }

    #[cfg(not(stm32g0))]
    unsafe fn set_channel_sample_time(ch: u8, sample_time: SampleTime) {
        if ch <= 9 {
            T::regs()
                .smpr1()
                .modify(|reg| reg.set_smp(ch as _, sample_time.sample_time()));
        } else {
            T::regs()
                .smpr2()
                .modify(|reg| reg.set_smp((ch - 10) as _, sample_time.sample_time()));
        }
    }
}

impl<'d, T: Instance> Drop for SingleChannel<'d, T> {
    fn drop(&mut self) {
        unsafe {
            T::regs().cr().modify(|reg| reg.set_addis(true));
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

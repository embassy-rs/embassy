use crate::adc::{AdcPin, Instance};
use core::marker::PhantomData;
use embassy::util::Unborrow;
use embassy_hal_common::unborrow;
use embedded_hal_02::blocking::delay::DelayUs;

pub const VDDA_CALIB_MV: u32 = 3000;

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

pub struct Adc<'d, T: Instance> {
    sample_time: SampleTime,
    calibrated_vdda: u32,
    resolution: Resolution,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Adc<'d, T> {
    pub fn new(_peri: impl Unborrow<Target = T> + 'd, delay: &mut impl DelayUs<u32>) -> Self {
        unborrow!(_peri);
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

        Self {
            sample_time: Default::default(),
            resolution: Resolution::default(),
            calibrated_vdda: VDDA_CALIB_MV,
            phantom: PhantomData,
        }
    }

    pub fn enable_vref(&self, delay: &mut impl DelayUs<u32>) -> Vref {
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

        Vref {}
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

    /// Calculates the system VDDA by sampling the internal VREF channel and comparing
    /// the result with the value stored at the factory. If the chip's VDDA is not stable, run
    /// this before each ADC conversion.
    #[cfg(not(stm32g0))] // TODO is this supposed to be public?
    #[allow(unused)] // TODO is this supposed to be public?
    fn calibrate(&mut self, vref: &mut Vref) {
        #[cfg(stm32l5)]
        let vref_cal: u32 = todo!();
        #[cfg(not(stm32l5))]
        let vref_cal = unsafe { crate::pac::VREFINTCAL.data().read().value() };
        let old_sample_time = self.sample_time;

        // "Table 24. Embedded internal voltage reference" states that the sample time needs to be
        // at a minimum 4 us. With 640.5 ADC cycles we have a minimum of 8 us at 80 MHz, leaving
        // some headroom.
        self.sample_time = SampleTime::Cycles640_5;

        // This can't actually fail, it's just in a result to satisfy hal trait
        let vref_samp = self.read(vref);

        self.sample_time = old_sample_time;

        self.calibrated_vdda = (VDDA_CALIB_MV * u32::from(vref_cal)) / u32::from(vref_samp);
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
        unsafe {
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
    }

    pub fn read(&mut self, pin: &mut impl AdcPin<T>) -> u16 {
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
            T::regs()
                .cfgr()
                .modify(|reg| reg.set_res(self.resolution.res()));
            #[cfg(stm32g0)]
            T::regs()
                .cfgr1()
                .modify(|reg| reg.set_res(self.resolution.res()));

            // Configure channel
            Self::set_channel_sample_time(pin.channel(), self.sample_time);

            // Select channel
            #[cfg(not(stm32g0))]
            T::regs().sqr1().write(|reg| reg.set_sq(0, pin.channel()));
            #[cfg(stm32g0)]
            T::regs()
                .chselr()
                .write(|reg| reg.set_chsel(pin.channel() as u32));

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
    }

    #[cfg(stm32g0)]
    unsafe fn set_channel_sample_time(_ch: u8, sample_time: SampleTime) {
        T::regs()
            .smpr()
            .modify(|reg| reg.set_smp1(sample_time.sample_time()));
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

use crate::adc::{AdcPin, Instance};
use core::marker::PhantomData;
use embassy::util::Unborrow;
use embassy_hal_common::unborrow;
use embedded_hal::blocking::delay::DelayUs;

pub const VDDA_CALIB_MV: u32 = 3000;

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
        0
    }
}

pub struct Temperature;
impl<T: Instance> AdcPin<T> for Temperature {}
impl<T: Instance> super::sealed::AdcPin<T> for Temperature {
    fn channel(&self) -> u8 {
        17
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
    fn sample_time(&self) -> crate::pac::adc::vals::SampleTime {
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

pub struct Adc<'d, T: Instance> {
    sample_time: SampleTime,
    calibrated_vdda: u32,
    resolution: Resolution,
    phantom: PhantomData<&'d mut T>,
}

impl<'d, T: Instance> Adc<'d, T> {
    pub fn new(_peri: impl Unborrow<Target = T> + 'd, delay: &mut impl DelayUs<u32>) -> Self {
        unborrow!(_peri);
        unsafe {
            T::regs().cr().modify(|reg| {
                reg.set_deeppwd(false);
                reg.set_advregen(true);
            });
        }

        delay.delay_us(20);

        unsafe {
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
    #[allow(unused)] // TODO is this supposed to be public?
    fn calibrate(&mut self, vref: &mut Vref) {
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
            T::regs()
                .cfgr()
                .modify(|reg| reg.set_res(self.resolution.res()));

            // Configure channel
            Self::set_channel_sample_time(pin.channel(), self.sample_time);

            // Select channel
            T::regs().sqr1().write(|reg| reg.set_sq(0, pin.channel()));

            // Start conversion
            T::regs().isr().modify(|reg| {
                reg.set_eos(true);
                reg.set_eoc(true);
            });
            T::regs().cr().modify(|reg| {
                reg.set_adstart(true);
            });

            while !T::regs().isr().read().eos() {
                // spin
            }

            // Read ADC value first time and discard it, as per errata sheet.
            // The errata states that if we do conversions slower than 1 kHz, the
            // first read ADC value can be corrupted, so we discard it and measure again.

            let _ = T::regs().dr().read();

            T::regs().isr().modify(|reg| {
                reg.set_eos(true);
                reg.set_eoc(true);
            });
            T::regs().cr().modify(|reg| {
                reg.set_adstart(true);
            });

            while !T::regs().isr().read().eos() {
                // spin
            }

            let val = T::regs().dr().read().0 as u16;

            T::regs().cr().modify(|reg| reg.set_addis(true));

            val
        }
    }

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

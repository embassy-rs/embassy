use core::marker::PhantomData;

use stm32_metapac::adc::regs::{Smpr0, Smpr1, Smpr2, Smpr3, Sqr1, Sqr2, Sqr3, Sqr4, Sqr5};

use super::Resolution;
use crate::adc::{Adc, AdcRegs, ConversionMode, DefaultInstance, Instance, SampleTime, Temperature, VrefInt};
use crate::interrupt::typelevel::Interrupt;
use crate::time::Hertz;
use crate::{Peri, interrupt, rcc};

const ADC_FREQ: Hertz = crate::rcc::HSI_FREQ;

pub const VDDA_CALIB_MV: u32 = 3300;
pub const ADC_MAX: u32 = (1 << 12) - 1;
pub const VREF_INT: u32 = 1230;

#[derive(Copy, Clone)]
pub enum AdcPowerMode {
    AlwaysOn,
    DelayOff,
    IdleOff,
    DelayIdleOff,
}

#[derive(Copy, Clone)]
pub enum Prescaler {
    Div1,
    Div2,
    Div3,
    Div4,
}

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: DefaultInstance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        if T::regs().sr().read().eoc() {
            T::regs().cr1().modify(|w| w.set_eocie(false));
        } else {
            return;
        }

        T::state().waker.wake();
    }
}

impl<T: Instance> super::ConverterFor<VrefInt> for T {
    const CHANNEL: u8 = 17;
}

impl<T: Instance> super::ConverterFor<Temperature> for T {
    const CHANNEL: u8 = 16;
}

pub struct Calibration {
    vref_cal: u16,
    vref_val: u16,
}

impl Calibration {
    /// The millivolts that the calibration value was measured at
    pub const CALIBRATION_UV: u32 = 3_000_000;

    /// Returns the measured VddA in microvolts (uV)
    pub fn vdda_uv(&self) -> u32 {
        (Self::CALIBRATION_UV * self.vref_cal as u32) / self.vref_val as u32
    }

    /// Returns the measured VddA as an f32
    pub fn vdda_f32(&self) -> f32 {
        (Self::CALIBRATION_UV as f32 / 1_000.0) * (self.vref_cal as f32 / self.vref_val as f32)
    }

    /// Returns a calibrated voltage value as in microvolts (uV)
    pub fn cal_uv(&self, raw: u16, resolution: super::Resolution) -> u32 {
        (self.vdda_uv() / super::resolution_to_max_count(resolution)) * raw as u32
    }

    /// Returns a calibrated voltage value as an f32
    pub fn cal_f32(&self, raw: u16, resolution: super::Resolution) -> f32 {
        raw as f32 * self.vdda_f32() / super::resolution_to_max_count(resolution) as f32
    }
}

impl AdcRegs for crate::pac::adc::Adc {
    fn data(&self) -> *mut u16 {
        crate::pac::adc::Adc::dr(*self).as_ptr() as *mut u16
    }

    fn enable(&self) {
        if !(self.sr().read().adons() || self.cr2().read().adon()) {
            self.cr2().modify(|w| w.set_adon(true));

            while !self.sr().read().adons() {}
        }

        // Wait for "sample ready"
        while self.sr().read().rcnr() {}
    }

    fn start(&self) {
        self.cr1().modify(|w| {
            w.set_eocie(true);
            w.set_scan(false);
        });

        self.cr2().modify(|w| {
            w.set_swstart(true);
            w.set_cont(false);
        });
    }

    fn stop(&self, disable: bool) {
        if self.cr2().read().adon() {
            while !self.csr().read().adons1() {}
        }

        self.cr2().modify(|w| w.set_adon(false));

        while self.csr().read().adons1() {}

        if disable {
            while !self.sr().read().adons() {}

            self.cr2().modify(|w| w.set_adon(false));
        }
    }

    fn wait_done(&self) -> bool {
        self.sr().read().eoc()
    }

    fn configure_dma(&self, conversion_mode: ConversionMode) {
        // Clear all status flags before configuring DMA.
        self.sr().modify(|w| {
            w.set_eoc(false);
            w.set_ovr(true);
        });

        self.cr1().modify(|w| {
            // Enable end of conversion interrupt only in repeated mode.
            w.set_eocie(true);
        });

        self.cr2().modify(|w| {
            w.set_dma(!matches!(conversion_mode, ConversionMode::NoDma));
            w.set_cont(false);
        });
    }

    fn configure_sequence(&self, sequence: impl ExactSizeIterator<Item = ((u8, bool), SampleTime)>) {
        let mut sqr1 = Sqr1::default();
        let mut sqr2 = Sqr2::default();
        let mut sqr3 = Sqr3::default();
        let mut sqr4 = Sqr4::default();
        let mut sqr5 = Sqr5::default();

        let mut smpr0 = Smpr0::default();
        let mut smpr1 = Smpr1::default();
        let mut smpr2 = Smpr2::default();
        let mut smpr3 = Smpr3::default();

        // Check the sequence is long enough
        sqr1.set_l((sequence.len() - 1).try_into().unwrap());

        for (i, ((ch, _), sample_time)) in sequence.enumerate() {
            match i {
                0..=5 => sqr5.set_sq(i, ch),
                6..=11 => sqr4.set_sq(i - 6, ch),
                12..=15 => sqr3.set_sq(i - 12, ch),
                18..=23 => sqr2.set_sq(i - 18, ch),
                24..=27 => sqr1.set_sq(i - 24, ch),
                _ => unreachable!(),
            }

            let sample_time = sample_time.into();
            match ch {
                0..=9 => smpr3.set_smp(ch as _, sample_time),
                10..=19 => smpr2.set_smp(ch as usize - 10, sample_time),
                20..=29 => smpr1.set_smp(ch as usize - 20, sample_time),
                30..=31 => smpr0.set_smp(ch as usize - 30, sample_time),
                _ => panic!("Invalid channel to sample"),
            }
        }

        self.sqr1().write_value(sqr1);
        self.sqr2().write_value(sqr2);
        self.sqr3().write_value(sqr3);
        self.sqr4().write_value(sqr4);
        self.sqr5().write_value(sqr5);

        self.smpr0().write_value(smpr0);
        self.smpr1().write_value(smpr1);
        self.smpr2().write_value(smpr2);
        self.smpr3().write_value(smpr3);
    }
}

impl<'d, T: DefaultInstance> Adc<'d, T> {
    pub fn new(
        adc: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        rcc::enable_and_reset::<T>();

        //let r = T::regs();
        //r.cr2().write(|w| w.set_align(true));

        T::Interrupt::unpend();
        unsafe {
            T::Interrupt::enable();
        }

        Self { adc }
    }

    fn freq() -> Hertz {
        let div = T::regs().ccr().read().adcpre() + 1;
        ADC_FREQ / div as u32
    }

    pub async fn set_resolution(&mut self, res: Resolution) {
        T::regs().stop(false);
        T::regs().cr1().modify(|w| w.set_res(res.into()));
    }

    pub fn resolution(&self) -> Resolution {
        T::regs().cr1().read().res()
    }

    pub fn enable_vref(&mut self) -> VrefInt {
        T::regs().ccr().modify(|w| w.set_tsvrefe(true));

        VrefInt {}
    }

    pub fn enable_temperature(&mut self) -> Temperature {
        T::regs().ccr().modify(|w| w.set_tsvrefe(true));

        Temperature {}
    }

    fn get_res_clks(res: Resolution) -> u32 {
        match res {
            Resolution::Bits12 => 12,
            Resolution::Bits10 => 11,
            Resolution::Bits8 => 9,
            Resolution::Bits6 => 7,
        }
    }

    fn get_sample_time_clks(sample_time: SampleTime) -> u32 {
        match sample_time {
            SampleTime::Cycles4 => 4,
            SampleTime::Cycles9 => 9,
            SampleTime::Cycles16 => 16,
            SampleTime::Cycles24 => 24,
            SampleTime::Cycles48 => 48,
            SampleTime::Cycles96 => 96,
            SampleTime::Cycles192 => 192,
            SampleTime::Cycles384 => 384,
        }
    }

    pub fn sample_time_for_us(&self, us: u32) -> SampleTime {
        let res_clks = Self::get_res_clks(self.resolution());
        let us_clks = us * Self::freq().0 / 1_000_000;
        let clks = us_clks.saturating_sub(res_clks);
        match clks {
            0..=4 => SampleTime::Cycles4,
            5..=9 => SampleTime::Cycles9,
            10..=16 => SampleTime::Cycles16,
            17..=24 => SampleTime::Cycles24,
            25..=48 => SampleTime::Cycles48,
            49..=96 => SampleTime::Cycles96,
            97..=192 => SampleTime::Cycles192,
            193.. => SampleTime::Cycles384,
        }
    }

    pub fn us_for_cfg(&self, res: Resolution, sample_time: SampleTime) -> u32 {
        let res_clks = Self::get_res_clks(res);
        let sample_clks = Self::get_sample_time_clks(sample_time);
        (res_clks + sample_clks) * 1_000_000 / Self::freq().0
    }
}

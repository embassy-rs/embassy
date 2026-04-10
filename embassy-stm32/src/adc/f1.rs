use core::marker::PhantomData;

use stm32_metapac::adc::regs::{Smpr1, Smpr2, Sqr1, Sqr2, Sqr3};

use super::blocking_delay_us;
use crate::adc::{Adc, AdcRegs, ConversionMode, DefaultInstance, Instance, SampleTime, VrefInt};
use crate::interrupt::typelevel::Interrupt;
use crate::interrupt::{self};
use crate::time::Hertz;
use crate::{Peri, rcc};

pub const VDDA_CALIB_MV: u32 = 3300;
pub const ADC_MAX: u32 = (1 << 12) - 1;
// No calibration data for F103, voltage should be 1.2v
pub const VREF_INT: u32 = 1200;

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: DefaultInstance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        if T::regs().sr().read().eoc() {
            T::regs().cr1().modify(|w| w.set_eocie(false)); // End of Convert interrupt disable
            T::state().waker.wake();
        }
    }
}

impl<T: Instance> super::ConverterFor<VrefInt> for T {
    const CHANNEL: u8 = 17;
}

impl<T: Instance> super::ConverterFor<super::Temperature> for T {
    const CHANNEL: u8 = 16;
}

impl AdcRegs for crate::pac::adc::Adc {
    fn data(&self) -> *mut u16 {
        crate::pac::adc::Adc::dr(*self).as_ptr() as *mut u16
    }

    fn enable(&self) {
        self.cr2().modify(|reg| {
            reg.set_adon(true);
        });

        blocking_delay_us(3);
    }

    fn start(&self) {
        self.sr().write(|reg| {
            reg.set_eoc(false);
        });

        // Begin ADC conversions
        self.cr2().modify(|reg| {
            reg.set_swstart(true);
        });
    }

    fn stop(&self, _disable: bool) {
        // Stop ADC
        self.cr2().modify(|reg| {
            // Stop ADC
            reg.set_swstart(false);
            // Stop ADC
            reg.set_adon(false);
            // Stop DMA
            reg.set_dma(false);
        });

        self.cr1().modify(|w| {
            // Disable interrupt for end of conversion
            w.set_eocie(false);
        });
    }

    fn wait_done(&self) -> bool {
        self.sr().read().eoc()
    }

    fn configure_dma(&self, conversion_mode: ConversionMode) {
        // Clear all status flags before configuring DMA.
        self.sr().modify(|regs| {
            regs.set_eoc(false);
            regs.set_strt(false);
        });

        self.cr1().modify(|w| {
            // Enable end of conversion interrupt only in repeated mode.
            w.set_eocie(true);
            // Scanning conversions of multiple channels.
            w.set_scan(true);
            // Disable discontinuous mode.
            w.set_discen(false);
        });

        self.cr2().modify(|w| {
            // Enable DMA mode
            w.set_dma(!matches!(conversion_mode, ConversionMode::NoDma));
            // EOC flag is set at the end of each conversion.
            w.set_cont(false);
        });
    }

    fn configure_sequence(&self, sequence: impl ExactSizeIterator<Item = ((u8, bool), SampleTime)>) {
        let mut sqr1 = Sqr1::default();
        let mut sqr2 = Sqr2::default();
        let mut sqr3 = Sqr3::default();

        let mut smpr1 = Smpr1::default();
        let mut smpr2 = Smpr2::default();

        // Check the sequence is long enough
        sqr1.set_l((sequence.len() - 1).try_into().unwrap());

        for (i, ((ch, _), sample_time)) in sequence.enumerate() {
            match i {
                0..=5 => sqr1.set_sq(i, ch),
                6..=11 => sqr2.set_sq(i - 6, ch),
                12..=15 => sqr3.set_sq(i - 12, ch),
                _ => unreachable!(),
            }

            let sample_time = sample_time.into();
            if ch <= 9 {
                smpr1.set_smp(ch as _, sample_time);
            } else {
                smpr2.set_smp((ch - 10) as _, sample_time);
            }
        }

        self.sqr1().write_value(sqr1);
        self.sqr2().write_value(sqr2);
        self.sqr3().write_value(sqr3);
        self.smpr1().write_value(smpr1);
        self.smpr2().write_value(smpr2);
    }
}

impl<'d, T: DefaultInstance> Adc<'d, T> {
    pub fn new(adc: Peri<'d, T>) -> Self {
        rcc::enable_and_reset::<T>();
        T::regs().cr2().modify(|reg| reg.set_adon(true));

        // 11.4: Before starting a calibration, the ADC must have been in power-on state (ADON bit = ‘1’)
        // for at least two ADC clock cycles.
        blocking_delay_us((1_000_000 * 2) / Self::freq().0 as u64 + 1);

        // Reset calibration
        T::regs().cr2().modify(|reg| reg.set_rstcal(true));
        while T::regs().cr2().read().rstcal() {
            // spin
        }

        // Calibrate
        T::regs().cr2().modify(|reg| reg.set_cal(true));
        while T::regs().cr2().read().cal() {
            // spin
        }

        // One cycle after calibration
        blocking_delay_us((1_000_000 * 1) / Self::freq().0 as u64 + 1);

        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };

        Self { adc }
    }

    fn freq() -> Hertz {
        T::frequency()
    }

    pub fn sample_time_for_us(&self, us: u32) -> SampleTime {
        match us * Self::freq().0 / 1_000_000 {
            0..=1 => SampleTime::CYCLES1_5,
            2..=7 => SampleTime::CYCLES7_5,
            8..=13 => SampleTime::CYCLES13_5,
            14..=28 => SampleTime::CYCLES28_5,
            29..=41 => SampleTime::CYCLES41_5,
            42..=55 => SampleTime::CYCLES55_5,
            56..=71 => SampleTime::CYCLES71_5,
            _ => SampleTime::CYCLES239_5,
        }
    }

    pub fn enable_vref(&mut self) -> super::VrefInt {
        T::regs().cr2().modify(|reg| {
            reg.set_tsvrefe(true);
        });
        super::VrefInt {}
    }

    pub fn enable_temperature(&mut self) -> super::Temperature {
        T::regs().cr2().modify(|reg| {
            reg.set_tsvrefe(true);
        });
        super::Temperature {}
    }
}

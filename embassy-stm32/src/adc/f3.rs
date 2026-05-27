use core::marker::PhantomData;

use stm32_metapac::adc::regs::{Smpr1, Smpr2, Sqr1, Sqr2, Sqr3};
use stm32_metapac::adc::vals::{Advregen, Dmacfg};

use crate::adc::{Adc, AdcRegs, ConversionMode, DefaultInstance, Instance, SampleTime, VrefInt};
use crate::interrupt::typelevel::Interrupt;
use crate::time::Hertz;
use crate::wait::block_for_us;
use crate::{Peri, interrupt, rcc};

pub const VDDA_CALIB_MV: u32 = 3300;
pub const ADC_MAX: u32 = (1 << 12) - 1;
pub const VREF_INT: u32 = 1230;

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: DefaultInstance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        if T::regs().isr().read().eoc() {
            T::regs().ier().modify(|w| w.set_eocie(false));
        } else {
            return;
        }

        T::state().waker.wake();
    }
}

impl<T: Instance> super::ConverterFor<VrefInt> for T {
    const CHANNEL: u8 = 18;
}

impl<T: Instance> super::ConverterFor<super::Temperature> for T {
    const CHANNEL: u8 = 16;
}

impl AdcRegs for crate::pac::adc::Adc {
    fn data(&self) -> *mut u16 {
        crate::pac::adc::Adc::dr(*self).as_ptr() as *mut u16
    }

    fn enable(&self) {
        // Enable the adc
        self.cr().modify(|w| w.set_aden(true));

        // Wait until the adc is ready
        while !self.isr().read().adrdy() {}
    }

    fn start(&self) {
        self.isr().modify(|reg| {
            reg.set_eos(true);
            reg.set_eoc(true);
        });

        self.cr().modify(|reg| {
            reg.set_adstart(true);
        });
    }

    fn stop(&self, disable: bool) {
        self.cr().modify(|w| w.set_adstp(true));

        while self.cr().read().adstp() {}

        self.cr().modify(|w| w.set_addis(true));

        while self.cr().read().aden() {}

        // Disable the adc regulator
        if disable {
            self.cr().modify(|w| w.set_advregen(Advregen::Intermediate));
            self.cr().modify(|w| w.set_advregen(Advregen::Disabled));
        }
    }

    fn wait_done(&self) -> bool {
        self.isr().read().eoc()
    }

    fn configure_dma(&self, conversion_mode: ConversionMode) {
        // Clear all status flags before configuring DMA.
        self.isr().modify(|w| {
            w.set_eoc(false);
            w.set_ovr(true);
        });

        self.ier().modify(|w| {
            // Enable end of conversion interrupt only in repeated mode.
            w.set_eocie(true);
        });

        self.cfgr().modify(|w| {
            w.set_discen(false);
            w.set_dmaen(!matches!(conversion_mode, ConversionMode::NoDma));
            w.set_cont(matches!(conversion_mode, ConversionMode::Repeated(None)));
            w.set_dmacfg(Dmacfg::Circular);

            if let ConversionMode::Repeated(Some((trigger, edge))) = conversion_mode {
                w.set_extsel(trigger);
                w.set_exten(edge);
            }
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
    pub fn new(
        adc: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        rcc::enable_and_reset::<T>();

        // Enable the adc regulator
        T::regs().cr().modify(|w| w.set_advregen(Advregen::Intermediate));
        T::regs().cr().modify(|w| w.set_advregen(Advregen::Enabled));

        // Wait for the regulator to stabilize
        block_for_us(10);

        assert!(!T::regs().cr().read().aden());

        // Begin calibration
        T::regs().cr().modify(|w| w.set_adcaldif(false));
        T::regs().cr().modify(|w| w.set_adcal(true));

        while T::regs().cr().read().adcal() {}

        // Wait more than 4 clock cycles after adcal is cleared (RM0364 p. 223).
        block_for_us((1_000_000 * 4) / Self::freq().0 as u64 + 1);

        // Enable the adc
        T::regs().enable();

        T::Interrupt::unpend();
        unsafe {
            T::Interrupt::enable();
        }

        Self { adc }
    }

    fn freq() -> Hertz {
        <T as crate::rcc::SealedRccPeripheral>::frequency()
    }

    pub fn sample_time_for_us(&self, us: u32) -> SampleTime {
        match us * Self::freq().0 / 1_000_000 {
            0..=1 => SampleTime::Cycles15,
            2..=4 => SampleTime::Cycles45,
            5..=7 => SampleTime::Cycles75,
            8..=19 => SampleTime::Cycles195,
            20..=61 => SampleTime::Cycles615,
            62..=181 => SampleTime::Cycles1815,
            _ => SampleTime::Cycles6015,
        }
    }

    pub fn enable_vref(&mut self) -> super::VrefInt {
        T::common_regs().ccr().modify(|w| w.set_vrefen(true));

        super::VrefInt {}
    }

    pub fn enable_temperature(&mut self) -> super::Temperature {
        T::common_regs().ccr().modify(|w| w.set_tsen(true));

        super::Temperature {}
    }
}

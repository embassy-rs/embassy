//! GFXTIM — graphics timer for display pipeline synchronization.

use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;

use crate::interrupt::typelevel::Interrupt;
use crate::pac::gfxtim::Gfxtim as Regs;
use crate::{Peri, interrupt, rcc};

/// GFXTIM driver.
pub struct Gfxtim<'d, T: Instance> {
    _peri: Peri<'d, T>,
}

impl<'d, T: Instance> Gfxtim<'d, T> {
    /// Create a GFXTIM instance.
    pub fn new(
        peri: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        rcc::enable_and_reset::<T>();
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };
        Self { _peri: peri }
    }

    /// Set line and frame counter reload values.
    pub fn set_counter_reload(&mut self, lines_per_frame: u16, pixels_per_line: u32) {
        let regs = T::regs();
        regs.lccrr().write(|w| w.set_reload(pixels_per_line));
        regs.fccrr().write(|w| w.set_reload(lines_per_frame));
    }

    /// Enable the absolute line and frame counters.
    pub fn start_counters(&mut self) {
        T::regs().tcr().modify(|w| {
            w.set_alcen(true);
            w.set_afcen(true);
        });
    }

    /// Disable the absolute line and frame counters.
    pub fn stop_counters(&mut self) {
        T::regs().tcr().modify(|w| {
            w.set_alcen(false);
            w.set_afcen(false);
        });
    }

    /// Read absolute line counter.
    pub fn line_counter(&self) -> u16 {
        T::regs().alcr().read().line()
    }

    /// Read absolute frame counter.
    pub fn frame_counter(&self) -> u32 {
        T::regs().afcr().read().frame()
    }
}

/// GFXTIM interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _marker: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let regs = T::regs();
        let pending = regs.isr().read().0;
        regs.icr().write(|w| w.0 = pending);
    }
}

trait SealedInstance: crate::rcc::RccPeripheral {
    fn regs() -> Regs;
}

/// GFXTIM instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {
    /// GFXTIM interrupt.
    type Interrupt: Interrupt;
}

foreach_interrupt!(
    ($inst:ident, gfxtim, GFXTIM, GLOBAL, $irq:ident) => {
        impl Instance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }

        impl SealedInstance for crate::peripherals::$inst {
            fn regs() -> Regs {
                crate::pac::$inst
            }
        }
    };
);

//! GPU2D (NeoChrom) 2D graphics accelerator.
//!
//! The GPU2D is primarily programmed via command lists in memory; the peripheral
//! registers expose status and interrupt control only.

use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;

use crate::interrupt::typelevel::Interrupt;
use crate::pac::gpu2d::Gpu2d as Regs;
use crate::{Peri, interrupt, rcc};

/// GPU2D driver error flag.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error {
    /// System error flag set.
    SystemError,
}

/// GPU2D driver.
pub struct Gpu2d<'d, T: Instance> {
    _peri: Peri<'d, T>,
}

impl<'d, T: Instance> Gpu2d<'d, T> {
    /// Create a GPU2D instance.
    pub fn new(
        peri: Peri<'d, T>,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        rcc::enable_and_reset::<T>();
        T::Interrupt::unpend();
        unsafe { T::Interrupt::enable() };
        Self { _peri: peri }
    }

    /// Last completed command list identifier.
    pub fn last_command_list_id(&self) -> u32 {
        T::regs().clid().read().id()
    }

    /// Returns true when the command-list-complete flag is set.
    pub fn command_list_complete(&self) -> bool {
        T::regs().itctrl().read().clc()
    }

    /// Clear the command-list-complete flag.
    pub fn clear_command_list_complete(&mut self) {
        T::regs().itctrl().modify(|w| w.set_clc(true));
    }

    /// Read and clear error status.
    pub fn take_error(&mut self) -> Result<(), Error> {
        let regs = T::regs();
        if regs.sys_interrupt().read().er() {
            regs.sys_interrupt().modify(|w| w.set_er(true));
            Err(Error::SystemError)
        } else {
            Ok(())
        }
    }
}

/// GPU2D error interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _marker: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let _ = T::regs();
    }
}

trait SealedInstance: crate::rcc::RccPeripheral {
    fn regs() -> Regs;
}

/// GPU2D instance trait.
#[allow(private_bounds)]
pub trait Instance: SealedInstance + PeripheralType + 'static {
    /// GPU2D error interrupt.
    type Interrupt: Interrupt;
}

foreach_interrupt!(
    ($inst:ident, gpu2d, GPU2D, ER, $irq:ident) => {
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

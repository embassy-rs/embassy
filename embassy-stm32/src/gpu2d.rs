//! GPU2D (NeoChrom) 2D graphics accelerator.
//!
//! The GPU2D is primarily programmed via command lists in memory; the peripheral
//! registers expose status and interrupt control only.

use core::marker::PhantomData;

use embassy_hal_internal::PeripheralType;
use embassy_sync::waitqueue::AtomicWaker;

use crate::interrupt::typelevel::{Binding, Handler, Interrupt};
use crate::pac::gpu2d::Gpu2d as Regs;
use crate::{Peri, interrupt, rcc};

// GPU2D has exactly one instance per chip today (see stm32-data), so a single
// static waker is sufficient — unlike peripherals with multiple instances
// (e.g. USART), which need one waker per instance.
static WAKER: AtomicWaker = AtomicWaker::new();

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

impl<'d, T: Instance + crate::rcc::RccPeripheral> Gpu2d<'d, T> {
    /// Create a GPU2D instance.
    pub fn new<H: Handler<T::Interrupt>>(peri: Peri<'d, T>, _irq: impl Binding<T::Interrupt, H> + 'd) -> Self {
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

    /// Wait for the current command list to complete.
    ///
    /// Note: [`Error::SystemError`] is not observed here — check
    /// [`Self::take_error`] separately if you need to detect hardware errors.
    pub async fn wait_command_list_complete(&mut self) {
        core::future::poll_fn(|cx| {
            WAKER.register(cx.waker());
            if self.command_list_complete() {
                self.clear_command_list_complete();
                // SAFETY: re-arming after consuming the flag we were woken for.
                unsafe { T::Interrupt::enable() };
                core::task::Poll::Ready(())
            } else {
                // First poll, or a spurious wake: make sure the interrupt is
                // armed before parking on the waker.
                unsafe { T::Interrupt::enable() };
                core::task::Poll::Pending
            }
        })
        .await
    }
}

/// GPU2D error interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _marker: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        // Neither CLC nor ER (see gpu2d_v1.yaml) has a separate
        // interrupt-enable field, so mask at the NVIC level to avoid an
        // interrupt storm; the waiting task clears the flag and re-enables.
        T::Interrupt::disable();
        WAKER.wake();
    }
}

trait SealedInstance {
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

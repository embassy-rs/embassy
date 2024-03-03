//! USB Type-C/USB Power Delivery Interface (UCPD)

use core::marker::PhantomData;

use crate::interrupt;
use crate::rcc::RccPeripheral;
use embassy_sync::waitqueue::AtomicWaker;

/// Interrupt handler.
pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        let sr = T::REGS.sr().read();

        // TODO: Disable interrupt which have fired.

        // Wake the task to handle and re-enabled interrupts.
        T::waker().wake();
    }
}

/// UCPD instance trait.
pub trait Instance: sealed::Instance + RccPeripheral {}

pub(crate) mod sealed {
    pub trait Instance {
        type Interrupt: crate::interrupt::typelevel::Interrupt;
        const REGS: crate::pac::ucpd::Ucpd;
        fn waker() -> &'static embassy_sync::waitqueue::AtomicWaker;
    }
}

foreach_interrupt!(
    ($inst:ident, ucpd, UCPD, GLOBAL, $irq:ident) => {
        impl sealed::Instance for crate::peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;

            const REGS: crate::pac::ucpd::Ucpd = crate::pac::$inst;

            fn waker() -> &'static AtomicWaker {
                static WAKER: AtomicWaker = AtomicWaker::new();
                &WAKER
            }
        }

        impl Instance for crate::peripherals::$inst {}
    };
);

pin_trait!(Cc1Pin, Instance);
pin_trait!(Cc2Pin, Instance);

dma_trait!(TxDma, Instance);
dma_trait!(RxDma, Instance);

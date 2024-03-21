#![macro_use]
#![allow(missing_docs)] // TODO

use core::future::poll_fn;
use core::marker::PhantomData;
use core::task::Poll;

use embassy_hal_internal::into_ref;

use crate::interrupt::typelevel::Interrupt;
use crate::peripherals;
use crate::{interrupt, Peripheral};

/// Touch Sensor Controller driver.
pub struct Tsc<'d, T: Instance> {
    tsc: crate::PeripheralRef<'d, T>,
}

pub struct InterruptHandler<T: Instance> {
    _phantom: PhantomData<T>,
}

impl<T: Instance> interrupt::typelevel::Handler<T::Interrupt> for InterruptHandler<T> {
    unsafe fn on_interrupt() {
        if T::regs().isr().read().eoaf() {
            T::regs().ier().modify(|w| w.set_eoaie(false));
        } else {
            return;
        }

        T::state().waker.wake();
    }
}

impl<'d, T: Instance> Tsc<'d, T> {
    pub fn new(
        tsc: impl Peripheral<P = T> + 'd,
        _irq: impl interrupt::typelevel::Binding<T::Interrupt, InterruptHandler<T>> + 'd,
    ) -> Self {
        into_ref!(tsc);
        T::enable_and_reset();

        T::regs().cr().modify(|w| w.set_tsce(true));

        T::regs().cr().modify(|w| {
            w.set_ctph(1);
            w.set_ctpl(1);
            w.set_sse(false);
            w.set_ssd(1);
            w.set_sspsc(false);
            w.set_pgpsc(2);
            w.set_mcv(5);
            w.set_iodef(false);
            w.set_syncpol(false);
            w.set_am(false);
        });

        T::regs().ier().modify(|w| {
            w.set_mceie(false);
            w.set_eoaie(true);
        });

        T::regs().iohcr().modify(|w| {
            w.set_g1_io1(true);
            w.set_g1_io2(true);
            w.set_g1_io3(true);
            w.set_g1_io4(true);

            w.set_g2_io1(true);
            w.set_g2_io2(true);
            w.set_g2_io3(true);
            w.set_g2_io4(true);

            w.set_g3_io1(true);
            w.set_g3_io2(true);
            w.set_g3_io3(true);
            w.set_g3_io4(true);

            w.set_g4_io1(true);
            w.set_g4_io2(true);
            w.set_g4_io3(true);
            w.set_g4_io4(true);

            w.set_g5_io1(true);
            w.set_g5_io2(true);
            w.set_g5_io3(true);
            w.set_g5_io4(true);

            w.set_g6_io1(false);
            w.set_g6_io2(false);
            w.set_g6_io3(true);
            w.set_g6_io4(false);

            w.set_g7_io1(true);
            w.set_g7_io2(true);
            w.set_g7_io3(true);
            w.set_g7_io4(true);
        });

        T::regs().ioccr().modify(|w| {
            w.set_g6_io1(true);
            w.set_g6_io2(true);
        });

        T::regs().ioscr().modify(|w| {
            w.set_g6_io4(true);
        });

        T::regs().iogcsr().modify(|w| {
            w.set_g6e(true);
        });

        T::regs().icr().modify(|w| {
            w.set_eoaic(true);
            w.set_mceic(true);
        });

        T::Interrupt::unpend();
        unsafe {
            T::Interrupt::enable();
        }

        Self { tsc }
    }

    pub async fn read(&mut self) -> u16 {
        // Request a new acquisition
        T::regs().cr().modify(|w| w.set_start(true));

        poll_fn(|cx| {
            T::state().waker.register(cx.waker());

            if T::regs().isr().read().eoaf() {
                // Make sure the interrupt is cleared
                T::regs().icr().modify(|w| w.set_eoaic(true));
                T::regs().ier().modify(|w| w.set_eoaie(true));
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await;

        T::regs().iogcr(5).read().cnt()
    }
}

pub(crate) mod sealed {
    use embassy_sync::waitqueue::AtomicWaker;

    pub struct State {
        pub waker: AtomicWaker,
    }

    impl State {
        pub const fn new() -> Self {
            Self {
                waker: AtomicWaker::new(),
            }
        }
    }

    pub trait InterruptableInstance {
        type Interrupt: crate::interrupt::typelevel::Interrupt;
    }

    pub trait Instance: InterruptableInstance {
        fn regs() -> crate::pac::tsc::Tsc;
        fn state() -> &'static State;
    }
}

pub trait Instance: sealed::Instance + crate::Peripheral<P = Self> + crate::rcc::RccPeripheral {}

foreach_interrupt!(
    ($inst:ident,tsc,TSC,GLOBAL,$irq:ident) => {
        impl crate::tsc::sealed::Instance for peripherals::$inst {
            fn regs() -> crate::pac::tsc::Tsc {
                crate::pac::$inst
            }

            fn state() -> &'static sealed::State {
                static STATE: sealed::State = sealed::State::new();
                &STATE
            }
        }

        impl sealed::InterruptableInstance for peripherals::$inst {
            type Interrupt = crate::interrupt::typelevel::$irq;
        }

        impl crate::tsc::Instance for peripherals::$inst {}
    };
);

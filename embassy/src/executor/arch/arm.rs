use core::marker::PhantomData;
use core::ptr;

use super::{raw, Spawner};
use crate::interrupt::{Interrupt, InterruptExt};

pub struct Executor {
    inner: raw::Executor,
    not_send: PhantomData<*mut ()>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            inner: raw::Executor::new(|_| cortex_m::asm::sev(), ptr::null_mut()),
            not_send: PhantomData,
        }
    }

    /// Runs the executor.
    ///
    /// This function never returns.
    pub fn run(&'static mut self, init: impl FnOnce(Spawner)) -> ! {
        init(unsafe { self.inner.spawner() });

        loop {
            unsafe { self.inner.run_queued() };
            cortex_m::asm::wfe();
        }
    }
}

fn pend_by_number(n: u16) {
    #[derive(Clone, Copy)]
    struct N(u16);
    unsafe impl cortex_m::interrupt::InterruptNumber for N {
        fn number(self) -> u16 {
            self.0
        }
    }
    cortex_m::peripheral::NVIC::pend(N(n))
}

pub struct InterruptExecutor<I: Interrupt> {
    irq: I,
    inner: raw::Executor,
    not_send: PhantomData<*mut ()>,
}

impl<I: Interrupt> InterruptExecutor<I> {
    pub fn new(irq: I) -> Self {
        let ctx = irq.number() as *mut ();
        Self {
            irq,
            inner: raw::Executor::new(|ctx| pend_by_number(ctx as u16), ctx),
            not_send: PhantomData,
        }
    }

    /// Start the executor.
    ///
    /// `init` is called in the interrupt context, then the interrupt is
    /// configured to run the executor.
    pub fn start(&'static mut self, init: impl FnOnce(Spawner) + Send) {
        self.irq.disable();

        init(unsafe { self.inner.spawner() });

        self.irq.set_handler(|ctx| unsafe {
            let executor = &*(ctx as *const raw::Executor);
            executor.run_queued();
        });
        self.irq.set_handler_context(&self.inner as *const _ as _);
        self.irq.enable();
    }
}

//! Executor specific to cortex-m devices.
use core::marker::PhantomData;

pub use embassy_executor::*;

use crate::interrupt::{Interrupt, InterruptExt};

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

/// Interrupt mode executor.
///
/// This executor runs tasks in interrupt mode. The interrupt handler is set up
/// to poll tasks, and when a task is woken the interrupt is pended from software.
///
/// This allows running async tasks at a priority higher than thread mode. One
/// use case is to leave thread mode free for non-async tasks. Another use case is
/// to run multiple executors: one in thread mode for low priority tasks and another in
/// interrupt mode for higher priority tasks. Higher priority tasks will preempt lower
/// priority ones.
///
/// It is even possible to run multiple interrupt mode executors at different priorities,
/// by assigning different priorities to the interrupts. For an example on how to do this,
/// See the 'multiprio' example for 'embassy-nrf'.
///
/// To use it, you have to pick an interrupt that won't be used by the hardware.
/// Some chips reserve some interrupts for this purpose, sometimes named "software interrupts" (SWI).
/// If this is not the case, you may use an interrupt from any unused peripheral.
///
/// It is somewhat more complex to use, it's recommended to use the thread-mode
/// [`Executor`] instead, if it works for your use case.
pub struct InterruptExecutor<I: Interrupt> {
    irq: I,
    inner: raw::Executor,
    not_send: PhantomData<*mut ()>,
}

impl<I: Interrupt> InterruptExecutor<I> {
    /// Create a new Executor.
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
    /// This initializes the executor, configures and enables the interrupt, and returns.
    /// The executor keeps running in the background through the interrupt.
    ///
    /// This returns a [`SendSpawner`] you can use to spawn tasks on it. A [`SendSpawner`]
    /// is returned instead of a [`Spawner`](embassy_executor::Spawner) because the executor effectively runs in a
    /// different "thread" (the interrupt), so spawning tasks on it is effectively
    /// sending them.
    ///
    /// To obtain a [`Spawner`](embassy_executor::Spawner) for this executor, use [`Spawner::for_current_executor()`](embassy_executor::Spawner::for_current_executor()) from
    /// a task running in it.
    ///
    /// This function requires `&'static mut self`. This means you have to store the
    /// Executor instance in a place where it'll live forever and grants you mutable
    /// access. There's a few ways to do this:
    ///
    /// - a [StaticCell](https://docs.rs/static_cell/latest/static_cell/) (safe)
    /// - a `static mut` (unsafe)
    /// - a local variable in a function you know never returns (like `fn main() -> !`), upgrading its lifetime with `transmute`. (unsafe)
    pub fn start(&'static mut self) -> SendSpawner {
        self.irq.disable();

        self.irq.set_handler(|ctx| unsafe {
            let executor = &*(ctx as *const raw::Executor);
            executor.poll();
        });
        self.irq.set_handler_context(&self.inner as *const _ as _);
        self.irq.enable();

        self.inner.spawner().make_send()
    }
}

use core::marker::PhantomData;
use core::mem;

use super::{raw, Spawner};
use crate::interrupt::{Interrupt, InterruptExt, NrWrap};

/// TODO
#[derive(Clone, Copy)]
pub struct ExecutorWaker {
    irqn: u16,
}

impl ExecutorWaker {
    /// TODO
    pub const unsafe fn poison() -> Self {
        Self { irqn: 0 }
    }

    /// TODO
    pub fn wake(&self) {
        let mut nvic: cortex_m::peripheral::NVIC = unsafe { mem::transmute(()) };
        nvic.request(NrWrap(self.irqn));
    }
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
pub struct Executor<I: Interrupt> {
    irq: I,
    inner: raw::Executor,
    not_send: PhantomData<*mut ()>,
}

impl<I: Interrupt> Executor<I> {
    /// Create a new Executor.
    pub fn new(irq: I) -> Self {
        let irqn = irq.number();
        Self {
            irq,
            inner: raw::Executor::new(ExecutorWaker { irqn }),
            not_send: PhantomData,
        }
    }

    /// Start the executor.
    ///
    /// The `init` closure is called from interrupt mode, with a [`Spawner`] that spawns tasks on
    /// this executor. Use it to spawn the initial task(s). After `init` returns,
    /// the interrupt is configured so that the executor starts running the tasks.
    /// Once the executor is started, `start` returns.
    ///
    /// To spawn more tasks later, you may keep copies of the [`Spawner`] (it is `Copy`),
    /// for example by passing it as an argument to the initial tasks.
    ///
    /// This function requires `&'static mut self`. This means you have to store the
    /// Executor instance in a place where it'll live forever and grants you mutable
    /// access. There's a few ways to do this:
    ///
    /// - a [Forever](crate::util::Forever) (safe)
    /// - a `static mut` (unsafe)
    /// - a local variable in a function you know never returns (like `fn main() -> !`), upgrading its lifetime with `transmute`. (unsafe)
    pub fn start(&'static mut self, init: impl FnOnce(Spawner) + Send) {
        self.irq.disable();

        init(self.inner.spawner());

        self.irq.set_handler(|ctx| unsafe {
            let executor = &*(ctx as *const raw::Executor);
            executor.poll();
        });
        self.irq.set_handler_context(&self.inner as *const _ as _);
        self.irq.enable();
    }
}

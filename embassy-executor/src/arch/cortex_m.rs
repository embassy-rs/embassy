#[export_name = "__pender"]
#[cfg(any(feature = "executor-thread", feature = "executor-interrupt"))]
fn __pender(context: *mut ()) {
    unsafe {
        // Safety: `context` is either `usize::MAX` created by `Executor::run`, or a valid interrupt
        // request number given to `InterruptExecutor::start`.

        let context = context as usize;

        #[cfg(feature = "executor-thread")]
        // Try to make Rust optimize the branching away if we only use thread mode.
        if !cfg!(feature = "executor-interrupt") || context == THREAD_PENDER {
            core::arch::asm!("sev");
            return;
        }

        #[cfg(feature = "executor-interrupt")]
        {
            use cortex_m::interrupt::InterruptNumber;
            use cortex_m::peripheral::NVIC;

            #[derive(Clone, Copy)]
            struct Irq(u16);
            unsafe impl InterruptNumber for Irq {
                fn number(self) -> u16 {
                    self.0
                }
            }

            let irq = Irq(context as u16);

            // STIR is faster, but is only available in v7 and higher.
            #[cfg(not(armv6m))]
            {
                let mut nvic: NVIC = core::mem::transmute(());
                nvic.request(irq);
            }

            #[cfg(armv6m)]
            NVIC::pend(irq);
        }
    }
}

#[cfg(feature = "executor-thread")]
pub use thread::*;
#[cfg(feature = "executor-thread")]
mod thread {
    pub(super) const THREAD_PENDER: usize = usize::MAX;

    use core::arch::asm;
    use core::marker::PhantomData;

    pub use embassy_executor_macros::main_cortex_m as main;

    use crate::{raw, Spawner};

    /// Thread mode executor, using WFE/SEV.
    ///
    /// This is the simplest and most common kind of executor. It runs on
    /// thread mode (at the lowest priority level), and uses the `WFE` ARM instruction
    /// to sleep when it has no more work to do. When a task is woken, a `SEV` instruction
    /// is executed, to make the `WFE` exit from sleep and poll the task.
    ///
    /// This executor allows for ultra low power consumption for chips where `WFE`
    /// triggers low-power sleep without extra steps. If your chip requires extra steps,
    /// you may use [`raw::Executor`] directly to program custom behavior.
    pub struct Executor {
        inner: raw::Executor,
        not_send: PhantomData<*mut ()>,
    }

    impl Executor {
        /// Create a new Executor.
        pub fn new() -> Self {
            Self {
                inner: raw::Executor::new(THREAD_PENDER as *mut ()),
                not_send: PhantomData,
            }
        }

        /// Run the executor.
        ///
        /// The `init` closure is called with a [`Spawner`] that spawns tasks on
        /// this executor. Use it to spawn the initial task(s). After `init` returns,
        /// the executor starts running the tasks.
        ///
        /// To spawn more tasks later, you may keep copies of the [`Spawner`] (it is `Copy`),
        /// for example by passing it as an argument to the initial tasks.
        ///
        /// This function requires `&'static mut self`. This means you have to store the
        /// Executor instance in a place where it'll live forever and grants you mutable
        /// access. There's a few ways to do this:
        ///
        /// - a [StaticCell](https://docs.rs/static_cell/latest/static_cell/) (safe)
        /// - a `static mut` (unsafe)
        /// - a local variable in a function you know never returns (like `fn main() -> !`), upgrading its lifetime with `transmute`. (unsafe)
        ///
        /// This function never returns.
        pub fn run(&'static mut self, init: impl FnOnce(Spawner)) -> ! {
            init(self.inner.spawner());

            loop {
                unsafe {
                    self.inner.poll();
                    asm!("wfe");
                };
            }
        }
    }
}

#[cfg(feature = "executor-interrupt")]
pub use interrupt::*;
#[cfg(feature = "executor-interrupt")]
mod interrupt {
    use core::cell::{Cell, UnsafeCell};
    use core::mem::MaybeUninit;

    use cortex_m::interrupt::InterruptNumber;
    use cortex_m::peripheral::NVIC;
    use critical_section::Mutex;

    use crate::raw;

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
    pub struct InterruptExecutor {
        started: Mutex<Cell<bool>>,
        executor: UnsafeCell<MaybeUninit<raw::Executor>>,
    }

    unsafe impl Send for InterruptExecutor {}
    unsafe impl Sync for InterruptExecutor {}

    impl InterruptExecutor {
        /// Create a new, not started `InterruptExecutor`.
        #[inline]
        pub const fn new() -> Self {
            Self {
                started: Mutex::new(Cell::new(false)),
                executor: UnsafeCell::new(MaybeUninit::uninit()),
            }
        }

        /// Executor interrupt callback.
        ///
        /// # Safety
        ///
        /// - You MUST call this from the interrupt handler, and from nowhere else.
        /// - You must not call this before calling `start()`.
        pub unsafe fn on_interrupt(&'static self) {
            let executor = unsafe { (&*self.executor.get()).assume_init_ref() };
            executor.poll();
        }

        /// Start the executor.
        ///
        /// This initializes the executor, enables the interrupt, and returns.
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
        /// # Interrupt requirements
        ///
        /// You must write the interrupt handler yourself, and make it call [`on_interrupt()`](Self::on_interrupt).
        ///
        /// This method already enables (unmasks) the interrupt, you must NOT do it yourself.
        ///
        /// You must set the interrupt priority before calling this method. You MUST NOT
        /// do it after.
        ///
        pub fn start(&'static self, irq: impl InterruptNumber) -> crate::SendSpawner {
            if critical_section::with(|cs| self.started.borrow(cs).replace(true)) {
                panic!("InterruptExecutor::start() called multiple times on the same executor.");
            }

            unsafe {
                (&mut *self.executor.get())
                    .as_mut_ptr()
                    .write(raw::Executor::new(irq.number() as *mut ()))
            }

            let executor = unsafe { (&*self.executor.get()).assume_init_ref() };

            unsafe { NVIC::unmask(irq) }

            executor.spawner().make_send()
        }

        /// Get a SendSpawner for this executor
        ///
        /// This returns a [`SendSpawner`] you can use to spawn tasks on this
        /// executor.
        ///
        /// This MUST only be called on an executor that has already been started.
        /// The function will panic otherwise.
        pub fn spawner(&'static self) -> crate::SendSpawner {
            if !critical_section::with(|cs| self.started.borrow(cs).get()) {
                panic!("InterruptExecutor::spawner() called on uninitialized executor.");
            }
            let executor = unsafe { (&*self.executor.get()).assume_init_ref() };
            executor.spawner().make_send()
        }
    }
}

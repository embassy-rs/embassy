#[cfg(feature = "executor-interrupt")]
compile_error!("`executor-interrupt` is not supported with `arch-xtensa`.");

#[cfg(feature = "executor-thread")]
pub use thread::*;
#[cfg(feature = "executor-thread")]
mod thread {
    use core::marker::PhantomData;
    use core::sync::atomic::{AtomicBool, Ordering};

    use crate::{raw, Spawner};

    /// global atomic used to keep track of whether there is work to do since sev() is not available on Xtensa
    static SIGNAL_WORK_THREAD_MODE: AtomicBool = AtomicBool::new(false);

    #[export_name = "__pender"]
    fn __pender(_context: *mut ()) {
        SIGNAL_WORK_THREAD_MODE.store(true, Ordering::SeqCst);
    }

    /// Xtensa Executor
    pub struct Executor {
        inner: raw::Executor,
        not_send: PhantomData<*mut ()>,
    }

    impl Executor {
        /// Create a new Executor.
        pub fn new() -> Self {
            Self {
                inner: raw::Executor::new(core::ptr::null_mut()),
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

                    // Manual critical section implementation that only masks interrupts handlers.
                    // We must not acquire the cross-core on dual-core systems because that would
                    // prevent the other core from doing useful work while this core is sleeping.
                    let token: critical_section::RawRestoreState;
                    core::arch::asm!("rsil {0}, 5", out(reg) token);

                    // we do not care about race conditions between the load and store operations, interrupts
                    // will only set this value to true.
                    // if there is work to do, loop back to polling
                    if SIGNAL_WORK_THREAD_MODE.load(Ordering::SeqCst) {
                        SIGNAL_WORK_THREAD_MODE.store(false, Ordering::SeqCst);

                        core::arch::asm!(
                        "wsr.ps {0}",
                        "rsync", in(reg) token)
                    } else {
                        // waiti sets the PS.INTLEVEL when slipping into sleep
                        // because critical sections in Xtensa are implemented via increasing
                        // PS.INTLEVEL the critical section ends here
                        // take care not add code after `waiti` if it needs to be inside the CS
                        core::arch::asm!("waiti 0"); // critical section ends here
                    }
                }
            }
        }
    }
}

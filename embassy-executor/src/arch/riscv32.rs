#[cfg(feature = "executor-interrupt")]
compile_error!("`executor-interrupt` is not supported with `arch-riscv32`.");

#[cfg(feature = "executor-thread")]
pub use thread::*;
#[cfg(feature = "executor-thread")]
mod thread {
    use core::marker::PhantomData;
    use core::sync::atomic::{AtomicBool, Ordering};

    pub use embassy_executor_macros::main_riscv as main;

    use crate::{Spawner, raw};

    /// global atomic used to keep track of whether there is work to do since sev() is not available on RISCV
    static SIGNAL_WORK_THREAD_MODE: AtomicBool = AtomicBool::new(false);

    #[unsafe(export_name = "__pender")]
    fn __pender(_context: *mut ()) {
        SIGNAL_WORK_THREAD_MODE.store(true, Ordering::SeqCst);
    }

    /// RISCV32 Executor
    ///
    /// This executor enters a low power mode using `WFI` instruction by default.
    /// If your chip (or application) requires extra steps, you may use the custom idle-hook
    /// feature via [`Executor::with_idle_hook`]. Alternatively, you may use the [`raw::Executor`]
    /// directly to program custom behavior.
    pub struct Executor {
        inner: raw::Executor,
        not_send: PhantomData<*mut ()>,
        #[cfg(feature = "idle-hook")]
        idle_hook: fn(&Executor),
    }

    impl Executor {
        /// Create a new Executor.
        pub fn new() -> Self {
            Self {
                inner: raw::Executor::new(core::ptr::null_mut()),
                not_send: PhantomData,
                #[cfg(feature = "idle-hook")]
                idle_hook: Executor::default_idle,
            }
        }

        /// Add idle-hook to Executor instance.
        #[cfg(feature = "idle-hook")]
        pub fn with_idle_hook(mut self, idle_hook: fn(&Executor)) -> Self {
            self.idle_hook = idle_hook;
            self
        }

        /// Put Executor into default idle state.
        ///
        /// This function might also be called from the application's context,
        /// e.g. from a custom idle-hook.
        #[inline(always)]
        pub fn default_idle(&self) {
            critical_section::with(|_| {
                // if there is work to do, loop back to polling, otherwise wait for interrupt
                // TODO can we relax this?
                if !SIGNAL_WORK_THREAD_MODE.swap(false, Ordering::SeqCst) {
                    unsafe { core::arch::asm!("wfi") };
                }
            });
            // if an interrupt occurred while waiting, it will be serviced here
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
        /// After all tasks have been polled, this function enters an idle state by either calling
        /// the inlined [`Executor::default_idle`] function or a custom idle-hook.
        ///
        /// This function never returns.
        pub fn run(&'static mut self, init: impl FnOnce(Spawner)) -> ! {
            init(self.inner.spawner());

            loop {
                unsafe { self.inner.poll() };

                #[cfg(feature = "idle-hook")]
                (self.idle_hook)(self);
                #[cfg(not(feature = "idle-hook"))]
                self.default_idle();
            }
        }
    }
}

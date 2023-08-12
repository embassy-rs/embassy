#[cfg(feature = "executor-interrupt")]
compile_error!("`executor-interrupt` is not supported with `arch-xtensa`.");

#[cfg(feature = "executor-thread")]
pub use thread::*;
#[cfg(feature = "executor-thread")]
mod thread {
    use core::marker::PhantomData;
    use core::sync::atomic::{AtomicBool, Ordering};

    use crate::raw::OpaqueThreadContext;
    use crate::thread::ThreadContext;

    /// global atomic used to keep track of whether there is work to do since sev() is not available on Xtensa
    static SIGNAL_WORK_THREAD_MODE: AtomicBool = AtomicBool::new(false);

    #[export_name = "__thread_mode_pender"]
    fn __thread_mode_pender(_context: OpaqueThreadContext) {
        SIGNAL_WORK_THREAD_MODE.store(true, Ordering::SeqCst);
    }

    /// TODO
    // Name pending
    #[derive(Default)] // Default enables Executor::new
    pub struct Context;

    impl ThreadContext for Context {
        #[cfg(feature = "thread-context")]
        fn context(&self) -> OpaqueThreadContext {
            // Enabling thread-context is not incorrect, just wasteful.
            OpaqueThreadContext(0)
        }

        fn wait(&mut self) {
            unsafe {
                // Manual critical section implementation that only masks interrupts handlers.
                // We must not acquire the cross-core on dual-core systems because that would
                // prevent the other core from doing useful work while this core is sleeping.
                let token: critical_section::RawRestoreState;
                core::arch::asm!("rsil {0}, 5", out(reg) token);

                // we do not care about race conditions between the load and store operations,
                // interrupts will only set this value to true.
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

    /// TODO
    // Type alias for backwards compatibility
    pub type Executor = crate::thread::ThreadModeExecutor<Context>;
}

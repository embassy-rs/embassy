#[cfg(feature = "executor-interrupt")]
compile_error!("`executor-interrupt` is not supported with `arch-riscv32`.");

#[cfg(feature = "executor-thread")]
pub use thread::*;
#[cfg(feature = "executor-thread")]
mod thread {
    use core::sync::atomic::{AtomicBool, Ordering};

    #[cfg(feature = "nightly")]
    pub use embassy_macros::main_riscv as main;

    use crate::raw::PenderContext;
    use crate::thread::ThreadContext;

    /// global atomic used to keep track of whether there is work to do since sev() is not available on RISCV
    static SIGNAL_WORK_THREAD_MODE: AtomicBool = AtomicBool::new(false);

    #[export_name = "__pender"]
    fn __thread_mode_pender(_context: PenderContext) {
        SIGNAL_WORK_THREAD_MODE.store(true, Ordering::SeqCst);
    }

    /// TODO
    // Name pending
    #[derive(Default)] // Default enables Executor::new
    pub struct Context;

    impl ThreadContext for Context {
        fn context(&self) -> PenderContext {
            unsafe { core::mem::transmute(0) }
        }

        fn wait(&mut self) {
            // We do not care about race conditions between the load and store operations,
            // interrupts will only set this value to true.
            critical_section::with(|_| {
                // if there is work to do, loop back to polling
                // TODO can we relax this?
                if SIGNAL_WORK_THREAD_MODE.load(Ordering::SeqCst) {
                    SIGNAL_WORK_THREAD_MODE.store(false, Ordering::SeqCst);
                }
                // if not, wait for interrupt
                else {
                    unsafe {
                        core::arch::asm!("wfi");
                    }
                }
            });
            // if an interrupt occurred while waiting, it will be serviced here
        }
    }

    /// TODO
    // Type alias for backwards compatibility
    pub type Executor = crate::thread::ThreadModeExecutor<Context>;
}

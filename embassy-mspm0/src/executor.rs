//! MSPM0-specific `embassy-executor` platform.
//!
//! This module provides an `embassy-executor` platform specific for MSPM0 chips that integrates [`low_power::sleep()`](crate::low_power::sleep) in the main loop.
//! Read the `embassy-executor` README for information about what "executor platforms" are and how they work.
//!
//! To use it:
//! - Enable the `executor-thread` feature on this crate.
//! - **Do not** enable features `platform-cortex-m`, `executor-thread` or `executor-interrupt` in the `embassy-executor` crate.
//! - Tell the `main` macro to use this executor like this:
//!
//! ```rust,no_run
//! #[embassy_executor::main(executor = "embassy_mspm0::executor::Executor", entry = "cortex_m_rt::entry")]
//! async fn main(spawner: Spawner) {
//!     let p = embassy_mspm0::init(Config::default());
//!     // ...
//! }
//! ```

#[unsafe(export_name = "__pender")]
#[cfg(feature = "executor-thread")]
fn __pender(_context: *mut ()) {
    thread::SIGNAL_WORK_THREAD_MODE.store(true, core::sync::atomic::Ordering::SeqCst);
}

#[cfg(feature = "executor-thread")]
pub use thread::*;
#[cfg(feature = "executor-thread")]
mod thread {
    use core::marker::PhantomData;
    use core::sync::atomic::{AtomicBool, Ordering};

    use embassy_executor::{Spawner, raw};

    const THREAD_PENDER: usize = usize::MAX;

    /// Set by the pender to signal pending work; checked before sleeping since `WFI` ignores `SEV`.
    pub(crate) static SIGNAL_WORK_THREAD_MODE: AtomicBool = AtomicBool::new(false);

    /// Thread-mode executor that deep-sleeps on idle via [`low_power::sleep`](crate::low_power::sleep).
    ///
    /// This is the simplest and most common kind of executor. It runs on
    /// thread mode (at the lowest priority level), and uses the `WFE` ARM instruction
    /// to sleep when it has no more work to do. When a task is woken, a `SEV` instruction
    /// is executed, to make the `WFE` exit from sleep and poll the task.
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

                    critical_section::with(|cs| {
                        if SIGNAL_WORK_THREAD_MODE.load(Ordering::SeqCst) {
                            SIGNAL_WORK_THREAD_MODE.store(false, Ordering::SeqCst);
                        } else {
                            crate::low_power::sleep(cs);
                        }
                    });
                }
            }
        }
    }

    impl Default for Executor {
        fn default() -> Self {
            Self::new()
        }
    }
}

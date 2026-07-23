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
//! ```rust,ignore
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

        /// Run the executor, calling `init` with a [`Spawner`] for the initial task(s).
        ///
        /// Never returns. See [`embassy_executor`]'s thread executor for how to obtain the
        /// `&'static mut self` this requires.
        pub fn run(&'static mut self, init: impl FnOnce(Spawner)) -> ! {
            init(self.inner.spawner());

            loop {
                unsafe {
                    self.inner.poll();

                    critical_section::with(|cs| {
                        // Interrupts only ever set this to true, so racing the load/store is fine.
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

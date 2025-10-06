#[cfg(feature = "executor-interrupt")]
compile_error!("`executor-interrupt` is not supported with `arch-cortex-ar`.");

#[unsafe(export_name = "__pender")]
#[cfg(any(feature = "executor-thread", feature = "executor-interrupt"))]
fn __pender(context: *mut ()) {
    // `context` is always `usize::MAX` created by `Executor::run`.
    let context = context as usize;

    #[cfg(feature = "executor-thread")]
    // Try to make Rust optimize the branching away if we only use thread mode.
    if !cfg!(feature = "executor-interrupt") || context == THREAD_PENDER {
        cortex_ar::asm::sev();
        return;
    }
}

#[cfg(feature = "executor-thread")]
pub use thread::*;
#[cfg(feature = "executor-thread")]
mod thread {
    pub(super) const THREAD_PENDER: usize = usize::MAX;

    use core::marker::PhantomData;

    use cortex_ar::asm::wfe;
    pub use embassy_executor_macros::main_cortex_ar as main;

    use crate::{Spawner, raw};

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
                }
                wfe();
            }
        }
    }
}

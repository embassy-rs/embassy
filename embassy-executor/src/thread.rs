//! Thread-mode executor.

use core::marker::PhantomData;

use crate::raw::{self, PenderContext};
use crate::Spawner;

/// Architecture-specific interface for a thread-mode executor. This trait describes what the
/// executor should do when idle, and what data should be passed to its pender.
// TODO: Name pending
pub trait ThreadContext: Sized {
    /// A pointer-sized piece of data that is passed to the pender function.
    ///
    /// For example, on multi-core systems, this can be used to store the ID of the core that
    /// should be woken up.
    fn context(&self) -> PenderContext;

    /// Waits for the executor to be waken.
    ///
    /// While it is valid for this function can be empty, it is recommended to use a WFE instruction
    /// or equivalent to let the CPU sleep.
    fn wait(&mut self);
}

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
pub struct ThreadModeExecutor<C: ThreadContext> {
    inner: raw::Executor,
    context: C,
    not_send: PhantomData<*mut ()>,
}

impl<C: ThreadContext> ThreadModeExecutor<C> {
    /// Create a new Executor.
    pub fn new() -> Self
    where
        C: Default,
    {
        Self::with_context(C::default())
    }

    /// Create a new Executor using the given thread context.
    pub fn with_context(context: C) -> Self {
        Self {
            inner: raw::Executor::new(context.context()),
            context,
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
                self.context.wait();
            };
        }
    }
}

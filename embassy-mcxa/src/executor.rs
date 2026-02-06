use core::{arch::asm, marker::PhantomData, sync::atomic::{AtomicBool, AtomicU8, Ordering}};
use embassy_executor::{Spawner, raw};

static TASKS_PENDING: AtomicBool = AtomicBool::new(false);
static EXECUTOR_ONCE: AtomicU8 = AtomicU8::new(0);

const EXECUTOR_UNINIT: u8 = 0;
const EXECUTOR_TAKEN: u8 = 1;
const EXECUTOR_INITING: u8 = 2;
const EXECUTOR_ACTIVE: u8 = 3;

// Use a sentinel value for context to denote the thread pender context
const THREAD_PENDER: usize = usize::MAX;

pub(crate) fn custom_executor_created() -> bool {
    EXECUTOR_ONCE.load(Ordering::Acquire) != EXECUTOR_UNINIT
}

pub struct Executor {
    inner: raw::Executor,
    not_send: PhantomData<*mut ()>,
}

#[unsafe(export_name = "__pender")]
fn __pender(context: *mut ()) {
    unsafe {
        // Safety: `context` is either `usize::MAX` created by `Executor::run`, or a valid interrupt
        // request number given to `InterruptExecutor::start`.

        let context = context as usize;

        // Try to make Rust optimize the branching away if we only use thread mode.
        if context == THREAD_PENDER {
            TASKS_PENDING.store(true, Ordering::Release);
            asm!("sev");
        }
    }
}

impl Executor {
    pub fn new() -> Self {
        let res = EXECUTOR_ONCE.compare_exchange(
            EXECUTOR_UNINIT,
            EXECUTOR_TAKEN,
            Ordering::AcqRel,
            Ordering::Relaxed,
        );

        if res.is_err() {
            panic!("Can only take the executor once");
        }

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
        // We can only create the executor once, so this must be the only one, meaning we
        // can store instead of exchange.
        EXECUTOR_ONCE.store(EXECUTOR_INITING, Ordering::Release);
        init(self.inner.spawner());
        EXECUTOR_ONCE.store(EXECUTOR_ACTIVE, Ordering::Release);

        loop {
            unsafe {
                self.inner.poll();
                // self.configure_pwr();
                // #[cfg(feature = "defmt")]
                // defmt::flush();
                asm!("wfe");
                // Self::on_wakeup_irq_or_event();
            }
        }
    }
}

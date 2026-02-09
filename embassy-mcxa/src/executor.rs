use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use embassy_executor::{Spawner, raw};

use crate::clocks::config::CoreSleep;

static TASKS_PENDING: AtomicBool = AtomicBool::new(false);
static EXECUTOR_ONCE: AtomicU8 = AtomicU8::new(0);

const EXECUTOR_UNINIT: u8 = 0;
const EXECUTOR_TAKEN: u8 = 1;
const EXECUTOR_INITING: u8 = 2;
const EXECUTOR_ACTIVE: u8 = 3;

// Use a sentinel value for context to denote the thread pender context
const THREAD_PENDER: usize = usize::MAX;

pub(crate) fn custom_executor_active() -> bool {
    EXECUTOR_ONCE.load(Ordering::Acquire) == EXECUTOR_ACTIVE
}

pub struct Executor {
    inner: raw::Executor,
    not_send: PhantomData<*mut ()>,
}

/// TODO: Taken from embassy-stm32, verify this is necessary or what we want
#[unsafe(export_name = "__pender")]
fn __pender(context: *mut ()) {
    // Safety: `context` is either `usize::MAX` created by `Executor::run`, or a valid interrupt
    // request number given to `InterruptExecutor::start`.

    let context = context as usize;

    // Try to make Rust optimize the branching away if we only use thread mode.
    if context == THREAD_PENDER {
        TASKS_PENDING.store(true, Ordering::Release);
        cortex_m::asm::sev();
    }
}

impl Executor {
    pub fn new() -> Self {
        let res = EXECUTOR_ONCE.compare_exchange(EXECUTOR_UNINIT, EXECUTOR_TAKEN, Ordering::AcqRel, Ordering::Relaxed);

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

        // TODO: We probably want to set SEVONPEND if we take a critical section while tearing
        // down and setting up for deep sleep!

        EXECUTOR_ONCE.store(EXECUTOR_ACTIVE, Ordering::Release);

        // Until we've performed HAL init, just do WFE sleep
        let power_depth = loop {
            unsafe {
                self.inner.poll();
                let sleep = crate::clocks::with_clocks(|c| c.core_sleep);
                if let Some(s) = sleep {
                    break s;
                }
                do_wfe();
                crate::perf_counters::incr_wfe_sleeps();
            }
        };

        match power_depth {
            // For Wfe sleep, our sleep target is constant. This means that
            // we don't need to do anything fancy here, just do a normal WFE
            // loop, since clock init already set our sleep mode parameters
            CoreSleep::WfeUngated | CoreSleep::WfeGated => loop {
                unsafe {
                    self.inner.poll();
                    do_wfe();
                    crate::perf_counters::incr_wfe_sleeps();
                }
            },
            CoreSleep::DeepSleep => loop {
                unsafe {
                    // For deep sleep, we need to be a bit more clever. First, poll any
                    // pending tasks
                    self.inner.poll();

                    // Next, we need to check if any high-power peripherals exist that should
                    // inhibit us from entering deep sleep. Take a critical section to check.
                    //
                    // We STAY in the CS for the deep sleep to ensure that we handle wake-up
                    // completely BEFORE yielding control flow back to interrupts.
                    let do_wfe_sleep = critical_section::with(|cs| !crate::clocks::deep_sleep_if_possible(&cs));

                    // Did we succeed at deep sleeping?
                    if do_wfe_sleep {
                        // Nope, WFE. We don't need a critical section here because we don't
                        // need to wait for clocks to resume before we service interrupts.
                        do_wfe();
                        crate::perf_counters::incr_wfe_sleeps();
                    } else {
                        // Yep!
                        crate::perf_counters::incr_deep_sleeps();
                    }
                }
            },
        }
    }
}

/// Every time we WFE, we want to do DSB; WFE.
#[inline(always)]
unsafe fn do_wfe() {
    cortex_m::asm::dsb();
    cortex_m::asm::wfe();
}

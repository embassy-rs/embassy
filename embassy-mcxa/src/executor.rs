use core::{
    arch::asm,
    marker::PhantomData,
    sync::atomic::{AtomicBool, AtomicU8, Ordering},
};
use critical_section::CriticalSection;
use embassy_executor::{Spawner, raw};
use nxp_pac::cmc::vals::CkctrlCkmode;

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
                asm!("wfe");
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
                    asm!("wfe");
                    crate::perf_counters::incr_wfe_sleeps();
                }
            },
            CoreSleep::DeepSleep => loop {
                // For deep sleep, we need to be a bit more clever. First, poll any
                // pending tasks
                unsafe {
                    self.inner.poll();
                }
                // Next, we need to check if any high-power peripherals exist that should
                // inhibit us from entering deep sleep. Take a critical section to check.
                let wfe_sleep = critical_section::with(|cs| {
                    let inhibit = crate::clocks::active_hp_tokens(&cs);
                    if inhibit {
                        return true;
                    }

                    // Yep, it's time to go to deep sleep. WHILE STILL IN the CS, get ready
                    setup_deep_sleep();

                    // Here we go!
                    //
                    // TODO: The C SDK does a weird procedure where they:
                    //
                    // ```asm
                    // sev
                    // dsb
                    // wfe ; guarantees sev state is cleared
                    // wfe ; actually goes to deep sleep
                    // ```
                    //
                    // Do we need to do the same?
                    unsafe {
                        asm!("wfe");
                    }

                    // Wakey wakey, eggs and bakey
                    recover_deep_sleep(&cs);

                    false
                });

                if wfe_sleep {
                    unsafe {
                        asm!("wfe");
                    }
                    crate::perf_counters::incr_wfe_sleeps();
                } else {
                    crate::perf_counters::incr_deep_sleeps();
                }
            },
        }
    }
}

fn setup_deep_sleep() {
    let cmc = nxp_pac::CMC;
    let spc = nxp_pac::SPC0;

    // Isolate/unpower external voltage domains
    spc.evd_cfg().write(|w| w.0 = 0);

    // To configure for Deep Sleep Low-Power mode entry:
    //
    // Write Fh to Clock Control (CKCTRL)
    cmc.ckctrl().modify(|w| w.set_ckmode(CkctrlCkmode::CKMODE1111));
    // Write 1h to Power Mode Protection (PMPROT)
    cmc.pmprot().write(|w| w.0 = 1);
    // Write 1h to Global Power Mode Control (GPMCTRL)
    cmc.gpmctrl().modify(|w| w.set_lpmode(0b0001));
    // Redundant?
    // cmc.pmctrlmain().modify(|w| w.set_lpmode(PmctrlmainLpmode::LPMODE0001));

    // SPC_LPWKUP_DELAY_LPWKUP_DELAY?
    // TODO: "When voltage levels are not the same between ACTIVE mode and Low Power mode, you must write a
    // nonzero value to this field."
    //
    // TODO: Do we need to ensure the CPU is on some kind of clock source that
    // is always-on, so we have a core clock source that we know is active when
    // we come back? How does this affect any peripherals that have main_clk selected
    // as a source?

    // From the C SDK:
    //
    // Before executing WFI instruction read back the last register to
    // ensure all registers writes have completed.
    let _ = cmc.gpmctrl().read();
}

fn recover_deep_sleep(cs: &CriticalSection) {
    let cmc = nxp_pac::CMC;

    // Restart any necessary clocks
    crate::clocks::restart_deep_sleep_clocks(cs);

    // Re-raise the sleep level to WFE sleep in the off chance that the
    // user decides to call `wfe` on their own accord, and to avoid having
    // to re-set if we chill in WFE sleep mostly
    cmc.ckctrl().modify(|w| w.set_ckmode(CkctrlCkmode::CKMODE0001));
}

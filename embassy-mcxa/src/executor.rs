use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};

use embassy_executor::{Spawner, raw};
use embassy_hal_internal::Peri;

use crate::clocks::config::CoreSleep;
use crate::gpio::{DriveStrength, GpioPin, Level, Output, SlewRate};
use crate::pac::gpio::vals::{Ptco, Ptso};

static TASKS_PENDING: AtomicBool = AtomicBool::new(false);
static EXECUTOR_ONCE: AtomicU8 = AtomicU8::new(0);

/// Information stored in format:
///
/// ```
/// 0bAxxx_xxxx_xxxx_xxxx_OOOO_OOOO_IIII_IIII
/// ```
///
/// Where:
///
/// * A: 1 bit, "is active", so we can differentiate between "never set"
///   and "use port 0 pin 0"
/// * O: 8 bits, "pOrt number"
/// * I: 8 bits, "pIn number"
///
/// Initially set to `0`, which makes "set lo" and "set hi" a no-op
static DEBUG_GPIO: AtomicU32 = AtomicU32::new(0);

const EXECUTOR_UNINIT: u8 = 0;
const EXECUTOR_TAKEN: u8 = 1;

// Use a sentinel value for context to denote the thread pender context
const THREAD_PENDER: usize = usize::MAX;

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
        // We can only create the executor once
        init(self.inner.spawner());

        // Until we've performed HAL init, just do WFE sleep
        let power_depth = loop {
            unsafe {
                self.inner.poll();

                if go_around() {
                    continue;
                }

                let sleep = crate::clocks::with_clocks(|c| c.core_sleep);
                if let Some(s) = sleep {
                    break s;
                }
                debug_lo();
                do_wfe();
                debug_hi();
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

                    if go_around() {
                        continue;
                    }

                    debug_lo();
                    do_wfe();
                    debug_hi();
                    crate::perf_counters::incr_wfe_sleeps();
                }
            },
            CoreSleep::DeepSleep => loop {
                unsafe {
                    // For deep sleep, we need to be a bit more clever. First, poll any
                    // pending tasks
                    self.inner.poll();

                    if go_around() {
                        continue;
                    }

                    // Next, we need to check if any high-power peripherals exist that should
                    // inhibit us from entering deep sleep. Take a critical section to check.
                    //
                    // We STAY in the CS for the deep sleep to ensure that we handle wake-up
                    // completely BEFORE yielding control flow back to interrupts.
                    debug_lo();
                    let do_wfe_sleep = critical_section::with(|cs| {
                        let did_deep_sleep = crate::clocks::deep_sleep_if_possible(&cs);
                        if did_deep_sleep {
                            debug_hi();
                        }
                        !did_deep_sleep
                    });

                    // Did we succeed at deep sleeping?
                    if do_wfe_sleep {
                        // Nope, WFE. We don't need a critical section here because we don't
                        // need to wait for clocks to resume before we service interrupts.
                        do_wfe();
                        debug_hi();
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

/// Function to go around to poll again, and clear a pending sev if we
/// know we will immediately wake anyway.
fn go_around() -> bool {
    let sev_pending = TASKS_PENDING.swap(false, Ordering::AcqRel);
    if sev_pending {
        // We know __pender has sent a sev, ack it with a wfe, which we
        // know will immediately return control flow to us.
        cortex_m::asm::wfe();
    }
    sev_pending
}

/// Dedicate a pin to be used for introspecting the state of the custom executor.
///
/// This pin will be driven low prior to entering WFE (light or deep sleep),
/// and be raised once control flow is returned to the executor.
///
/// For deep sleep, the pin will be raised *before* exiting the critical section,
/// meaning that the level will go high before servicing any interrupts. For light
/// sleep, no critical section is taken prior to deep sleep, so the pin will go
/// high once the executor resumes, likely after processing any pending interrupts.
///
/// This exact behavior is not considered semver stable, and may change at any time.
///
/// This function consumes the given pin, overwriting any previous pin set as the executor
/// debug pin. If this function is called multiple times, the previously assigned pin(s)
/// will not be disabled, and will remain at their last updated state.
pub fn set_executor_debug_gpio(pin: Peri<'static, impl GpioPin>) {
    let pin_num = pin.pin();
    let port_num = pin.port();
    // Setup GPIO as output, initially high
    let output = Output::new(pin, Level::High, DriveStrength::Normal, SlewRate::Slow);

    let number = 0x8000_0000 | ((port_num as u32) << 8) | (pin_num as u32);
    core::mem::forget(output);
    DEBUG_GPIO.store(number, Ordering::Relaxed);
}

/// Set low if we have a debug gpio set, using raw pac methods
fn debug_lo() {
    let num = DEBUG_GPIO.load(Ordering::Relaxed);
    if num == 0 {
        return;
    }
    let port_num = (num >> 8) as u8;
    let pin_num = (num & 0xFF) as usize;
    match port_num {
        0 => crate::pac::GPIO0.pcor(),
        1 => crate::pac::GPIO1.pcor(),
        2 => crate::pac::GPIO2.pcor(),
        3 => crate::pac::GPIO3.pcor(),
        4 => crate::pac::GPIO4.pcor(),
        _ => return,
    }
    .write(|w| w.set_ptco(pin_num, Ptco::PTCO1));
}

/// Set high if we have a debug gpio set, using raw pac methods
fn debug_hi() {
    let num = DEBUG_GPIO.load(Ordering::Relaxed);
    if num == 0 {
        return;
    }
    let port_num = (num >> 8) as u8;
    let pin_num = (num & 0xFF) as usize;
    match port_num {
        0 => crate::pac::GPIO0.psor(),
        1 => crate::pac::GPIO1.psor(),
        2 => crate::pac::GPIO2.psor(),
        3 => crate::pac::GPIO3.psor(),
        4 => crate::pac::GPIO4.psor(),
        _ => return,
    }
    .write(|w| w.set_ptso(pin_num, Ptso::PTSO1));
}

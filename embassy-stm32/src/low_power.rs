//! Low-power support.
//!
//! The STM32 line of microcontrollers support various deep-sleep modes which exploit clock-gating
//! to reduce power consumption. `embassy-stm32` provides a low-power executor, [`Executor`] which
//! can use knowledge of which peripherals are currently blocked upon to transparently and safely
//! enter such low-power modes (currently, only `STOP2`) when idle.
//!
//! The executor determines which peripherals are active by their RCC state; consequently,
//! low-power states can only be entered if all peripherals have been `drop`'d. There are a few
//! exceptions to this rule:
//!
//!  * `GPIO`
//!  * `RTC`
//!
//! Since entering and leaving low-power modes typically incurs a significant latency, the
//! low-power executor will only attempt to enter when the next timer event is at least
//! [`time_driver::MIN_STOP_PAUSE`] in the future.
//!
//! Currently there is no macro analogous to `embassy_executor::main` for this executor;
//! consequently one must define their entrypoint manually. Moreover, you must relinquish control
//! of the `RTC` peripheral to the executor. This will typically look like
//!
//! ```rust,no_run
//! use embassy_executor::Spawner;
//! use embassy_stm32::low_power::Executor;
//! use embassy_stm32::rtc::{Rtc, RtcConfig};
//! use static_cell::StaticCell;
//!
//! #[cortex_m_rt::entry]
//! fn main() -> ! {
//!     Executor::take().run(|spawner| {
//!         spawner.spawn(unwrap!(async_main(spawner)));
//!     });
//! }
//!
//! #[embassy_executor::task]
//! async fn async_main(spawner: Spawner) {
//!     // initialize the platform...
//!     let mut config = embassy_stm32::Config::default();
//!     // when enabled the power-consumption is much higher during stop, but debugging and RTT is working
//!     config.enable_debug_during_sleep = false;
//!     let p = embassy_stm32::init(config);
//!
//!     // give the RTC to the executor...
//!     let mut rtc = Rtc::new(p.RTC, RtcConfig::default());
//!     embassy_stm32::low_power::stop_with_rtc(rtc);
//!
//!     // your application here...
//! }
//! ```

// TODO: Usage of `static mut` here is unsound. Fix then remove this `allow`.`
#![allow(static_mut_refs)]

use core::arch::asm;
use core::marker::PhantomData;
use core::sync::atomic::{Ordering, compiler_fence};

use cortex_m::peripheral::SCB;
use embassy_executor::*;

use crate::interrupt;
use crate::time_driver::{RtcDriver, get_driver};

const THREAD_PENDER: usize = usize::MAX;

use crate::rtc::Rtc;

static mut EXECUTOR: Option<Executor> = None;

#[cfg(stm32wlex)]
pub(crate) use self::busy::DeviceBusy;
#[cfg(stm32wlex)]
mod busy {
    use core::sync::atomic::{AtomicU32, Ordering};

    // Count of devices blocking STOP
    static STOP_BLOCKED: AtomicU32 = AtomicU32::new(0);

    /// Check if STOP1 is blocked.
    pub(crate) fn stop_blocked() -> bool {
        STOP_BLOCKED.load(Ordering::SeqCst) > 0
    }

    /// When ca device goes busy it will construct one of these where it will be dropped when the device goes idle.
    pub(crate) struct DeviceBusy {}

    impl DeviceBusy {
        /// Create a new DeviceBusy.
        pub(crate) fn new() -> Self {
            STOP_BLOCKED.fetch_add(1, Ordering::SeqCst);
            trace!("low power: device busy: Stop:{}", STOP_BLOCKED.load(Ordering::SeqCst));
            Self {}
        }
    }

    impl Drop for DeviceBusy {
        fn drop(&mut self) {
            STOP_BLOCKED.fetch_sub(1, Ordering::SeqCst);
            trace!("low power: device idle: Stop:{}", STOP_BLOCKED.load(Ordering::SeqCst));
        }
    }
}
#[cfg(not(stm32u0))]
foreach_interrupt! {
    (RTC, rtc, $block:ident, WKUP, $irq:ident) => {
        #[interrupt]
        #[allow(non_snake_case)]
        unsafe fn $irq() {
            EXECUTOR.as_mut().unwrap().on_wakeup_irq();
        }
    };
}

#[cfg(stm32u0)]
foreach_interrupt! {
    (RTC, rtc, $block:ident, TAMP, $irq:ident) => {
        #[interrupt]
        #[allow(non_snake_case)]
        unsafe fn $irq() {
            EXECUTOR.as_mut().unwrap().on_wakeup_irq();
        }
    };
}

#[allow(dead_code)]
pub(crate) unsafe fn on_wakeup_irq() {
    if EXECUTOR.is_some() {
        trace!("low power: wakeup irq");
        EXECUTOR.as_mut().unwrap().on_wakeup_irq();
    }
}

/// Configure STOP mode with RTC.
pub fn stop_with_rtc(rtc: Rtc) {
    unsafe { EXECUTOR.as_mut().unwrap() }.stop_with_rtc(rtc)
}

/// Reconfigure the RTC, if set.
pub fn reconfigure_rtc(f: impl FnOnce(&mut Rtc)) {
    unsafe { EXECUTOR.as_mut().unwrap() }.reconfigure_rtc(f);
}

/// Get whether the core is ready to enter the given stop mode.
///
/// This will return false if some peripheral driver is in use that
/// prevents entering the given stop mode.
pub fn stop_ready(stop_mode: StopMode) -> bool {
    match unsafe { EXECUTOR.as_mut().unwrap() }.stop_mode() {
        Some(StopMode::Stop2) => true,
        Some(StopMode::Stop1) => stop_mode == StopMode::Stop1,
        None => false,
    }
}

/// Available Stop modes.
#[non_exhaustive]
#[derive(PartialEq)]
pub enum StopMode {
    /// STOP 1
    Stop1,
    /// STOP 2
    Stop2,
}

#[cfg(any(stm32l4, stm32l5, stm32u5, stm32wba, stm32wlex, stm32u0))]
use stm32_metapac::pwr::vals::Lpms;

#[cfg(any(stm32l4, stm32l5, stm32u5, stm32wba, stm32wlex, stm32u0))]
impl Into<Lpms> for StopMode {
    fn into(self) -> Lpms {
        match self {
            StopMode::Stop1 => Lpms::STOP1,
            #[cfg(not(stm32wba))]
            StopMode::Stop2 => Lpms::STOP2,
            #[cfg(stm32wba)]
            StopMode::Stop2 => Lpms::STOP1, // TODO: WBA has no STOP2?
        }
    }
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
pub struct Executor {
    inner: raw::Executor,
    not_send: PhantomData<*mut ()>,
    scb: SCB,
    time_driver: &'static RtcDriver,
}

impl Executor {
    /// Create a new Executor.
    pub fn take() -> &'static mut Self {
        critical_section::with(|_| unsafe {
            assert!(EXECUTOR.is_none());

            EXECUTOR = Some(Self {
                inner: raw::Executor::new(THREAD_PENDER as *mut ()),
                not_send: PhantomData,
                scb: cortex_m::Peripherals::steal().SCB,
                time_driver: get_driver(),
            });

            let executor = EXECUTOR.as_mut().unwrap();

            executor
        })
    }

    unsafe fn on_wakeup_irq(&mut self) {
        #[cfg(stm32wlex)]
        {
            let extscr = crate::pac::PWR.extscr().read();
            if extscr.c1stop2f() || extscr.c1stopf() {
                // when we wake from any stop mode we need to re-initialize the rcc
                crate::rcc::apply_resume_config();
                if extscr.c1stop2f() {
                    // when we wake from STOP2, we need to re-initialize the time driver
                    critical_section::with(|cs| crate::time_driver::init_timer(cs));
                    // reset the refcounts for STOP2 and STOP1 (initializing the time driver will increment one of them for the timer)
                    // and given that we just woke from STOP2, we can reset them
                    crate::rcc::REFCOUNT_STOP2 = 0;
                    crate::rcc::REFCOUNT_STOP1 = 0;
                }
            }
        }
        self.time_driver.resume_time();
        trace!("low power: resume");
    }

    pub(self) fn stop_with_rtc(&mut self, rtc: Rtc) {
        self.time_driver.set_rtc(rtc);
        self.time_driver.reconfigure_rtc(|rtc| rtc.enable_wakeup_line());

        trace!("low power: stop with rtc configured");
    }

    pub(self) fn reconfigure_rtc(&mut self, f: impl FnOnce(&mut Rtc)) {
        self.time_driver.reconfigure_rtc(f);
    }

    fn stop1_ok_to_enter(&self) -> bool {
        #[cfg(stm32wlex)]
        if self::busy::stop_blocked() {
            return false;
        }
        unsafe { crate::rcc::REFCOUNT_STOP1 == 0 }
    }

    fn stop2_ok_to_enter(&self) -> bool {
        self.stop1_ok_to_enter() && unsafe { crate::rcc::REFCOUNT_STOP2 == 0 }
    }

    fn stop_mode(&self) -> Option<StopMode> {
        if self.stop2_ok_to_enter() {
            Some(StopMode::Stop2)
        } else if self.stop1_ok_to_enter() {
            Some(StopMode::Stop1)
        } else {
            None
        }
    }

    #[allow(unused_variables)]
    fn configure_stop(&mut self, stop_mode: StopMode) {
        #[cfg(any(stm32l4, stm32l5, stm32u5, stm32u0, stm32wba, stm32wlex))]
        crate::pac::PWR.cr1().modify(|m| m.set_lpms(stop_mode.into()));
        #[cfg(stm32h5)]
        crate::pac::PWR.pmcr().modify(|v| {
            use crate::pac::pwr::vals;
            v.set_lpms(vals::Lpms::STOP);
            v.set_svos(vals::Svos::SCALE3);
        });
    }

    fn configure_pwr(&mut self) {
        self.scb.clear_sleepdeep();
        // Clear any previous stop flags
        #[cfg(stm32wlex)]
        crate::pac::PWR.extscr().modify(|w| {
            w.set_c1cssf(true);
        });

        compiler_fence(Ordering::SeqCst);

        let stop_mode = self.stop_mode();

        if stop_mode.is_none() {
            trace!("low power: not ready to stop");
            return;
        }

        if self.time_driver.pause_time().is_err() {
            trace!("low power: failed to pause time");
            return;
        }

        let stop_mode = stop_mode.unwrap();
        match stop_mode {
            StopMode::Stop1 => trace!("low power: stop 1"),
            StopMode::Stop2 => trace!("low power: stop 2"),
        }
        self.configure_stop(stop_mode);

        #[cfg(not(feature = "low-power-debug-with-sleep"))]
        self.scb.set_sleepdeep();
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
        let executor = unsafe { EXECUTOR.as_mut().unwrap() };
        init(executor.inner.spawner());

        loop {
            unsafe {
                executor.inner.poll();
                self.configure_pwr();
                asm!("wfe");
                #[cfg(stm32wlex)]
                {
                    let es = crate::pac::PWR.extscr().read();
                    match (es.c1stopf(), es.c1stop2f()) {
                        (true, false) => debug!("low power: wake from STOP1"),
                        (false, true) => debug!("low power: wake from STOP2"),
                        (true, true) => debug!("low power: wake from STOP1 and STOP2 ???"),
                        (false, false) => trace!("low power: stop mode not entered"),
                    };
                    crate::pac::PWR.extscr().modify(|w| {
                        w.set_c1cssf(false);
                    });
                }
            };
        }
    }
}

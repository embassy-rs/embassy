//! Low-power support.
//!
//! The STM32 line of microcontrollers support various deep-sleep modes which exploit clock-gating
//! to reduce power consumption. `embassy-stm32` provides a low-power executor, [`Executor`] which
//! can use knowledge of which peripherals are currently blocked upon to transparently and safely
//! enter such low-power modes including `STOP1` and `STOP2` when idle.
//!
//! The executor determines which peripherals are active by their RCC state; consequently,
//! low-power states can only be entered if peripherals which block stop have been `drop`'d and if
//! peripherals that do not block stop are busy. Peripherals which never block stop include:
//!
//!  * `GPIO`
//!  * `RTC`
//!
//! Other peripherals which block stop when busy include (this list may be stale):
//!
//!  * `I2C`
//!  * `USART`
//!
//! Since entering and leaving low-power modes typically incurs a significant latency, the
//! low-power executor will only attempt to enter when the next timer event is at least
//! [`config.min_stop_pause`] in the future.
//!
//!
//! ```rust,no_run
//! use embassy_executor::Spawner;
//! use embassy_time::Duration;
//!
//! #[embassy_executor::main(executor = "embassy_stm32::Executor", entry = "cortex_m_rt::entry")]
//! async fn main(spawner: Spawner) {
//!     // initialize the platform...
//!     let mut config = embassy_stm32::Config::default();
//!     // the default value, but can be adjusted
//!     config.min_stop_pause = Duration::from_millis(250);
//!     // when enabled the power-consumption is much higher during stop, but debugging and RTT is working
//!     config.enable_debug_during_sleep = false;
//!     let p = embassy_stm32::init(config);
//!
//!     // your application here...
//! }
//! ```

use core::arch::asm;
use core::marker::PhantomData;
use core::mem;
use core::sync::atomic::{AtomicBool, Ordering, compiler_fence};

use cortex_m::peripheral::SCB;
use critical_section::CriticalSection;
use embassy_executor::*;

use crate::interrupt;
pub use crate::rcc::StopMode;
use crate::rcc::{BusyPeripheral, RCC_CONFIG, REFCOUNT_STOP1, REFCOUNT_STOP2};
use crate::time_driver::get_driver;

const THREAD_PENDER: usize = usize::MAX;

static EXECUTOR_TAKEN: AtomicBool = AtomicBool::new(false);
#[cfg(feature = "low-power-pender")]
static TASKS_PENDING: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "low-power-pender")]
#[unsafe(export_name = "__pender")]
fn __pender(context: *mut ()) {
    unsafe {
        // Safety: `context` is either `usize::MAX` created by `Executor::run`, or a valid interrupt
        // request number given to `InterruptExecutor::start`.

        let context = context as usize;

        // Try to make Rust optimize the branching away if we only use thread mode.
        if context == THREAD_PENDER {
            TASKS_PENDING.store(true, Ordering::Release);
            core::arch::asm!("sev");
            return;
        }
    }
}

/// Prevent the device from going into the stop mode if held
pub struct DeviceBusy {
    _stop_mode: BusyPeripheral<StopMode>,
}

impl DeviceBusy {
    /// Create a new DeviceBusy with stop1.
    pub fn new_stop1() -> Self {
        Self::new(StopMode::Stop1)
    }

    /// Create a new DeviceBusy with stop2.
    pub fn new_stop2() -> Self {
        Self::new(StopMode::Stop2)
    }

    /// Create a new DeviceBusy.
    pub fn new(stop_mode: StopMode) -> Self {
        Self {
            _stop_mode: BusyPeripheral::new(stop_mode),
        }
    }
}

#[cfg(not(stm32u0))]
foreach_interrupt! {
    (RTC, rtc, $block:ident, WKUP, $irq:ident) => {
        #[interrupt]
        #[allow(non_snake_case)]
        unsafe fn $irq() {
            Executor::on_wakeup_irq_or_event();
        }
    };
}

#[cfg(stm32u0)]
foreach_interrupt! {
    (RTC, rtc, $block:ident, TAMP, $irq:ident) => {
        #[interrupt]
        #[allow(non_snake_case)]
        unsafe fn $irq() {
            Executor::on_wakeup_irq_or_event();
        }
    };
}

/// Get whether the core is ready to enter the given stop mode.
///
/// This will return false if some peripheral driver is in use that
/// prevents entering the given stop mode.
pub fn stop_ready(stop_mode: StopMode) -> bool {
    critical_section::with(|cs| match Executor::stop_mode(cs) {
        Some(StopMode::Standby | StopMode::Stop2) => true,
        Some(StopMode::Stop1) => stop_mode == StopMode::Stop1,
        None => false,
    })
}

#[cfg(any(stm32l4, stm32l5, stm32u5, stm32wba, stm32wb, stm32wl, stm32u0))]
use crate::pac::pwr::vals::Lpms;

#[cfg(any(stm32l4, stm32l5, stm32u5, stm32wba, stm32wb, stm32wl, stm32u0))]
impl Into<Lpms> for StopMode {
    fn into(self) -> Lpms {
        match self {
            StopMode::Stop1 => Lpms::STOP1,
            #[cfg(not(stm32wba))]
            StopMode::Standby | StopMode::Stop2 => Lpms::STOP2,
            #[cfg(stm32wba)]
            StopMode::Standby | StopMode::Stop2 => Lpms::STOP1, // TODO: WBA has no STOP2?
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
}

impl Executor {
    /// Create a new Executor.
    pub fn new() -> Self {
        if EXECUTOR_TAKEN.load(Ordering::Acquire) {
            panic!("Low power executor can only be taken once.");
        } else {
            EXECUTOR_TAKEN.store(true, Ordering::Release);
        }

        Self {
            inner: raw::Executor::new(THREAD_PENDER as *mut ()),
            not_send: PhantomData,
        }
    }

    pub(crate) unsafe fn on_wakeup_irq_or_event() {
        if !get_driver().is_stopped() {
            trace!("low power: time driver not stopped!");
            return;
        }

        critical_section::with(|cs| {
            #[cfg(any(stm32wl, stm32wb))]
            {
                let es = crate::pac::PWR.extscr().read();
                #[cfg(stm32wl)]
                match (es.c1stopf(), es.c1stop2f()) {
                    (true, false) => debug!("low power: cpu1 wake from STOP1"),
                    (false, true) => debug!("low power: cpu1 wake from STOP2"),
                    (true, true) => debug!("low power: cpu1 wake from STOP1 and STOP2 ???"),
                    (false, false) => trace!("low power: cpu1 stop mode not entered"),
                };
                #[cfg(stm32wl5x)]
                // TODO: only for the current cpu
                match (es.c2stopf(), es.c2stop2f()) {
                    (true, false) => debug!("low power: cpu2 wake from STOP1"),
                    (false, true) => debug!("low power: cpu2 wake from STOP2"),
                    (true, true) => debug!("low power: cpu2 wake from STOP1 and STOP2 ???"),
                    (false, false) => trace!("low power: cpu2 stop mode not entered"),
                };

                #[cfg(stm32wb)]
                match (es.c1stopf(), es.c2stopf()) {
                    (true, false) => debug!("low power: cpu1 wake from STOP"),
                    (false, true) => debug!("low power: cpu2 wake from STOP"),
                    (true, true) => debug!("low power: cpu1 and cpu2 wake from STOP"),
                    (false, false) => trace!("low power: stop mode not entered"),
                };

                let _has_stopped2 = {
                    #[cfg(stm32wb)]
                    {
                        es.c2stopf()
                    }

                    #[cfg(stm32wlex)]
                    {
                        es.c1stop2f()
                    }

                    #[cfg(stm32wl5x)]
                    {
                        // TODO: I think we could just use c1stop2f() here as it won't enter a stop mode unless BOTH cpus will enter it.
                        es.c1stop2f() | es.c2stop2f()
                    }
                };

                #[cfg(not(stm32wb))]
                if es.c1stopf() || _has_stopped2 {
                    // when we wake from any stop mode we need to re-initialize the rcc
                    crate::rcc::init(RCC_CONFIG.unwrap());
                    if _has_stopped2 {
                        // when we wake from STOP2, we need to re-initialize the time driver
                        get_driver().init_timer(cs);
                        // reset the refcounts for STOP2 and STOP1 (initializing the time driver will increment one of them for the timer)
                        // and given that we just woke from STOP2, we can reset them
                        REFCOUNT_STOP2 = 0;
                        REFCOUNT_STOP1 = 0;
                    }
                }
                // Clear all stop flags
                #[cfg(stm32wl)]
                crate::pac::PWR.extscr().modify(|w| {
                    w.set_c1cssf(true);
                    #[cfg(stm32wl5x)]
                    w.set_c2cssf(true);
                });
            }
            get_driver().resume_time(cs);

            trace!("low power: resume time");
        });
    }

    const fn get_scb() -> SCB {
        unsafe { mem::transmute(()) }
    }

    fn stop_mode(_cs: CriticalSection) -> Option<StopMode> {
        // We cannot enter standby because we will lose program state.
        if unsafe { REFCOUNT_STOP2 == 0 && REFCOUNT_STOP1 == 0 } {
            Some(StopMode::Stop2)
        } else if unsafe { REFCOUNT_STOP1 == 0 } {
            Some(StopMode::Stop1)
        } else {
            trace!("low power: not ready to stop (refcount_stop1: {})", unsafe {
                REFCOUNT_STOP1
            });
            None
        }
    }

    #[cfg(all(stm32wb, feature = "low-power"))]
    fn configure_stop_stm32wb(
        &self,
        _cs: CriticalSection,
    ) -> Result<crate::hsem::HardwareSemaphoreMutex<'_, crate::peripherals::HSEM>, ()> {
        use core::task::Poll;

        use embassy_futures::poll_once;

        use crate::hsem::HardwareSemaphoreChannel;
        use crate::pac::rcc::vals::{Smps, Sw};
        use crate::pac::{PWR, RCC};

        trace!("low power: trying to get sem3");

        let sem3_mutex = match poll_once(HardwareSemaphoreChannel::<crate::peripherals::HSEM>::new(3).lock(0)) {
            Poll::Pending => None,
            Poll::Ready(mutex) => Some(mutex),
        }
        .ok_or(())?;

        trace!("low power: got sem3");

        let sem4_mutex = HardwareSemaphoreChannel::<crate::peripherals::HSEM>::new(4).try_lock(0);
        if let Some(sem4_mutex) = sem4_mutex {
            trace!("low power: got sem4");

            if PWR.extscr().read().c2ds() {
                drop(sem4_mutex);
            } else {
                return Ok(sem3_mutex);
            }
        }

        // Sem4 not granted
        // Set HSION
        RCC.cr().modify(|w| {
            w.set_hsion(true);
        });

        // Wait for HSIRDY
        while !RCC.cr().read().hsirdy() {}

        // Set SW to HSI
        RCC.cfgr().modify(|w| {
            w.set_sw(Sw::HSI);
        });

        // Wait for SWS to report HSI
        while !RCC.cfgr().read().sws().eq(&Sw::HSI) {}

        // Set SMPSSEL to HSI
        RCC.smpscr().modify(|w| {
            w.set_smpssel(Smps::HSI);
        });

        Ok(sem3_mutex)
    }

    #[allow(unused_variables)]
    fn configure_stop(&self, _cs: CriticalSection, stop_mode: StopMode) -> Result<(), ()> {
        #[cfg(stm32wb)]
        let mutex = {
            use crate::pac::{PWR, RCC};

            let mutex = self.configure_stop_stm32wb(_cs)?;

            // on PWR
            RCC.apb1enr1().modify(|r| r.0 |= 1 << 28);
            cortex_m::asm::dsb();

            // off SMPS, on Bypass
            PWR.cr5().modify(|r| {
                let mut val = r.0;
                val &= !(1 << 15); // sdeb = 0 (off SMPS)
                val |= 1 << 14; // sdben = 1 (on Bypass)
                r.0 = val
            });

            cortex_m::asm::delay(1000);

            mutex
        };

        #[cfg(any(stm32l4, stm32l5, stm32u5, stm32u0, stm32wb, stm32wba, stm32wl))]
        crate::pac::PWR.cr1().modify(|m| m.set_lpms(stop_mode.into()));
        #[cfg(stm32h5)]
        crate::pac::PWR.pmcr().modify(|v| {
            use crate::pac::pwr::vals;
            v.set_lpms(vals::Lpms::STOP);
            v.set_svos(vals::Svos::SCALE3);
        });

        #[cfg(stm32wb)]
        drop(mutex);

        Ok(())
    }

    fn configure_pwr(&self) {
        Self::get_scb().clear_sleepdeep();
        // Clear any previous stop flags
        #[cfg(stm32wl)]
        crate::pac::PWR.extscr().modify(|w| {
            w.set_c1cssf(true);
            #[cfg(stm32wl5x)]
            w.set_c2cssf(true);
        });

        #[cfg(feature = "low-power-pender")]
        if TASKS_PENDING.load(Ordering::Acquire) {
            TASKS_PENDING.store(false, Ordering::Release);

            return;
        }

        compiler_fence(Ordering::Acquire);

        critical_section::with(|cs| {
            let _ = unsafe { RCC_CONFIG }?;
            let stop_mode = Self::stop_mode(cs)?;
            get_driver().pause_time(cs).ok()?;
            self.configure_stop(cs, stop_mode).ok()?;

            Some(stop_mode)
        })
        .map(|stop_mode| {
            trace!("low power: enter stop: {}", stop_mode);

            #[cfg(not(feature = "low-power-debug-with-sleep"))]
            Self::get_scb().set_sleepdeep();
        });
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
                self.configure_pwr();
                #[cfg(feature = "defmt")]
                defmt::flush();
                asm!("wfe");
                Self::on_wakeup_irq_or_event();
            };
        }
    }
}

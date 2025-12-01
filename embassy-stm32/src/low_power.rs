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
//! [`time_driver::min_stop_pause`] in the future.
//!
//! Currently there is no macro analogous to `embassy_executor::main` for this executor;
//! consequently one must define their entrypoint manually. Moreover, you must relinquish control
//! of the `RTC` peripheral to the executor. This will typically look like
//!
//! ```rust,no_run
//! use embassy_executor::Spawner;
//! use embassy_stm32::low_power;
//! use embassy_stm32::rtc::{Rtc, RtcConfig};
//! use embassy_time::Duration;
//!
//! #[embassy_executor::main(executor = "low_power::Executor")]
//! async fn async_main(spawner: Spawner) {
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
use core::sync::atomic::{Ordering, compiler_fence};

use cortex_m::peripheral::SCB;
use critical_section::CriticalSection;
use embassy_executor::*;

use crate::interrupt;
pub use crate::rcc::StopMode;
use crate::rcc::{BusyPeripheral, RCC_CONFIG, REFCOUNT_STOP1, REFCOUNT_STOP2};
use crate::time_driver::get_driver;

const THREAD_PENDER: usize = usize::MAX;

static mut EXECUTOR_TAKEN: bool = false;

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
            Executor::on_wakeup_irq();
        }
    };
}

#[cfg(stm32u0)]
foreach_interrupt! {
    (RTC, rtc, $block:ident, TAMP, $irq:ident) => {
        #[interrupt]
        #[allow(non_snake_case)]
        unsafe fn $irq() {
            Executor::on_wakeup_irq();
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

#[cfg(any(stm32l4, stm32l5, stm32u5, stm32wba, stm32wb, stm32wlex, stm32u0))]
use crate::pac::pwr::vals::Lpms;

#[cfg(any(stm32l4, stm32l5, stm32u5, stm32wba, stm32wb, stm32wlex, stm32u0))]
impl Into<Lpms> for StopMode {
    fn into(self) -> Lpms {
        match self {
            StopMode::Stop1 => Lpms::STOP1,
            #[cfg(not(any(stm32wb, stm32wba)))]
            StopMode::Standby | StopMode::Stop2 => Lpms::STOP2,
            #[cfg(any(stm32wb, stm32wba))]
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
        unsafe {
            if EXECUTOR_TAKEN {
                panic!("Low power executor can only be taken once.");
            } else {
                EXECUTOR_TAKEN = true;
            }
        }

        Self {
            inner: raw::Executor::new(THREAD_PENDER as *mut ()),
            not_send: PhantomData,
        }
    }

    pub(crate) unsafe fn on_wakeup_irq() {
        critical_section::with(|cs| {
            #[cfg(stm32wlex)]
            {
                use crate::pac::rcc::vals::Sw;
                use crate::pac::{PWR, RCC};
                use crate::rcc::init as init_rcc;

                let extscr = PWR.extscr().read();
                if extscr.c1stop2f() || extscr.c1stopf() {
                    // when we wake from any stop mode we need to re-initialize the rcc
                    while RCC.cfgr().read().sws() != Sw::MSI {}

                    init_rcc(RCC_CONFIG.unwrap());

                    if extscr.c1stop2f() {
                        // when we wake from STOP2, we need to re-initialize the time driver
                        get_driver().init_timer(cs);
                        // reset the refcounts for STOP2 and STOP1 (initializing the time driver will increment one of them for the timer)
                        // and given that we just woke from STOP2, we can reset them
                        REFCOUNT_STOP2 = 0;
                        REFCOUNT_STOP1 = 0;
                    }
                }
            }
            get_driver().resume_time(cs);
            trace!("low power: resume");
        });
    }

    const fn get_scb() -> SCB {
        unsafe { mem::transmute(()) }
    }

    fn stop_mode(_cs: CriticalSection) -> Option<StopMode> {
        // We cannot enter standby because we will lose program state.
        if unsafe { REFCOUNT_STOP2 == 0 && REFCOUNT_STOP1 == 0 } {
            trace!("low power: stop 2");
            Some(StopMode::Stop2)
        } else if unsafe { REFCOUNT_STOP1 == 0 } {
            trace!("low power: stop 1");
            Some(StopMode::Stop1)
        } else {
            trace!("low power: not ready to stop (refcount_stop1: {})", unsafe {
                REFCOUNT_STOP1
            });
            None
        }
    }

    #[cfg(all(stm32wb, feature = "low-power"))]
    fn configure_stop_stm32wb(&self, _cs: CriticalSection) -> Result<(), ()> {
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
                return Ok(());
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

        drop(sem3_mutex);

        Ok(())
    }

    #[allow(unused_variables)]
    fn configure_stop(&self, _cs: CriticalSection, stop_mode: StopMode) -> Result<(), ()> {
        #[cfg(all(stm32wb, feature = "low-power"))]
        self.configure_stop_stm32wb(_cs)?;

        #[cfg(any(stm32l4, stm32l5, stm32u5, stm32u0, stm32wb, stm32wba, stm32wlex))]
        crate::pac::PWR.cr1().modify(|m| m.set_lpms(stop_mode.into()));
        #[cfg(stm32h5)]
        crate::pac::PWR.pmcr().modify(|v| {
            use crate::pac::pwr::vals;
            v.set_lpms(vals::Lpms::STOP);
            v.set_svos(vals::Svos::SCALE3);
        });

        Ok(())
    }

    fn configure_pwr(&self) {
        Self::get_scb().clear_sleepdeep();
        // Clear any previous stop flags
        #[cfg(stm32wlex)]
        crate::pac::PWR.extscr().modify(|w| {
            w.set_c1cssf(true);
        });

        compiler_fence(Ordering::SeqCst);

        critical_section::with(|cs| {
            let _ = unsafe { RCC_CONFIG }?;
            let stop_mode = Self::stop_mode(cs)?;
            get_driver().pause_time(cs).ok()?;
            self.configure_stop(cs, stop_mode).ok()?;

            Some(())
        })
        .map(|_| {
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

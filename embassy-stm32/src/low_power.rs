use core::arch::asm;
use core::marker::PhantomData;
use core::sync::atomic::{compiler_fence, Ordering};

use cortex_m::peripheral::SCB;
use embassy_executor::*;

use crate::interrupt;
use crate::time_driver::{get_driver, RtcDriver};

const THREAD_PENDER: usize = usize::MAX;

use crate::rtc::Rtc;

static mut EXECUTOR: Option<Executor> = None;

foreach_interrupt! {
    (RTC, rtc, $block:ident, WKUP, $irq:ident) => {
        #[interrupt]
        unsafe fn $irq() {
            EXECUTOR.as_mut().unwrap().on_wakeup_irq();
        }
    };
}

#[allow(dead_code)]
pub(crate) unsafe fn on_wakeup_irq() {
    EXECUTOR.as_mut().unwrap().on_wakeup_irq();
}

pub fn stop_with_rtc(rtc: &'static Rtc) {
    unsafe { EXECUTOR.as_mut().unwrap() }.stop_with_rtc(rtc)
}

pub fn stop_ready(stop_mode: StopMode) -> bool {
    unsafe { EXECUTOR.as_mut().unwrap() }.stop_ready(stop_mode)
}

#[non_exhaustive]
pub enum StopMode {
    Stop2,
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
        unsafe {
            assert!(EXECUTOR.is_none());

            EXECUTOR = Some(Self {
                inner: raw::Executor::new(THREAD_PENDER as *mut ()),
                not_send: PhantomData,
                scb: cortex_m::Peripherals::steal().SCB,
                time_driver: get_driver(),
            });

            EXECUTOR.as_mut().unwrap()
        }
    }

    unsafe fn on_wakeup_irq(&mut self) {
        self.time_driver.resume_time();
        trace!("low power: resume");
    }

    pub(self) fn stop_with_rtc(&mut self, rtc: &'static Rtc) {
        self.time_driver.set_rtc(rtc);

        rtc.enable_wakeup_line();

        trace!("low power: stop with rtc configured");
    }

    fn stop_ready(&self, stop_mode: StopMode) -> bool {
        match stop_mode {
            StopMode::Stop2 => unsafe { crate::rcc::REFCOUNT_STOP2 == 0 },
        }
    }

    fn configure_pwr(&mut self) {
        self.scb.clear_sleepdeep();

        compiler_fence(Ordering::SeqCst);

        if !self.stop_ready(StopMode::Stop2) {
            trace!("low power: not ready to stop");
        } else if self.time_driver.pause_time().is_err() {
            trace!("low power: failed to pause time");
        } else {
            trace!("low power: stop");
            self.scb.set_sleepdeep();
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
        init(unsafe { EXECUTOR.as_mut().unwrap() }.inner.spawner());

        loop {
            unsafe {
                EXECUTOR.as_mut().unwrap().inner.poll();
                self.configure_pwr();
                asm!("wfe");
            };
        }
    }
}

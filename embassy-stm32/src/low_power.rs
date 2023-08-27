use core::arch::asm;
use core::marker::PhantomData;

use cortex_m::peripheral::SCB;
use embassy_executor::*;
use embassy_time::Duration;

use crate::interrupt;
use crate::interrupt::typelevel::Interrupt;
use crate::pac::EXTI;
use crate::rcc::low_power_ready;
use crate::time_driver::{get_driver, RtcDriver};

const THREAD_PENDER: usize = usize::MAX;

use crate::rtc::Rtc;

static mut EXECUTOR: Option<Executor> = None;

foreach_interrupt! {
    (RTC, rtc, $block:ident, WKUP, $irq:ident) => {
        #[interrupt]
        unsafe fn $irq() {
            unsafe { EXECUTOR.as_mut().unwrap() }.on_wakeup_irq();
        }
    };
}

// pub fn timer_driver_pause_time() {
//     pause_time();
// }

pub fn stop_with_rtc(rtc: &'static Rtc) {
    unsafe { EXECUTOR.as_mut().unwrap() }.stop_with_rtc(rtc)
}

// pub fn start_wakeup_alarm(requested_duration: embassy_time::Duration) {
//     let rtc_instant = unsafe { EXECUTOR.as_mut().unwrap() }
//         .rtc
//         .unwrap()
//         .start_wakeup_alarm(requested_duration);
//
//     unsafe { EXECUTOR.as_mut().unwrap() }.last_stop = Some(rtc_instant);
// }
//
// pub fn set_sleepdeep() {
//     unsafe { EXECUTOR.as_mut().unwrap() }.scb.set_sleepdeep();
// }
//
// pub fn stop_wakeup_alarm() -> RtcInstant {
//     unsafe { EXECUTOR.as_mut().unwrap() }.rtc.unwrap().stop_wakeup_alarm()
// }

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
    stop_time: embassy_time::Duration,
    next_alarm: embassy_time::Duration,
    wfe: u8,
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
                stop_time: Duration::from_ticks(0),
                next_alarm: Duration::from_ticks(u64::MAX),
                wfe: 0,
            });

            EXECUTOR.as_mut().unwrap()
        }
    }

    unsafe fn on_wakeup_irq(&mut self) {
        trace!("low power: on wakeup irq");

        if crate::pac::RTC.isr().read().wutf() {
            trace!("low power: wutf set");
        } else {
            trace!("low power: wutf not set");
        }

        self.time_driver.resume_time();
        trace!("low power: resume time");

        crate::interrupt::typelevel::RTC_WKUP::disable();

        // cortex_m::asm::bkpt();

        //        let time_elasped = self.rtc.unwrap().stop_wakeup_alarm() - self.last_stop.take().unwrap();
        //
        //        trace!("low power: {} ms elapsed", time_elasped.as_millis());
        //
        //        resume_time(time_elasped);
        //        trace!("low power: resume time");
        //
        //        self.scb.clear_sleepdeep();

        //       cortex_m::asm::bkpt();
        //        Self::get_scb().set_sleeponexit();
        //
        //        return;
        //
        //        let elapsed = RTC_INSTANT.take().unwrap() - stop_wakeup_alarm();
        //
        //        STOP_TIME += elapsed;
        //        // let to_next = NEXT_ALARM - STOP_TIME;
        //        let to_next = Duration::from_secs(3);
        //
        //        trace!("on wakeup irq: to next: {}", to_next);
        //        if to_next > THRESHOLD {
        //            trace!("start wakeup alarm");
        //            RTC_INSTANT.replace(start_wakeup_alarm(to_next));
        //
        //            trace!("set sleeponexit");
        //            Self::get_scb().set_sleeponexit();
        //        } else {
        //            Self::get_scb().clear_sleeponexit();
        //            Self::get_scb().clear_sleepdeep();
        //        }
    }

    pub(self) fn stop_with_rtc(&mut self, rtc: &'static Rtc) {
        trace!("low power: stop with rtc configured");

        self.time_driver.set_rtc(rtc);

        crate::interrupt::typelevel::RTC_WKUP::unpend();
        unsafe { crate::interrupt::typelevel::RTC_WKUP::enable() };

        EXTI.rtsr(0).modify(|w| w.set_line(22, true));
        EXTI.imr(0).modify(|w| w.set_line(22, true));
    }

    fn configure_pwr(&mut self) {
        trace!("low power: configure_pwr");

        self.scb.clear_sleepdeep();
        if !low_power_ready() {
            trace!("low power: configure_pwr: low power not ready");
            return;
        }

        if self.time_driver.pause_time().is_err() {
            trace!("low power: configure_pwr: time driver failed to pause");
            return;
        }

        trace!("low power: enter stop...");
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

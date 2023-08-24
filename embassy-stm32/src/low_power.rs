use core::arch::asm;
use core::marker::PhantomData;

use cortex_m::peripheral::SCB;
use embassy_executor::*;
use embassy_time::Duration;

use crate::interrupt;
use crate::interrupt::typelevel::Interrupt;
use crate::pac::EXTI;
use crate::rcc::low_power_ready;

const THREAD_PENDER: usize = usize::MAX;
const THRESHOLD: Duration = Duration::from_millis(500);

use crate::rtc::{Rtc, RtcInstant};

static mut RTC: Option<&'static Rtc> = None;

foreach_interrupt! {
    (RTC, rtc, $block:ident, WKUP, $irq:ident) => {
        #[interrupt]
        unsafe fn $irq() {
            Executor::on_wakeup_irq();
        }
    };
}

pub fn stop_with_rtc(rtc: &'static Rtc) {
    crate::interrupt::typelevel::RTC_WKUP::unpend();
    unsafe { crate::interrupt::typelevel::RTC_WKUP::enable() };

    EXTI.rtsr(0).modify(|w| w.set_line(22, true));
    EXTI.imr(0).modify(|w| w.set_line(22, true));

    unsafe { RTC = Some(rtc) };
}

pub fn start_wakeup_alarm(requested_duration: embassy_time::Duration) -> RtcInstant {
    unsafe { RTC }.unwrap().start_wakeup_alarm(requested_duration)
}

pub fn stop_wakeup_alarm() -> RtcInstant {
    unsafe { RTC }.unwrap().stop_wakeup_alarm()
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
        Self {
            inner: raw::Executor::new(THREAD_PENDER as *mut ()),
            not_send: PhantomData,
        }
    }

    unsafe fn on_wakeup_irq() {
        info!("on wakeup irq");

        cortex_m::asm::bkpt();
    }

    fn time_until_next_alarm(&self) -> Duration {
        Duration::from_secs(3)
    }

    fn get_scb() -> SCB {
        unsafe { cortex_m::Peripherals::steal() }.SCB
    }

    fn configure_pwr(&self) {
        trace!("configure_pwr");

        if !low_power_ready() {
            return;
        }

        let time_until_next_alarm = self.time_until_next_alarm();
        if time_until_next_alarm < THRESHOLD {
            return;
        }

        trace!("low power stop required");

        critical_section::with(|_| {
            trace!("executor: set wakeup alarm...");

            start_wakeup_alarm(time_until_next_alarm);

            trace!("low power wait for rtc ready...");

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
            };
        }
    }
}

use atomic_polyfill::{AtomicBool, Ordering};
use cortex_m;
use cortex_m::peripheral::NVIC;
use embassy_time::{Duration, TICK_HZ};

use crate::interrupt::typelevel::Interrupt;
use crate::pac::RTC;
use crate::rcc::{get_freqs, low_power_ready};
use crate::time_driver::{pause_time, resume_time, time_until_next_alarm};
use crate::{interrupt, rtc};

static STOPPED: AtomicBool = AtomicBool::new(false);
const THRESHOLD: Duration = Duration::from_millis(25);

foreach_interrupt! {
    (RTC, rtc, $block:ident, WKUP, $irq:ident) => {
        #[interrupt]
        fn $irq() {
            resume_time_irq_handler();
        }
    };
}

pub(crate) fn resume_time_irq_handler() {
    trace!("low power resume time");

    crate::interrupt::typelevel::RTC_WKUP::disable();

    if !STOPPED.load(Ordering::SeqCst) {
        return;
    }

    critical_section::with(|_| {
        let mut scb = unsafe { cortex_m::Peripherals::steal().SCB };

        scb.clear_sleepdeep();

        // TODO: load time from rtc timer, if enabled
        let offset = 0;
        resume_time(offset);

        STOPPED.store(false, Ordering::SeqCst);

        // TODO: synchronize the timer after a stop
    });
}

#[no_mangle]
fn _embassy_executor_arch_cortex_m_low_power_before_wfe() {
    trace!("low power before wfe");

    if !low_power_ready() {
        return;
    }

    let time_until_next_alarm = time_until_next_alarm();
    if time_until_next_alarm < THRESHOLD.as_ticks() {
        return;
    }

    trace!("low power stop required");

    let rtc_hz = unsafe { get_freqs() }.rtc.unwrap().0 as u64;
    let rtc_ticks = time_until_next_alarm * rtc_hz / TICK_HZ;
    let rtc_ticks = if rtc_ticks > u16::MAX as u64 {
        u16::MAX
    } else {
        rtc_ticks as u16
    };

    trace!("computed rtc ticks: {}", rtc_ticks);

    crate::interrupt::typelevel::RTC_WKUP::unpend();
    unsafe { crate::interrupt::typelevel::RTC_WKUP::enable() };

    critical_section::with(|_| {
        pause_time();

        STOPPED.store(true, Ordering::SeqCst);

        // Set the wake-up timer
        // RM0434 p919

        trace!("low power wait for rtc ready...");

        RTC.cr().modify(|w| w.set_wute(false));
        // while !RTC.isr().read().wutf() {}

        // TODO: configure the prescaler, if any

        trace!("low power configure rtc");

        RTC.wutr().modify(|w| w.set_wut(rtc_ticks));
        RTC.cr().modify(|w| w.set_wute(true));

        let mut scb = unsafe { cortex_m::Peripherals::steal().SCB };

        trace!("low power sleep");

        cortex_m::asm::bkpt();

        scb.set_sleepdeep();
    });
}

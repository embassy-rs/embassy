use atomic_polyfill::{AtomicBool, Ordering};
use embassy_time::Duration;

use crate::rcc::low_power_ready;
use crate::time_driver::{pause_time, resume_time, time_until_next_alarm};

static STOPPED: AtomicBool = AtomicBool::new(false);
const THRESHOLD: Duration = Duration::from_millis(25);

pub(crate) fn resume_time_irq_handler() {
    if !STOPPED.load(Ordering::SeqCst) {
        return;
    }

    critical_section::with(|_| {
        // TODO: load time from rtc timer, if enabled
        let offset = 0;
        resume_time(offset);

        STOPPED.store(false, Ordering::SeqCst);

        // TODO: synchronize the timer after a stop
    });
}

#[no_mangle]
fn _embassy_executor_arch_cortex_m_low_power_before_wfe() {
    if !low_power_ready() || time_until_next_alarm() < THRESHOLD.as_ticks() {
        return;
    }

    critical_section::with(|_| {
        pause_time();

        STOPPED.store(true, Ordering::SeqCst);
        // TODO: stop
    });
}

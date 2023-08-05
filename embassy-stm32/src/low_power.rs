use embassy_time::Duration;

use crate::rcc::low_power_ready;
use crate::time_driver::time_until_next_alarm;

const THRESHOLD: Duration = Duration::from_millis(25);

#[no_mangle]
fn _embassy_executor_arch_cortex_m_low_power_before_wfe() {
    if !low_power_ready() || time_until_next_alarm() < THRESHOLD.as_ticks() {
        return;
    }

    // TODO: stop
    // TODO: synchronize the timer after a stop
}

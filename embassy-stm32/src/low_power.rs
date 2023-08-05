use crate::rcc::low_power_ready;

#[no_mangle]
fn _embassy_executor_arch_cortex_m_low_power_before_wfe() {
    if !low_power_ready() {
        return;
    }

    // TODO: determine if the next wake is after the minimum stop time
}

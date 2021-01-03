pub mod peripheral;
pub mod ring_buffer;

/// Low power blocking wait loop using WFE/SEV.
pub fn low_power_wait_until(mut condition: impl FnMut() -> bool) {
    while !condition() {
        // WFE might "eat" an event that would have otherwise woken the executor.
        cortex_m::asm::wfe();
    }
    // Retrigger an event to be transparent to the executor.
    cortex_m::asm::sev();
}

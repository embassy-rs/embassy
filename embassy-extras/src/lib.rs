#![no_std]

#[macro_use]
extern crate embassy_fmt;

pub mod interrupt;
mod macros;
pub mod peripheral;
pub mod peripheral_shared;
pub mod ring_buffer;
pub mod usb;

/// Low power blocking wait loop using WFE/SEV.
pub fn low_power_wait_until(mut condition: impl FnMut() -> bool) {
    while !condition() {
        // WFE might "eat" an event that would have otherwise woken the executor.
        cortex_m::asm::wfe();
    }
    // Retrigger an event to be transparent to the executor.
    cortex_m::asm::sev();
}

#![no_std]
#![allow(clippy::missing_safety_doc)]

//! Shared board-specific helpers for the FRDM-MCXA276 examples.
//! These live with the examples so the HAL stays generic.

use hal::{clocks, pins};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

/// Initialize clocks and pin muxing for ADC.
pub unsafe fn init_adc_pins() {
    // NOTE: Lpuart has been updated to properly enable + reset its own clocks.
    // GPIO has not.
    _ = clocks::enable_and_reset::<hal::peripherals::PORT1>(&clocks::periph_helpers::NoConfig);
    pins::configure_adc_pins();
}

//! MCXN947 start-up.
//!
//! `init` verifies the reset-default main clock (see `crate::clocks`), enables
//! interrupts, and ungates PORT0-4/GPIO0-4 so the pins can be driven through
//! `crate::gpio`. The clock tree does not switch the main clock source away
//! from its reset default (`fro_hf`).

use crate::_generated::Peripherals;

/// Initialize the HAL and return the peripheral singletons.
///
/// Call once, before using any driver.
///
/// # Panics
///
/// Panics if called more than once, or if `main_clk` is not on its reset
/// source (see [`crate::clocks::ClockError::UnsupportedMainClockSource`]).
pub fn init(cfg: crate::config::Config) -> Peripherals {
    let peripherals = Peripherals::take();

    // Verify the core clocks and record their frequencies.
    crate::clocks::init(cfg.clock_cfg).unwrap();

    // Enable interrupts.
    unsafe {
        cortex_m::interrupt::enable();
    }

    // Enable PORT/GPIO clocks. GPIO5/PORT5 are always-on (see `clocks::gate`)
    // and need no explicit enable.
    unsafe {
        crate::clocks::enable_and_reset::<crate::peripherals::PORT0>();
        crate::clocks::enable_and_reset::<crate::peripherals::GPIO0>();

        crate::clocks::enable_and_reset::<crate::peripherals::PORT1>();
        crate::clocks::enable_and_reset::<crate::peripherals::GPIO1>();

        crate::clocks::enable_and_reset::<crate::peripherals::PORT2>();
        crate::clocks::enable_and_reset::<crate::peripherals::GPIO2>();

        crate::clocks::enable_and_reset::<crate::peripherals::PORT3>();
        crate::clocks::enable_and_reset::<crate::peripherals::GPIO3>();

        crate::clocks::enable_and_reset::<crate::peripherals::PORT4>();
        crate::clocks::enable_and_reset::<crate::peripherals::GPIO4>();
    }

    peripherals
}

#![no_std]
#![allow(clippy::missing_safety_doc)]

//! Shared board-specific helpers for the FRDM-MCXA276 examples.
//! These live with the examples so the HAL stays generic.

use hal::{clocks, pins};
use {embassy_mcxa as hal, panic_probe as _};

/// Initialize clocks and pin muxing for UART2 debug console.
/// Safe to call multiple times; writes are idempotent for our use.
pub unsafe fn init_uart2_pins(_p: &hal::pac::Peripherals) {
    // NOTE: Lpuart has been updated to properly enable + reset its own clocks.
    // GPIO has not.
    _ = clocks::enable_and_reset::<hal::peripherals::PORT2>(&clocks::NoConfig);
    pins::configure_uart2_pins_port2();
}

/// Initialize clocks for the LED GPIO/PORT used by the blink example.
pub unsafe fn init_led_gpio_clocks(_p: &hal::pac::Peripherals) {
    _ = clocks::enable_and_reset::<hal::peripherals::PORT3>(&clocks::NoConfig);
    _ = clocks::enable_and_reset::<hal::peripherals::GPIO3>(&clocks::NoConfig);
}

/// Initialize clocks and pin muxing for ADC.
pub unsafe fn init_adc_pins(_p: &hal::pac::Peripherals) {
    // NOTE: Lpuart has been updated to properly enable + reset its own clocks.
    // GPIO has not.
    _ = clocks::enable_and_reset::<hal::peripherals::PORT1>(&clocks::NoConfig);
    pins::configure_adc_pins();
}

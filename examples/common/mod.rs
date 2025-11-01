//! Shared board-specific helpers for the FRDM-MCXA276 examples.
//! These live with the examples so the HAL stays generic.

use embassy_mcxa276 as hal;
use hal::{clocks, pins, reset};

/// Initialize clocks and pin muxing for UART2 debug console.
/// Safe to call multiple times; writes are idempotent for our use.
#[allow(dead_code)]
pub unsafe fn init_uart2(p: &hal::pac::Peripherals) {
    clocks::ensure_frolf_running(p);
    clocks::enable_uart2_port2(p);
    reset::release_reset_port2(p);
    reset::release_reset_lpuart2(p);
    pins::configure_uart2_pins_port2();
    clocks::select_uart2_clock(p);
}

/// Initialize clocks for the LED GPIO/PORT used by the blink example.
#[allow(dead_code)]
pub unsafe fn init_led(p: &hal::pac::Peripherals) {
    clocks::enable_led_port(p);
    reset::release_reset_gpio3(p);
    reset::release_reset_port3(p);
}

/// Initialize clocks for OSTIMER0 (1 MHz source).
#[allow(dead_code)]
pub unsafe fn init_ostimer0(p: &hal::pac::Peripherals) {
    clocks::ensure_frolf_running(p);
    clocks::enable_ostimer0(p);
    reset::release_reset_ostimer0(p);
    clocks::select_ostimer0_clock_1m(p);
}

/// Initialize clocks and pin muxing for ADC.
#[allow(dead_code)]
pub unsafe fn init_adc(p: &hal::pac::Peripherals) {
    clocks::ensure_frolf_running(p);
    clocks::enable_adc(p);
    reset::release_reset_port1(p);
    reset::release_reset_adc1(p);
    pins::configure_adc_pins();
    clocks::select_adc_clock(p);
}

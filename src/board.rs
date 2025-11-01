use crate::{clocks, pac, pins};

/// Initialize clocks and pin muxing for UART2 debug console.
pub unsafe fn init_uart2(p: &pac::Peripherals) {
    clocks::ensure_frolf_running(p);
    clocks::enable_uart2_port2(p);
    pins::configure_uart2_pins_port2();
    clocks::select_uart2_clock(p);
}

/// Initialize clocks for the LED GPIO/PORT used by the blink example.
pub unsafe fn init_led(p: &pac::Peripherals) {
    clocks::enable_led_port(p);
}

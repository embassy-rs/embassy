//! MCXA1xx development support.

use crate::_generated::Peripherals;
use crate::interrupt::InterruptExt;

/// Initialize the MCXA156 development HAL on its documented 12 MHz reset clock.
pub fn init(cfg: crate::config::Config) -> Peripherals {
    let peripherals = Peripherals::take();

    crate::interrupt::GPIO0.set_priority(cfg.gpio_interrupt_priority);
    crate::interrupt::GPIO3.set_priority(cfg.gpio_interrupt_priority);

    crate::clocks::init(cfg.clock_cfg).unwrap();
    crate::init_time_driver(cfg.time_interrupt_priority);

    unsafe {
        cortex_m::interrupt::enable();

        _ = crate::clocks::enable_and_reset::<crate::peripherals::PORT0>(&crate::clocks::periph_helpers::NoConfig);
        _ = crate::clocks::enable_and_reset::<crate::peripherals::GPIO0>(&crate::clocks::periph_helpers::NoConfig);
        _ = crate::clocks::enable_and_reset::<crate::peripherals::PORT3>(&crate::clocks::periph_helpers::NoConfig);
        _ = crate::clocks::enable_and_reset::<crate::peripherals::GPIO3>(&crate::clocks::periph_helpers::NoConfig);
    }

    peripherals
}

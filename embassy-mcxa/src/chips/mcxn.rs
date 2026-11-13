//! Initial MCXN primary-core startup.

use crate::Peripherals;

/// Initialize the supported CPU0/FRO12/output/OSTIMER slice once.
pub fn init(config: crate::config::Config) -> Peripherals {
    critical_section::with(|_| {
        crate::clocks::init(config.clock_cfg).unwrap();
        let peripherals = Peripherals::take();
        crate::clocks::init_gpio_clocks().unwrap();
        crate::init_time_driver(config.time_interrupt_priority);
        peripherals
    })
}

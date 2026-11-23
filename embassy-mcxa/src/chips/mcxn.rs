//! Initial MCXN primary-core startup.

use crate::Peripherals;
use crate::interrupt::InterruptExt;

/// Initialize the supported CPU0/FRO12/output/OSTIMER slice once.
pub fn init(config: crate::config::Config) -> Peripherals {
    critical_section::with(|_| {
        crate::clocks::init(config.clock_cfg).unwrap();
        let peripherals = Peripherals::take();
        crate::clocks::init_gpio_clocks().unwrap();
        // The async GPIO driver routes every pin to the port's first interrupt line.
        for irq in [
            crate::interrupt::GPIO00,
            crate::interrupt::GPIO10,
            crate::interrupt::GPIO20,
            crate::interrupt::GPIO30,
            crate::interrupt::GPIO40,
            crate::interrupt::GPIO50,
        ] {
            irq.set_priority(config.gpio_interrupt_priority);
        }
        crate::init_time_driver(config.time_interrupt_priority);
        peripherals
    })
}

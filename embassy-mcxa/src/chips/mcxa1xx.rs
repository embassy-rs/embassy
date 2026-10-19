use crate::_generated::Peripherals;
use crate::interrupt::InterruptExt;

/// Initialize MCXA156 GPIO banks and the selected time driver, assuming the 12 MHz reset state.
pub fn init(cfg: crate::config::Config) -> Peripherals {
    // Mutable only when a dedicated-pin opt-in is selected.
    #[allow(unused_mut)]
    let mut peripherals = Peripherals::take();

    crate::interrupt::GPIO0.set_priority(cfg.gpio_interrupt_priority);
    crate::interrupt::GPIO1.set_priority(cfg.gpio_interrupt_priority);
    crate::interrupt::GPIO2.set_priority(cfg.gpio_interrupt_priority);
    crate::interrupt::GPIO3.set_priority(cfg.gpio_interrupt_priority);
    crate::interrupt::GPIO4.set_priority(cfg.gpio_interrupt_priority);

    crate::clocks::init(cfg.clock_cfg).unwrap();
    crate::init_time_driver(cfg.time_interrupt_priority);

    unsafe {
        let _ = crate::clocks::enable_and_reset::<crate::peripherals::PORT0>(&crate::clocks::periph_helpers::NoConfig)
            .unwrap();
        let _ = crate::clocks::enable_and_reset::<crate::peripherals::GPIO0>(&crate::clocks::periph_helpers::NoConfig)
            .unwrap();
        let _ = crate::clocks::enable_and_reset::<crate::peripherals::PORT1>(&crate::clocks::periph_helpers::NoConfig)
            .unwrap();
        let _ = crate::clocks::enable_and_reset::<crate::peripherals::GPIO1>(&crate::clocks::periph_helpers::NoConfig)
            .unwrap();
        let _ = crate::clocks::enable_and_reset::<crate::peripherals::PORT2>(&crate::clocks::periph_helpers::NoConfig)
            .unwrap();
        let _ = crate::clocks::enable_and_reset::<crate::peripherals::GPIO2>(&crate::clocks::periph_helpers::NoConfig)
            .unwrap();
        let _ = crate::clocks::enable_and_reset::<crate::peripherals::PORT3>(&crate::clocks::periph_helpers::NoConfig)
            .unwrap();
        let _ = crate::clocks::enable_and_reset::<crate::peripherals::GPIO3>(&crate::clocks::periph_helpers::NoConfig)
            .unwrap();
        let _ = crate::clocks::enable_and_reset::<crate::peripherals::PORT4>(&crate::clocks::periph_helpers::NoConfig)
            .unwrap();
        let _ = crate::clocks::enable_and_reset::<crate::peripherals::GPIO4>(&crate::clocks::periph_helpers::NoConfig)
            .unwrap();
    }

    #[allow(unused_imports)]
    use crate::gpio::SealedPin;

    #[cfg(feature = "swd-as-gpio")]
    {
        peripherals.P0_0.set_as_disabled();
        peripherals.P0_1.set_as_disabled();
    }
    #[cfg(feature = "swd-swo-as-gpio")]
    peripherals.P0_2.set_as_disabled();
    #[cfg(feature = "jtag-extras-as-gpio")]
    {
        peripherals.P0_3.set_as_disabled();
        // Package-dependent ISP inputs: reserve both in the package union.
        peripherals.P0_6.set_as_disabled();
        peripherals.P3_29.set_as_disabled();
    }
    #[cfg(feature = "dangerous-reset-as-gpio")]
    peripherals.P1_29.set_as_disabled();
    #[cfg(feature = "sosc-as-gpio")]
    {
        peripherals.P1_30.set_as_disabled();
        peripherals.P1_31.set_as_disabled();
    }

    // Enable interrupts after the bank and dedicated-pin setup is complete.
    unsafe { cortex_m::interrupt::enable() };

    peripherals
}

//! Module for MCXA2xx family

use crate::_generated::Peripherals;
use crate::interrupt::InterruptExt;

/// Initialize HAL with configuration (mirrors embassy-imxrt style). Minimal: just take peripherals.
/// Also applies configurable NVIC priority for the OSTIMER OS_EVENT interrupt (no enabling).
pub fn init(cfg: crate::config::Config) -> Peripherals {
    // Might not need to be mutable if none of the `...-as-gpio` features are active.
    #[allow(unused_mut)]
    let mut peripherals = Peripherals::take();

    crate::interrupt::RTC.set_priority(cfg.rtc_interrupt_priority);
    crate::interrupt::GPIO0.set_priority(cfg.gpio_interrupt_priority);
    crate::interrupt::GPIO1.set_priority(cfg.gpio_interrupt_priority);
    crate::interrupt::GPIO2.set_priority(cfg.gpio_interrupt_priority);
    crate::interrupt::GPIO3.set_priority(cfg.gpio_interrupt_priority);
    crate::interrupt::GPIO4.set_priority(cfg.gpio_interrupt_priority);

    // Configure clocks
    crate::clocks::init(cfg.clock_cfg).unwrap();

    // Initialize embassy-time global driver backed by OSTIMER0
    // NOTE: As early as possible, but MUST be AFTER clocks!
    crate::ostimer::init(cfg.time_interrupt_priority);

    // Initialize the INPUTMUX peripheral
    crate::inputmux::init();

    // Enable interrupts
    unsafe {
        cortex_m::interrupt::enable();
    }

    // Initialize DMA controller (clock, reset, configuration)
    crate::dma::init();

    // Enable GPIO clocks
    unsafe {
        _ = crate::clocks::enable_and_reset::<crate::peripherals::PORT0>(&crate::clocks::periph_helpers::NoConfig);
        _ = crate::clocks::enable_and_reset::<crate::peripherals::GPIO0>(&crate::clocks::periph_helpers::NoConfig);

        _ = crate::clocks::enable_and_reset::<crate::peripherals::PORT1>(&crate::clocks::periph_helpers::NoConfig);
        _ = crate::clocks::enable_and_reset::<crate::peripherals::GPIO1>(&crate::clocks::periph_helpers::NoConfig);

        _ = crate::clocks::enable_and_reset::<crate::peripherals::PORT2>(&crate::clocks::periph_helpers::NoConfig);
        _ = crate::clocks::enable_and_reset::<crate::peripherals::GPIO2>(&crate::clocks::periph_helpers::NoConfig);

        _ = crate::clocks::enable_and_reset::<crate::peripherals::PORT3>(&crate::clocks::periph_helpers::NoConfig);
        _ = crate::clocks::enable_and_reset::<crate::peripherals::GPIO3>(&crate::clocks::periph_helpers::NoConfig);

        _ = crate::clocks::enable_and_reset::<crate::peripherals::PORT4>(&crate::clocks::periph_helpers::NoConfig);
        _ = crate::clocks::enable_and_reset::<crate::peripherals::GPIO4>(&crate::clocks::periph_helpers::NoConfig);
    }

    // import may be unused if none of the following features are set
    #[allow(unused_imports)]
    use crate::gpio::SealedPin;

    // If we are not using SWD pins for SWD reasons, make them floating inputs
    #[cfg(feature = "swd-as-gpio")]
    {
        peripherals.P0_0.set_as_disabled();
        peripherals.P0_1.set_as_disabled();
    }
    #[cfg(feature = "swd-swo-as-gpio")]
    {
        peripherals.P0_2.set_as_disabled();
    }
    #[cfg(feature = "jtag-extras-as-gpio")]
    {
        peripherals.P0_3.set_as_disabled();
        peripherals.P0_6.set_as_disabled();
    }
    #[cfg(feature = "dangerous-reset-as-gpio")]
    {
        // DANGER DANGER DANGER
        peripherals.P1_29.set_as_disabled();
    }

    peripherals
}

/// This module contains implementations of MRCC APIs, specifically of the [`Gate`] trait,
/// for various low level peripherals.
pub(crate) mod peripheral_gating {
    use paste::paste;

    use crate::clocks::Gate;
    use crate::clocks::periph_helpers::{
        AdcConfig, CTimerConfig, Clk1MConfig, I3cConfig, Lpi2cConfig, LpspiConfig, LpuartConfig, NoConfig,
        OsTimerConfig,
    };
    use crate::{impl_cc_gate, pac};

    // These peripherals have no additional upstream clocks or configuration required
    // other than enabling through the MRCC gate. Currently, these peripherals will
    // ALWAYS return `Ok(0)` when calling [`enable_and_reset()`] and/or
    // [`SPConfHelper::post_enable_config()`].
    impl_cc_gate!(PORT0, mrcc_glb_cc1, mrcc_glb_rst1, port0, NoConfig);
    impl_cc_gate!(PORT1, mrcc_glb_cc1, mrcc_glb_rst1, port1, NoConfig);
    impl_cc_gate!(PORT2, mrcc_glb_cc1, mrcc_glb_rst1, port2, NoConfig);
    impl_cc_gate!(PORT3, mrcc_glb_cc1, mrcc_glb_rst1, port3, NoConfig);
    impl_cc_gate!(PORT4, mrcc_glb_cc1, mrcc_glb_rst1, port4, NoConfig);

    impl_cc_gate!(CRC0, mrcc_glb_cc0, mrcc_glb_rst0, crc0, NoConfig);
    impl_cc_gate!(INPUTMUX0, mrcc_glb_cc0, mrcc_glb_rst0, inputmux0, NoConfig);

    // These peripherals DO have meaningful configuration, and could fail if the system
    // clocks do not match their needs.
    impl_cc_gate!(ADC0, mrcc_glb_cc1, mrcc_glb_rst1, adc0, AdcConfig);
    impl_cc_gate!(ADC1, mrcc_glb_cc1, mrcc_glb_rst1, adc1, AdcConfig);

    impl_cc_gate!(I3C0, mrcc_glb_cc0, mrcc_glb_rst0, i3c0, I3cConfig);
    impl_cc_gate!(CTIMER0, mrcc_glb_cc0, mrcc_glb_rst0, ctimer0, CTimerConfig);
    impl_cc_gate!(CTIMER1, mrcc_glb_cc0, mrcc_glb_rst0, ctimer1, CTimerConfig);
    impl_cc_gate!(CTIMER2, mrcc_glb_cc0, mrcc_glb_rst0, ctimer2, CTimerConfig);
    impl_cc_gate!(CTIMER3, mrcc_glb_cc0, mrcc_glb_rst0, ctimer3, CTimerConfig);
    impl_cc_gate!(CTIMER4, mrcc_glb_cc0, mrcc_glb_rst0, ctimer4, CTimerConfig);
    impl_cc_gate!(OSTIMER0, mrcc_glb_cc1, mrcc_glb_rst1, ostimer0, OsTimerConfig);

    // TRNG peripheral - uses NoConfig since it has no selectable clock source
    impl_cc_gate!(TRNG0, mrcc_glb_cc1, mrcc_glb_rst1, trng0, NoConfig);

    // Peripherals that use ACC instead of CC!
    impl_cc_gate!(LPUART0, mrcc_glb_acc0, mrcc_glb_rst0, lpuart0, LpuartConfig);
    impl_cc_gate!(LPUART1, mrcc_glb_acc0, mrcc_glb_rst0, lpuart1, LpuartConfig);
    impl_cc_gate!(LPUART2, mrcc_glb_acc0, mrcc_glb_rst0, lpuart2, LpuartConfig);
    impl_cc_gate!(LPUART3, mrcc_glb_acc0, mrcc_glb_rst0, lpuart3, LpuartConfig);
    impl_cc_gate!(LPUART4, mrcc_glb_acc0, mrcc_glb_rst0, lpuart4, LpuartConfig);
    impl_cc_gate!(LPUART5, mrcc_glb_acc1, mrcc_glb_rst1, lpuart5, LpuartConfig);

    // DMA0 peripheral - uses NoConfig since it has no selectable clock source
    impl_cc_gate!(DMA0, mrcc_glb_acc0, mrcc_glb_rst0, dma0, NoConfig);

    impl_cc_gate!(GPIO0, mrcc_glb_acc2, mrcc_glb_rst2, gpio0, NoConfig);
    impl_cc_gate!(GPIO1, mrcc_glb_acc2, mrcc_glb_rst2, gpio1, NoConfig);
    impl_cc_gate!(GPIO2, mrcc_glb_acc2, mrcc_glb_rst2, gpio2, NoConfig);
    impl_cc_gate!(GPIO3, mrcc_glb_acc2, mrcc_glb_rst2, gpio3, NoConfig);
    impl_cc_gate!(GPIO4, mrcc_glb_acc2, mrcc_glb_rst2, gpio4, NoConfig);

    impl_cc_gate!(LPI2C0, mrcc_glb_acc0, mrcc_glb_rst0, lpi2c0, Lpi2cConfig);
    impl_cc_gate!(LPI2C1, mrcc_glb_acc0, mrcc_glb_rst0, lpi2c1, Lpi2cConfig);
    impl_cc_gate!(LPI2C2, mrcc_glb_acc1, mrcc_glb_rst1, lpi2c2, Lpi2cConfig);
    impl_cc_gate!(LPI2C3, mrcc_glb_acc1, mrcc_glb_rst1, lpi2c3, Lpi2cConfig);

    impl_cc_gate!(LPSPI0, mrcc_glb_acc0, mrcc_glb_rst0, lpspi0, LpspiConfig);
    impl_cc_gate!(LPSPI1, mrcc_glb_acc0, mrcc_glb_rst0, lpspi1, LpspiConfig);

    impl_cc_gate!(WWDT0, mrcc_glb_acc0, wwdt0, Clk1MConfig);
}

pub(crate) mod clock_limits {
    use crate::chips::ClockLimits;

    // TODO: Different for different CPUs?
    pub const VDD_CORE_MID_DRIVE_WAIT_STATE_LIMITS: &[(u32, u8)] = &[(22_500_000, 0b0000)];
    pub const VDD_CORE_MID_DRIVE_MAX_WAIT_STATES: u8 = 0b0001;

    pub const VDD_CORE_OVER_DRIVE_WAIT_STATE_LIMITS: &[(u32, u8)] = &[
        (40_000_000, 0b0000),
        (80_000_000, 0b0001),
        (120_000_000, 0b0010),
        (160_000_000, 0b0011),
    ];
    pub const VDD_CORE_OVER_DRIVE_MAX_WAIT_STATES: u8 = 0b0100;

    impl ClockLimits {
        pub const MID_DRIVE: Self = Self {
            fro_hf: 90_000_000,
            fro_hf_div: 45_000_000,
            pll1_clk: 48_000_000,
            main_clk: 90_000_000,
            cpu_clk: 45_000_000,
            pll1_clk_div: 48_000_000,
            // clk_16k: 16_384,
            // clk_in: 50_000_000,
            // clk_48m: 48_000_000,
            // fro_12m: 24_000_000, // what?
            // fro_12m_div: 24_000_000, // what?
            // clk_1m: 1_000_000,
            // system_clk: 45_000_000,
            // bus_clk: 22_500_000,
            // slow_clk: 7_500_000,
        };

        pub const OVER_DRIVE: Self = Self {
            fro_hf: 180_000_000,
            fro_hf_div: 180_000_000,
            pll1_clk: 240_000_000,
            main_clk: 180_000_000,
            cpu_clk: 180_000_000,
            pll1_clk_div: 240_000_000,
            // clk_16k: 16_384,
            // clk_in: 50_000_000,
            // clk_48m: 48_000_000,
            // fro_12m: 24_000_000, // what?
            // fro_12m_div: 24_000_000, // what?
            // clk_1m: 1_000_000,
            // system_clk: 180_000_000,
            // bus_clk: 90_000_000,
            // slow_clk: 36_000_000,
        };
    }
}

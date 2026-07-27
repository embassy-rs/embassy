//! Module for MCXA5xx family

use crate::_generated::Peripherals;
use crate::interrupt::InterruptExt;

/// Initialize HAL with configuration (mirrors embassy-imxrt style). Minimal: just take peripherals.
/// Also applies configurable NVIC priority for the OSTIMER OS_EVENT interrupt (no enabling).
pub fn init(cfg: crate::config::Config) -> Peripherals {
    // Might not need to be mutable if none of the `...-as-gpio` features are active.
    #[allow(unused_mut)]
    let mut peripherals = Peripherals::take();

    // crate::interrupt::RTC.set_priority(cfg.rtc_interrupt_priority);
    crate::interrupt::GPIO0.set_priority(cfg.gpio_interrupt_priority);
    crate::interrupt::GPIO1.set_priority(cfg.gpio_interrupt_priority);
    crate::interrupt::GPIO2.set_priority(cfg.gpio_interrupt_priority);
    crate::interrupt::GPIO3.set_priority(cfg.gpio_interrupt_priority);
    crate::interrupt::GPIO4.set_priority(cfg.gpio_interrupt_priority);
    crate::interrupt::GPIO5.set_priority(cfg.gpio_interrupt_priority);

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

        _ = crate::clocks::enable_and_reset::<crate::peripherals::PORT5>(&crate::clocks::periph_helpers::NoConfig);
        _ = crate::clocks::enable_and_reset::<crate::peripherals::GPIO5>(&crate::clocks::periph_helpers::NoConfig);
    }

    // import may be unused if none of the following features are set
    #[allow(unused_imports)]
    use crate::gpio::SealedPin;

    // If we are not using pins for specialized purposes, set them as disabled
    #[cfg(feature = "rosc-32k-as-gpio")]
    {
        peripherals.P5_0.set_as_disabled();
        peripherals.P5_1.set_as_disabled();
    }
    #[cfg(feature = "sosc-as-gpio")]
    {
        peripherals.P1_30.set_as_disabled();
        peripherals.P1_31.set_as_disabled();
    }
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

pub(crate) mod clock_limits {
    #![allow(dead_code)]

    use crate::chips::ClockLimits;

    pub const VDD_CORE_MID_DRIVE_WAIT_STATE_LIMITS: &[(u32, u8)] = &[(24_000_000, 0b0000)];
    // <= 48MHz
    pub const VDD_CORE_MID_DRIVE_MAX_WAIT_STATES: u8 = 0b0001;

    pub const VDD_CORE_NORMAL_DRIVE_WAIT_STATE_LIMITS: &[(u32, u8)] =
        &[(30_000_000, 0b0000), (60_000_000, 0b0001), (90_000_000, 0b0010)];
    // <= 120MHz
    pub const VDD_CORE_NORMAL_DRIVE_MAX_WAIT_STATES: u8 = 0b0011;

    pub const VDD_CORE_OVER_DRIVE_WAIT_STATE_LIMITS: &[(u32, u8)] = &[
        (40_000_000, 0b0000),
        (80_000_000, 0b0001),
        (120_000_000, 0b0010),
        (160_000_000, 0b0011),
        (200_000_000, 0b0100),
    ];
    // <= 250MHz
    pub const VDD_CORE_OVER_DRIVE_MAX_WAIT_STATES: u8 = 0b0101;

    impl ClockLimits {
        pub const MID_DRIVE: Self = Self {
            fro_hf: 96_000_000,
            fro_hf_div: 48_000_000,
            pll1_clk: 100_000_000,
            pll1_clk_div: 100_000_000,
            main_clk: 96_000_000,
            cpu_clk: 48_000_000,
            // clk_16k: 16_384,
            // clk_in: 50_000_000,
            // clk_48m: 48_000_000,
            // fro_12m: 12_000_000,
            // fro_12m_div: 12_000_000,
            // clk_1m: 1_000_000,
            // system_clk: cpu_clk,
            // bus_clk: cpu_clk / 2,
            // slow_clk: cpu_clk / 6,
        };

        pub const NORMAL_DRIVE: Self = Self {
            fro_hf: 192_000_000,
            fro_hf_div: 192_000_000,
            pll1_clk: 300_000_000,
            pll1_clk_div: 150_000_000,
            main_clk: 120_000_000,
            cpu_clk: 120_000_000,
            // clk_16k: 16_384,
            // clk_in: 50_000_000,
            // clk_48m: 48_000_000,
            // fro_12m: 12_000_000,
            // fro_12m_div: 12_000_000,
            // clk_1m: 1_000_000,
            // system_clk: cpu_clk,
            // bus_clk: cpu_clk / 2,
            // slow_clk: cpu_clk / 6,
        };

        pub const OVER_DRIVE: Self = Self {
            fro_hf: 192_000_000,
            fro_hf_div: 192_000_000,
            pll1_clk: 400_000_000,
            pll1_clk_div: 200_000_000,
            main_clk: 240_000_000,
            cpu_clk: 240_000_000,
            // clk_16k: 16_384,
            // clk_in: 50_000_000,
            // clk_48m: 48_000_000,
            // fro_12m: 12_000_000,
            // fro_12m_div: 12_000_000,
            // clk_1m: 1_000_000,
            // system_clk: cpu_clk,
            // bus_clk: cpu_clk / 2,
            // slow_clk: cpu_clk / 6,
        };
    }
}

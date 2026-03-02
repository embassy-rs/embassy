// From Table 165 - Max Clock Frequencies
#[allow(dead_code)]
pub(crate) struct ClockLimits {
    pub(crate) fro_hf: u32,
    pub(crate) fro_hf_div: u32,
    pub(crate) pll1_clk: u32,
    pub(crate) main_clk: u32,
    pub(crate) cpu_clk: u32,
    // The following items are LISTED in Table 165, but are not necessary
    // to check at runtime either because they are physically fixed, the
    // HAL exposes no way for them to exceed their limits, or they cannot
    // exceed their limits due to some upstream clock enforcement. They
    // are included here as documentation.
    //
    // clk_16k: u32,        // fixed (16.384kHz), no need to check
    // clk_in: u32,         // Checked already in configure_sosc method, 50MHz in all modes
    // clk_48m: u32,        // clk_48m is fixed (to 45mhz actually)
    // fro_12m: u32,        // We don't allow modifying from 12mhz
    // fro_12m_div: u32,    // div can never exceed 12mhz
    // pll1_clk_div: u32,   // if pll1_clk is in range, so is pll1_clk_div
    // clk_1m: u32,         // fro_12m / 12 can never exceed 12mhz
    // system_clk: u32,     // cpu_clk == system_clk
    // bus_clk: u32,        // bus_clk == (cpu_clk / 2), if cpu_clk is good so is bus_clk
    // slow_clk: u32,       // slow_clk == (cpu_clk / 6), if cpu_clk is good so is slow_clock
}

#[cfg(feature = "mcxa2xx")]
pub(crate) use mcxa2xx::*;
#[allow(unused_imports)]
#[cfg(feature = "mcxa5xx")]
pub(crate) use mcxa5xx::*;

#[cfg(feature = "mcxa2xx")]
mod mcxa2xx {
    use super::ClockLimits;

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
            // clk_16k: 16_384,
            // clk_in: 50_000_000,
            // clk_48m: 48_000_000,
            // fro_12m: 24_000_000, // what?
            // fro_12m_div: 24_000_000, // what?
            // pll1_clk_div: 48_000_000,
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
            // clk_16k: 16_384,
            // clk_in: 50_000_000,
            // clk_48m: 48_000_000,
            // fro_12m: 24_000_000, // what?
            // fro_12m_div: 24_000_000, // what?
            // pll1_clk_div: 240_000_000,
            // clk_1m: 1_000_000,
            // system_clk: 180_000_000,
            // bus_clk: 90_000_000,
            // slow_clk: 36_000_000,
        };
    }
}

#[cfg(feature = "mcxa5xx")]
mod mcxa5xx {
    #![allow(dead_code)]
    // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    // !UNVERIFIED! JUST COPY AND PASTED!
    // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

    use super::ClockLimits;

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
            // clk_16k: 16_384,
            // clk_in: 50_000_000,
            // clk_48m: 48_000_000,
            // fro_12m: 24_000_000, // what?
            // fro_12m_div: 24_000_000, // what?
            // pll1_clk_div: 48_000_000,
            // clk_1m: 1_000_000,
            // system_clk: 45_000_000,
            // bus_clk: 22_500_000,
            // slow_clk: 7_500_000,
        };

        // TODO: add standard drive

        pub const OVER_DRIVE: Self = Self {
            fro_hf: 180_000_000,
            fro_hf_div: 180_000_000,
            pll1_clk: 240_000_000,
            main_clk: 180_000_000,
            cpu_clk: 180_000_000,
            // clk_16k: 16_384,
            // clk_in: 50_000_000,
            // clk_48m: 48_000_000,
            // fro_12m: 24_000_000, // what?
            // fro_12m_div: 24_000_000, // what?
            // pll1_clk_div: 240_000_000,
            // clk_1m: 1_000_000,
            // system_clk: 180_000_000,
            // bus_clk: 90_000_000,
            // slow_clk: 36_000_000,
        };
    }
}

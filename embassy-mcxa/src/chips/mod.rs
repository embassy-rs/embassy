#[cfg(feature = "mcxa2xx")]
pub mod mcxa2xx;

#[cfg(feature = "mcxa5xx")]
pub mod mcxa5xx;

#[cfg(feature = "mcxa2xx")]
pub(crate) use mcxa2xx::clock_limits;
#[cfg(feature = "mcxa5xx")]
pub(crate) use mcxa5xx::clock_limits;

// From Table 165 - Max Clock Frequencies (mcxa2xx)
// From Table 375 - Max. Clock Frequency (mcxa5xx)
#[allow(dead_code)]
pub(crate) struct ClockLimits {
    pub(crate) fro_hf: u32,
    pub(crate) fro_hf_div: u32,
    pub(crate) pll1_clk: u32,
    pub(crate) main_clk: u32,
    pub(crate) cpu_clk: u32,
    pub(crate) pll1_clk_div: u32,
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
    // clk_1m: u32,         // fro_12m / 12 can never exceed 12mhz
    // system_clk: u32,     // cpu_clk == system_clk
    // bus_clk: u32,        // bus_clk == (cpu_clk / 2), if cpu_clk is good so is bus_clk
    // slow_clk: u32,       // slow_clk == (cpu_clk / 6), if cpu_clk is good so is slow_clock
}

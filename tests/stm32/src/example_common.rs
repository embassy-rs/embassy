#![macro_use]

pub use defmt::*;
#[allow(unused)]
use embassy_stm32::time::Hertz;
use embassy_stm32::Config;
use {defmt_rtt as _, panic_probe as _};

pub fn config() -> Config {
    #[allow(unused_mut)]
    let mut config = Config::default();

    #[cfg(feature = "stm32h755zi")]
    {
        config.rcc.sys_ck = Some(Hertz(400_000_000));
        config.rcc.pll1.q_ck = Some(Hertz(100_000_000));
    }

    #[cfg(feature = "stm32u585ai")]
    {
        config.rcc.mux = embassy_stm32::rcc::ClockSrc::MSI(embassy_stm32::rcc::MSIRange::Range48mhz);
    }

    config
}

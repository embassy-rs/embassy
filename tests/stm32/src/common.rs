#![macro_use]

pub use defmt::*;
#[allow(unused)]
use embassy_stm32::time::Hertz;
use embassy_stm32::Config;
use {defmt_rtt as _, panic_probe as _};

#[cfg(feature = "stm32f103c8")]
teleprobe_meta::target!(b"bluepill-stm32f103c8");
#[cfg(feature = "stm32g491re")]
teleprobe_meta::target!(b"nucleo-stm32g491re");
#[cfg(feature = "stm32g071rb")]
teleprobe_meta::target!(b"nucleo-stm32g071rb");
#[cfg(feature = "stm32f429zi")]
teleprobe_meta::target!(b"nucleo-stm32f429zi");
#[cfg(feature = "stm32wb55rg")]
teleprobe_meta::target!(b"nucleo-stm32wb55rg");
#[cfg(feature = "stm32h755zi")]
teleprobe_meta::target!(b"nucleo-stm32h755zi");
#[cfg(feature = "stm32u585ai")]
teleprobe_meta::target!(b"iot-stm32u585ai");
#[cfg(feature = "stm32h563zi")]
teleprobe_meta::target!(b"nucleo-stm32h563zi");
#[cfg(feature = "stm32c031c6")]
teleprobe_meta::target!(b"nucleo-stm32c031c6");

pub fn config() -> Config {
    #[allow(unused_mut)]
    let mut config = Config::default();

    #[cfg(feature = "stm32h755zi")]
    {
        config.rcc.sys_ck = Some(Hertz(400_000_000));
        config.rcc.pll1.q_ck = Some(Hertz(100_000_000));
        config.rcc.adc_clock_source = embassy_stm32::rcc::AdcClockSource::PerCk;
    }

    #[cfg(feature = "stm32u585ai")]
    {
        config.rcc.mux = embassy_stm32::rcc::ClockSrc::MSI(embassy_stm32::rcc::MSIRange::Range48mhz);
    }

    config
}

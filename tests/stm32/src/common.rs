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
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(Hsi::Mhz64);
        config.rcc.csi = true;
        config.rcc.pll_src = PllSource::Hsi;
        config.rcc.pll1 = Some(Pll {
            prediv: 4,
            mul: 50,
            divp: Some(2),
            divq: Some(8), // SPI1 cksel defaults to pll1_q
            divr: None,
        });
        config.rcc.pll2 = Some(Pll {
            prediv: 4,
            mul: 50,
            divp: Some(8), // 100mhz
            divq: None,
            divr: None,
        });
        config.rcc.sys = Sysclk::Pll1P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
        config.rcc.adc_clock_source = AdcClockSource::PLL2_P;
    }

    #[cfg(feature = "stm32u585ai")]
    {
        config.rcc.mux = embassy_stm32::rcc::ClockSrc::MSI(embassy_stm32::rcc::MSIRange::Range48mhz);
    }

    config
}

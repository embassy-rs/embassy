#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::rcc::*;
use embassy_stm32::rtc::{DateTime, DayOfWeek, Rtc, RtcConfig};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

pub fn pll_init(config: &mut Config) {
    config.rcc.pll1 = Some(embassy_stm32::rcc::Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV1,  // PLLM = 1 → HSI / 1 = 16 MHz
        mul: PllMul::MUL30,       // PLLN = 30 → 16 MHz * 30 = 480 MHz VCO
        divr: Some(PllDiv::DIV5), // PLLR = 5 → 96 MHz (Sysclk)
        // divq: Some(PllDiv::DIV10), // PLLQ = 10 → 48 MHz (NOT USED)
        divq: None,
        divp: Some(PllDiv::DIV30), // PLLP = 30 → 16 MHz (USBOTG)
        frac: Some(0),             // Fractional part (enabled)
    });

    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV1;
    config.rcc.apb2_pre = APBPrescaler::DIV1;
    config.rcc.apb7_pre = APBPrescaler::DIV1;
    config.rcc.ahb5_pre = AHB5Prescaler::DIV4;

    // voltage scale for max performance
    config.rcc.voltage_scale = VoltageScale::RANGE1;
    // route PLL1_P into the USB‐OTG‐HS block
    config.rcc.sys = Sysclk::PLL1_R;
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();

    pll_init(&mut config);

    let p = embassy_stm32::init(config);

    let mut rtc = Rtc::new(p.RTC, RtcConfig::default());

    // Setting datetime
    let initial_datetime = DateTime::from(1970, 1, 1, DayOfWeek::Thursday, 0, 00, 00, 0).unwrap();
    match rtc.0.set_datetime(initial_datetime) {
        Ok(()) => info!("RTC set successfully."),
        Err(e) => error!("Failed to set RTC date/time: {:?}", e),
    }

    // Reading datetime every 1s
    loop {
        match rtc.1.now() {
            Ok(result) => info!("{}", result),
            Err(e) => error!("Failed to set RTC date/time: {:?}", e),
        }

        Timer::after_millis(1000).await;
    }
}

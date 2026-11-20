#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::rcc::*;
use embassy_stm32::rtc::{RawDateTime, Rtc, RtcConfig};
use embassy_time::Timer;
use panic_probe as _;

pub fn pll_init(config: &mut Config) {
    config.rcc.pll1 = Some(embassy_stm32::rcc::Pll {
        source: PllSource::Hsi,
        prediv: PllPreDiv::Div1,  // PLLM = 1 → HSI / 1 = 16 MHz
        mul: PllMul::Mul30,       // PLLN = 30 → 16 MHz * 30 = 480 MHz VCO
        divr: Some(PllDiv::Div5), // PLLR = 5 → 96 MHz (Sysclk)
        // divq: Some(PllDiv::Div10), // PLLQ = 10 → 48 MHz (NOT USED)
        divq: None,
        divp: Some(PllDiv::Div30), // PLLP = 30 → 16 MHz (USBOTG)
        frac: Some(0),             // Fractional part (enabled)
    });

    config.rcc.ahb_pre = AHBPrescaler::Div1;
    config.rcc.apb1_pre = APBPrescaler::Div1;
    config.rcc.apb2_pre = APBPrescaler::Div1;
    config.rcc.apb7_pre = APBPrescaler::Div1;
    config.rcc.ahb5_pre = AHB5Prescaler::Div4;

    // voltage scale for max performance
    config.rcc.voltage_scale = VoltageScale::Range1;
    // route PLL1_P into the USB‐OTG‐HS block
    config.rcc.sys = Sysclk::Pll1R;
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();

    pll_init(&mut config);

    let p = embassy_stm32::init(config);

    let mut rtc = Rtc::new(p.RTC, RtcConfig::default());

    // Setting datetime
    // The raw API does no validation: these values are written to the RTC registers as is.
    let initial_datetime = RawDateTime {
        year: 0, // 2000
        month: 1,
        day: 1,
        day_of_week: 6, // Saturday
        hour: 0,
        minute: 0,
        second: 0,
        subsecond: 0,
    };
    rtc.set_raw(initial_datetime);

    // Reading datetime every 1s
    loop {
        info!("{}", rtc.read_raw());

        Timer::after_millis(1000).await;
    }
}

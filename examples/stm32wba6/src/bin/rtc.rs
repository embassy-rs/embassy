#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::rcc::*;
use embassy_stm32::rtc::DayOfWeek;
use embassy_stm32::rtc::{DateTime, Rtc, RtcApi, RtcConfig, RtcError, RtcInstance, RtcTimeProvider};
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

// STM32-specific wrapper that implements RtcInstance
pub struct Stm32Rtc {
    rtc_tuple: (Rtc, RtcTimeProvider),
}

impl Stm32Rtc {
    pub fn new(rtc_tuple: (Rtc, RtcTimeProvider)) -> Self {
        Self { rtc_tuple }
    }
}

impl RtcInstance for Stm32Rtc {
    fn set_date_time(&mut self, new_date_time: DateTime) -> Result<(), RtcError> {
        self.rtc_tuple.0.set_datetime(new_date_time)
    }

    fn get_date_time(&mut self) -> Result<DateTime, RtcError> {
        self.rtc_tuple.1.now()
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();

    pll_init(&mut config);

    let p = embassy_stm32::init(config);

    let rtc_tuple = Rtc::new(p.RTC, RtcConfig::default());

    let stm32_rtc = Stm32Rtc::new(rtc_tuple);
    let mut my_rtc = RtcApi::new(stm32_rtc);

    // Setting datetime using API format
    let initial_datetime = DateTime::from(2022, 12, 18, DayOfWeek::Sunday, 0, 0, 0, 0).unwrap();

    match my_rtc.set_date_time(initial_datetime) {
        Ok(()) => info!("RTC set successfully."),
        Err(e) => error!("Failed to set RTC date/time: {:?}", e),
    }

    // Reading datetime every 1s
    loop {
        match my_rtc.get_date_time() {
            Ok(result) => info!(
                "Date: {} {}/{}/{} Time: {}:{}:{}",
                result.day_of_week(),
                result.year(),
                result.month(),
                result.day(),
                result.hour(),
                result.minute(),
                result.second(),
            ),
            Err(e) => error!("Failed to get RTC date/time: {:?}", e),
        }
        Timer::after_millis(1000).await;
    }
}

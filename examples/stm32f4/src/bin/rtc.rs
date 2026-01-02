#![no_std]
#![no_main]

use chrono::{NaiveDate, NaiveDateTime};
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
// use embassy_stm32::rtc::{DateTime, DayOfWeek, Rtc, RtcApi, RtcConfig, RtcError, RtcInstance, RtcTimeProvider};
use embassy_stm32::rtc::{DateTime, Rtc, RtcApi, RtcConfig, RtcError, RtcInstance, RtcTimeProvider};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

// STM32-specific wrapper that implements RtcInstance
pub struct Stm32Rtc {
    rtc_tuple: (Rtc, RtcTimeProvider),
    // time_provider: RtcTimeProvider,
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
    let config = Config::default();
    let p = embassy_stm32::init(config);

    info!("Hello World!");

    let now = NaiveDate::from_ymd_opt(2020, 5, 15)
        .unwrap()
        .and_hms_opt(10, 30, 15)
        .unwrap();

    let rtc = Rtc::new(p.RTC, RtcConfig::default());
    let rtc_instance = Stm32Rtc::new(rtc);
    let mut rtc_api = RtcApi::new(rtc_instance);
    //

    // rtc.set_datetime(now.into()).expect("datetime not set");
    rtc_api.set_date_time(now.into());

    loop {
        match rtc_api.get_date_time() {
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

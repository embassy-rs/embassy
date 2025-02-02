#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::rtc::{Alarm, DateTime, DayOfWeek, Rtc, RtcAlarmMatch, RtcConfig};
use embassy_stm32::Config;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();
    let p = embassy_stm32::init(config);

    info!("Hello World!");

    let now = DateTime::from(2023, 6, 14, DayOfWeek::Friday, 15, 59, 10);

    let mut rtc = Rtc::new(p.RTC, RtcConfig::default());

    rtc.set_datetime(now.unwrap()).expect("datetime not set");

    rtc.set_alarm(
        Alarm::A,
        RtcAlarmMatch {
            subsecond: None,
            second: None,
            minute: None,
            hour: None,
            hour_is_pm: false,
            date: None,
        },
    );

    loop {
        let now: DateTime = rtc.now().unwrap().into();

        info!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second());

        rtc.wait_for_alarm(Alarm::A).await;
    }
}

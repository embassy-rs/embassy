//! This example shows how to use RTC (Real Time Clock) for scheduling alarms and reacting to them.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_rp::bind_interrupts;
use embassy_rp::rtc::{DateTime, DateTimeFilter, DayOfWeek, Rtc};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

// Bind the RTC interrupt to the handler
bind_interrupts!(struct Irqs {
    RTC_IRQ => embassy_rp::rtc::InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut rtc = Rtc::new(p.RTC, Irqs);

    if !rtc.is_running() {
        info!("Start RTC");
        let now = DateTime {
            year: 2000,
            month: 1,
            day: 1,
            day_of_week: DayOfWeek::Saturday,
            hour: 0,
            minute: 0,
            second: 0,
        };
        rtc.set_datetime(now).unwrap();
    }

    loop {
        // Wait for 5 seconds or until the alarm is triggered
        match select(Timer::after_secs(5), rtc.wait_for_alarm()).await {
            // Timer expired
            Either::First(_) => {
                let dt = rtc.now().unwrap();
                info!(
                    "Now: {}-{:02}-{:02} {}:{:02}:{:02}",
                    dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second,
                );

                // See if the alarm is already scheduled, if not, schedule it
                if rtc.alarm_scheduled().is_none() {
                    info!("Scheduling alarm for 30 seconds from now");
                    rtc.schedule_alarm(DateTimeFilter::default().second((dt.second + 30) % 60));
                    info!("Alarm scheduled: {}", rtc.alarm_scheduled().unwrap());
                }
            }
            // Alarm triggered
            Either::Second(_) => {
                let dt = rtc.now().unwrap();
                info!(
                    "ALARM TRIGGERED! Now: {}-{:02}-{:02} {}:{:02}:{:02}",
                    dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second,
                );
            }
        }
    }
}

#![no_std]
#![no_main]
#[cfg(feature = "rp2040")]
teleprobe_meta::target!(b"rpi-pico");

use defmt::{assert, *};
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_rp::bind_interrupts;
use embassy_rp::rtc::{DateTime, DateTimeFilter, DayOfWeek, Rtc};
use embassy_time::{Duration, Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

// Bind the RTC interrupt to the handler
bind_interrupts!(struct Irqs {
    RTC_IRQ => embassy_rp::rtc::InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut rtc = Rtc::new(p.RTC, Irqs);

    info!("RTC test started");

    // Initialize RTC if not running
    if !rtc.is_running() {
        info!("Starting RTC");
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
        Timer::after_millis(100).await;
    }

    // Test 1: Basic RTC functionality - read current time
    let initial_time = rtc.now().unwrap();
    info!(
        "Initial time: {}-{:02}-{:02} {}:{:02}:{:02}",
        initial_time.year,
        initial_time.month,
        initial_time.day,
        initial_time.hour,
        initial_time.minute,
        initial_time.second
    );

    // Test 2: Schedule and wait for alarm
    info!("Testing alarm scheduling");

    // Wait until we're at a predictable second, then schedule for a future second
    loop {
        let current = rtc.now().unwrap();
        if current.second <= 55 {
            break;
        }
        Timer::after_millis(100).await;
    }

    // Now schedule alarm for 3 seconds from current time
    let current_time = rtc.now().unwrap();
    let alarm_second = (current_time.second + 3) % 60;
    let alarm_filter = DateTimeFilter::default().second(alarm_second);

    info!("Scheduling alarm for second: {}", alarm_second);
    rtc.schedule_alarm(alarm_filter);

    // Verify alarm is scheduled
    let scheduled = rtc.alarm_scheduled();
    assert!(scheduled.is_some(), "Alarm should be scheduled");
    info!("Alarm scheduled successfully: {}", scheduled.unwrap());

    // Wait for alarm with timeout
    let alarm_start = Instant::now();
    match select(Timer::after_secs(5), rtc.wait_for_alarm()).await {
        Either::First(_) => {
            core::panic!("Alarm timeout - alarm should have triggered by now");
        }
        Either::Second(_) => {
            let alarm_duration = Instant::now() - alarm_start;
            info!("ALARM TRIGGERED after {:?}", alarm_duration);

            // Verify timing is reasonable (should be around 3 seconds)
            assert!(
                alarm_duration >= Duration::from_secs(2) && alarm_duration <= Duration::from_secs(4),
                "Alarm timing incorrect: {:?}",
                alarm_duration
            );
        }
    }

    // Test 3: Verify RTC is still running and time has advanced
    let final_time = rtc.now().unwrap();
    info!(
        "Final time: {}-{:02}-{:02} {}:{:02}:{:02}",
        final_time.year, final_time.month, final_time.day, final_time.hour, final_time.minute, final_time.second
    );

    // Verify time has advanced (allowing for minute/hour rollover)
    let time_diff = if final_time.second >= initial_time.second {
        final_time.second - initial_time.second
    } else {
        60 - initial_time.second + final_time.second
    };

    assert!(time_diff >= 3, "RTC should have advanced by at least 3 seconds");
    info!("Time advanced by {} seconds", time_diff);

    // Test 4: Verify alarm is no longer scheduled after triggering
    let post_alarm_scheduled = rtc.alarm_scheduled();
    assert!(
        post_alarm_scheduled.is_none(),
        "Alarm should not be scheduled after triggering"
    );
    info!("Alarm correctly cleared after triggering");

    info!("Test OK");
    cortex_m::asm::bkpt();
}

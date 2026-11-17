#![no_std]
#![no_main]
#[cfg(feature = "rp2040")]
teleprobe_meta::target!(b"rpi-pico");
teleprobe_meta::timeout!(60);

use defmt::{assert, assert_eq, *};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_rp::bind_interrupts;
use embassy_rp::rtc::{DateTimeFilter, InvalidDateTimeError, OutOfRangeError, RawDateTime, Rtc};
use embassy_time::{Duration, Instant, Timer};
use jiff::civil::date;
use panic_probe as _;

// Bind the RTC interrupt to the handler
bind_interrupts!(struct Irqs {
    RTC_IRQ => embassy_rp::rtc::InterruptHandler;
});

fn raw(year: u16, month: u8, day: u8, day_of_week: u8, hour: u8, minute: u8, second: u8) -> RawDateTime {
    RawDateTime {
        year,
        month,
        day,
        day_of_week,
        hour,
        minute,
        second,
    }
}

/// Set the RTC to `from`, wait for it to tick once, and return what it reads.
async fn tick(rtc: &mut Rtc<'_, embassy_rp::peripherals::RTC>, from: RawDateTime) -> RawDateTime {
    rtc.set_raw(from);
    // The registers show the old time for a couple of `clk_rtc` cycles after the load.
    Timer::after_millis(1).await;
    let start = Instant::now();
    loop {
        let now = rtc.read_raw();
        if now != from {
            return now;
        }
        assert!(start.elapsed() < Duration::from_millis(1500), "RTC did not tick");
    }
}

/// `t` one second later, within the same minute.
fn next_second(t: RawDateTime) -> RawDateTime {
    RawDateTime {
        second: t.second + 1,
        ..t
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut rtc = Rtc::new(p.RTC, Irqs);

    info!("RTC test started");

    // The raw API keeps the day of week, even when it doesn't match the date.
    info!("raw: set and read");
    let t = raw(2020, 5, 15, 2, 10, 30, 15);
    rtc.set_raw(t);
    assert!(rtc.is_running());
    Timer::after_millis(1).await;
    // The RTC ticks right after it is started, so it can already be a second later.
    let read = rtc.read_raw();
    info!("wrote {}, read {}", t, read);
    assert!(read == t || read == next_second(t));
    assert_eq!(tick(&mut rtc, read).await, next_second(read));

    // The day of week is incremented with the date, wrapping from Saturday to Sunday.
    info!("raw: midnight rollover");
    assert_eq!(
        tick(&mut rtc, raw(2000, 1, 1, 6, 23, 59, 59)).await,
        raw(2000, 1, 2, 0, 0, 0, 0)
    );
    assert_eq!(
        tick(&mut rtc, raw(2000, 12, 31, 0, 23, 59, 59)).await,
        raw(2001, 1, 1, 1, 0, 0, 0)
    );

    // The RTC takes every year divisible by 4 as a leap year...
    info!("leap year check");
    assert_eq!(
        tick(&mut rtc, raw(2024, 2, 28, 3, 23, 59, 59)).await,
        raw(2024, 2, 29, 4, 0, 0, 0)
    );
    assert_eq!(
        tick(&mut rtc, raw(2023, 2, 28, 2, 23, 59, 59)).await,
        raw(2023, 3, 1, 3, 0, 0, 0)
    );
    // ... including 2100, a date `read` rejects ...
    assert_eq!(
        tick(&mut rtc, raw(2100, 2, 28, 0, 23, 59, 59)).await,
        raw(2100, 2, 29, 1, 0, 0, 0)
    );
    assert!(rtc.read() == Err(InvalidDateTimeError));
    // ... unless the leap year check is disabled. `set_raw` clears the setting, so it is changed
    // after it, with a second to spare before midnight.
    rtc.set_raw(raw(2100, 2, 28, 0, 23, 59, 58));
    rtc.set_leap_year_check(false);
    Timer::after_millis(1).await;
    let start = Instant::now();
    while rtc.read_raw().day == 28 {
        assert!(
            start.elapsed() < Duration::from_millis(2500),
            "RTC did not reach midnight"
        );
    }
    info!("leap year check disabled: {}", rtc.read_raw());
    assert_eq!(rtc.read_raw(), raw(2100, 3, 1, 1, 0, 0, 0));
    assert!(rtc.read() == Ok(date(2100, 3, 1).at(0, 0, 0, 0)));
    rtc.set_leap_year_check(true);

    info!("jiff: set and read");
    rtc.set(date(2020, 5, 15).at(10, 30, 15, 0)).unwrap();
    Timer::after_millis(1).await;
    let read = rtc.read().unwrap();
    assert!(read == date(2020, 5, 15).at(10, 30, 15, 0) || read == date(2020, 5, 15).at(10, 30, 16, 0));
    // `set` writes a dummy day of week.
    assert_eq!(rtc.read_raw().day_of_week, 0);
    assert_eq!(rtc.set(date(4096, 1, 1).at(0, 0, 0, 0)), Err(OutOfRangeError));
    assert_eq!(rtc.set(date(-1, 12, 31).at(0, 0, 0, 0)), Err(OutOfRangeError));

    info!("jiff: invalid date in the RTC");
    rtc.set_raw(raw(2023, 2, 30, 0, 0, 0, 0));
    Timer::after_millis(1).await;
    assert!(rtc.read() == Err(InvalidDateTimeError));

    // Every field of the alarm filter reads back, including the raw day of week.
    info!("alarm: scheduled filter");
    rtc.set_raw(raw(2020, 5, 15, 5, 10, 30, 15));
    let filter = DateTimeFilter::default()
        .year(2021)
        .month(6)
        .day(7)
        .day_of_week(3)
        .hour(8)
        .minute(9)
        .second(10);
    rtc.schedule_alarm(filter);
    assert_eq!(rtc.alarm_scheduled(), Some(filter));
    rtc.disable_alarm();
    assert_eq!(rtc.alarm_scheduled(), None);

    info!("alarm: wait");
    let alarm_second = (rtc.read_raw().second + 3) % 60;
    rtc.schedule_alarm(DateTimeFilter::default().second(alarm_second));
    let alarm_start = Instant::now();
    match select(Timer::after_secs(5), rtc.wait_for_alarm()).await {
        Either::First(_) => core::panic!("alarm did not trigger"),
        Either::Second(_) => {
            let alarm_duration = Instant::now() - alarm_start;
            info!("alarm triggered after {}", alarm_duration);
            assert!(alarm_duration >= Duration::from_secs(2) && alarm_duration <= Duration::from_secs(4));
        }
    }
    assert_eq!(rtc.read_raw().second, alarm_second);
    assert_eq!(rtc.alarm_scheduled(), None);

    info!("Test OK");
    cortex_m::asm::bkpt();
}

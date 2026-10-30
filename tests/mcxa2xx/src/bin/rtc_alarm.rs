#![no_std]
#![no_main]

teleprobe_meta::target!(b"frdm-mcx-a266");

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_mcxa::bind_interrupts;
use embassy_time::{Duration, Instant, Timer, with_timeout};
use hal::peripherals::RTC0;
use hal::rtc::{DateTime, InterruptHandler, Rtc};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    RTC => InterruptHandler<RTC0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(hal::config::Config::default());

    let mut rtc = Rtc::new(p.RTC0, Irqs, Default::default());

    let now = DateTime {
        year: 2026,
        month: 10,
        day: 30,
        hour: 14,
        minute: 59,
        second: 58,
    };

    rtc.set_datetime(now);

    let read = rtc.get_datetime();
    assert_eq!(read.year, 2026);
    assert_eq!(read.month, 10);
    assert_eq!(read.day, 30);
    assert_eq!(read.hour, 14);
    assert_eq!(read.minute, 59);
    assert_eq!(read.second, 58);

    // Time must advance without arming an alarm.
    Timer::after_secs(3).await;
    let read = rtc.get_datetime();
    assert_eq!(read.hour, 15);
    assert_eq!(read.minute, 0);
    assert!(read.second <= 3);

    // Re-setting a running counter must work, including the prescaler reset.
    rtc.set_datetime(now);

    let mut alarm = now;
    alarm.hour = 15;
    alarm.minute = 0;
    alarm.second = 0;

    // Exercise rollover and rearming after an IRQ, not just a single wakeup.
    for second in [0, 3] {
        alarm.second = second;
        let start = Instant::now();
        with_timeout(Duration::from_secs(10), rtc.wait_for_alarm(alarm))
            .await
            .expect("RTC alarm timed out");
        assert!(start.elapsed() >= Duration::from_millis(500), "RTC alarm fired early");

        let read = rtc.get_datetime();
        assert_eq!(read.hour, 15);
        assert_eq!(read.minute, 0);
        assert!(read.second >= second && read.second <= second + 2);
    }

    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}

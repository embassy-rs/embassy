#![no_std]
#![no_main]

teleprobe_meta::target!(b"frdm-mcx-a266");
teleprobe_meta::timeout!(30);

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_mcxa as hal;
use embassy_mcxa::bind_interrupts;
use embassy_time::{Duration, Instant, Timer, with_timeout};
use hal::peripherals::RTC0;
use hal::rtc::{DateTime, InterruptHandler, Rtc, convert_datetime_to_seconds, convert_seconds_to_datetime};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    RTC => InterruptHandler<RTC0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = hal::init(hal::config::Config::default());

    let (mut rtc, time_provider) = Rtc::new(p.RTC0, Irqs, Default::default());
    let reader = time_provider.clone();

    let now = DateTime {
        year: 2026,
        month: 10,
        day: 30,
        hour: 14,
        minute: 59,
        second: 58,
    };

    rtc.set_datetime(now);

    let read = reader.now();
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
        let (result, ()) = join(
            with_timeout(Duration::from_secs(10), rtc.wait_for_alarm(alarm)),
            async {
                // The alarm is pending, but another owner can still read UTC.
                Timer::after_millis(100).await;
                let read = reader.now();
                assert_eq!(read.year, 2026);
                assert!(read.hour == 14 || read.hour == 15);
                assert!(start.elapsed() < Duration::from_secs(1));
            },
        )
        .await;
        result.expect("RTC alarm timed out");
        assert!(start.elapsed() >= Duration::from_millis(500), "RTC alarm fired early");
        assert!(!hal::pac::RTC0.ier().read().taie());

        let read = rtc.get_datetime();
        assert_eq!(read.hour, 15);
        assert_eq!(read.minute, 0);
        assert!(read.second >= second && read.second <= second + 2);
    }

    // Constructing and dropping an unpolled wait must not arm the hardware.
    let before = hal::pac::RTC0.tar().read().0;
    drop(rtc.wait_for_alarm(alarm));
    assert_eq!(hal::pac::RTC0.tar().read().0, before);
    assert!(!hal::pac::RTC0.ier().read().taie());

    // Cancellation must disable interrupts, even before the alarm has matched.
    let seconds = convert_datetime_to_seconds(&time_provider.now());
    let abandoned = convert_seconds_to_datetime(seconds + 2);
    assert!(
        with_timeout(Duration::from_millis(100), rtc.wait_for_alarm(abandoned))
            .await
            .is_err()
    );
    assert!(!hal::pac::RTC0.ier().read().taie());
    Timer::after_secs(4).await;
    assert!(!hal::pac::RTC0.ier().read().taie());

    // A match after cancellation must not make the next wait complete early.
    let seconds = convert_datetime_to_seconds(&time_provider.now());
    let next = convert_seconds_to_datetime(seconds + 2);
    let start = Instant::now();
    with_timeout(Duration::from_secs(5), rtc.wait_for_alarm(next))
        .await
        .expect("RTC alarm did not rearm after cancellation");
    assert!(start.elapsed() >= Duration::from_secs(1));

    // A reader retains its own clock/power guard, even without the alarm handle.
    drop(rtc);
    let before = convert_datetime_to_seconds(&time_provider.now());
    Timer::after_secs(2).await;
    let after = convert_datetime_to_seconds(&time_provider.now());
    assert!(after >= before + 1);

    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}

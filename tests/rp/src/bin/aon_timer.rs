#![no_std]
#![no_main]
#[cfg(feature = "rp235xa")]
teleprobe_meta::target!(b"rpi-pico-2");
#[cfg(feature = "rp235xb")]
teleprobe_meta::target!(b"pimoroni-pico-plus-2");

use defmt::{assert, *};
use embassy_executor::Spawner;
use embassy_rp::aon_timer::{AlarmWakeMode, AonTimer, ClockSource, Config, Error};
use embassy_rp::bind_interrupts;
use embassy_time::{Duration, Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    POWMAN_IRQ_TIMER => embassy_rp::aon_timer::InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut p = embassy_rp::init(Default::default());
    info!("Hello World!");

    // Basic timer operations
    {
        let mut timer = AonTimer::new(p.POWMAN.reborrow(), Irqs, Config::default());
        assert!(!timer.is_running());

        timer.set_counter(0);
        timer.start();
        assert!(timer.is_running());

        Timer::after_millis(100).await;
        let val = timer.now();
        assert!(val >= 90 && val <= 120);

        timer.stop();
        assert!(!timer.is_running());
    }

    // Counter precision
    {
        let mut timer = AonTimer::new(p.POWMAN.reborrow(), Irqs, Config::default());
        timer.set_counter(0);
        timer.start();

        Timer::after_millis(50).await;
        let first = timer.now();

        Timer::after_millis(100).await;
        let second = timer.now();

        let elapsed = second - first;
        assert!(elapsed >= 90 && elapsed <= 120);

        timer.stop();
    }

    // Set alarm at specific time
    {
        let mut timer = AonTimer::new(p.POWMAN.reborrow(), Irqs, Config::default());
        timer.set_counter(0);
        timer.start();

        Timer::after_millis(50).await;

        let current = timer.now();
        let alarm_time = current + 200;
        timer.set_alarm(alarm_time).unwrap();

        Timer::after_millis(250).await;
        assert!(timer.alarm_fired());

        timer.stop();
    }

    // Set alarm relative to now
    {
        let mut timer = AonTimer::new(p.POWMAN.reborrow(), Irqs, Config::default());
        timer.set_counter(0);
        timer.start();

        Timer::after_millis(50).await;

        timer.set_alarm_after(Duration::from_millis(150)).unwrap();

        Timer::after_millis(200).await;
        assert!(timer.alarm_fired());

        timer.stop();
    }

    // Alarm in past error
    {
        let mut timer = AonTimer::new(p.POWMAN.reborrow(), Irqs, Config::default());
        timer.set_counter(1000);
        timer.start();

        Timer::after_millis(50).await;

        let result = timer.set_alarm(500);
        assert!(matches!(result, Err(Error::AlarmInPast)));

        timer.stop();
    }

    // Clear alarm
    {
        let mut timer = AonTimer::new(p.POWMAN.reborrow(), Irqs, Config::default());
        timer.set_counter(0);
        timer.start();

        Timer::after_millis(50).await;

        timer.set_alarm_after(Duration::from_millis(100)).unwrap();
        Timer::after_millis(150).await;

        assert!(timer.alarm_fired());
        timer.clear_alarm();
        assert!(!timer.alarm_fired());

        timer.stop();
    }

    // Disable/enable alarm
    {
        let mut timer = AonTimer::new(p.POWMAN.reborrow(), Irqs, Config::default());
        timer.set_counter(0);
        timer.start();

        Timer::after_millis(50).await;

        timer.set_alarm_after(Duration::from_millis(100)).unwrap();
        timer.disable_alarm();

        Timer::after_millis(150).await;
        assert!(!timer.alarm_fired());

        timer.stop();
    }

    // Async alarm wait
    {
        let mut timer = AonTimer::new(p.POWMAN.reborrow(), Irqs, Config::default());
        timer.set_counter(0);
        timer.start();

        Timer::after_millis(50).await;

        timer.set_alarm_after(Duration::from_millis(200)).unwrap();
        timer.wait_for_alarm().await;

        timer.stop();
    }

    // LPOSC clock source
    {
        let config = Config {
            clock_source: ClockSource::Lposc,
            clock_freq_khz: 32,
            alarm_wake_mode: AlarmWakeMode::WfiOnly,
        };
        let mut timer = AonTimer::new(p.POWMAN.reborrow(), Irqs, config);
        timer.set_counter(0);
        timer.start();

        Timer::after_millis(100).await;
        let value = timer.now();
        assert!(value >= 50 && value <= 150);

        timer.stop();
    }

    // Counter overflow edge case
    {
        let mut timer = AonTimer::new(p.POWMAN.reborrow(), Irqs, Config::default());

        let near_max = 0xFFFF_FFFF_FFFF_F000u64;
        timer.set_counter(near_max);
        timer.start();

        Timer::after_millis(50).await;

        let read1 = timer.now();
        assert!(read1 >= near_max);

        Timer::after_millis(100).await;

        let read2 = timer.now();
        assert!(read2 > read1);

        timer.stop();
    }

    // Rapid alarms
    {
        let mut timer = AonTimer::new(p.POWMAN.reborrow(), Irqs, Config::default());
        timer.set_counter(0);
        timer.start();

        Timer::after_millis(50).await;

        for _ in 0..100 {
            timer.set_alarm_after(Duration::from_millis(10)).unwrap();
            timer.wait_for_alarm().await;
        }

        timer.stop();
    }

    // Long-running stability
    {
        let mut timer = AonTimer::new(p.POWMAN.reborrow(), Irqs, Config::default());
        timer.set_counter(0);
        timer.start();

        Timer::after_millis(50).await;

        let start_time = timer.now();
        let wall_start = Instant::now();

        Timer::after_secs(5).await;

        let timer_elapsed = timer.now() - start_time;
        let wall_elapsed = wall_start.elapsed().as_millis();
        let drift = (timer_elapsed as i64) - (wall_elapsed as i64);

        assert!(drift.abs() < 100);

        timer.stop();
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}

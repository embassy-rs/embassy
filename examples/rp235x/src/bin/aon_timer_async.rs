//! AON (Always-On) Timer Example using Embassy Async API
//!
//! This example demonstrates async alarm support with the AON Timer.
//! The alarm triggers an interrupt (POWMAN_IRQ_TIMER) which wakes the CPU
//! from WFI (Wait For Interrupt) low-power mode.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::aon_timer::{AlarmWakeMode, AonTimer, ClockSource, Config};
use embassy_rp::{bind_interrupts, gpio};
use embassy_time::{Duration, Timer};
use gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    POWMAN_IRQ_TIMER => embassy_rp::aon_timer::InterruptHandler;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, Level::Low);

    info!("AON Timer Async example starting");

    // Small delay for debug probe
    Timer::after_millis(10).await;

    // Configure the AON Timer with XOSC at 12 MHz
    let config = Config {
        clock_source: ClockSource::Xosc,
        clock_freq_khz: 12000,
        alarm_wake_mode: AlarmWakeMode::WfiOnly,
    };

    let mut aon = AonTimer::new(p.POWMAN, Irqs, config);

    // Set counter to 0 (start counting from boot)
    info!("Setting counter to 0");
    aon.set_counter(0);

    // Start the timer
    aon.start();
    info!("AON Timer started");

    // Verify timer is running
    Timer::after_millis(100).await;
    let initial_ms = aon.now();
    info!("Counter value after 100ms: {} ms", initial_ms);

    // Main loop: set alarms and wait asynchronously
    for i in 1..=5 {
        info!("=== Round {} ===", i);

        // Set an alarm for 2 seconds from now
        let alarm_duration = Duration::from_secs(2);
        let current = aon.now();
        info!("Current time: {} ms", current);
        info!("Setting alarm for {} seconds from now", alarm_duration.as_secs());

        aon.set_alarm_after(alarm_duration).unwrap();

        // Blink LED while waiting
        led.set_high();

        // Wait asynchronously for the alarm
        // The CPU will enter WFI low-power mode during this time
        info!("Waiting for alarm...");
        aon.wait_for_alarm().await;

        led.set_low();

        // Alarm fired!
        let elapsed = aon.elapsed();
        info!(
            "ALARM FIRED! Counter: {} ms ({}.{:03} seconds)",
            aon.now(),
            elapsed.as_secs(),
            elapsed.as_millis() % 1000
        );

        // Wait a bit before next round
        Timer::after_secs(1).await;
    }

    info!("Demo complete! Looping with longer alarms...");

    // Continue with longer alarms
    loop {
        info!("Setting alarm for 5 seconds");
        aon.set_alarm_after(Duration::from_secs(5)).unwrap();

        led.toggle();
        aon.wait_for_alarm().await;
        led.toggle();

        info!("Alarm fired at {} ms", aon.now());
        Timer::after_secs(1).await;
    }
}

#![no_std]
#![no_main]
#[cfg(feature = "rp2040")]
teleprobe_meta::target!(b"rpi-pico");

use defmt::{assert, assert_eq, info};
use embassy_executor::Spawner;
use embassy_rp::config::Config;
use embassy_rp::gpio::{Input, Pull};
use embassy_rp::pwm::{Config as PwmConfig, Pwm};
use embassy_time::{Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

#[cfg(feature = "rp2040")]
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize with 200MHz clock configuration for RP2040
    let mut config = Config::default();
    config.clocks = embassy_rp::clocks::ClockConfig::at_sys_frequency_mhz(200);

    let p = embassy_rp::init(config);

    info!("RP2040 overclocked to 200MHz!");
    info!("System clock frequency: {} Hz", embassy_rp::clocks::clk_sys_freq());

    // Test 1: Timer accuracy at 200MHz
    info!("Testing timer accuracy at 200MHz...");
    let start = Instant::now();
    Timer::after_millis(100).await;
    let end = Instant::now();
    let ms = (end - start).as_millis();
    info!("slept for {} ms", ms);
    assert!(ms >= 99);
    assert!(ms < 110);
    info!("Timer test passed!");

    // Test 2: PWM functionality at 200MHz
    info!("Testing PWM functionality at 200MHz...");
    let pwm_cfg = {
        let mut c = PwmConfig::default();
        c.divider = ((embassy_rp::clocks::clk_sys_freq() / 1_000_000) as u8).into();
        c.top = 10000;
        c.compare_a = 5000;
        c.compare_b = 5000;
        c
    };

    // Test PWM output
    let pin1 = Input::new(p.PIN_9, Pull::None);
    let _pwm = Pwm::new_output_a(p.PWM_SLICE3, p.PIN_6, pwm_cfg);
    Timer::after_millis(1).await;
    let initial_state = pin1.is_low();
    Timer::after_millis(5).await;
    assert_eq!(pin1.is_high(), initial_state);
    Timer::after_millis(5).await;
    assert_eq!(pin1.is_low(), initial_state);
    info!("PWM test passed!");

    info!("All tests passed at 200MHz!");
    info!("Overclock test successful");
    cortex_m::asm::bkpt();
}

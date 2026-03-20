//! Comparator (COMP) example for STM32G474.
//!
//! Demonstrates using the comparator peripheral to compare an analog input
//! against an internal reference (Vref ~ 1.2V).
//!
//! Hardware setup:
//! - Connect a variable voltage (0-3.3V) to PA7 (COMP2 INP0)
//!
//! The comparator will read HIGH when PA7 > Vref, LOW otherwise.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::comp::{self, Comp, Config, InvertingInput};
use embassy_stm32::{Config as SysConfig, bind_interrupts};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    COMP1_2_3 => comp::InterruptHandler<embassy_stm32::peripherals::COMP2>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(SysConfig::default());
    info!("COMP example starting!");

    let mut cfg = Config::default();
    cfg.inverting_input = InvertingInput::Vref;

    let mut comp2 = Comp::new(p.COMP2, p.PA7, Irqs, cfg);
    comp2.enable();

    info!("Comparator configured. Comparing PA7 against Vref (~1.2V)");

    // Polling
    for _ in 0..5 {
        info!("output: {}", comp2.output_level());
        Timer::after_millis(500).await;
    }

    // Async edge detection
    info!("Waiting for edges...");
    loop {
        comp2.wait_for_rising_edge().await;
        info!("Rising edge!");
        Timer::after_millis(100).await;

        comp2.wait_for_falling_edge().await;
        info!("Falling edge!");
        Timer::after_millis(100).await;
    }
}

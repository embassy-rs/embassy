//! Comparator (COMP) example for STM32WBA6.
//!
//! This example demonstrates how to use the comparator peripheral to compare
//! an analog input voltage against an internal reference (half Vref).
//!
//! Hardware setup:
//! - Connect a variable voltage (0-3.3V) to PA2 (COMP1 INP)
//! - The comparator will compare PA2 against 1/2 Vref (~1.65V)
//!
//! The example shows:
//! - Polling the comparator output level
//! - Using async edge detection to wait for transitions

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::comp::{Comp, Config, Hysteresis, InvertingInput, OutputPolarity, PowerMode};
use embassy_stm32::rcc::{
    AHB5Prescaler, AHBPrescaler, APBPrescaler, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk, VoltageScale,
};
use embassy_stm32::{bind_interrupts, comp};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    COMP => comp::InterruptHandler<embassy_stm32::peripherals::COMP1>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();

    // Configure PLL for system clock
    config.rcc.pll1 = Some(embassy_stm32::rcc::Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV1,
        mul: PllMul::MUL30,
        divr: Some(PllDiv::DIV5),
        divq: None,
        divp: Some(PllDiv::DIV30),
        frac: Some(0),
    });

    config.rcc.ahb_pre = AHBPrescaler::DIV1;
    config.rcc.apb1_pre = APBPrescaler::DIV1;
    config.rcc.apb2_pre = APBPrescaler::DIV1;
    config.rcc.apb7_pre = APBPrescaler::DIV1;
    config.rcc.ahb5_pre = AHB5Prescaler::DIV4;
    config.rcc.voltage_scale = VoltageScale::RANGE1;
    config.rcc.sys = Sysclk::PLL1_R;

    let p = embassy_stm32::init(config);
    info!("COMP example starting!");

    // Configure comparator:
    // - Non-inverting input (INP): PA2
    // - Inverting input (INM): Internal reference (1/2 Vref)
    // - High speed power mode
    // - Low hysteresis for noise immunity
    let comp_config = Config {
        power_mode: PowerMode::HighSpeed,
        hysteresis: Hysteresis::Low,
        output_polarity: OutputPolarity::NotInverted,
        inverting_input: InvertingInput::HalfVref,
        ..Default::default()
    };

    let mut comp1 = Comp::new(p.COMP1, p.PA2, Irqs, comp_config);

    info!("Comparator configured. Comparing PA2 against 1/2 Vref (~1.65V)");
    info!("Output HIGH when PA2 > 1.65V, LOW when PA2 < 1.65V");

    // Enable the comparator
    comp1.enable();

    // Polling example: read the comparator output every second
    info!("Starting polling mode - reading comparator output every second...");
    for _ in 0..5 {
        let output = comp1.output_level();
        if output {
            info!("Comparator output: HIGH (PA2 > 1/2 Vref)");
        } else {
            info!("Comparator output: LOW (PA2 < 1/2 Vref)");
        }
        Timer::after_millis(1000).await;
    }

    // Async edge detection example
    info!("Switching to async mode - waiting for edges...");

    loop {
        info!("Waiting for rising edge (PA2 going above 1/2 Vref)...");
        comp1.wait_for_rising_edge().await;
        info!("Rising edge detected!");

        // Small delay to debounce
        Timer::after_millis(100).await;

        info!("Waiting for falling edge (PA2 going below 1/2 Vref)...");
        comp1.wait_for_falling_edge().await;
        info!("Falling edge detected!");

        // Small delay to debounce
        Timer::after_millis(100).await;
    }
}

//! Comparator Window Mode example for STM32WBA6.
//!
//! This example demonstrates how to use two comparators in window mode to detect
//! when an analog input voltage is within a specific voltage window.
//!
//! Window mode configuration:
//! - COMP1: Upper threshold at 3/4 Vref (~2.48V)
//! - COMP2: Lower threshold at 1/4 Vref (~0.83V)
//! - Both comparators share the same input signal
//!
//! The window detection works as follows:
//! - Signal BELOW window (< 1/4 Vref): COMP1=LOW, COMP2=LOW
//! - Signal WITHIN window (1/4 to 3/4 Vref): COMP1=LOW, COMP2=HIGH
//! - Signal ABOVE window (> 3/4 Vref): COMP1=HIGH, COMP2=HIGH
//!
//! Hardware setup:
//! - Connect a variable voltage (0-3.3V) to PA2 (COMP1 INP)
//! - COMP2 will use COMP1's input in window mode

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::comp::{
    Comp, Config, Hysteresis, InvertingInput, OutputPolarity, PowerMode, WindowMode, WindowOutput,
};
use embassy_stm32::rcc::{
    AHB5Prescaler, AHBPrescaler, APBPrescaler, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk, VoltageScale,
};
use embassy_stm32::{bind_interrupts, comp};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    COMP => comp::InterruptHandler<embassy_stm32::peripherals::COMP1>,
            comp::InterruptHandler<embassy_stm32::peripherals::COMP2>;
});

/// Represents the position of the signal relative to the voltage window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, defmt::Format)]
enum WindowPosition {
    /// Signal is below the window (< lower threshold)
    BelowWindow,
    /// Signal is within the window (between thresholds)
    WithinWindow,
    /// Signal is above the window (> upper threshold)
    AboveWindow,
}

/// Determine the window position based on both comparator outputs.
fn get_window_position(comp1_high: bool, comp2_high: bool) -> WindowPosition {
    match (comp1_high, comp2_high) {
        (false, false) => WindowPosition::BelowWindow,
        (false, true) => WindowPosition::WithinWindow,
        (true, true) => WindowPosition::AboveWindow,
        // This state shouldn't happen with proper thresholds
        (true, false) => WindowPosition::BelowWindow,
    }
}

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
    info!("COMP Window Mode example starting!");

    // Configure COMP1 as the upper threshold comparator
    // - Non-inverting input (INP): PA2 (the signal to monitor)
    // - Inverting input (INM): 3/4 Vref (~2.48V) - upper threshold
    // - Window mode disabled (this is the primary comparator)
    let comp1_config = Config {
        power_mode: PowerMode::HighSpeed,
        hysteresis: Hysteresis::Low,
        output_polarity: OutputPolarity::NotInverted,
        inverting_input: InvertingInput::ThreeQuarterVref,
        window_mode: WindowMode::Disabled,
        window_output: WindowOutput::OwnValue,
        ..Default::default()
    };

    // Configure COMP2 as the lower threshold comparator
    // - Non-inverting input: Uses COMP1's input (PA2) via window mode
    // - Inverting input (INM): 1/4 Vref (~0.83V) - lower threshold
    // - Window mode enabled to share COMP1's input
    let comp2_config = Config {
        power_mode: PowerMode::HighSpeed,
        hysteresis: Hysteresis::Low,
        output_polarity: OutputPolarity::NotInverted,
        inverting_input: InvertingInput::OneQuarterVref,
        window_mode: WindowMode::Enabled, // Use COMP1's INP
        window_output: WindowOutput::OwnValue,
        ..Default::default()
    };

    // Create COMP1 with its input pin (PA2)
    let mut comp1 = Comp::new(p.COMP1, p.PA2, Irqs, comp1_config);

    // Create COMP2 - in window mode, it uses COMP1's input
    // We still need to provide a pin for the constructor, but it won't be used
    // since window mode routes COMP1's input to COMP2
    let mut comp2 = Comp::new(p.COMP2, p.PA0, Irqs, comp2_config);

    info!("Window comparator configured:");
    info!("  - Upper threshold: 3/4 Vref (~2.48V)");
    info!("  - Lower threshold: 1/4 Vref (~0.83V)");
    info!("  - Input signal: PA2");
    info!("");
    info!("Window detection:");
    info!("  - BELOW:  Signal < 0.83V");
    info!("  - WITHIN: 0.83V < Signal < 2.48V");
    info!("  - ABOVE:  Signal > 2.48V");

    // Enable both comparators
    comp1.enable();
    comp2.enable();

    // Allow comparators to stabilize
    Timer::after_millis(10).await;

    info!("");
    info!("Starting window detection - monitoring every 500ms...");

    let mut last_position = None;

    loop {
        // Read both comparator outputs
        let comp1_output = comp1.output_level();
        let comp2_output = comp2.output_level();

        // Determine window position
        let position = get_window_position(comp1_output, comp2_output);

        // Only log when position changes to reduce output noise
        if last_position != Some(position) {
            match position {
                WindowPosition::BelowWindow => {
                    info!(
                        "BELOW WINDOW - Signal < 0.83V (COMP1={}, COMP2={})",
                        comp1_output, comp2_output
                    );
                }
                WindowPosition::WithinWindow => {
                    info!(
                        "WITHIN WINDOW - 0.83V < Signal < 2.48V (COMP1={}, COMP2={})",
                        comp1_output, comp2_output
                    );
                }
                WindowPosition::AboveWindow => {
                    info!(
                        "ABOVE WINDOW - Signal > 2.48V (COMP1={}, COMP2={})",
                        comp1_output, comp2_output
                    );
                }
            }
            last_position = Some(position);
        }

        Timer::after_millis(500).await;
    }
}

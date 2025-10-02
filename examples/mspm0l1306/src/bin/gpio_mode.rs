#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_mspm0::gpio::{Input, Level, Output, OutputOpenDrain, Pull};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

/// GPIO configuration validation tests for MSPM0L1306
///
/// Hardware setup:
/// - PA0: Connect to external 3.3V pull-up resistor (e.g., 4.7kΩ) via J9
///        for OutputOpenDrain tests
/// - PA4: Standard push-pull output for comparison
/// - PA12: Floating pin for internal pull-up test
/// - PA13: Floating pin for internal pull-down test
/// - PA23: Floating pin for no internal pull test
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_mspm0::init(Default::default());
    info!("MSPM0 GPIO Configuration Tests");

    // Test 1: Standard push-pull output
    info!("\n=== Test 1: Standard Output (Push-Pull) ===");
    let mut standard_out = Output::new(p.PA4, Level::Low);
    info!("Initial: is_set_high={}", standard_out.is_set_high());

    standard_out.set_high();
    info!("After set_high(): is_set_high={}", standard_out.is_set_high());

    standard_out.set_low();
    info!("After set_low(): is_set_low={}", standard_out.is_set_low());

    // Test 2: OutputOpenDrain with external pull-up (requires hardware setup on PA0)
    info!("\n=== Test 2: OutputOpenDrain + External Pull-up ===");
    info!("Hardware: PA0 connected to 3.3V via pull-up resistor (J9)");
    let mut open_drain = OutputOpenDrain::new(p.PA0, Level::Low);

    info!(
        "Initial (Low): is_high={}, get_level={:?}, is_set_high={}",
        open_drain.is_high(),
        open_drain.get_level(),
        open_drain.is_set_high()
    );

    open_drain.set_high();
    Timer::after_millis(10).await; // Allow time for pull-up
    info!(
        "After set_high(): is_high={}, get_level={:?}, is_set_high={}",
        open_drain.is_high(),
        open_drain.get_level(),
        open_drain.is_set_high()
    );

    open_drain.set_low();
    Timer::after_millis(10).await;
    info!(
        "After set_low(): is_high={}, get_level={:?}, is_set_low={}",
        open_drain.is_high(),
        open_drain.get_level(),
        open_drain.is_set_low()
    );

    open_drain.toggle();
    Timer::after_millis(10).await;
    info!(
        "After toggle(): is_high={}, get_level={:?}",
        open_drain.is_high(),
        open_drain.get_level()
    );

    // Test 3: Input with internal pull-up
    info!("\n=== Test 3: Input with Internal Pull-Up ===");
    info!("Pin: PA12 (floating, internal pull-up enabled)");
    let input_pu = Input::new(p.PA12, Pull::Up);
    info!("is_high={}, get_level={:?}", input_pu.is_high(), input_pu.get_level());
    drop(input_pu);

    // Test 4: Input with internal pull-down
    info!("\n=== Test 4: Input with Internal Pull-Down ===");
    info!("Pin: PA13 (floating, internal pull-down enabled)");
    let input_pd = Input::new(p.PA13, Pull::Down);
    info!("is_high={}, get_level={:?}", input_pd.is_high(), input_pd.get_level());
    drop(input_pd);

    // Test 5: Input floating (no pull resistor)
    info!("\n=== Test 5: Input Floating (No Pull) ===");
    info!("Pin: PA23 (floating, no internal pull, may be unstable)");
    let input_float = Input::new(p.PA23, Pull::None);
    info!(
        "is_high={}, get_level={:?} (may be unstable)",
        input_float.is_high(),
        input_float.get_level()
    );
    Timer::after_millis(10).await;
    info!(
        "After 10ms: is_high={}, get_level={:?}",
        input_float.is_high(),
        input_float.get_level()
    );
    drop(input_float);

    info!("\n=== All GPIO configuration tests completed ===");
    info!("Entering toggle demo on open-drain output...");

    // Continuous toggle demo
    loop {
        Timer::after_millis(500).await;
        open_drain.toggle();
        info!("OpenDrain toggled: is_high={}", open_drain.is_high());
    }
}

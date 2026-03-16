//! This example demonstrates automatic clock gating. This example needs to be run
//! with:
//!
//! ```sh
//! cargo run --release --no-default-features --features=executor-platform --bin power-deepsleep-gpio-int
//! ```

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clkout::{self, ClockOut, ClockOutSel, Div4};
use embassy_mcxa::clocks::config::{
    CoreSleep, Div8, FircConfig, FircFreqSel, FlashSleep, MainClockConfig, MainClockSource, VddDriveStrength, VddLevel,
};
use embassy_mcxa::clocks::{PoweredClock, WakeGuard};
use embassy_mcxa::gpio::{self, Async, Input, Pull};
use embassy_mcxa::{bind_interrupts, peripherals};
use embassy_time::{Duration, Instant, Timer};
use hal::gpio::{DriveStrength, Level, Output, SlewRate};
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(struct Irqs {
    GPIO3 => gpio::InterruptHandler<peripherals::GPIO3>;
});

#[cfg_attr(
    feature = "executor-platform",
    embassy_executor::main(executor = "embassy_mcxa::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "executor-platform"), embassy_executor::main)]
async fn main(spawner: Spawner) {
    // Do a short delay in order to allow for us to attach the debugger/start
    // a flash in case some setting below is wrong, and the CPU gets stuck
    // in deep sleep with debugging disabled.
    defmt::info!("Pre-power delay!");
    // Experimentally: about 5-6s or so.
    cortex_m::asm::delay(48_000_000);
    defmt::info!("Pre-power delay complete!");
    let mut cfg = hal::config::Config::default();

    // Enable 192M FIRC
    let mut fcfg = FircConfig::default();
    fcfg.frequency = FircFreqSel::Mhz192;
    fcfg.power = PoweredClock::NormalEnabledDeepSleepDisabled;
    fcfg.fro_hf_enabled = true;
    fcfg.clk_hf_fundamental_enabled = false;
    fcfg.fro_hf_div = Some(const { Div8::from_divisor(192).unwrap() });
    cfg.clock_cfg.firc = Some(fcfg);

    // Enable 12M osc to use as ostimer clock
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = None;
    cfg.clock_cfg.sirc.power = PoweredClock::AlwaysEnabled;

    // Disable 16K osc
    cfg.clock_cfg.fro16k = None;

    // Disable external osc
    cfg.clock_cfg.sosc = None;

    // Disable PLL
    cfg.clock_cfg.spll = None;

    // Feed core from 180M osc
    cfg.clock_cfg.main_clock = MainClockConfig {
        source: MainClockSource::FircHfRoot,
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
        ahb_clk_div: Div8::no_div(),
    };

    // Set the core in high power active mode
    cfg.clock_cfg.vdd_power.active_mode.level = VddLevel::OverDriveMode;
    cfg.clock_cfg.vdd_power.active_mode.drive = VddDriveStrength::Normal;
    // Set the core in low power sleep mode
    cfg.clock_cfg.vdd_power.low_power_mode.level = VddLevel::MidDriveMode;
    cfg.clock_cfg.vdd_power.low_power_mode.drive = VddDriveStrength::Low { enable_bandgap: false };

    // Set "deep sleep" mode
    cfg.clock_cfg.vdd_power.core_sleep = CoreSleep::DeepSleep;

    // Set flash doze, allowing internal flash clocks to be gated on sleep
    cfg.clock_cfg.vdd_power.flash_sleep = FlashSleep::FlashDoze;

    let p = hal::init(cfg);

    #[cfg(feature = "executor-platform")]
    embassy_mcxa::executor::set_executor_debug_gpio(p.P2_3);

    let mut pin = p.P4_2;
    let mut clkout = p.CLKOUT;
    const K250_CONFIG: clkout::Config = clkout::Config {
        // 192MHz / 192 -> 1MHz
        sel: ClockOutSel::FroHfDiv,
        // 1MHz / 4 -> 250kHz
        div: const { Div4::from_divisor(4).unwrap() },
        level: PoweredClock::NormalEnabledDeepSleepDisabled,
    };

    // We create a clkout peripheral so that we can view the activity of the clock
    // in a logic analyzer. We use `new_unchecked` here, which doesn't participate
    // in verifying that the clock sources are always valid, and does not take
    // a WaitGuard token, which would prevent us from entering deep sleep.
    let _clock_out = unsafe { ClockOut::new_unchecked(clkout.reborrow(), pin.reborrow(), K250_CONFIG) };

    defmt::info!("Going to sleep shortly...");
    cortex_m::asm::delay(48_000_000 / 4);

    let mut red = Output::new(p.P2_14, Level::High, DriveStrength::Normal, SlewRate::Slow);

    // Setup a second LED, and use the button labeled "WAKEUP" as an input source
    let blue = Output::new(p.P2_23, Level::High, DriveStrength::Normal, SlewRate::Slow);
    let btn = Input::new_async(p.P3_17, Irqs, Pull::Up);
    spawner.spawn(press_toggler(btn, blue).unwrap());

    loop {
        // We sleep a little longer than usual, to make it easier to distinguish between
        // timer wakeups and GPIO wakeups.
        Timer::after_millis(4900).await;

        // For the 100ms the LED is low, we manually take a wakeguard to prevent the
        // system from returning to deep sleep, which drastically increases our power
        // usage, but also prevents these clock sources from being disabled automatically.
        red.set_low();
        let _wg = WakeGuard::new();
        let start = Instant::now();

        // for the first 20ms, busyloop
        while start.elapsed() < Duration::from_millis(20) {}

        // then wfe sleep for 80ms
        Timer::after_millis(80).await;

        red.set_high();
        // The WakeGuard is dropped here before returning to the top of the loop. When this
        // happens, we will enter deep sleep automatically on our next .await.
    }
}

/// A task that toggles the given LED every time the button falls low. No fancy
/// debouncing, but useful to look at with the scope and the custom executor
/// debug pin (or a power analyzer) to measure the time delta between the
/// WAKEUP pin going low and the executor resuming from deep sleep.
///
/// At the time of writing, it takes us roughly 20us from the GPIO falling to
/// the assertion of the executor debug gpio pin. This button can be observed
/// using the mikro-bus header pin labeled "RST".
#[embassy_executor::task]
async fn press_toggler(mut button: Input<'static, Async>, mut led: Output<'static>) {
    loop {
        button.wait_for_low().await;
        led.toggle();
        button.wait_for_high().await;
    }
}

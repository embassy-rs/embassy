//! Timer and SW2 wake from deep sleep; SW2 toggles the blue LED.
//! Requires `--no-default-features --features executor-platform`.

#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_mcxa as hal;
use embassy_mcxa::clkout::{self, ClockOut, ClockOutSel, Div4};
use embassy_mcxa::clocks::config::{
    CoreSleep, Div8, FircConfig, FircFreqSel, FlashSleep, MainClockConfig, MainClockSource, VddDriveStrength, VddLevel,
};
use embassy_mcxa::clocks::{PoweredClock, WakeGuard};
use embassy_mcxa::gpio::{self, Async, Input, Pull};
use embassy_mcxa::{bind_interrupts, peripherals};
use embassy_time::{Duration, Instant, Timer};
use hal::gpio::{DriveStrength, Level, Output, SlewRate};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    GPIO3 => gpio::InterruptHandler<peripherals::GPIO3>;
});

#[cfg_attr(
    feature = "executor-platform",
    embassy_executor::main(executor = "embassy_mcxa::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "executor-platform"), embassy_executor::main)]
async fn main(spawner: Spawner) {
    // Allow debugger recovery before deep sleep.
    defmt::info!("Pre-power delay!");
    cortex_m::asm::delay(48_000_000);
    defmt::info!("Pre-power delay complete!");
    let mut cfg = hal::config::Config::default();

    let mut fcfg = FircConfig::default();
    fcfg.frequency = FircFreqSel::Mhz192;
    fcfg.power = PoweredClock::NormalEnabledDeepSleepDisabled;
    fcfg.fro_hf_enabled = true;
    fcfg.clk_hf_fundamental_enabled = false;
    fcfg.fro_hf_div = Some(const { Div8::from_divisor(192).unwrap() });
    cfg.clock_cfg.firc = Some(fcfg);

    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = None;
    cfg.clock_cfg.sirc.power = PoweredClock::AlwaysEnabled;
    cfg.clock_cfg.fro16k = None;
    cfg.clock_cfg.sosc = None;
    cfg.clock_cfg.spll = None;
    cfg.clock_cfg.main_clock = MainClockConfig {
        source: MainClockSource::FircHfRoot,
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
        ahb_clk_div: Div8::no_div(),
    };

    cfg.clock_cfg.vdd_power.active_mode.level = VddLevel::OverDriveMode;
    cfg.clock_cfg.vdd_power.active_mode.drive = VddDriveStrength::Normal;
    cfg.clock_cfg.vdd_power.low_power_mode.level = VddLevel::MidDriveMode;
    cfg.clock_cfg.vdd_power.low_power_mode.drive = VddDriveStrength::Low { enable_bandgap: false };
    cfg.clock_cfg.vdd_power.core_sleep = CoreSleep::DeepSleep;
    cfg.clock_cfg.vdd_power.flash_sleep = FlashSleep::FlashDoze;

    let p = hal::init(cfg);

    #[cfg(feature = "executor-platform")]
    embassy_mcxa::executor::set_executor_debug_gpio(p.P2_3);

    let mut pin = p.P4_2;
    let mut clkout = p.CLKOUT;
    const K250_CONFIG: clkout::Config = clkout::Config {
        sel: ClockOutSel::FroHfDiv,
        // 192 MHz / 192 / 4 = 250 kHz.
        div: const { Div4::from_divisor(4).unwrap() },
        level: PoweredClock::NormalEnabledDeepSleepDisabled,
    };

    // Observe clock gating without holding a WakeGuard.
    let _clock_out = unsafe { ClockOut::new_unchecked(clkout.reborrow(), pin.reborrow(), K250_CONFIG) };

    defmt::info!("Going to sleep shortly...");
    cortex_m::asm::delay(48_000_000 / 4);

    let mut red = Output::new(p.P2_14, Level::High, DriveStrength::Normal, SlewRate::Slow);
    let blue = Output::new(p.P2_23, Level::High, DriveStrength::Normal, SlewRate::Slow);
    let btn = Input::new_async(p.P3_17, Irqs, Pull::Up);
    spawner.spawn(press_toggler(btn, blue).unwrap());

    loop {
        Timer::after_millis(4900).await;
        red.set_low();
        let _wg = WakeGuard::new();
        let start = Instant::now();

        // Busy for 20 ms, then gated WFE for 80 ms.
        while start.elapsed() < Duration::from_millis(20) {}
        Timer::after_millis(80).await;
        red.set_high();
    }
}

#[embassy_executor::task]
async fn press_toggler(mut button: Input<'static, Async>, mut led: Output<'static>) {
    loop {
        button.wait_for_low().await;
        led.toggle();
        button.wait_for_high().await;
    }
}

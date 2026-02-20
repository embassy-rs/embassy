//! LPUART DMA example for MCXA276.
//!
//! This example demonstrates using DMA for UART TX and RX operations.
//! It sends a message using DMA, then waits for 16 characters to be received
//! via DMA and echoes them back.
//!
//! The DMA request sources are automatically derived from the LPUART instance type.
//! DMA clock/reset/init is handled automatically by the HAL.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clocks::PoweredClock;
use embassy_mcxa::gpio::{DriveStrength, Level, Output, SlewRate};
use embassy_mcxa::{bind_interrupts, lpuart};
use embassy_mcxa::clocks::config::{CoreSleep, Div8, FlashSleep, MainClockConfig, MainClockSource, VddDriveStrength, VddLevel};
use embassy_mcxa::lpuart::{Config, LpuartBbqRx};
use embassy_time::Timer;
use static_cell::ConstStaticCell;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(struct Irqs {
    LPUART3 => lpuart::BbqInterruptHandler::<hal::peripherals::LPUART3>;
});

static TX_BUF: ConstStaticCell<[u8; 4096]> = ConstStaticCell::new([0u8; 4096]);

#[cfg_attr(
    feature = "custom-executor",
    embassy_executor::main(executor = "embassy_mcxa::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "custom-executor"), embassy_executor::main)]
async fn main(_spawner: Spawner) {
    // Do a short delay in order to allow for us to attach the debugger/start
    // a flash in case some setting below is wrong, and the CPU gets stuck
    // in deep sleep with debugging disabled.
    defmt::info!("Pre-power delay!");
    // Experimentally: about 5-6s or so.
    cortex_m::asm::delay(45_000_000);
    defmt::info!("Pre-power delay complete!");

    let mut cfg = hal::config::Config::default();

    // Disable 45M osc
    cfg.clock_cfg.firc = None;

    // Enable 12M osc to use as core clock
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
    cfg.clock_cfg.sirc.power = PoweredClock::AlwaysEnabled;

    // Disable 16K osc
    cfg.clock_cfg.fro16k = None;

    // Disable external osc
    cfg.clock_cfg.sosc = None;

    // Disable PLL
    cfg.clock_cfg.spll = None;

    // Feed core from 12M osc
    cfg.clock_cfg.main_clock = MainClockConfig {
        source: MainClockSource::SircFro12M,
        power: PoweredClock::AlwaysEnabled,
        ahb_clk_div: Div8::no_div(),
    };

    // Set lowest core power, disable bandgap LDO reference
    cfg.clock_cfg.vdd_power.active_mode.level = VddLevel::MidDriveMode;
    cfg.clock_cfg.vdd_power.low_power_mode.level = VddLevel::MidDriveMode;
    cfg.clock_cfg.vdd_power.active_mode.drive = VddDriveStrength::Low { enable_bandgap: false };
    cfg.clock_cfg.vdd_power.low_power_mode.drive = VddDriveStrength::Low { enable_bandgap: false };

    // Set "deep sleep" mode
    cfg.clock_cfg.vdd_power.core_sleep = CoreSleep::WfeUngated;

    // Set flash doze, allowing internal flash clocks to be gated on sleep
    // cfg.clock_cfg.vdd_power.flash_sleep = FlashSleep::FlashDoze;

    let p = hal::init(cfg);

    defmt::info!("LPUART DMA example starting...");

    // Create UART configuration
    let config = Config {
        baudrate_bps: 115_200,
        power: PoweredClock::AlwaysEnabled,
        ..Default::default()
    };

    let tx_buf = TX_BUF.take();

    // Create UART instance with DMA channels
    let mut lpuart = LpuartBbqRx::new(
        p.LPUART3,
        p.P4_2,
        Irqs,
        tx_buf,
        p.DMA_CH0,
        config,
    ).unwrap();

    // // let mut gpio = Output::new(p.P4_2, Level::Low, DriveStrength::Normal, SlewRate::Slow);

    let mut to_send = [0u8; 256];
    to_send.iter_mut().enumerate().for_each(|(i, b)| *b = i as u8);

    Timer::after_millis(1000).await;

    let mut red = Output::new(p.P3_18, Level::High, DriveStrength::Normal, SlewRate::Fast);

    // #[cfg(feature = "custom-executor")]
    // embassy_mcxa::executor::set_executor_debug_gpio(p.P4_2);

    loop {
        // let mut window = to_send.as_slice();
        // while !window.is_empty() {
        //     let sent = lpuart.write(window).await.unwrap();
        //     let (_now, later) = window.split_at(sent);
        //     window = later;
        // }
        Timer::after_millis(3000).await;
        red.toggle();
    }

}

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
use embassy_mcxa::clocks::config::{
    CoreSleep, Div8, FircConfig, FircFreqSel, FlashSleep, MainClockConfig, MainClockSource, VddDriveStrength, VddLevel,
};
use embassy_mcxa::clocks::periph_helpers::LpuartClockSel;
use embassy_mcxa::gpio::{DriveStrength, Level, Output, SlewRate};
use embassy_mcxa::lpuart::{Config, LpuartBbqRx};
use embassy_mcxa::{bind_interrupts, lpuart};
use static_cell::ConstStaticCell;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(struct Irqs {
    LPUART3 => lpuart::BbqInterruptHandler::<hal::peripherals::LPUART3>;
});

const SIZE: usize = 4096;
static RX_BUF: ConstStaticCell<[u8; SIZE]> = ConstStaticCell::new([0u8; SIZE]);

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
    let mut fcfg = FircConfig::default();
    fcfg.frequency = FircFreqSel::Mhz180;
    fcfg.power = PoweredClock::NormalEnabledDeepSleepDisabled;
    fcfg.fro_hf_enabled = true;
    fcfg.clk_45m_enabled = false;
    fcfg.fro_hf_div = Some(const { Div8::from_divisor(4).unwrap() });
    cfg.clock_cfg.firc = Some(fcfg);

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
        source: MainClockSource::FircHfRoot,
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
        ahb_clk_div: Div8::no_div(),
    };

    // Set lowest core power, disable bandgap LDO reference
    cfg.clock_cfg.vdd_power.active_mode.level = VddLevel::OverDriveMode;
    cfg.clock_cfg.vdd_power.low_power_mode.level = VddLevel::MidDriveMode;
    cfg.clock_cfg.vdd_power.active_mode.drive = VddDriveStrength::Normal;
    cfg.clock_cfg.vdd_power.low_power_mode.drive = VddDriveStrength::Low { enable_bandgap: false };

    // Set "deep sleep" mode
    // cfg.clock_cfg.vdd_power.core_sleep = CoreSleep::DeepSleep; // FIX DELAY
    cfg.clock_cfg.vdd_power.core_sleep = CoreSleep::WfeUngated; // FIX DELAY

    // Set flash doze, allowing internal flash clocks to be gated on sleep
    cfg.clock_cfg.vdd_power.flash_sleep = FlashSleep::FlashDoze;

    let p = hal::init(cfg);

    defmt::info!("LPUART DMA example starting...");

    // Create UART configuration
    let config = Config {
        baudrate_bps: 4_000_000,
        power: PoweredClock::NormalEnabledDeepSleepDisabled,
        rx_fifo_watermark: 0,
        source: LpuartClockSel::FroHfDiv,
        ..Default::default()
    };

    let rx_buf = RX_BUF.take();

    // Create UART instance with DMA channels
    let mut lpuart = LpuartBbqRx::new(p.LPUART3, p.P4_2, Irqs, rx_buf, p.DMA_CH0, config).unwrap();

    // // let mut gpio = Output::new(p.P4_2, Level::Low, DriveStrength::Normal, SlewRate::Slow);
    let mut red = Output::new(p.P3_18, Level::High, DriveStrength::Normal, SlewRate::Fast);

    // #[cfg(feature = "custom-executor")]
    embassy_mcxa::executor::set_executor_debug_gpio(p.P1_0);

    let mut streak = 0;
    let mut idx = 0u8;
    let mut buf = [0u8; 1024];

    loop {
        // let used = match lpuart.read(&mut buf).with_timeout(Duration::from_millis(1500)).await {
        //     Ok(Ok(used)) => used,
        //     Ok(Err(_)) => panic!(),
        //     Err(_) => {
        //         defmt::info!("Timeout!");
        //         continue
        //     },
        // };
        defmt::info!("waiting!");
        let used = lpuart.read(&mut buf).await.unwrap();
        for byte in &buf[..used] {
            if *byte == idx {
                streak += 1;
                idx = idx.wrapping_add(1);
            } else {
                defmt::error!("{=u8} != {=u8}", idx, *byte);
                streak = 0;
                idx = (*byte).wrapping_add(1);
            }
        }
        defmt::info!("Got {=usize}, Streak: {=usize}", used, streak);
    }
}

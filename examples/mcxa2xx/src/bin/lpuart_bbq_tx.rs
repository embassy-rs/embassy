//! LPUART BBQueue example for MCXA276.
//!
//! This scenario is meant to be coupled with another device receiving using the
//! `lpuart_bbq_rx` example. In this scenario:
//!
//! * We send a short 16-byte "knock" packet
//! * We wait 1ms for the other device to wake up
//! * We send a larger 768-byte "data" packet
//!
//! See `lpuart_bbq_rx` for more information. This half is not sleepy, it just sends
//! regularly without concern for power consumption.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_mcxa::clocks::PoweredClock;
use embassy_mcxa::clocks::config::{
    CoreSleep, Div8, FircConfig, FircFreqSel, FlashSleep, MainClockConfig, MainClockSource, VddDriveStrength, VddLevel,
};
use embassy_mcxa::clocks::periph_helpers::LpuartClockSel;
use embassy_mcxa::dma::DmaChannel;
use embassy_mcxa::gpio::{DriveStrength, Level, Output, SlewRate};
use embassy_mcxa::lpuart::{BbqConfig, BbqHalfParts, LpuartBbqTx};
use embassy_mcxa::{bind_interrupts, lpuart};
use embassy_time::Timer;
use static_cell::ConstStaticCell;
use {defmt_rtt as _, embassy_mcxa as hal, panic_probe as _};

bind_interrupts!(struct Irqs {
    LPUART3 => lpuart::BbqInterruptHandler::<hal::peripherals::LPUART3>;
});

const SIZE: usize = 4096;
static TX_BUF: ConstStaticCell<[u8; SIZE]> = ConstStaticCell::new([0u8; SIZE]);

#[cfg_attr(
    feature = "custom-executor",
    embassy_executor::main(executor = "embassy_mcxa::executor::Executor", entry = "cortex_m_rt::entry")
)]
#[cfg_attr(not(feature = "custom-executor"), embassy_executor::main)]
async fn main(_spawner: Spawner) {
    let mut cfg = hal::config::Config::default();

    // Enable 180MHz clock source
    let mut fcfg = FircConfig::default();
    fcfg.frequency = FircFreqSel::Mhz180;
    fcfg.power = PoweredClock::NormalEnabledDeepSleepDisabled;
    fcfg.fro_hf_enabled = true;
    fcfg.clk_45m_enabled = false;
    fcfg.fro_hf_div = Some(const { Div8::from_divisor(4).unwrap() });
    cfg.clock_cfg.firc = Some(fcfg);

    // Enable 12M osc
    cfg.clock_cfg.sirc.fro_12m_enabled = true;
    cfg.clock_cfg.sirc.fro_lf_div = Some(Div8::no_div());
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

    // We don't sleep, set relatively high power
    cfg.clock_cfg.vdd_power.active_mode.level = VddLevel::OverDriveMode;
    cfg.clock_cfg.vdd_power.low_power_mode.level = VddLevel::MidDriveMode;
    cfg.clock_cfg.vdd_power.active_mode.drive = VddDriveStrength::Normal;
    cfg.clock_cfg.vdd_power.low_power_mode.drive = VddDriveStrength::Low { enable_bandgap: false };

    // Set "never sleep" mode
    cfg.clock_cfg.vdd_power.core_sleep = CoreSleep::WfeUngated;

    // Set flash doze, allowing internal flash clocks to be gated on sleep
    cfg.clock_cfg.vdd_power.flash_sleep = FlashSleep::FlashDoze;

    let p = hal::init(cfg);

    defmt::info!("LPUART DMA example starting...");

    // Create UART configuration
    let mut config = BbqConfig::default();
    config.baudrate_bps = 4_000_000;
    config.power = PoweredClock::NormalEnabledDeepSleepDisabled;
    config.source = LpuartClockSel::FroHfDiv;

    let tx_buf = TX_BUF.take();

    // Create UART instance with DMA channels
    let dma_channel = DmaChannel::new(p.DMA_CH0);
    let parts = BbqHalfParts::new_tx_half(p.LPUART3, Irqs, p.P4_5, tx_buf, dma_channel);
    let mut lpuart = LpuartBbqTx::new(parts, config).unwrap();
    let mut to_knock = [0u8; 16];
    let mut to_send = [0u8; 768];
    to_knock.iter_mut().for_each(|b| *b = 0xFF);
    to_send.iter_mut().enumerate().for_each(|(i, b)| *b = (i as u8) & 0x7F);

    Timer::after_millis(1000).await;

    let mut red = Output::new(p.P3_18, Level::High, DriveStrength::Normal, SlewRate::Fast);

    #[cfg(feature = "custom-executor")]
    embassy_mcxa::executor::set_executor_debug_gpio(p.P4_2);

    loop {
        // Send a small 16-byte "knock" packet in case the other device is sleeping
        defmt::info!("knock!");
        let mut window = to_knock.as_slice();
        while !window.is_empty() {
            let sent = lpuart.write(window).await.unwrap();
            let (_now, later) = window.split_at(sent);
            window = later;
        }
        defmt::info!("Knocked, flushing...");
        lpuart.flush().await;
        // Wait a small amount of time AFTER knocking to allow the device to wake up
        Timer::after_millis(1).await;

        defmt::info!("Sending!");
        let mut window = to_send.as_slice();
        while !window.is_empty() {
            let sent = lpuart.write(window).await.unwrap();
            let (_now, later) = window.split_at(sent);
            window = later;
        }
        defmt::info!("Sent, flushing...");
        lpuart.flush().await;

        defmt::info!("Flushed.");

        // Now wait a bit to let the other device go back to sleep
        Timer::after_millis(3000).await;
        red.toggle();
    }
}

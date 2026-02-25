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
use embassy_mcxa::dma::DmaChannel;
use embassy_mcxa::gpio::{DriveStrength, Input, Level, Output, Pull, SlewRate};
use embassy_mcxa::lpuart::{BbqConfig, BbqHalfParts, LpuartBbqRx};
use embassy_mcxa::{bind_interrupts, lpuart};
use embassy_time::{WithTimeout, Duration};
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
    let mut cfg = hal::config::Config::default();

    // Enable 180MHz clock source
    let mut fcfg = FircConfig::default();
    fcfg.frequency = FircFreqSel::Mhz180;
    fcfg.power = PoweredClock::NormalEnabledDeepSleepDisabled;
    fcfg.fro_hf_enabled = true;
    fcfg.clk_45m_enabled = false;
    fcfg.fro_hf_div = Some(Div8::no_div());
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
    cfg.clock_cfg.vdd_power.active_mode.drive = VddDriveStrength::Normal;
    cfg.clock_cfg.vdd_power.low_power_mode.level = VddLevel::MidDriveMode;
    cfg.clock_cfg.vdd_power.low_power_mode.drive = VddDriveStrength::Low { enable_bandgap: false };

    // Useful for measuring prints, but power will be higher.
    cfg.clock_cfg.vdd_power.core_sleep = CoreSleep::DeepSleep;

    // Set flash doze, allowing internal flash clocks to be gated on sleep
    cfg.clock_cfg.vdd_power.flash_sleep = FlashSleep::FlashDoze;

    let p = hal::init(cfg);

    defmt::info!("LPUART DMA example starting...");

    // Create UART configuration
    let mut config = BbqConfig::default();
    config.baudrate_bps = 4_000_000;
    config.power = PoweredClock::NormalEnabledDeepSleepDisabled;
    config.source = LpuartClockSel::FroHfDiv;

    let rx_buf = RX_BUF.take();

    // Create UART instance with DMA channels
    let dma = DmaChannel::new(p.DMA_CH0);
    let mut parts = BbqHalfParts::new_rx_half(p.LPUART3, Irqs, p.P4_2, rx_buf, dma);
    let mut red = Output::new(p.P3_18, Level::High, DriveStrength::Normal, SlewRate::Fast);
    let mut debug = Output::new(p.P3_28, Level::High, DriveStrength::Normal, SlewRate::Fast);

    #[cfg(feature = "custom-executor")]
    embassy_mcxa::executor::set_executor_debug_gpio(p.P1_0);

    loop {
        defmt::info!("Waiting for falling edge");
        red.set_high();
        debug.set_high();
        {
            let mut input = Input::new(parts.pin(), Pull::Up);
            input.wait_for_low().await;
        }

        let mut lpuart = LpuartBbqRx::new(parts, config).unwrap();
        red.set_low();
        debug.set_low();
        defmt::info!("got wake, listening");

        let mut streak = 0;
        let mut idx = 0u8;
        let mut buf = [0u8; 64];
        let mut got = 0;
        let mut dummies = 0;

        'wake: loop {
            match lpuart.read(&mut buf).with_timeout(Duration::from_millis(100)).await {
                Ok(res) => {
                    let used = res.unwrap();
                    for byte in &buf[..used] {
                        if *byte > 0x7F {
                            dummies += 1;
                        } else if *byte == idx {
                            streak += 1;
                            idx = idx.wrapping_add(1) & 0x7F;
                        } else {
                            streak = 0;
                            idx = (*byte).wrapping_add(1) & 0x7F;
                        }
                    }
                    got += used;
                }
                Err(_) => {
                    break 'wake;
                }
            }
        }

        defmt::info!("Going to sleep, got: {=usize}, streak: {=usize}, dummies: {=usize}", got, streak, dummies);

        parts = lpuart.teardown();
    }
}

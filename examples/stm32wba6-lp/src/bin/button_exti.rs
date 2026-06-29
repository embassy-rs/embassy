//! EXTI button example with STOP mode.
//!
//! The MCU enters STOP mode while waiting for a button press on PC13.
//! The EXTI line wakes the core from STOP — no polling required.
//! Current draw drops to ~1 µA between presses.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::exti::{self, ExtiInput};
use embassy_stm32::gpio::Pull;
use embassy_stm32::rcc::*;
use embassy_stm32::{bind_interrupts, interrupt};
use {defmt_rtt as _, panic_probe as _};

const MEASURE_POWER: bool = true;

bind_interrupts!(
    pub struct Irqs {
        EXTI13 => exti::InterruptHandler<interrupt::typelevel::EXTI13>;
    }
);

#[embassy_executor::main(executor = "embassy_stm32::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();

    // HSI 16 MHz as sysclk.
    config.rcc.sys = Sysclk::Hsi;
    config.rcc.ahb5_pre = AHB5Prescaler::Div1;

    // LSI 32 kHz for the RTC — the time driver uses the RTC wakeup
    // alarm to bring the core back from STOP mode.
    config.rcc.ls = LsConfig {
        rtc: RtcClockSource::Lsi,
        lsi: true,
        lse: None,
    };

    // SAI1 clock mux defaults to PLL1_P — override to HSI since
    // PLL1 is not configured in this demo.
    config.rcc.mux.sai1sel = Sai1sel::Hsi;

    if MEASURE_POWER {
        // Lowest leakage profile for current measurements.
        config.enable_debug_during_sleep = false;
        config.min_stop_pause = embassy_time::Duration::from_millis(100);
    } else {
        // Easier debug/reflash profile, but with higher STOP current.
        config.enable_debug_during_sleep = true;
        config.min_stop_pause = embassy_time::Duration::from_millis(20);
    }

    let p = embassy_stm32::init(config);
    info!("Hello from STM32WBA6 low-power button example!");
    info!(
        "Power profile: {}",
        if MEASURE_POWER { "measurement" } else { "debug-friendly" }
    );
    info!("Press the USER button (PC13)...");

    let mut button = ExtiInput::new(p.PC13, p.EXTI13, Pull::Up, Irqs);

    loop {
        // MCU enters STOP while waiting for the falling edge.
        button.wait_for_falling_edge().await;
        info!("Pressed!");

        button.wait_for_rising_edge().await;
        info!("Released!");
    }
}

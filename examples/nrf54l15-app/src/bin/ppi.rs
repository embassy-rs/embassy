//! PPI and PPIB (PPI bridge) demo:
//! - PPI: a TIMER compare event toggles LED1 (both in the PERI domain).
//! - PPIB: a button in the MCU domain toggles LED2 in the PERI domain, forwarded
//!   across the PPIB30 <-> PPIB22 bridge as one DPPI hop per domain.

#![no_std]
#![no_main]

use core::future::pending;

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, OutputDrive, Pull};
use embassy_nrf::gpiote::{InputChannel, InputChannelPolarity, OutputChannel, OutputChannelPolarity};
use embassy_nrf::ppi::Ppi;
use embassy_nrf::ppib::Ppib;
use embassy_nrf::timer::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("Starting!");

    // GPIOTE20 drives P1 pins (PERI domain), GPIOTE30 drives P0 pins (MCU domain).
    let led1 = OutputChannel::new(
        p.GPIOTE20_CH0,
        p.P1_10,
        Level::Low,
        OutputDrive::Standard,
        OutputChannelPolarity::Toggle,
    );
    let led3 = OutputChannel::new(
        p.GPIOTE20_CH1,
        p.P1_14,
        Level::Low,
        OutputDrive::Standard,
        OutputChannelPolarity::Toggle,
    );
    let button = InputChannel::new(p.GPIOTE30_CH0, p.P0_04, Pull::Up, InputChannelPolarity::HiToLo);

    // Periodic CC[0] compare event, every 0.5 s at the default 1 MHz.
    let timer = Timer::new(p.TIMER20);
    timer.cc(0).write(500_000);
    timer.cc(0).short_compare_clear();
    timer.start();

    let mut blink = Ppi::new_one_to_one(p.PPI20_CH0, timer.cc(0).event_compare(), led1.task_out());
    blink.enable();

    let bridge = Ppib::new(p.PPIB30_CH0, p.PPIB22_CH0);
    let mut src = Ppi::new_one_to_one(p.PPI30_CH0, button.event_in(), bridge.task());
    src.enable();
    let mut dst = Ppi::new_one_to_one(p.PPI20_CH1, bridge.event(), led3.task_out());
    dst.enable();

    info!("LED1 blinks off the timer; press the button to toggle LED2.");

    // Keep the drivers alive — dropping them tears the wiring down.
    pending::<()>().await;
}

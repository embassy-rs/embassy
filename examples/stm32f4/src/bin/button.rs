#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::*;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use {defmt_rtt as _, panic_probe as _};

/// This example is written for the nucleo-stm32f429zi, with a stm32f429zi chip.
///
/// If you are using a different board or chip, make sure you update the following:
///
/// * [ ] Update .cargo/config.toml with the correct `probe-rs run --chip STM32F429ZITx`chip name.
/// * [ ] Update Cargo.toml to have the correct `embassy-stm32` feature, it is
///       currently `stm32f429zi`.
/// * [ ] If your board has a special clock or power configuration, make sure that it is
///       set up appropriately.
/// * [ ] If your board has different pin mapping, update any pin numbers or peripherals
///       to match your schematic
///
/// If you are unsure, please drop by the Embassy Matrix chat for support, and let us know:
///
/// * Which example you are trying to run
/// * Which chip and board you are using
///
/// Embassy Chat: https://matrix.to/#/#embassy-rs:matrix.org
#[entry]
fn main() -> ! {
    info!("Hello World!");

    let p = embassy_stm32::init(Default::default());

    let button = Input::new(p.PC13, Pull::Down);
    let mut led1 = Output::new(p.PB0, Level::High, Speed::Low);
    let _led2 = Output::new(p.PB7, Level::High, Speed::Low);
    let mut led3 = Output::new(p.PB14, Level::High, Speed::Low);

    loop {
        if button.is_high() {
            info!("high");
            led1.set_high();
            led3.set_low();
        } else {
            info!("low");
            led1.set_low();
            led3.set_high();
        }
    }
}

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::time::khz;
use embassy_stm32::timer::input_capture::{CapturePin, InputCapture};
use embassy_stm32::timer::{self, Channel};
use embassy_stm32::{bind_interrupts, peripherals};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

/// Connect PB2 and PB10 with a 1k Ohm resistor

#[embassy_executor::task]
async fn blinky(led: peripherals::PB2) {
    let mut led = Output::new(led, Level::High, Speed::Low);

    loop {
        info!("high");
        led.set_high();
        Timer::after_millis(300).await;

        info!("low");
        led.set_low();
        Timer::after_millis(300).await;
    }
}

bind_interrupts!(struct Irqs {
    TIM2 => timer::CaptureCompareInterruptHandler<peripherals::TIM2>;
});

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
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");

    unwrap!(spawner.spawn(blinky(p.PB2)));

    let ch3 = CapturePin::new_ch3(p.PB10, Pull::None);
    let mut ic = InputCapture::new(p.TIM2, None, None, Some(ch3), None, Irqs, khz(1000), Default::default());

    loop {
        info!("wait for risign edge");
        ic.wait_for_rising_edge(Channel::Ch3).await;

        let capture_value = ic.get_capture_value(Channel::Ch3);
        info!("new capture! {}", capture_value);
    }
}

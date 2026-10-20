#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;

use defmt::{assert, info};
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Flex, Input, Level, Output, OutputDrive, Pull};
use embassy_time::Timer;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    let mut a = peri!(p, PIN_A);
    let mut b = peri!(p, PIN_B);

    {
        let input = Input::new(a.reborrow(), Pull::Up);
        let mut output = Output::new(b.reborrow(), Level::Low, OutputDrive::Standard);

        output.set_low();
        assert!(output.is_set_low());
        Timer::after_millis(10).await;
        assert!(input.is_low());

        output.set_high();
        assert!(output.is_set_high());
        Timer::after_millis(10).await;
        assert!(input.is_high());
    }

    // Test is_input / is_output / is_disconnected and set_as_disconnected
    {
        let mut b = Flex::new(b.reborrow());
        assert!(b.is_disconnected());
        assert!(!b.is_input());
        assert!(!b.is_output());

        b.set_as_input(Pull::Up);
        assert!(b.is_input());
        assert!(!b.is_output());
        assert!(!b.is_disconnected());

        let mut a = Flex::new(a.reborrow());
        a.set_low();
        a.set_as_output(OutputDrive::Standard);
        assert!(a.is_output());
        assert!(!a.is_input());
        assert!(!a.is_disconnected());
        Timer::after_millis(10).await;
        assert!(b.is_low());

        a.set_as_input_output(Pull::None, OutputDrive::Standard0Disconnect1);
        assert!(a.is_input());
        assert!(a.is_output());
        assert!(!a.is_disconnected());
        Timer::after_millis(10).await;
        assert!(a.is_low());
        assert!(b.is_low());

        // A disconnected pin doesn't drive the line.
        a.set_as_disconnected();
        assert!(a.is_disconnected());
        assert!(!a.is_input());
        assert!(!a.is_output());
        Timer::after_millis(10).await;
        assert!(b.is_high());

        // set_as_input must connect the input buffer again.
        a.set_as_input(Pull::None);
        assert!(a.is_input());
        assert!(!a.is_disconnected());
        Timer::after_millis(10).await;
        assert!(a.is_high());

        a.set_as_disconnected();
        a.set_as_output(OutputDrive::Standard);
        assert!(a.is_output());
        assert!(!a.is_disconnected());
        Timer::after_millis(10).await;
        assert!(b.is_low());
    }

    info!("Test OK");
    cortex_m::asm::bkpt();
}

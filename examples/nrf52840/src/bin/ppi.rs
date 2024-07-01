#![no_std]
#![no_main]

use core::future::pending;

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pull};
use embassy_nrf::gpiote::{self, InputChannel, InputChannelPolarity};
use embassy_nrf::ppi::Ppi;
use gpiote::{OutputChannel, OutputChannelPolarity};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("Starting!");

    let button1 = InputChannel::new(
        p.GPIOTE_CH0,
        Input::new(p.P0_11, Pull::Up),
        InputChannelPolarity::HiToLo,
    );
    let button2 = InputChannel::new(
        p.GPIOTE_CH1,
        Input::new(p.P0_12, Pull::Up),
        InputChannelPolarity::HiToLo,
    );
    let button3 = InputChannel::new(
        p.GPIOTE_CH2,
        Input::new(p.P0_24, Pull::Up),
        InputChannelPolarity::HiToLo,
    );
    let button4 = InputChannel::new(
        p.GPIOTE_CH3,
        Input::new(p.P0_25, Pull::Up),
        InputChannelPolarity::HiToLo,
    );

    let led1 = OutputChannel::new(
        p.GPIOTE_CH4,
        Output::new(p.P0_13, Level::Low, OutputDrive::Standard),
        OutputChannelPolarity::Toggle,
    );

    let led2 = OutputChannel::new(
        p.GPIOTE_CH5,
        Output::new(p.P0_14, Level::Low, OutputDrive::Standard),
        OutputChannelPolarity::Toggle,
    );

    let mut ppi = Ppi::new_one_to_one(p.PPI_CH0, button1.event_in(), led1.task_out());
    ppi.enable();

    let mut ppi = Ppi::new_one_to_one(p.PPI_CH1, button2.event_in(), led1.task_clr());
    ppi.enable();

    let mut ppi = Ppi::new_one_to_one(p.PPI_CH2, button3.event_in(), led1.task_set());
    ppi.enable();

    let mut ppi = Ppi::new_one_to_two(p.PPI_CH3, button4.event_in(), led1.task_out(), led2.task_out());
    ppi.enable();

    info!("PPI setup!");
    info!("Press button 1 to toggle LED 1");
    info!("Press button 2 to turn on LED 1");
    info!("Press button 3 to turn off LED 1");
    info!("Press button 4 to toggle LEDs 1 and 2");

    // Block forever so the above drivers don't get dropped
    pending::<()>().await;
}

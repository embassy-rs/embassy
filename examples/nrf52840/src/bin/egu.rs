//! This example shows the use of the EGU peripheral combined with PPI.
//!
//! It chains events from button -> egu0-trigger0 -> egu0-trigger1 -> led
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::egu::{Egu, TriggerNumber};
use embassy_nrf::gpio::{Level, OutputDrive, Pull};
use embassy_nrf::gpiote::{InputChannel, InputChannelPolarity, OutputChannel, OutputChannelPolarity};
use embassy_nrf::peripherals::{PPI_CH0, PPI_CH1, PPI_CH2};
use embassy_nrf::ppi::Ppi;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    let mut egu1 = Egu::new(p.EGU0);
    let led1 = OutputChannel::new(
        p.GPIOTE_CH0,
        p.P0_13,
        Level::High,
        OutputDrive::Standard,
        OutputChannelPolarity::Toggle,
    );
    let btn1 = InputChannel::new(p.GPIOTE_CH1, p.P0_11, Pull::Up, InputChannelPolarity::LoToHi);

    let trigger0 = egu1.trigger(TriggerNumber::Trigger0);
    let trigger1 = egu1.trigger(TriggerNumber::Trigger1);

    let mut ppi1: Ppi<PPI_CH0, 1, 1> = Ppi::new_one_to_one(p.PPI_CH0, btn1.event_in(), trigger0.task());
    ppi1.enable();

    let mut ppi2: Ppi<PPI_CH1, 1, 1> = Ppi::new_one_to_one(p.PPI_CH1, trigger0.event(), trigger1.task());
    ppi2.enable();

    let mut ppi3: Ppi<PPI_CH2, 1, 1> = Ppi::new_one_to_one(p.PPI_CH2, trigger1.event(), led1.task_out());
    ppi3.enable();

    defmt::info!("Push the button to toggle the LED");
    loop {
        Timer::after(Duration::from_secs(60)).await;
    }
}

#![no_std]
#![no_main]

use defmt::unwrap;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

enum LedState {
    On,
    Off,
}

static CHANNEL: Channel<ThreadModeRawMutex, LedState, 1> = Channel::new();

#[embassy_executor::task]
async fn my_task() {
    loop {
        CHANNEL.send(LedState::On).await;
        Timer::after_secs(1).await;
        CHANNEL.send(LedState::Off).await;
        Timer::after_secs(1).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut led = Output::new(p.P0_13, Level::Low, OutputDrive::Standard);

    unwrap!(spawner.spawn(my_task()));

    loop {
        match CHANNEL.receive().await {
            LedState::On => led.set_high(),
            LedState::Off => led.set_low(),
        }
    }
}

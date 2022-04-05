#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::unwrap;
use embassy::blocking_mutex::raw::ThreadModeRawMutex;
use embassy::channel::channel::Channel;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::Peripherals;

use defmt_rtt as _; // global logger
use panic_probe as _;

enum LedState {
    On,
    Off,
}

static CHANNEL: Channel<ThreadModeRawMutex, LedState, 1> = Channel::new();

#[embassy::task]
async fn my_task() {
    loop {
        CHANNEL.send(LedState::On).await;
        Timer::after(Duration::from_secs(1)).await;
        CHANNEL.send(LedState::Off).await;
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy::main]
async fn main(spawner: Spawner, p: Peripherals) {
    let mut led = Output::new(p.P0_13, Level::Low, OutputDrive::Standard);

    unwrap!(spawner.spawn(my_task()));

    loop {
        match CHANNEL.recv().await {
            LedState::On => led.set_high(),
            LedState::Off => led.set_low(),
        }
    }
}

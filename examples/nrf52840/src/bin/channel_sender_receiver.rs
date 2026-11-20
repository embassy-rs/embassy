#![no_std]
#![no_main]

use defmt::unwrap;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_time::Timer;
use panic_probe as _;
use static_cell::StaticCell;

enum LedState {
    On,
    Off,
}

static CHANNEL: StaticCell<Channel<NoopRawMutex, LedState, 1>> = StaticCell::new();

#[embassy_executor::task]
async fn send_task(sender: Sender<'static, NoopRawMutex, LedState, 1>) {
    loop {
        sender.send(LedState::On).await;
        Timer::after_secs(1).await;
        sender.send(LedState::Off).await;
        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn recv_task(mut led: Output<'static>, receiver: Receiver<'static, NoopRawMutex, LedState, 1>) {
    loop {
        match receiver.receive().await {
            LedState::On => led.set_low(),
            LedState::Off => led.set_high(),
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let channel = CHANNEL.init(Channel::new());

    spawner.spawn(unwrap!(send_task(channel.sender())));
    let led = Output::new(p.P0_13, Level::Low, OutputDrive::Standard);
    spawner.spawn(unwrap!(recv_task(led, channel.receiver())));
}

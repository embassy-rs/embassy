#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use defmt::unwrap;
use embassy::blocking_mutex::raw::NoopRawMutex;
use embassy::channel::mpsc::{self, Channel, Sender, TryRecvError};
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy::util::Forever;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::Peripherals;

enum LedState {
    On,
    Off,
}

static CHANNEL: Forever<Channel<NoopRawMutex, LedState, 1>> = Forever::new();

#[embassy::task(pool_size = 1)]
async fn my_task(sender: Sender<'static, NoopRawMutex, LedState, 1>) {
    loop {
        let _ = sender.send(LedState::On).await;
        Timer::after(Duration::from_secs(1)).await;
        let _ = sender.send(LedState::Off).await;
        Timer::after(Duration::from_secs(1)).await;
    }
}

#[embassy::main]
async fn main(spawner: Spawner, p: Peripherals) {
    let mut led = Output::new(p.P0_13, Level::Low, OutputDrive::Standard);

    let channel = CHANNEL.put(Channel::new());
    let (sender, mut receiver) = mpsc::split(channel);

    unwrap!(spawner.spawn(my_task(sender)));

    // We could just loop on `receiver.recv()` for simplicity. The code below
    // is optimized to drain the queue as fast as possible in the spirit of
    // handling events as fast as possible. This optimization is benign when in
    // thread mode, but can be useful when interrupts are sending messages
    // with the channel having been created via with_critical_sections.
    loop {
        let maybe_message = match receiver.try_recv() {
            m @ Ok(..) => m.ok(),
            Err(TryRecvError::Empty) => receiver.recv().await,
            Err(TryRecvError::Closed) => break,
        };
        match maybe_message {
            Some(LedState::On) => led.set_high(),
            Some(LedState::Off) => led.set_low(),
            _ => (),
        }
    }
}

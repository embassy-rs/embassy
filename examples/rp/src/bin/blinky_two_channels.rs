#![no_std]
#![no_main]
/// This example demonstrates how to access a given pin from more than one embassy task
/// The on-board LED is toggled by two tasks with slightly different periods, leading to the
/// apparent duty cycle of the LED increasing, then decreasing, linearly. The phenomenon is similar
/// to interference and the 'beats' you can hear if you play two frequencies close to one another
/// [Link explaining it](https://www.physicsclassroom.com/class/sound/Lesson-3/Interference-and-Beats)
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{Channel, Sender};
use embassy_time::{Duration, Ticker};
use gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

enum LedState {
    Toggle,
}
static CHANNEL: Channel<ThreadModeRawMutex, LedState, 64> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, Level::High);

    let dt = 100 * 1_000_000;
    let k = 1.003;

    unwrap!(spawner.spawn(toggle_led(CHANNEL.sender(), Duration::from_nanos(dt))));
    unwrap!(spawner.spawn(toggle_led(
        CHANNEL.sender(),
        Duration::from_nanos((dt as f64 * k) as u64)
    )));

    loop {
        match CHANNEL.receive().await {
            LedState::Toggle => led.toggle(),
        }
    }
}

#[embassy_executor::task(pool_size = 2)]
async fn toggle_led(control: Sender<'static, ThreadModeRawMutex, LedState, 64>, delay: Duration) {
    let mut ticker = Ticker::every(delay);
    loop {
        control.send(LedState::Toggle).await;
        ticker.next().await;
    }
}

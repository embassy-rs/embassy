#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::task::Poll;
use defmt::{info, unwrap};
use embassy::executor::Spawner;
use embassy::time::{Duration, Instant, Timer};
use embassy_nrf::Peripherals;

use defmt_rtt as _; // global logger
use panic_probe as _;

#[embassy::task]
async fn run1() {
    loop {
        info!("DING DONG");
        Timer::after(Duration::from_ticks(16000)).await;
    }
}

#[embassy::task]
async fn run2() {
    loop {
        Timer::at(Instant::from_ticks(0)).await;
    }
}

#[embassy::task]
async fn run3() {
    futures::future::poll_fn(|cx| {
        cx.waker().wake_by_ref();
        Poll::<()>::Pending
    })
    .await;
}

#[embassy::main]
async fn main(spawner: Spawner, _p: Peripherals) {
    unwrap!(spawner.spawn(run1()));
    unwrap!(spawner.spawn(run2()));
    unwrap!(spawner.spawn(run3()));
}

#![no_std]
#![no_main]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use core::task::Poll;
use defmt::panic;
use embassy::executor::{task, Spawner};
use embassy::time::{Duration, Instant, Timer};
use embassy_nrf::interrupt;

#[task]
async fn run1() {
    loop {
        info!("DING DONG");
        Timer::after(Duration::from_ticks(16000)).await;
    }
}

#[task]
async fn run2() {
    loop {
        Timer::at(Instant::from_ticks(0)).await;
    }
}

#[task]
async fn run3() {
    futures::future::poll_fn(|cx| {
        cx.waker().wake_by_ref();
        Poll::<()>::Pending
    })
    .await;
}

#[embassy::main]
async fn main(spawner: Spawner) {
    unwrap!(spawner.spawn(run1()));
    unwrap!(spawner.spawn(run2()));
    unwrap!(spawner.spawn(run3()));
}

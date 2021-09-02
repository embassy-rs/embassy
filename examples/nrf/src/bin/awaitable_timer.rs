#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;
use embassy_nrf::interrupt;
use embassy_nrf::timer::Timer;
use embassy_nrf::Peripherals;
use example_common::info;

use embassy::executor::Spawner;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let mut t = Timer::new_awaitable(p.TIMER0, interrupt::take!(TIMER0));
    // default frequency is 1MHz, so this triggers every second
    t.cc(0).write(1_000_000);
    // clear the timer value on cc[0] compare match
    t.cc(0).short_compare_clear();
    t.start();

    loop {
        // wait for compare match
        t.cc(0).wait().await;
        info!("hardware timer tick");
    }
}

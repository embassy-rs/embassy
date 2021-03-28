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
use cortex_m_rt::entry;
use defmt::panic;
use embassy::executor::{task, Executor};
use embassy::time::{Duration, Instant, Timer};
use embassy::util::Forever;
use embassy_nrf::peripherals;
use embassy_nrf::{interrupt, rtc};

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

static RTC: Forever<rtc::RTC<peripherals::RTC1>> = Forever::new();
static ALARM: Forever<rtc::Alarm<peripherals::RTC1>> = Forever::new();
static EXECUTOR: Forever<Executor> = Forever::new();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let p = unwrap!(embassy_nrf::Peripherals::take());

    unsafe { embassy_nrf::system::configure(Default::default()) };
    let rtc = RTC.put(rtc::RTC::new(p.RTC1, interrupt::take!(RTC1)));
    rtc.start();
    unsafe { embassy::time::set_clock(rtc) };

    let alarm = ALARM.put(rtc.alarm0());
    let executor = EXECUTOR.put(Executor::new());
    executor.set_alarm(alarm);
    executor.run(|spawner| {
        unwrap!(spawner.spawn(run1()));
        unwrap!(spawner.spawn(run2()));
        unwrap!(spawner.spawn(run3()));
    });
}

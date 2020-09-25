#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use core::mem::MaybeUninit;
use cortex_m_rt::entry;
use embassy::executor::{task, Executor, WfeModel};
use embassy::time::{Duration, Instant, Timer};
use embassy_nrf::pac;
use embassy_nrf::rtc;
use nrf52840_hal::clocks;

#[task]
async fn run1() {
    loop {
        info!("BIG INFREQUENT TICK");
        Timer::after(Duration::from_ticks(64000)).await;
    }
}

#[task]
async fn run2() {
    loop {
        info!("tick");
        Timer::after(Duration::from_ticks(13000)).await;
    }
}

static mut RTC: MaybeUninit<rtc::RTC<pac::RTC1>> = MaybeUninit::uninit();
static mut EXECUTOR: MaybeUninit<Executor<WfeModel, rtc::Alarm<pac::RTC1>>> = MaybeUninit::uninit();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let p = embassy_nrf::pac::Peripherals::take().dewrap();

    clocks::Clocks::new(p.CLOCK)
        .enable_ext_hfosc()
        .set_lfclk_src_external(clocks::LfOscConfiguration::NoExternalNoBypass)
        .start_lfclk();

    let rtc: &'static _ = unsafe {
        let ptr = RTC.as_mut_ptr();
        ptr.write(rtc::RTC::new(p.RTC1));
        &*ptr
    };

    rtc.start();
    unsafe { embassy::time::set_clock(|| RTC.as_ptr().as_ref().unwrap().now()) };

    let executor: &'static _ = unsafe {
        let ptr = EXECUTOR.as_mut_ptr();
        ptr.write(Executor::new(rtc.alarm0()));
        &*ptr
    };

    unsafe {
        executor.spawn(run1()).dewrap();
        executor.spawn(run2()).dewrap();

        executor.run()
    }
}

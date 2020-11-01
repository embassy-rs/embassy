#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use core::mem::MaybeUninit;
use cortex_m_rt::entry;
use nrf52840_hal::clocks;

use embassy::executor::{task, TimerExecutor};
use embassy::time::{Clock, Duration, Timer};
use embassy::util::Forever;
use embassy_nrf::pac;
use embassy_nrf::rtc;

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

static RTC: Forever<rtc::RTC<pac::RTC1>> = Forever::new();
static EXECUTOR: Forever<TimerExecutor<rtc::Alarm<pac::RTC1>>> = Forever::new();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let p = unwrap!(embassy_nrf::pac::Peripherals::take());

    clocks::Clocks::new(p.CLOCK)
        .enable_ext_hfosc()
        .set_lfclk_src_external(clocks::LfOscConfiguration::NoExternalNoBypass)
        .start_lfclk();

    let rtc = RTC.put(rtc::RTC::new(p.RTC1));
    rtc.start();

    unsafe { embassy::time::set_clock(rtc) };

    let executor = EXECUTOR.put(TimerExecutor::new(rtc.alarm0(), cortex_m::asm::sev));

    unwrap!(executor.spawn(run1()));
    unwrap!(executor.spawn(run2()));

    loop {
        executor.run();
        cortex_m::asm::wfe();
    }
}

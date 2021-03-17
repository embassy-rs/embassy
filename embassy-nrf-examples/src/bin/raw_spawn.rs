#![no_std]
#![no_main]

#[path = "../example_common.rs"]
mod example_common;
use core::mem;

use embassy::executor::raw::Task;
use example_common::*;

use cortex_m_rt::entry;
use defmt::panic;
use embassy::executor::Executor;
use embassy::time::{Duration, Timer};
use embassy::util::Forever;
use embassy_nrf::pac;
use embassy_nrf::{interrupt, rtc};
use nrf52840_hal::clocks;

async fn run1() {
    loop {
        info!("BIG INFREQUENT TICK");
        Timer::after(Duration::from_ticks(64000)).await;
    }
}

async fn run2() {
    loop {
        info!("tick");
        Timer::after(Duration::from_ticks(13000)).await;
    }
}

static RTC: Forever<rtc::RTC<pac::RTC1>> = Forever::new();
static ALARM: Forever<rtc::Alarm<pac::RTC1>> = Forever::new();
static EXECUTOR: Forever<Executor> = Forever::new();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let p = unwrap!(embassy_nrf::pac::Peripherals::take());

    clocks::Clocks::new(p.CLOCK)
        .enable_ext_hfosc()
        .set_lfclk_src_external(clocks::LfOscConfiguration::NoExternalNoBypass)
        .start_lfclk();

    let rtc = RTC.put(rtc::RTC::new(p.RTC1, interrupt::take!(RTC1)));
    rtc.start();

    unsafe { embassy::time::set_clock(rtc) };

    let alarm = ALARM.put(rtc.alarm0());
    let executor = EXECUTOR.put(Executor::new());
    executor.set_alarm(alarm);

    let run1_task = Task::new();
    let run2_task = Task::new();

    // Safety: these variables do live forever if main never returns.
    let run1_task = unsafe { make_static(&run1_task) };
    let run2_task = unsafe { make_static(&run2_task) };

    executor.run(|spawner| {
        unwrap!(spawner.spawn(run1_task.spawn(|| run1())));
        unwrap!(spawner.spawn(run2_task.spawn(|| run2())));
    });
}

unsafe fn make_static<T>(t: &T) -> &'static T {
    mem::transmute(t)
}

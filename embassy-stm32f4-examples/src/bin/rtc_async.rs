#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use cortex_m_rt::entry;
use defmt::panic;
use embassy::executor::{task, Executor};
use embassy::time::{Duration, Timer};
use embassy::util::Forever;
use embassy_stm32f4::{interrupt, pac, rtc};
use stm32f4xx_hal::prelude::*;

#[task]
async fn run1() {
    loop {
        info!("BIG INFREQUENT TICK");
        Timer::after(Duration::from_ticks(32768 * 2)).await;
    }
}

#[task]
async fn run2() {
    loop {
        info!("tick");
        Timer::after(Duration::from_ticks(13000)).await;
    }
}

static RTC: Forever<rtc::RTC<pac::TIM2>> = Forever::new();
static ALARM: Forever<rtc::Alarm<pac::TIM2>> = Forever::new();
static EXECUTOR: Forever<Executor> = Forever::new();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let p = unwrap!(pac::Peripherals::take());

    p.RCC.ahb1enr.modify(|_, w| w.dma1en().enabled());
    let rcc = p.RCC.constrain();
    let clocks = rcc.cfgr.freeze();

    p.DBGMCU.cr.modify(|_, w| {
        w.dbg_sleep().set_bit();
        w.dbg_standby().set_bit();
        w.dbg_stop().set_bit()
    });

    let rtc = RTC.put(rtc::RTC::new(p.TIM2, interrupt::take!(TIM2), clocks));
    rtc.start();

    unsafe { embassy::time::set_clock(rtc) };

    let alarm = ALARM.put(rtc.alarm1());
    let executor = EXECUTOR.put(Executor::new());
    executor.set_alarm(alarm);
    executor.run(|spawner| {
        unwrap!(spawner.spawn(run1()));
        unwrap!(spawner.spawn(run2()));
    });
}

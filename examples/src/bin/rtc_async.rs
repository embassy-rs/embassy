#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use core::mem::MaybeUninit;
use cortex_m_rt::entry;
use embassy::clock::Monotonic;
use embassy_nrf::rtc;
use futures_intrusive::timer::{Clock, LocalTimer, LocalTimerService};
use nrf52840_hal::clocks;
use static_executor::{task, Executor};

struct RtcClock<T>(rtc::RTC<T>);

impl<T: rtc::Instance> Clock for RtcClock<T> {
    fn now(&self) -> u64 {
        self.0.now()
    }
}

#[task]
async fn run1(rtc: &'static rtc::RTC<embassy_nrf::pac::RTC1>, timer: &'static LocalTimerService) {
    loop {
        info!("tick 1");
        timer.deadline(rtc.now() + 64000).await;
    }
}

#[task]
async fn run2(rtc: &'static rtc::RTC<embassy_nrf::pac::RTC1>, timer: &'static LocalTimerService) {
    loop {
        info!("tick 2");
        timer.deadline(rtc.now() + 23000).await;
    }
}

static EXECUTOR: Executor = Executor::new(cortex_m::asm::sev);
static mut RTC: MaybeUninit<RtcClock<embassy_nrf::pac::RTC1>> = MaybeUninit::uninit();
static mut TIMER: MaybeUninit<LocalTimerService> = MaybeUninit::uninit();

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
        ptr.write(RtcClock(rtc::RTC::new(p.RTC1)));
        &*ptr
    };

    rtc.0.start();

    let timer: &'static _ = unsafe {
        let ptr = TIMER.as_mut_ptr();
        ptr.write(LocalTimerService::new(rtc));
        &*ptr
    };

    unsafe {
        EXECUTOR.spawn(run1(&rtc.0, timer)).dewrap();
        EXECUTOR.spawn(run2(&rtc.0, timer)).dewrap();

        loop {
            timer.check_expirations();

            EXECUTOR.run();

            match timer.next_expiration() {
                // If this is in the past, set_alarm will immediately trigger the alarm,
                // which will make the wfe immediately return so we do another loop iteration.
                Some(at) => rtc.0.set_alarm(at, cortex_m::asm::sev),
                None => rtc.0.clear_alarm(),
            }

            cortex_m::asm::wfe();
        }
    }
}

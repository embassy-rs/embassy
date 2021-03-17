#![no_std]
#![no_main]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use core::mem::MaybeUninit;
use cortex_m_rt::entry;
use defmt::panic;
use embassy::time::{Alarm, Clock};
use embassy_nrf::{interrupt, rtc};
use nrf52840_hal::clocks;

static mut RTC: MaybeUninit<rtc::RTC<embassy_nrf::pac::RTC1>> = MaybeUninit::uninit();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let p = unwrap!(embassy_nrf::pac::Peripherals::take());

    clocks::Clocks::new(p.CLOCK)
        .enable_ext_hfosc()
        .set_lfclk_src_external(clocks::LfOscConfiguration::NoExternalNoBypass)
        .start_lfclk();

    let irq = interrupt::take!(RTC1);

    let rtc: &'static _ = unsafe {
        let ptr = RTC.as_mut_ptr();
        ptr.write(rtc::RTC::new(p.RTC1, irq));
        &*ptr
    };

    let alarm = rtc.alarm0();

    rtc.start();

    alarm.set_callback(|_| info!("ALARM TRIGGERED"), core::ptr::null_mut());
    alarm.set(53719);

    info!("initialized!");

    let mut val = 0;
    let mut printval = 0;
    loop {
        let val2 = rtc.now();
        if val2 < val {
            info!("timer ran backwards! {} -> {}", val as u32, val2 as u32);
        }
        val = val2;

        if val > printval + 32768 {
            info!("tick {}", val as u32);
            printval = val;
        }
    }
}

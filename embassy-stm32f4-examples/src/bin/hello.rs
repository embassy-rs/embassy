#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::{panic, *};

use cortex_m::singleton;
use cortex_m_rt::entry;
use embassy::executor::{task, Executor};
use embassy::uart::Uart;
use embassy::util::Forever;
use embassy_stm32f4::interrupt;
use embassy_stm32f4::serial;
use stm32f4xx_hal::serial::config::Config;
use stm32f4xx_hal::stm32;
use stm32f4xx_hal::{prelude::*, serial::config};

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let p = stm32f4xx_hal::stm32::Peripherals::take().unwrap();
    let gpioa = p.GPIOA.split();
    let gpioc = p.GPIOC.split();

    let mut led = gpioc.pc13.into_push_pull_output();
    let button = gpioa.pa0.into_pull_up_input();
    led.set_low().unwrap();

    loop {
        if button.is_high().unwrap() {
            led.set_low().unwrap();
        } else {
            led.set_high().unwrap();
        }
    }
}

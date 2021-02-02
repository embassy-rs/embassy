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
use embassy::gpio::*;
use embassy::util::Forever;
use embassy_stm32f4::exti;
use embassy_stm32f4::exti::*;
use embassy_stm32f4::interrupt;
use embassy_stm32f4::serial;
use futures::pin_mut;
use stm32f4xx_hal::serial::config::Config;
use stm32f4xx_hal::stm32;
use stm32f4xx_hal::syscfg;
use stm32f4xx_hal::{prelude::*, serial::config};

static EXTI: Forever<exti::ExtiManager> = Forever::new();

#[task]
async fn run(dp: stm32::Peripherals, cp: cortex_m::Peripherals) {
    let gpioa = dp.GPIOA.split();

    let button = gpioa.pa0.into_pull_up_input();

    let exti = EXTI.put(exti::ExtiManager::new(dp.EXTI, dp.SYSCFG.constrain()));
    let pin = exti.new_pin(button, interrupt::take!(EXTI0));
    pin_mut!(pin);

    info!("Starting loop");

    loop {
        pin.as_mut().wait_for_rising_edge().await;
        info!("edge detected!");
    }
}

static EXECUTOR: Forever<Executor> = Forever::new();

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    let executor = EXECUTOR.put(Executor::new());
    executor.run(|spawner| {
        unwrap!(spawner.spawn(run(dp, cp)));
    });
}

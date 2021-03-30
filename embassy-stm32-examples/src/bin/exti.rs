#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::{panic, *};

use cortex_m_rt::entry;
use embassy::executor::Executor;
use embassy::traits::gpio::*;
use embassy::util::Forever;
use embassy_stm32::exti::ExtiPin;
use embassy_stm32::interrupt;
use futures::pin_mut;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::stm32;

#[embassy::task]
async fn run(dp: stm32::Peripherals, _cp: cortex_m::Peripherals) {
    let gpioa = dp.GPIOA.split();

    let button = gpioa.pa0.into_pull_up_input();
    let mut syscfg = dp.SYSCFG.constrain();

    let pin = ExtiPin::new(button, interrupt::take!(EXTI0), &mut syscfg);
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

    dp.DBGMCU.cr.modify(|_, w| {
        w.dbg_sleep().set_bit();
        w.dbg_standby().set_bit();
        w.dbg_stop().set_bit()
    });
    dp.RCC.ahb1enr.modify(|_, w| w.dma1en().enabled());

    let executor = EXECUTOR.put(Executor::new());
    executor.run(|spawner| {
        unwrap!(spawner.spawn(run(dp, cp)));
    });
}

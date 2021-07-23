#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;
use embassy::executor::Executor;
use embassy::util::Forever;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Pull};
use embassy_traits::gpio::{WaitForFallingEdge, WaitForRisingEdge};
use example_common::*;

use cortex_m_rt::entry;
use embassy_stm32::pac;

#[embassy::task]
async fn main_task() {
    let p = embassy_stm32::init(Default::default());

    let button = Input::new(p.PC13, Pull::Down);
    let mut button = ExtiInput::new(button, p.EXTI13);

    info!("Press the USER button...");

    loop {
        button.wait_for_rising_edge().await;
        info!("Pressed!");
        button.wait_for_falling_edge().await;
        info!("Released!");
    }
}

static EXECUTOR: Forever<Executor> = Forever::new();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    unsafe {
        pac::DBGMCU.cr().modify(|w| {
            w.set_dbg_sleep(true);
            w.set_dbg_standby(true);
            w.set_dbg_stop(true);
        });

        // EXTI clock
        pac::RCC.apb2enr().modify(|w| {
            w.set_syscfgen(true);
        });
    }

    let executor = EXECUTOR.put(Executor::new());

    executor.run(|spawner| {
        unwrap!(spawner.spawn(main_task()));
    })
}

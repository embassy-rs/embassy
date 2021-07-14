#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;
use cortex_m_rt::entry;
use embassy::executor::Executor;
use embassy::time::Clock;
use embassy::util::Forever;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Pull};
use embassy_stm32::pac;
use embassy_traits::gpio::{WaitForFallingEdge, WaitForRisingEdge};
use example_common::*;

#[embassy::task]
async fn main_task() {
    let p = embassy_stm32::init(Default::default());

    let button = Input::new(p.PC13, Pull::Up);
    let mut button = ExtiInput::new(button, p.EXTI13);

    info!("Press the USER button...");

    loop {
        button.wait_for_falling_edge().await;
        info!("Pressed!");
        button.wait_for_rising_edge().await;
        info!("Released!");
    }
}

struct ZeroClock;

impl Clock for ZeroClock {
    fn now(&self) -> u64 {
        0
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

        pac::RCC.apb2enr().modify(|w| {
            w.set_syscfgen(true);
        });

        pac::RCC.ahb2enr().modify(|w| {
            w.set_gpioaen(true);
            w.set_gpioben(true);
            w.set_gpiocen(true);
            w.set_gpioden(true);
            w.set_gpioeen(true);
            w.set_gpiofen(true);
        });
    }

    unsafe { embassy::time::set_clock(&ZeroClock) };

    let executor = EXECUTOR.put(Executor::new());

    executor.run(|spawner| {
        unwrap!(spawner.spawn(main_task()));
    })
}

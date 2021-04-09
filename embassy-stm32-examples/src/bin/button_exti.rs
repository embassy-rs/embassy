#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use embassy::executor::Executor;
use embassy::time::Clock;
use embassy::util::Forever;
use embassy_stm32::exti::{self, ExtiInput};
use embassy_stm32::gpio::{Input, Pull};
use embassy_traits::gpio::{WaitForFallingEdge, WaitForRisingEdge};
use example_common::*;

use cortex_m_rt::entry;
use pac::{interrupt, NVIC};
use stm32f4::stm32f429 as pac;

#[embassy::task]
async fn main_task() {
    let p = embassy_stm32::Peripherals::take().unwrap();
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

    let pp = pac::Peripherals::take().unwrap();

    pp.DBGMCU.cr.modify(|_, w| {
        w.dbg_sleep().set_bit();
        w.dbg_standby().set_bit();
        w.dbg_stop().set_bit()
    });
    pp.RCC.ahb1enr.modify(|_, w| w.dma1en().enabled());

    pp.RCC.ahb1enr.modify(|_, w| {
        w.gpioaen().enabled();
        w.gpioben().enabled();
        w.gpiocen().enabled();
        w.gpioden().enabled();
        w.gpioeen().enabled();
        w.gpiofen().enabled();
        w
    });
    pp.RCC.apb2enr.modify(|_, w| {
        w.syscfgen().enabled();
        w
    });

    unsafe { embassy::time::set_clock(&ZeroClock) };

    unsafe {
        NVIC::unmask(interrupt::EXTI0);
        NVIC::unmask(interrupt::EXTI1);
        NVIC::unmask(interrupt::EXTI2);
        NVIC::unmask(interrupt::EXTI3);
        NVIC::unmask(interrupt::EXTI4);
        NVIC::unmask(interrupt::EXTI9_5);
        NVIC::unmask(interrupt::EXTI15_10);
    }

    let executor = EXECUTOR.put(Executor::new());

    executor.run(|spawner| {
        unwrap!(spawner.spawn(main_task()));
    })
}

// TODO for now irq handling is done by user code using the old pac, until we figure out how interrupts work in the metapac

#[interrupt]
unsafe fn EXTI0() {
    exti::on_irq()
}

#[interrupt]
unsafe fn EXTI1() {
    exti::on_irq()
}

#[interrupt]
unsafe fn EXTI2() {
    exti::on_irq()
}

#[interrupt]
unsafe fn EXTI3() {
    exti::on_irq()
}

#[interrupt]
unsafe fn EXTI4() {
    exti::on_irq()
}

#[interrupt]
unsafe fn EXTI9_5() {
    exti::on_irq()
}

#[interrupt]
unsafe fn EXTI15_10() {
    exti::on_irq()
}

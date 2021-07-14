#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]
#![allow(incomplete_features)]

#[path = "../example_common.rs"]
mod example_common;
use core::fmt::Write;
use cortex_m_rt::entry;
use embassy::executor::Executor;
use embassy::time::Clock;
use embassy::util::Forever;
use embassy_stm32::dma_traits::NoDma;
use embassy_stm32::pac;
use embassy_stm32::usart::{Config, Uart};
use embassy_traits::uart::Write as _;
use example_common::*;
use heapless::String;

#[embassy::task]
async fn main_task() {
    let p = embassy_stm32::init(Default::default());

    let config = Config::default();
    let mut usart = Uart::new(p.UART4, p.PA1, p.PA0, p.DMA1_3, NoDma, config);

    for n in 0u32.. {
        let mut s: String<128> = String::new();
        core::write!(&mut s, "Hello DMA World {}!\r\n", n).unwrap();

        usart.write(s.as_bytes()).await.ok();

        info!("wrote DMA");
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

        pac::RCC.ahb1enr().modify(|w| {
            w.set_dmamux1en(true);
            w.set_dma1en(true);
            w.set_dma2en(true);
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

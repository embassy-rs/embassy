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
use embassy_stm32::usart::{Config, Uart};
use example_common::*;
use heapless::String;
use stm32l4::stm32l4x5 as pac;

#[embassy::task]
async fn main_task() {
    let mut p = embassy_stm32::init(Default::default());

    let config = Config::default();
    let mut usart = Uart::new(p.UART4, p.PA1, p.PA0, config);

    for n in 0u32.. {
        let mut s: String<128> = String::new();
        core::write!(&mut s, "Hello DMA World {}!\r\n", n).unwrap();

        usart
            .write_dma(&mut p.DMA1_3, s.as_bytes())
            .await
            .unwrap();
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

    let pp = pac::Peripherals::take().unwrap();

    pp.DBGMCU.cr.modify(|_, w| {
        w.dbg_sleep().set_bit();
        w.dbg_standby().set_bit();
        w.dbg_stop().set_bit()
    });

    pp.RCC.ahb1enr.modify(|_, w| {
        unsafe {
            w.bits( 0x07 );
        }
        w
        //w.dmamuxen().set_bit();
        //w.dma1en().set_bit();
        //w.dma2en().set_bit();
        //w
    });

    pp.RCC.ahb2enr.modify(|_, w| {
        w.gpioaen().set_bit();
        w.gpioben().set_bit();
        w.gpiocen().set_bit();
        w.gpioden().set_bit();
        w.gpioeen().set_bit();
        w.gpiofen().set_bit();
        w
    });

    pp.RCC.apb2enr.modify(|_, w| {
        w.syscfgen().set_bit();
        w
    });


    unsafe { embassy::time::set_clock(&ZeroClock) };

    let executor = EXECUTOR.put(Executor::new());

    executor.run(|spawner| {
        unwrap!(spawner.spawn(main_task()));
    })
}

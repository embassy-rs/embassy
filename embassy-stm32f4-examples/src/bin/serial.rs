#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::{panic, *};

use cortex_m::singleton;
use cortex_m_rt::entry;
use embassy::executor::{task, Executor};
use embassy::traits::uart::Uart;
use embassy::util::Forever;
use embassy_stm32f4::interrupt;
use embassy_stm32f4::serial;
use stm32f4xx_hal::dma::StreamsTuple;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::serial::config::Config;
use stm32f4xx_hal::stm32;

#[task]
async fn run(dp: stm32::Peripherals, _cp: cortex_m::Peripherals) {
    dp.DBGMCU.cr.modify(|_, w| {
        w.dbg_sleep().set_bit();
        w.dbg_standby().set_bit();
        w.dbg_stop().set_bit()
    });
    dp.RCC.ahb1enr.modify(|_, w| w.dma1en().enabled());

    // https://gist.github.com/thalesfragoso/a07340c5df6eee3b04c42fdc69ecdcb1
    let gpioa = dp.GPIOA.split();
    let rcc = dp.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(16.mhz())
        .sysclk(48.mhz())
        .pclk1(24.mhz())
        .freeze();

    let streams = StreamsTuple::new(dp.DMA2);

    let mut serial = unsafe {
        serial::Serial::new(
            dp.USART1,
            (streams.7, streams.2),
            (
                gpioa.pa9.into_alternate_af7(),
                gpioa.pa10.into_alternate_af7(),
            ),
            interrupt::take!(DMA2_STREAM7),
            interrupt::take!(DMA2_STREAM2),
            interrupt::take!(USART1),
            Config::default().baudrate(9600.bps()),
            clocks,
        )
    };

    let streams = StreamsTuple::new(dp.DMA1);

    let mut serial = unsafe {
        serial::Serial::new(
            dp.USART2,
            (streams.6, streams.5),
            (
                gpioa.pa2.into_alternate_af7(),
                gpioa.pa3.into_alternate_af7(),
            ),
            interrupt::take!(DMA1_STREAM6),
            interrupt::take!(DMA1_STREAM5),
            interrupt::take!(USART2),
            Config::default().baudrate(9600.bps()),
            clocks,
        )
    };
    let buf = singleton!(: [u8; 30] = [0; 30]).unwrap();

    buf[5] = 0x01;
    serial.send(buf).await.unwrap();
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

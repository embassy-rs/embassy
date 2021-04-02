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
use embassy::executor::{Executor, Spawner};
use embassy::traits::uart::{Read, ReadUntilIdle, Write};
use embassy::util::Forever;
use embassy_stm32::interrupt;
use embassy_stm32::serial;
use futures::pin_mut;
use stm32f4xx_hal::dma::StreamsTuple;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::serial::config::Config;
use stm32f4xx_hal::stm32;

#[embassy::main(use_hse = 16, sysclk = 48, pclk1 = 24)]
async fn main(spawner: Spawner) {
    let (dp, clocks) = embassy_stm32::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    dp.DBGMCU.cr.modify(|_, w| {
        w.dbg_sleep().set_bit();
        w.dbg_standby().set_bit();
        w.dbg_stop().set_bit()
    });

    // https://gist.github.com/thalesfragoso/a07340c5df6eee3b04c42fdc69ecdcb1
    let gpioa = dp.GPIOA.split();
    let streams = StreamsTuple::new(dp.DMA2);

    let _serial = unsafe {
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
    pin_mut!(serial);

    let buf = singleton!(: [u8; 30] = [0; 30]).unwrap();

    buf[5] = 0x01;
    serial.as_mut().write(buf).await.unwrap();
    serial.as_mut().read_until_idle(buf).await.unwrap();
}

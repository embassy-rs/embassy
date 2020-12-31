#![no_std]
#![no_main]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]

// extern crate panic_halt;

use cortex_m::singleton;
use cortex_m_rt::entry;
use embassy::executor::{task, Executor};
use embassy::util::Forever;
use embassy_stm32f4::serial;
use stm32f4xx_hal::stm32;
use stm32f4xx_hal::{prelude::*, serial::config};

#[task]
async fn run(dp: stm32::Peripherals, cp: cortex_m::Peripherals) {
    // https://gist.github.com/thalesfragoso/a07340c5df6eee3b04c42fdc69ecdcb1
    let gpioa = dp.GPIOA.split();
    let rcc = dp.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(16.mhz())
        .sysclk(48.mhz())
        .pclk1(24.mhz())
        .freeze();

    let mut serial = serial::Serial::new(
        gpioa.pa10.into_alternate_af7(),
        gpioa.pa9.into_alternate_af7(),
        dp.DMA2,
        dp.USART1,
        config::Parity::ParityNone,
        9600.bps(),
        clocks,
    );

    let buf = singleton!(: [u8; 30] = [0; 30]).unwrap();

    buf[5] = 0x01;

    serial.send(buf).await;
}

static EXECUTOR: Forever<Executor> = Forever::new();

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    let executor = EXECUTOR.put(Executor::new(cortex_m::asm::sev));
    executor.spawn(run(dp, cp));

    loop {
        executor.run();
        cortex_m::asm::wfe();
    }
}

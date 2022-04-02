#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::fmt::Write;
use defmt::*;
use defmt_rtt as _; // global logger
use embassy::executor::Executor;
use embassy::util::Forever;
use embassy_stm32::dma::NoDma;
use embassy_stm32::usart::{Config, Uart};
use panic_probe as _;

use cortex_m_rt::entry;
use heapless::String;

#[embassy::task]
async fn main_task() {
    let p = embassy_stm32::init(Default::default());

    let config = Config::default();
    let mut usart = Uart::new(p.UART7, p.PF6, p.PF7, p.DMA1_CH0, NoDma, config);

    for n in 0u32.. {
        let mut s: String<128> = String::new();
        core::write!(&mut s, "Hello DMA World {}!\r\n", n).unwrap();

        usart.write(s.as_bytes()).await.ok();

        info!("wrote DMA");
    }
}

static EXECUTOR: Forever<Executor> = Forever::new();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let executor = EXECUTOR.put(Executor::new());

    executor.run(|spawner| {
        unwrap!(spawner.spawn(main_task()));
    })
}

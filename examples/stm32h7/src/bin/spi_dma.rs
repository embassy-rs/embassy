#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _; // global logger
use panic_probe as _;

use core::fmt::Write;
use core::str::from_utf8;
use cortex_m_rt::entry;
use defmt::*;
use embassy::executor::Executor;
use embassy::util::Forever;
use embassy_stm32::peripherals::{DMA1_CH3, DMA1_CH4, SPI3};
use embassy_stm32::spi;
use embassy_stm32::time::U32Ext;
use embassy_stm32::Config;
use heapless::String;

pub fn config() -> Config {
    let mut config = Config::default();
    config.rcc.sys_ck = Some(400.mhz().into());
    config.rcc.hclk = Some(200.mhz().into());
    config.rcc.pll1.q_ck = Some(100.mhz().into());
    config
}

#[embassy::task]
async fn main_task(mut spi: spi::Spi<'static, SPI3, DMA1_CH3, DMA1_CH4>) {
    for n in 0u32.. {
        let mut write: String<128> = String::new();
        let mut read = [0; 128];
        core::write!(&mut write, "Hello DMA World {}!\r\n", n).unwrap();
        // transfer will slice the &mut read down to &write's actual length.
        spi.transfer(&mut read, write.as_bytes()).await.ok();
        info!("read via spi+dma: {}", from_utf8(&read).unwrap());
    }
}

static EXECUTOR: Forever<Executor> = Forever::new();

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let p = embassy_stm32::init(config());

    let spi = spi::Spi::new(
        p.SPI3,
        p.PB3,
        p.PB5,
        p.PB4,
        p.DMA1_CH3,
        p.DMA1_CH4,
        1.mhz(),
        spi::Config::default(),
    );

    let executor = EXECUTOR.put(Executor::new());

    executor.run(|spawner| {
        unwrap!(spawner.spawn(main_task(spi)));
    })
}

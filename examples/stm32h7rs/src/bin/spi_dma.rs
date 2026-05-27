#![no_std]
#![no_main]

use core::fmt::Write;
use core::str::from_utf8;

use cortex_m_rt::entry;
use defmt::*;
use embassy_executor::Executor;
use embassy_stm32::mode::Async;
use embassy_stm32::time::mhz;
use embassy_stm32::{bind_interrupts, dma, peripherals, spi};
use heapless::String;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    GPDMA1_CHANNEL0 => dma::InterruptHandler<peripherals::GPDMA1_CH0>;
    GPDMA1_CHANNEL1 => dma::InterruptHandler<peripherals::GPDMA1_CH1>;
});

#[embassy_executor::task]
async fn main_task(mut spi: spi::Spi<'static, Async, spi::mode::Master>) {
    for n in 0u32.. {
        let mut write: String<128> = String::new();
        let mut read = [0; 128];
        core::write!(&mut write, "Hello DMA World {}!\r\n", n).unwrap();
        // transfer will slice the &mut read down to &write's actual length.
        spi.transfer(&mut read, write.as_bytes()).await.ok();
        info!("read via spi+dma: {}", from_utf8(&read).unwrap());
    }
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[entry]
fn main() -> ! {
    info!("Hello World!");
    let p = embassy_stm32::init(Default::default());

    let mut spi_config = spi::Config::default();
    spi_config.frequency = mhz(1);

    let spi = spi::Spi::new(
        p.SPI3,
        p.PB3,
        p.PB5,
        p.PB4,
        p.GPDMA1_CH0,
        p.GPDMA1_CH1,
        Irqs,
        spi_config,
    );

    let executor = EXECUTOR.init(Executor::new());

    executor.run(|spawner| {
        spawner.spawn(unwrap!(main_task(spi)));
    })
}

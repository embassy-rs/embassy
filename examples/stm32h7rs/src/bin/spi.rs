#![no_std]
#![no_main]

use core::fmt::Write;
use core::str::from_utf8;

use cortex_m_rt::entry;
use defmt::*;
use embassy_executor::Executor;
use embassy_stm32::mode::Blocking;
use embassy_stm32::spi;
use embassy_stm32::time::mhz;
use heapless::String;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn main_task(mut spi: spi::Spi<'static, Blocking>) {
    for n in 0u32.. {
        let mut write: String<128> = String::new();
        core::write!(&mut write, "Hello DMA World {}!\r\n", n).unwrap();
        unsafe {
            let result = spi.blocking_transfer_in_place(write.as_bytes_mut());
            if let Err(_) = result {
                defmt::panic!("crap");
            }
        }
        info!("read via spi: {}", from_utf8(write.as_bytes()).unwrap());
    }
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[entry]
fn main() -> ! {
    info!("Hello World!");
    let p = embassy_stm32::init(Default::default());

    let mut spi_config = spi::Config::default();
    spi_config.frequency = mhz(1);

    let spi = spi::Spi::new_blocking(p.SPI3, p.PB3, p.PB5, p.PB4, spi_config);

    let executor = EXECUTOR.init(Executor::new());

    executor.run(|spawner| {
        spawner.spawn(unwrap!(main_task(spi)));
    })
}

#![no_std]
#![no_main]

use core::fmt::Write;
use core::str::from_utf8;

use cortex_m_rt::entry;
use defmt::*;
use embassy_executor::Executor;
use embassy_stm32::dma::NoDma;
use embassy_stm32::peripherals::SPI3;
use embassy_stm32::time::mhz;
use embassy_stm32::{spi, Config};
use heapless::String;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::task]
async fn main_task(mut spi: spi::Spi<'static, SPI3, NoDma, NoDma>) {
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

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV8), // used by SPI3. 100Mhz.
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
    }
    let p = embassy_stm32::init(config);

    let mut spi_config = spi::Config::default();
    spi_config.frequency = mhz(1);

    let spi = spi::Spi::new(p.SPI3, p.PB3, p.PB5, p.PB4, NoDma, NoDma, spi_config);

    let executor = EXECUTOR.init(Executor::new());

    executor.run(|spawner| {
        unwrap!(spawner.spawn(main_task(spi)));
    })
}

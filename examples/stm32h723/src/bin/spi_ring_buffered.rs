//! This example showcases:
//! - tx only SPI master
//! - rx only SPI slave
//! - ring buffered rx with deselect detection
#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::mode::Async;
use embassy_stm32::spi::mode::Master;
use embassy_stm32::spi::{self, RingBufferedSpiRx, Spi};
use embassy_stm32::{Config, bind_interrupts, dma, exti, interrupt, peripherals};
use embassy_time::Timer;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    DMA2_STREAM2 => dma::InterruptHandler<peripherals::DMA2_CH2>;
    DMA1_STREAM3 => dma::InterruptHandler<peripherals::DMA1_CH3>;
    EXTI15_10 => exti::InterruptHandler<interrupt::typelevel::EXTI15_10>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::Div1);
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div16,
            mul: PllMul::Mul200,
            divp: Some(PllDiv::Div2), // 400 MHz
            divq: Some(PllDiv::Div8), // 100 MHz
            divr: Some(PllDiv::Div2), // 400 MHz
        });
        config.rcc.mux.spi123sel = mux::Saisel::Pll1Q; // 100 MHz
    }
    let p = embassy_stm32::init(config);

    let mut spi_config = spi::Config::default();
    spi_config.nss_polarity = spi::SlaveSelectPolarity::ActiveHigh;

    let tx_spi = Spi::new_txonly(p.SPI1, p.PA5, p.PA7, p.DMA2_CH2, Irqs, spi_config);
    let tx_cs = Output::new(p.PA4, Level::Low, Speed::VeryHigh);
    spawner.spawn(tx_task(tx_spi, tx_cs).unwrap());

    let rx_spi = Spi::new_rxonly_slave(p.SPI2, p.PB13, p.PB15, p.PB12, p.DMA1_CH3, Irqs, spi_config);

    #[unsafe(link_section = ".sram1")]
    static mut BUF: [u8; 64] = [0; 64];
    let buf = unsafe { &mut BUF[..] };

    let rx_spi = rx_spi.into_ring_buffered(buf, p.EXTI12, Irqs);
    spawner.spawn(rx_task(rx_spi).unwrap());
}

#[embassy_executor::task]
async fn tx_task(mut spi: Spi<'static, Async, Master>, mut cs: Output<'static>) {
    loop {
        Timer::after_secs(1).await;
        cs.set_high();
        match spi.write::<u8>(&[0, 1, 2, 3, 4, 5, 6, 7]).await {
            Ok(()) => info!("tx ok"),
            Err(e) => info!("tx err {}", e),
        }

        Timer::after_secs(1).await;
        match spi.write::<u8>(&[8, 9, 10, 11, 12, 13, 14, 15]).await {
            Ok(()) => info!("tx ok"),
            Err(e) => info!("tx err {}", e),
        }
        cs.set_low();
    }
}

#[embassy_executor::task]
async fn rx_task(mut spi: RingBufferedSpiRx<'static, u8>) {
    let mut buf = [0u8; 64];
    loop {
        match spi.read(&mut buf).await {
            Ok(len) => info!("rx ok {} buf {}", len, buf[..len]),
            Err(e) => info!("rx err {}", e),
        }
    }
}

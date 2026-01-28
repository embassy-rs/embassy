#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_imxrt::bind_interrupts;
use embassy_imxrt::flexcomm::spi::{InterruptHandler, Spi};
use embassy_imxrt::peripherals::FLEXCOMM5;
use {defmt_rtt as _, embassy_imxrt_examples as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    FLEXCOMM5 => InterruptHandler<FLEXCOMM5>;
});

const BUFLEN: usize = 1024;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_imxrt::init(Default::default());

    info!("Initializing SPI");

    let mut spi = Spi::new_async(p.FLEXCOMM5, p.PIO1_3, p.PIO1_5, p.PIO1_4, Irqs, Default::default());

    let mut rxbuf = [0x55; BUFLEN];
    let txbuf = [0xaa; BUFLEN];

    for _ in 0..10 {
        spi.async_transfer(&mut rxbuf, &txbuf).await.unwrap();
        assert!(rxbuf.iter().all(|b| *b == 0xaa));
        rxbuf.fill(0x55);
    }

    info!("SPI transfers succeeded");
}

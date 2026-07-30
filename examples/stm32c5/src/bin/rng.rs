#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::rng::Rng;
use embassy_stm32::{Config, bind_interrupts, peripherals, rng};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.rcc.mux.clk48sel = embassy_stm32::rcc::mux::Clksel::Hsidiv3;
    let p = embassy_stm32::init(config);
    info!("Hello World!");

    let mut rng = Rng::new(p.RNG, Irqs);

    let mut buf = [0u8; 16];
    unwrap!(rng.async_fill_bytes(&mut buf).await);
    info!("random bytes: {:02x}", buf);
}

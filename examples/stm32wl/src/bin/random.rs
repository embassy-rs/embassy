#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::{bind_interrupts, pac, peripherals};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs{
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    config.rcc.mux = embassy_stm32::rcc::ClockSrc::HSE32;
    config.rcc.enable_lsi = true; //Needed for RNG to work

    let p = embassy_stm32::init(config);
    pac::RCC.ccipr().modify(|w| {
        w.set_rngsel(0b01);
    });

    info!("Hello World!");

    let mut rng = Rng::new(p.RNG, Irqs);

    let mut buf = [0u8; 16];
    unwrap!(rng.async_fill_bytes(&mut buf).await);
    info!("random bytes: {:02x}", buf);

    loop {}
}

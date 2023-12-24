#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::rcc::{ClockSrc, Pll, PllMul, PllPreDiv, PllQDiv, PllRDiv, PllSource};
use embassy_stm32::rng::Rng;
use embassy_stm32::{bind_interrupts, peripherals, rng, Config};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.rcc.mux = ClockSrc::PLL1_R;
    config.rcc.hsi = true;
    config.rcc.pll = Some(Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV1,
        mul: PllMul::MUL18,
        divp: None,
        divq: Some(PllQDiv::DIV6), // 48Mhz (16 / 1 * 18 / 6)
        divr: Some(PllRDiv::DIV4), // sysclk 72Mhz clock (16 / 1 * 18 / 4)
    });
    let p = embassy_stm32::init(config);

    info!("Hello World!");

    let mut rng = Rng::new(p.RNG, Irqs);

    let mut buf = [0u8; 16];
    unwrap!(rng.async_fill_bytes(&mut buf).await);
    info!("random bytes: {:02x}", buf);
}

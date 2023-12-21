#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::rng::{self, Rng};
use embassy_stm32::time::Hertz;
use embassy_stm32::{bind_interrupts, peripherals};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs{
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_stm32::Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(32_000_000),
            mode: HseMode::Bypass,
            prescaler: HsePrescaler::DIV1,
        });
        config.rcc.mux = ClockSrc::PLL1_R;
        config.rcc.pll = Some(Pll {
            source: PllSource::HSE,
            prediv: PllPreDiv::DIV2,
            mul: PllMul::MUL6,
            divp: None,
            divq: Some(PllQDiv::DIV2), // PLL1_Q clock (32 / 2 * 6 / 2), used for RNG
            divr: Some(PllRDiv::DIV2), // sysclk 48Mhz clock (32 / 2 * 6 / 2)
        });
    }
    let p = embassy_stm32::init(config);

    info!("Hello World!");

    let mut rng = Rng::new(p.RNG, Irqs);

    let mut buf = [0u8; 16];
    unwrap!(rng.async_fill_bytes(&mut buf).await);
    info!("random bytes: {:02x}", buf);

    loop {}
}

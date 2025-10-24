#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::rcc::mux::Clk48sel;
use embassy_stm32::rng::Rng;
use embassy_stm32::{Config, bind_interrupts, peripherals, rng};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RNG_CRYP => rng::InterruptHandler<peripherals::RNG>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = true;
        config.rcc.pll = Some(Pll {
            source: PllSource::HSI, // 16 MHz
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL7, // 16 * 7 = 112 MHz
            divp: None,
            divq: None,
            divr: Some(PllRDiv::DIV2), // 112 / 2 = 56 MHz
        });
        config.rcc.sys = Sysclk::PLL1_R;
        config.rcc.hsi48 = Some(Hsi48Config { sync_from_usb: false }); // needed for RNG
        config.rcc.mux.clk48sel = Clk48sel::HSI48; // needed for RNG (or use MSI or PLLQ if you want)
    }

    let p = embassy_stm32::init(config);

    info!("Hello World!");

    let mut rng = Rng::new(p.RNG, Irqs);

    let mut buf = [0u8; 16];
    unwrap!(rng.async_fill_bytes(&mut buf).await);
    info!("random bytes: {:02x}", buf);
}

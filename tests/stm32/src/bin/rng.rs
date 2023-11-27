// required-features: rng
#![no_std]
#![no_main]

#[path = "../common.rs"]
mod common;
use common::*;
use embassy_executor::Spawner;
use embassy_stm32::rng::Rng;
use embassy_stm32::{bind_interrupts, peripherals, rng};
use {defmt_rtt as _, panic_probe as _};

#[cfg(any(
    feature = "stm32l4a6zg",
    feature = "stm32h755zi",
    feature = "stm32h753zi",
    feature = "stm32f429zi"
))]
bind_interrupts!(struct Irqs {
   HASH_RNG => rng::InterruptHandler<peripherals::RNG>;
});
#[cfg(any(feature = "stm32l073rz"))]
bind_interrupts!(struct Irqs {
   RNG_LPUART1 => rng::InterruptHandler<peripherals::RNG>;
});
#[cfg(not(any(
    feature = "stm32l4a6zg",
    feature = "stm32l073rz",
    feature = "stm32h755zi",
    feature = "stm32h753zi",
    feature = "stm32f429zi"
)))]
bind_interrupts!(struct Irqs {
   RNG => rng::InterruptHandler<peripherals::RNG>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p: embassy_stm32::Peripherals = embassy_stm32::init(config());

    let mut rng = Rng::new(p.RNG, Irqs);

    let mut buf1 = [0u8; 16];
    unwrap!(rng.async_fill_bytes(&mut buf1).await);
    info!("random bytes: {:02x}", buf1);

    let mut buf2 = [0u8; 16];
    unwrap!(rng.async_fill_bytes(&mut buf2).await);
    info!("random bytes: {:02x}", buf2);

    defmt::assert!(buf1 != buf2);

    info!("Test OK");
    cortex_m::asm::bkpt();
}

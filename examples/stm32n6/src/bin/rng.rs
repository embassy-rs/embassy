#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::rng::Rng;
use embassy_stm32::{Config, bind_interrupts, peripherals, rng};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut core_peri = unsafe { cortex_m::Peripherals::steal() };
    core_peri.SCB.invalidate_icache();
    core_peri.SCB.enable_icache();

    // On STM32N6, the RNG kernel clock (rng_ker_ck) is hardwired to hsis_osc_ck (48 MHz internal RC
    // oscillator) with no mux - RM0486 Table 73. No explicit kernel clock selection is needed.
    let config = Config::default();

    let p = embassy_stm32::init(config);
    info!("Hello World!");

    let mut rng = Rng::new(p.RNG, Irqs);

    let mut buf = [0u8; 16];
    unwrap!(rng.async_fill_bytes(&mut buf).await);
    info!("random bytes: {:02x}", buf);
}

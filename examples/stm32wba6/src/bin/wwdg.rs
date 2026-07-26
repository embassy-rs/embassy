#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_stm32::wdg::WindowWatchdog;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();
    // Fine-tune PLL1 dividers/multipliers
    // voltage scale for max performance
    // route PLL1_R into Sysclk
    let p = embassy_stm32::init(config);
    info!("WWDG example");

    // 200 ms total period; the first 100 ms is the closed window.
    // Petting the watchdog within 100 ms of the last reload causes an immediate reset.
    let mut wdg = WindowWatchdog::new(p.WWDG, 200_000, 100_000);

    loop {
        // Wait until we are inside the open window (100–200 ms after last reload).
        Timer::after_millis(150).await;
        info!("pet");
        wdg.pet();
    }
}

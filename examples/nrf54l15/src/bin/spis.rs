#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::spis::{Config, Spis};
use embassy_nrf::{bind_interrupts, peripherals, spis};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SERIAL20 => spis::InterruptHandler<peripherals::TWISPI20>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("Running!");

    let mut spis = Spis::new(p.TWISPI20, Irqs, p.P2_05, p.P2_01, p.P2_02, p.P2_04, Config::default());

    loop {
        let mut rx_buf = [0_u8; 64];
        let tx_buf = [1_u8, 2, 3, 4, 5, 6, 7, 8];
        if let Ok((n_rx, n_tx)) = spis.transfer(&mut rx_buf, &tx_buf).await {
            info!("RX: {:?}", rx_buf[..n_rx]);
            info!("TX: {:?}", tx_buf[..n_tx]);
        }
    }
}

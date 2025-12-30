#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::{bind_interrupts, peripherals, spim};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    SERIAL00 => spim::InterruptHandler<peripherals::SERIAL00>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut config = spim::Config::default();
    config.frequency = spim::Frequency::M32;
    let mut spim = spim::Spim::new(p.SERIAL00, Irqs, p.P2_05, p.P2_09, p.P2_08, config.clone());
    let data = [
        0x42, 0x43, 0x44, 0x45, 0x66, 0x12, 0x23, 0x34, 0x45, 0x19, 0x91, 0xaa, 0xff, 0xa5, 0x5a, 0x77,
    ];
    let mut buf = [0u8; 16];

    buf.fill(0);
    spim.blocking_transfer(&mut buf, &data).unwrap();
    assert_eq!(data, buf);

    buf.fill(0);
    spim.transfer(&mut buf, &data).await.unwrap();
    assert_eq!(data, buf);
}

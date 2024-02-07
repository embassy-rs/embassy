#![no_std]
#![no_main]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_nrf::{bind_interrupts, peripherals, radio};
use embassy_time::Timer;
use jewel::phy::Radio;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    RADIO => radio::InterruptHandler<peripherals::RADIO>;
});

// For a high-level API look on jewel examples
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_nrf::config::Config::default();
    config.hfclk_source = embassy_nrf::config::HfclkSource::ExternalXtal;
    let p = embassy_nrf::init(config);

    info!("Starting BLE radio");
    let mut radio = radio::ble::Radio::new(p.RADIO, Irqs);

    let pdu = [
        0x46u8, // ADV_NONCONN_IND, Random address,
        0x18,   // Length of payload
        0x27, 0xdc, 0xd0, 0xe8, 0xe1, 0xff, // Adress
        0x02, 0x01, 0x06, // Flags
        0x03, 0x03, 0x09, 0x18, // Complete list of 16-bit UUIDs available
        0x0A, 0x09, // Length, Type: Device name
        b'H', b'e', b'l', b'l', b'o', b'R', b'u', b's', b't',
    ];

    unwrap!(radio.set_buffer(pdu.as_ref()));

    loop {
        info!("Sending packet");
        radio.transmit().await;
        Timer::after_millis(500).await;
    }
}

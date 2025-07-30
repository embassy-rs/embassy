#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::config::{Config, HfclkSource};
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::radio::ieee802154::{self, Packet};
use embassy_nrf::{peripherals, radio};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

embassy_nrf::bind_interrupts!(struct Irqs {
    RADIO => radio::InterruptHandler<peripherals::RADIO>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    config.hfclk_source = HfclkSource::ExternalXtal;
    let peripherals = embassy_nrf::init(config);

    // assumes LED on P0_15 with active-high polarity
    let mut gpo_led = Output::new(peripherals.P0_15, Level::Low, OutputDrive::Standard);

    let mut radio = ieee802154::Radio::new(peripherals.RADIO, Irqs);
    let mut packet = Packet::new();

    loop {
        gpo_led.set_low();
        let rv = radio.receive(&mut packet).await;
        gpo_led.set_high();
        match rv {
            Err(_) => defmt::error!("receive() Err"),
            Ok(_) => defmt::info!("receive() {:?}", *packet),
        }
        Timer::after_millis(100u64).await;
    }
}

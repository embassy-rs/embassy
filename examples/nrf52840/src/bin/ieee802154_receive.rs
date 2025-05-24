#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::{
    gpio::{
        Level,
        Output,
        OutputDrive
    },
    peripherals,
    radio::{
        self,
        ieee802154::{
            self,
            Packet,
        },
    },
};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

embassy_nrf::bind_interrupts!(struct Irqs {
    RADIO => radio::InterruptHandler<peripherals::RADIO>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let peripherals = embassy_nrf::init(Default::default());
    let mut gpo_led = Output::new(peripherals.P0_30, Level::Low, OutputDrive::Standard);
    let mut radio = ieee802154::Radio::new(peripherals.RADIO, Irqs);
    let mut packet = Packet::new();


    Timer::after_millis(100_u64).await;
    loop {
        gpo_led.set_high();
        match radio.receive(&mut packet).await {
            Err(_) => defmt::error!("receive() Err"),
            Ok(_) => defmt::info!("receive() {:?}", *packet),
        }
        Timer::after_millis(10u64).await;
        gpo_led.set_low();
    }
}

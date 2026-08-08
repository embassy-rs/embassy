//! USB1 high-speed CDC-ACM serial port example for the LPC55S6x.
//!
//! Creates a USB serial port on the USB1 (high-speed) port that echoes
//! received data. The device enumerates at 480 Mbps.

#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nxp::config::MainClock;
use embassy_nxp::usb::{Driver, InterruptHandler, Memory};
use embassy_nxp::{bind_interrupts, peripherals};
use embassy_nxp_lpc55s69_examples::serial::{self, Params};
use embassy_usb::UsbDeviceSpeed;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB1 => InterruptHandler<peripherals::USBHSD>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_nxp::config::Config::default();
    config.main_clock = MainClock::FroHf96;
    let p = embassy_nxp::init(config);
    info!("Initialization complete");

    let mut resources = serial::Resources::new();
    let driver = Driver::<peripherals::USBHSD>::new(p.USBHSD, Irqs, Memory::usb1_sram());
    serial::run::<_, 512>(
        driver,
        &mut resources,
        Params {
            pid: 0xcafe,
            product: "USB-HS serial example",
            serial: "12345678",
            max_speed: UsbDeviceSpeed::High,
        },
        "Connected",
        "Disconnected",
    )
    .await;
}

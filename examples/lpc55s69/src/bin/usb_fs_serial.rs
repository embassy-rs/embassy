//! USB0 full-speed CDC-ACM serial port example for the LPC55S6x.
//!
//! Creates a USB serial port on the USB0 (full-speed) port that echoes
//! received data. The device enumerates at 12 Mbps.
//!
//! Uses the default HAL config on purpose: the full-speed controller brings up
//! its own 48 MHz clock from FRO-HF, so unlike the high-speed examples it needs
//! no `MainClock::FroHf96` and no crystal.
//!
//! Cable on connector **P10**.

#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nxp::usb::{Driver, InterruptHandler, Memory};
use embassy_nxp::{bind_interrupts, peripherals};
use embassy_nxp_lpc55s69_examples::serial::{self, Params};
use embassy_usb::UsbDeviceSpeed;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB0 => InterruptHandler<peripherals::USB0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nxp::init(embassy_nxp::config::Config::default());
    info!("Initialization complete");

    let mut resources = serial::Resources::new();
    let driver = Driver::<peripherals::USB0>::new(p.USB0, Irqs, p.PIO0_22, Memory::usb1_sram());
    serial::run::<_, 64>(
        driver,
        &mut resources,
        Params {
            pid: 0xcafe,
            product: "USB-FS serial example",
            serial: "12345678",
            max_speed: UsbDeviceSpeed::Full,
        },
        "Connected",
        "Disconnected",
    )
    .await;
}

//! USB1 high-speed CDC-ACM throughput test firmware for the LPC55S6x.
//!
//! Command-driven benchmark peer for the host throughput script. The host
//! sends a 5-byte header: a command byte followed by a u32 LE byte count.
//!
//! - `b'I'`: device sources exactly `count` bytes to the host.
//! - `b'O'`: device sinks exactly `count` bytes, then replies with the received
//!   count as a u32 LE acknowledgement.
//!
//! Payload bytes in the same packet as an `b'O'` header count toward the total.

#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nxp::config::MainClock;
use embassy_nxp::usb::{Driver, InterruptHandler, Memory};
use embassy_nxp::{bind_interrupts, peripherals};
use embassy_nxp_lpc55s69_examples::throughput_device::{self, Params};
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

    let mut resources = throughput_device::Resources::new();
    let driver = Driver::<peripherals::USBHSD>::new(p.USBHSD, Irqs, Memory::usb1_sram());
    throughput_device::run::<_, 512, 3584, 3584, 512>(
        driver,
        &mut resources,
        Params {
            pid: 0xcb08,
            product: "USB-HS throughput test",
            serial: "12345678",
            max_speed: UsbDeviceSpeed::High,
        },
    )
    .await;
}

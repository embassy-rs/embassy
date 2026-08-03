//! USB0 full-speed bulk throughput benchmark for the LPC55S6x.
//!
//! Speaks the same wire protocol as `usb_hs_throughput`, so one host script
//! drives both: the host sends a 5-byte header on the CDC tty, a command byte
//! followed by a little-endian u32 byte count.
//!
//! - `b'I'` + N: the device streams N bytes to the host.
//! - `b'O'` + N: the host sends N bytes, then the device replies with the
//!   received count as a little-endian u32.
//!
//! Drive it with `examples/lpc55s69/scripts/usb_throughput.py`. Expect roughly
//! 1.0-1.2 MB/s, against a full-speed bulk ceiling of 1.216 MB/s.
//!
//! Uses the default HAL config: the full-speed controller brings up its own
//! 48 MHz clock. Cable on connector **P10**.

#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nxp::usb::{Driver, InterruptHandler, Memory};
use embassy_nxp::{bind_interrupts, peripherals};
use embassy_nxp_lpc55s69_examples::throughput_device::{self, Params};
use embassy_usb::UsbDeviceSpeed;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB0 => InterruptHandler<peripherals::USB0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nxp::init(embassy_nxp::config::Config::default());
    info!("Initialization complete");

    let mut resources = throughput_device::Resources::new();
    let driver = Driver::<peripherals::USB0>::new(p.USB0, Irqs, p.PIO0_22, Memory::usb1_sram());
    throughput_device::run::<_, 64, 512, 64, 64>(
        driver,
        &mut resources,
        Params {
            pid: 0xcb07,
            product: "USB-FS throughput test",
            serial: "12345678",
            max_speed: UsbDeviceSpeed::Full,
        },
    )
    .await;
}

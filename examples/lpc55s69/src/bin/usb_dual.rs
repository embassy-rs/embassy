//! Both LPC55 USB controllers running at the same time.
//!
//! Brings up USB1 (high-speed, connector **P9**) on the dedicated USB1 SRAM and
//! USB0 (full-speed, connector **P10**) on a buffer in main SRAM, then runs two
//! independent `UsbDevice`s concurrently. Each is a CDC-ACM echo port with its
//! own product string and PID, so the host sees two distinct devices:
//!
//! - `c0de:cafe` "USB-HS dual echo" at 480 Mbps
//! - `c0de:caff` "USB-FS dual echo" at 12 Mbps

#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_nxp::config::MainClock;
use embassy_nxp::usb::{Driver, InterruptHandler, Memory};
use embassy_nxp::{bind_interrupts, peripherals};
use embassy_nxp_lpc55s69_examples::serial::{self, Params};
use embassy_usb::UsbDeviceSpeed;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB0 => InterruptHandler<peripherals::USB0>;
    USB1 => InterruptHandler<peripherals::USBHSD>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_nxp::config::Config::default();
    config.main_clock = MainClock::FroHf96;
    let p = embassy_nxp::init(config);
    info!("Initialization complete");

    // Only one controller can own the dedicated USB1 SRAM.
    let hs_driver = Driver::<peripherals::USBHSD>::new(p.USBHSD, Irqs, Memory::usb1_sram());
    let mut ep_mem = [0u8; 4096];
    let fs_driver = Driver::<peripherals::USB0>::new(p.USB0, Irqs, p.PIO0_22, Memory::buffer(&mut ep_mem));

    let mut hs_resources = serial::Resources::new();
    let mut fs_resources = serial::Resources::new();
    let hs = serial::run::<_, 512>(
        hs_driver,
        &mut hs_resources,
        Params {
            pid: 0xcafe,
            product: "USB-HS dual echo",
            serial: "12345678",
            max_speed: UsbDeviceSpeed::High,
        },
        "HS connected",
        "HS disconnected",
    );
    let fs = serial::run::<_, 64>(
        fs_driver,
        &mut fs_resources,
        Params {
            pid: 0xcaff,
            product: "USB-FS dual echo",
            serial: "87654321",
            max_speed: UsbDeviceSpeed::Full,
        },
        "FS connected",
        "FS disconnected",
    );
    join(hs, fs).await;
}

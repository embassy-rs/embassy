//! USB0 full-speed enumeration test. Requires a host cable on **P10**.
//!
//! Uses the default HAL config on purpose: the full-speed driver brings up its
//! own 48 MHz clock, and proving that is part of the test.

#![no_std]
#![no_main]

use core::sync::atomic::AtomicBool;

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nxp::usb::{Driver, InterruptHandler, Memory};
use embassy_nxp::{bind_interrupts, pac, peripherals};
use embassy_nxp_lpc55_tests::enumerate::{self, Params, Resources};
use embassy_usb::UsbDeviceSpeed;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB0 => InterruptHandler<peripherals::USB0>;
});

static CONFIGURED: AtomicBool = AtomicBool::new(false);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nxp::init(embassy_nxp::config::Config::default());
    let mut resources = Resources::new(&CONFIGURED);
    let driver = Driver::<peripherals::USB0>::new(p.USB0, Irqs, p.PIO0_22, Memory::usb1_sram());

    enumerate::run::<_, _, 64>(
        driver,
        &mut resources,
        Params {
            product: "USB-FS enumeration test",
            max_speed: UsbDeviceSpeed::Full,
        },
        || {
            defmt::assert!(
                pac::USB0.devcmdstat().read().dev_addr() != 0,
                "host configured us but DEVCMDSTAT.DEV_ADDR is still 0"
            );
        },
    )
    .await;
}

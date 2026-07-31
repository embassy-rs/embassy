//! USB1 high-speed enumeration test. Requires a host cable on **P9**.

#![no_std]
#![no_main]

use core::sync::atomic::AtomicBool;

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nxp::config::MainClock;
use embassy_nxp::usb::{Driver, InterruptHandler, Memory};
use embassy_nxp::{bind_interrupts, pac, peripherals};
use embassy_nxp_lpc55_tests::enumerate::{self, Params, Resources};
use embassy_usb::UsbDeviceSpeed;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB1 => InterruptHandler<peripherals::USBHSD>;
});

static CONFIGURED: AtomicBool = AtomicBool::new(false);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_nxp::config::Config::default();
    // The high-speed controller requires a system clock of at least 96 MHz.
    config.main_clock = MainClock::FroHf96;
    let p = embassy_nxp::init(config);
    let mut resources = Resources::new(&CONFIGURED);
    let driver = Driver::<peripherals::USBHSD>::new(p.USBHSD, Irqs, Memory::usb1_sram());

    enumerate::run::<_, _, 512>(
        driver,
        &mut resources,
        Params {
            product: "USB-HS enumeration test",
            max_speed: UsbDeviceSpeed::High,
        },
        || {
            // 10b = high speed, per the DEVCMDSTAT.SPEED field docs.
            defmt::assert_eq!(
                pac::USBHSD.devcmdstat().read().speed(),
                0b10,
                "device did not negotiate high speed"
            );
            defmt::assert!(
                pac::USBHSD.devcmdstat().read().dev_addr() != 0,
                "host configured us but DEVCMDSTAT.DEV_ADDR is still 0"
            );
        },
    )
    .await;
}

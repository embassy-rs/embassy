//! USB1 high-speed conformance test. Requires a host cable on **P9** and the
//! host driver `host/conformance.py --pid 0xcb01 --speed hs`.

#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nxp::config::MainClock;
use embassy_nxp::usb::{Driver, InterruptHandler, Memory};
use embassy_nxp::{bind_interrupts, peripherals};
use embassy_nxp_lpc55_tests::conformance::{self, Params};
use embassy_usb::UsbDeviceSpeed;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB1 => InterruptHandler<peripherals::USBHSD>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let mut hal_config = embassy_nxp::config::Config::default();
    // The high-speed controller requires a system clock of at least 96 MHz.
    hal_config.main_clock = MainClock::FroHf96;
    let p = embassy_nxp::init(hal_config);

    // Only one controller runs in this binary, so it may take the dedicated
    // USB1 SRAM. Worst case there: list 128 + setup 64 + EP0 128 + bulk IN
    // 2x3584 + bulk OUT 2x512 + interrupt IN 2x64 + iso OUT 2x1024 + iso IN
    // 2x1024 = 12736 B of 16384 B.
    let driver = Driver::<peripherals::USBHSD>::new(p.USBHSD, Irqs, Memory::usb1_sram());

    conformance::run(
        driver,
        Params {
            pid: 0xcb01,
            product: "USB-HS conformance",
            serial: "hs-conf",
            max_speed: UsbDeviceSpeed::High,
            bulk_mps: 512,
            int_mps: 16,
            iso_out_mps: 1024,
            // 1023 and not 1024: erratum USB.6 caps high-speed isochronous IN.
            iso_in_mps: 1023,
        },
    )
    .await
}

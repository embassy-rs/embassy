//! USB0 full-speed conformance test. Requires a host cable on **P10** and the
//! host driver `host/conformance.py --pid 0xcb02 --speed fs`.

#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nxp::usb::{Driver, InterruptHandler, Memory};
use embassy_nxp::{bind_interrupts, peripherals};
use embassy_nxp_lpc55_tests::conformance::{self, Params};
use embassy_usb::UsbDeviceSpeed;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB0 => InterruptHandler<peripherals::USB0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    // Default main clock: the full-speed driver brings up its own 48 MHz.
    let p = embassy_nxp::init(embassy_nxp::config::Config::default());

    // Only one controller runs in this binary, so it may take the dedicated
    // USB1 SRAM even though it drives USB0.
    let driver = Driver::<peripherals::USB0>::new(p.USB0, Irqs, p.PIO0_22, Memory::usb1_sram());

    conformance::run(
        driver,
        Params {
            pid: 0xcb02,
            product: "USB-FS conformance",
            serial: "fs-conf",
            max_speed: UsbDeviceSpeed::Full,
            bulk_mps: 64,
            int_mps: 16,
            // 512 and not the full-speed maximum of 1023. Two 1023-byte
            // isochronous endpoints do not fit the 1 ms full-speed frame's 90 %
            // periodic budget (1350 B), and a 1023-byte full-speed isochronous
            // IN through a high-speed hub's transaction translator only survives
            // one packet per URB on this host - multi-packet URBs come back
            // truncated or with EPIPE. The 1023-byte allocation boundary itself
            // is covered by `usb_alloc` on both controllers.
            iso_out_mps: 512,
            iso_in_mps: 512,
        },
    )
    .await
}

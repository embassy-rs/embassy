//! Endpoint allocation limits for both LPC55 USB controllers.
//!
//! Needs no host cable: neither driver is ever started, so no bus traffic
//! happens. This is the only test that catches a pure allocator regression.
//!
//! Endpoint indices, not bytes, are what the exhaustion assertions probe: they
//! use small endpoints so the relevant limit is `Instance::EP_COUNT` rather
//! than the size of the endpoint memory, which would make the test brittle.

#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nxp::config::MainClock;
use embassy_nxp::usb::{Driver, InterruptHandler, Memory};
use embassy_nxp::{bind_interrupts, peripherals};
use embassy_usb::driver::{Direction, Driver as _, EndpointAddress, EndpointType};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB0 => InterruptHandler<peripherals::USB0>;
    USB1 => InterruptHandler<peripherals::USBHSD>;
});

/// Endpoint memory for USB0, in main SRAM. 4 KiB is enough for the command
/// list plus every full-speed endpoint this test allocates.
#[repr(C, align(256))]
struct EpMem([u8; 4096]);
static mut EP_MEM: EpMem = EpMem([0; 4096]);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_nxp::config::Config::default();
    // The high-speed controller requires a system clock of at least 96 MHz.
    config.main_clock = MainClock::FroHf96;
    let p = embassy_nxp::init(config);

    // ---------------------------------------------------------------- USB1 HS
    // EP_COUNT = 6, so data endpoint indices 1..=5 are usable per direction.
    let mut hs = Driver::<peripherals::USBHSD>::new(p.USBHSD, Irqs, Memory::usb1_sram());

    defmt::assert!(
        hs.alloc_endpoint_out(EndpointType::Bulk, None, 512, 0).is_ok(),
        "hs 1: bulk OUT 512 must be accepted (index 1 OUT)"
    );
    defmt::assert!(
        hs.alloc_endpoint_out(EndpointType::Bulk, None, 1024, 0).is_err(),
        "hs 2: bulk OUT 1024 exceeds the 512-byte high-speed bulk cap"
    );
    defmt::assert!(
        hs.alloc_endpoint_in(EndpointType::Interrupt, None, 1024, 0).is_ok(),
        "hs 3: interrupt IN 1024 must be accepted (index 1 IN)"
    );
    defmt::assert!(
        hs.alloc_endpoint_out(EndpointType::Isochronous, None, 1024, 0).is_ok(),
        "hs 4: isochronous OUT 1024 must be accepted (index 2 OUT)"
    );
    defmt::assert!(
        hs.alloc_endpoint_in(EndpointType::Isochronous, None, 1024, 0).is_err(),
        "hs 5: isochronous IN 1024 must be rejected (erratum USB.6)"
    );
    defmt::assert!(
        hs.alloc_endpoint_in(EndpointType::Isochronous, None, 1023, 0).is_ok(),
        "hs 6: isochronous IN 1023 must be accepted (index 2 IN)"
    );
    defmt::assert!(
        hs.alloc_endpoint_in(EndpointType::Control, None, 64, 0).is_err(),
        "hs 7: control endpoints beyond EP0 must be rejected"
    );
    defmt::assert!(
        hs.alloc_endpoint_in(
            EndpointType::Bulk,
            Some(EndpointAddress::from_parts(1, Direction::In)),
            512,
            0
        )
        .is_err(),
        "hs 8: index 1 IN is already taken"
    );

    // The 4096-byte multi-packet slots no longer fit alongside the isochronous
    // endpoints allocated above, so this exercises the driver's single-packet
    // fallback: it must succeed anyway.
    defmt::assert!(
        hs.alloc_endpoint_out(EndpointType::Bulk, None, 512, 0).is_ok(),
        "hs 9: bulk OUT 512 must fall back to single-packet slots (index 3 OUT)"
    );

    // Index exhaustion on the IN side, using 64-byte endpoints so the limit
    // reached is EP_COUNT and not the remaining endpoint memory. Indices 1 and
    // 2 IN are taken, so exactly three more must fit.
    for i in 3..=5 {
        defmt::assert!(
            hs.alloc_endpoint_in(EndpointType::Interrupt, None, 64, 0).is_ok(),
            "hs 10: interrupt IN 64 must be accepted at index {}",
            i
        );
    }
    defmt::assert!(
        hs.alloc_endpoint_in(EndpointType::Interrupt, None, 64, 0).is_err(),
        "hs 11: no IN endpoint index is left"
    );

    // ---------------------------------------------------------------- USB0 FS
    // EP_COUNT = 5, so data endpoint indices 1..=4 are usable per direction.
    let mem = Memory::buffer(unsafe { &mut (*(&raw mut EP_MEM)).0 });
    let mut fs = Driver::<peripherals::USB0>::new(p.USB0, Irqs, p.PIO0_22, mem);

    defmt::assert!(
        fs.alloc_endpoint_out(EndpointType::Bulk, None, 64, 0).is_ok(),
        "fs 1: bulk OUT 64 must be accepted (index 1 OUT)"
    );
    defmt::assert!(
        fs.alloc_endpoint_out(EndpointType::Bulk, None, 128, 0).is_err(),
        "fs 2: bulk OUT 128 exceeds the 64-byte full-speed bulk cap"
    );
    defmt::assert!(
        fs.alloc_endpoint_in(EndpointType::Interrupt, None, 64, 0).is_ok(),
        "fs 3: interrupt IN 64 must be accepted (index 1 IN)"
    );
    defmt::assert!(
        fs.alloc_endpoint_in(EndpointType::Interrupt, None, 128, 0).is_err(),
        "fs 4: interrupt IN 128 exceeds the 64-byte full-speed interrupt cap"
    );
    defmt::assert!(
        fs.alloc_endpoint_out(EndpointType::Isochronous, None, 1023, 0).is_ok(),
        "fs 5: isochronous OUT 1023 must be accepted (index 2 OUT)"
    );
    defmt::assert!(
        fs.alloc_endpoint_out(EndpointType::Isochronous, None, 1024, 0).is_err(),
        "fs 6: isochronous OUT 1024 exceeds the full-speed isochronous cap"
    );

    // Indices 3 and 4 OUT, then index exhaustion.
    for i in 3..=4 {
        defmt::assert!(
            fs.alloc_endpoint_out(EndpointType::Bulk, None, 64, 0).is_ok(),
            "fs 7: bulk OUT 64 must be accepted at index {}",
            i
        );
    }
    defmt::assert!(
        fs.alloc_endpoint_out(EndpointType::Bulk, None, 64, 0).is_err(),
        "fs 8: no OUT endpoint index is left"
    );

    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}

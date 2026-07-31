//! USB0 endpoint allocation at the 512-byte minimum.
//!
//! This test needs no host cable. It checks that EP0 remains reserved when the
//! remaining memory can hold only one double-buffered 64-byte data endpoint.

#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_nxp::usb::{Driver, InterruptHandler, Memory};
use embassy_nxp::{bind_interrupts, peripherals};
use embassy_usb::driver::{Driver as _, EndpointAllocError, EndpointType};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB0 => InterruptHandler<peripherals::USB0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nxp::init(embassy_nxp::config::Config::default());

    let mut storage = [0u8; 512 + 255];
    let offset = storage.as_ptr().align_offset(256);
    defmt::assert!(offset <= 255, "failed to find 256-byte alignment");
    let endpoint_memory = &mut storage[offset..offset + 512];
    defmt::assert_eq!(endpoint_memory.as_ptr() as usize & 0xff, 0);

    let memory = Memory::buffer(endpoint_memory);
    let mut driver = Driver::<peripherals::USB0>::new(p.USB0, Irqs, p.PIO0_22, memory);

    let endpoint = match driver.alloc_endpoint_out(EndpointType::Bulk, None, 64, 0) {
        Ok(endpoint) => endpoint,
        Err(_) => defmt::panic!("first 64-byte endpoint did not fit in 512 bytes"),
    };
    defmt::assert!(
        matches!(
            driver.alloc_endpoint_out(EndpointType::Bulk, None, 64, 0),
            Err(EndpointAllocError)
        ),
        "second 64-byte endpoint fit in 512 bytes"
    );

    let (bus, control) = driver.start(64);
    drop(endpoint);
    drop(control);
    drop(bus);

    defmt::info!("Test OK");
    cortex_m::asm::bkpt();
}

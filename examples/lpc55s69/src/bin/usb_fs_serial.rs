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

use defmt::{info, panic};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_nxp::usb::{Driver, InterruptHandler, Memory};
use embassy_nxp::{bind_interrupts, peripherals};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB0 => InterruptHandler<peripherals::USB0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nxp::init(embassy_nxp::config::Config::default());

    info!("Initialization complete");

    // Create the driver, from the HAL. PIO0_22 is the USB0_VBUS sense pin.
    let driver = Driver::<peripherals::USB0>::new(p.USB0, Irqs, p.PIO0_22, Memory::usb1_sram());

    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-FS serial example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut state = State::new();

    let mut builder = embassy_usb::Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    // Create classes on the builder. 64-byte bulk max packet size for FS.
    let mut class = CdcAcmClass::new(&mut builder, &mut state, 64);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    // Do stuff with the class!
    let echo_fut = async {
        loop {
            class.wait_connection().await;
            info!("Connected");
            let _ = echo(&mut class).await;
            info!("Disconnected");
        }
    };

    // Run everything concurrently.
    join(usb_fut, echo_fut).await;
}

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

async fn echo<'d>(class: &mut CdcAcmClass<'d, Driver<'d, peripherals::USB0>>) -> Result<(), Disconnected> {
    const MPS: usize = 64;
    let mut buf = [0; MPS];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        info!("data: {:x}", data);
        class.write_packet(data).await?;
        // An echo of exactly one max-packet-size packet needs a zero-length
        // packet to end the transfer: without it the host keeps waiting for
        // more and never delivers the reply.
        if n == MPS {
            class.write_packet(&[]).await?;
        }
    }
}

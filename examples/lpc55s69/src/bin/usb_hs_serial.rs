//! USB1 high-speed CDC-ACM serial port example for the LPC55S6x.
//!
//! Creates a USB serial port on the USB1 (high-speed) port that echoes
//! received data. The device enumerates at 480 Mbps.

#![no_std]
#![no_main]

use defmt::{info, panic};
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_nxp::config::MainClock;
use embassy_nxp::usb::{Driver, InterruptHandler};
use embassy_nxp::{bind_interrupts, peripherals};
use embassy_usb::UsbDeviceSpeed;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USB1 => InterruptHandler<peripherals::USBHSD>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = embassy_nxp::config::Config::default();
    // USB-HS requires a system clock of at least 96 MHz.
    config.main_clock = MainClock::FroHf96;
    let p = embassy_nxp::init(config);

    info!("Initialization complete");

    // Create the driver, from the HAL.
    let driver = Driver::new(p.USBHSD, Irqs);

    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-HS serial example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;
    config.max_speed = UsbDeviceSpeed::High;

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

    // Create classes on the builder. 512-byte bulk max packet size for HS.
    let mut class = CdcAcmClass::new(&mut builder, &mut state, 512);

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

async fn echo<'d>(class: &mut CdcAcmClass<'d, Driver<'d, peripherals::USBHSD>>) -> Result<(), Disconnected> {
    let mut buf = [0; 512];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        info!("data: {:x}", data);
        class.write_packet(data).await?;
    }
}

//! USB CDC-ACM serial example with VBUS detection.
//!
//! The NUCLEO-G0B1RE (MB1360) has no USB connector: USB_DM and USB_DP are on
//! CN10 pins 14 and 12. Wire a USB connector there, and wire VBUS through a
//! divider to PA9 (D8, CN10-21) so that PA9 never exceeds VDD. For example:
//!
//! ```text
//! VBUS --- 10k ---+--- PA9
//!                 |
//!                15k
//!                 |
//!                GND
//! ```
//!
//! With VBUS at 5 V this gives 3.0 V on PA9. Adjust the divider for your board;
//! the input must stay within the MCU's supply range.
//!
//! When VBUS goes low, `Driver` reports `PowerRemoved` and disables the USB
//! endpoints, so `read_packet`/`write_packet` return `EndpointError::Disabled`
//! instead of blocking until a timeout. When VBUS returns, the device
//! re-enumerates.

#![no_std]
#![no_main]

use defmt::{panic, *};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::exti::{self, ExtiInput};
use embassy_stm32::gpio::Pull;
use embassy_stm32::usb::{Driver, Instance};
use embassy_stm32::{Config, bind_interrupts, interrupt, peripherals, usb};
use embassy_usb::Builder;
use embassy_usb::class::cdc_acm::device::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use panic_probe as _;

bind_interrupts!(struct Irqs {
    USB_UCPD1_2 => usb::InterruptHandler<peripherals::USB>;
    EXTI4_15 => exti::InterruptHandler<interrupt::typelevel::EXTI4_15>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi48 = Some(Hsi48Config { sync_from_usb: true });
        config.rcc.mux.usbsel = mux::Usbsel::Hsi48;
    }
    let p = embassy_stm32::init(config);

    info!("Hello World!");

    // VBUS from the USB connector, divided down to a 3.3 V-safe level.
    let vbus = ExtiInput::new(p.PA9, p.EXTI9, Pull::None, Irqs);

    // Create the driver, from the HAL.
    let driver = Driver::new_with_vbus(p.USB, p.PA12, p.PA11, vbus, Irqs);

    // Create embassy-usb Config
    let config = embassy_usb::Config::new(0xc0de, 0xcafe);
    //config.max_packet_size_0 = 64;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 7];

    let mut state = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [], // no msos descriptors
        &mut control_buf,
    );

    // Create classes on the builder.
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
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
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

async fn echo<'d, T: Instance + 'd>(class: &mut CdcAcmClass<'d, Driver<'d, T>>) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    loop {
        let n = class.read_packet(&mut buf).await?;
        let data = &buf[..n];
        info!("data: {:x}", data);
        class.write_packet(data).await?;
    }
}

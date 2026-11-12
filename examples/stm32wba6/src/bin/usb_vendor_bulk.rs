//! This example demonstrates the Generic Vendor Bulk USB class on STM32WBA6.
//!
//! Exposes a high-throughput raw bulk IN and bulk OUT interface paired with:
//! - Microsoft OS 2.0 WinUSB descriptors (driverless auto-binding on Windows).
//! - WebUSB Platform Capability (direct access via browser / Chrome `navigator.usb`).
//! - Echo loop using `split()` into separate concurrent reader and writer endpoints.

#![no_std]
#![no_main]

use defmt::{info, warn};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::usb::Driver;
use embassy_stm32::{Config, bind_interrupts, peripherals, usb};
use embassy_usb::Builder;
use embassy_usb::class::vendor::{Config as VendorConfig, State, Url, VendorClass};
use embassy_usb::driver::EndpointError;
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USB_OTG_HS => usb::InterruptHandler<peripherals::USB_OTG_HS>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Starting STM32WBA6 USB Vendor Bulk example...");

    let mut config = Config::default();

    {
        use embassy_stm32::rcc::*;
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hsi,
            prediv: PllPreDiv::Div1,   // PLLM = 1 → HSI / 1 = 16 MHz
            mul: PllMul::Mul30,        // PLLN = 30 → 16 MHz * 30 = 480 MHz VCO
            divr: Some(PllDiv::Div5),  // PLLR = 5 → 96 MHz (Sysclk)
            divq: Some(PllDiv::Div10), // PLLQ = 10 → 48 MHz
            divp: Some(PllDiv::Div30), // PLLP = 30 → 16 MHz (USB_OTG_HS)
            frac: Some(0),             // Fractional part (disabled)
        });
        config.rcc.mux.otghssel = mux::Otghssel::Pll1P;
    }

    let p = embassy_stm32::init(config);

    // Create the driver from the HAL.
    static EP_OUT_BUFFER: StaticCell<[u8; 256]> = StaticCell::new();
    let mut usb_config = embassy_stm32::usb::Config::default();
    usb_config.vbus_detection = false;
    let driver = Driver::new_hs(
        p.USB_OTG_HS,
        p.PD6,
        p.PD7,
        Irqs,
        EP_OUT_BUFFER.init([0u8; 256]),
        usb_config,
    );

    // Create embassy-usb DeviceBuilder.
    let mut usb_builder_config = embassy_usb::Config::new(0xc0de, 0xcafe);
    usb_builder_config.manufacturer = Some("Embassy");
    usb_builder_config.product = Some("Vendor Bulk Demo");
    usb_builder_config.serial_number = Some("12345678");
    usb_builder_config.max_power = 100;
    usb_builder_config.max_packet_size_0 = 64;

    static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static MSOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 128]> = StaticCell::new();

    let mut builder = Builder::new(
        driver,
        usb_builder_config,
        CONFIG_DESCRIPTOR.init([0; 256]),
        BOS_DESCRIPTOR.init([0; 256]),
        MSOS_DESCRIPTOR.init([0; 256]),
        CONTROL_BUF.init([0; 128]),
    );

    // Create vendor class with WinUSB and WebUSB support.
    static VENDOR_STATE: StaticCell<State<'static>> = StaticCell::new();
    let vendor_config = VendorConfig::new()
        .with_interface_name("Vendor Bulk Interface")
        .with_winusb(&["{2494f6f8-7443-41bb-a537-c1c5e478566a}"])
        .with_webusb(Some(Url::new("https://github.com/embassy-rs/embassy")), 0x01);

    let vendor = VendorClass::new(&mut builder, VENDOR_STATE.init(State::new()), vendor_config);

    let mut usb = builder.build();

    let usb_fut = usb.run();

    let (mut sender, mut receiver) = vendor.split();

    let echo_fut = async {
        loop {
            receiver.wait_connection().await;
            info!("USB Vendor connected!");

            let mut buf = [0u8; 64];
            loop {
                match receiver.read_packet(&mut buf).await {
                    Ok(n) => {
                        info!("Received {} bytes, echoing back...", n);
                        if let Err(e) = sender.write_packet(&buf[..n]).await {
                            warn!("Send failed: {:?}", defmt::Debug2Format(&e));
                            break;
                        }
                    }
                    Err(EndpointError::Disabled) => {
                        info!("USB disconnected");
                        break;
                    }
                    Err(EndpointError::BufferOverflow) => {
                        warn!("Buffer overflow on read");
                        break;
                    }
                }
            }
        }
    };

    join(usb_fut, echo_fut).await;
}

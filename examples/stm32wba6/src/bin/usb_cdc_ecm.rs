//! This example demonstrates the CDC-ECM USB Ethernet class on STM32WBA6.
//!
//! Exposes a standard USB Ethernet interface:
//! - Reports Ethernet networking control and data interfaces.
//! - MAC address configuration descriptor.
//! - Receives incoming Ethernet frames and logs them via defmt.

#![no_std]
#![no_main]

use defmt::{info, warn};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::usb::Driver;
use embassy_stm32::{Config, bind_interrupts, peripherals, usb};
use embassy_usb::Builder;
use embassy_usb::class::cdc_ecm::device::{CdcEcmClass, State};
use embassy_usb::driver::EndpointError;
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USB_OTG_HS => usb::InterruptHandler<peripherals::USB_OTG_HS>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Starting STM32WBA6 USB CDC-ECM Ethernet example...");

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
    let mut usb_builder_config = embassy_usb::Config::new(0x1209, 0x0004);
    usb_builder_config.manufacturer = Some("Embassy");
    usb_builder_config.product = Some("CDC-ECM Ethernet Demo");
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

    static ECM_STATE: StaticCell<State<'static>> = StaticCell::new();
    let ecm_state = ECM_STATE.init(State::new());

    let mac_address = [0x02, 0x00, 0x00, 0x00, 0x00, 0x01];
    let ecm = CdcEcmClass::new(&mut builder, ecm_state, mac_address, 64);

    let mut usb = builder.build();

    let usb_fut = usb.run();

    let (mut _sender, mut receiver) = ecm.split();

    let net_fut = async {
        loop {
            if let Err(e) = receiver.wait_connection().await {
                warn!("wait_connection failed: {:?}", defmt::Debug2Format(&e));
                continue;
            }
            info!("CDC-ECM USB Ethernet connected!");

            let mut buf = [0u8; 1514];
            loop {
                match receiver.read_packet(&mut buf).await {
                    Ok(n) => {
                        info!("Received Ethernet frame ({} bytes)", n);
                    }
                    Err(EndpointError::Disabled) => {
                        info!("CDC-ECM disconnected");
                        break;
                    }
                    Err(EndpointError::BufferOverflow) => {
                        warn!("CDC-ECM buffer overflow");
                        break;
                    }
                }
            }
        }
    };

    join(usb_fut, net_fut).await;
}

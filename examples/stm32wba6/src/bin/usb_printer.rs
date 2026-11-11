//! This example demonstrates the USB Printer Device class on STM32WBA6.
//!
//! Emulates a bidirectional USB thermal receipt printer:
//! - Reports IEEE 1284 Device ID query.
//! - Implements standard printer port status (Paper present, Selected, No error).
//! - Receives incoming print jobs and logs them via defmt.

#![no_std]
#![no_main]

use defmt::{info, warn};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_stm32::usb::Driver;
use embassy_stm32::{Config, bind_interrupts, peripherals, usb};
use embassy_usb::Builder;
use embassy_usb::class::printer::PortStatus;
use embassy_usb::class::printer::device::{Config as PrinterConfig, PrinterClass, State};
use embassy_usb::driver::EndpointError;
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USB_OTG_HS => usb::InterruptHandler<peripherals::USB_OTG_HS>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Starting STM32WBA6 USB Printer example...");

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
    let mut usb_builder_config = embassy_usb::Config::new(0x1209, 0x0003);
    usb_builder_config.manufacturer = Some("Embassy");
    usb_builder_config.product = Some("Virtual Receipt Printer");
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

    static PRINTER_STATE: StaticCell<State<'static>> = StaticCell::new();
    let printer_state = PRINTER_STATE.init(State::new());

    let printer_config = PrinterConfig {
        device_id: Some("MFG:Embassy;CMD:ESC/POS;MDL:WBA65 Receipt Printer;CLS:PRINTER;"),
        initial_status: PortStatus::default(),
        ..Default::default()
    };

    let mut printer = PrinterClass::new(&mut builder, printer_state, printer_config);

    let mut usb = builder.build();

    let usb_fut = usb.run();

    let print_fut = async {
        let mut buf = [0u8; 64];
        loop {
            match printer.read(&mut buf).await {
                Ok(n) => {
                    info!("Received print job data: {} bytes", n);
                    if let Ok(text) = core::str::from_utf8(&buf[..n]) {
                        info!("Print data: {:?}", text);
                    } else {
                        info!("Print raw bytes: {=[u8]:x}", &buf[..n]);
                    }
                }
                Err(EndpointError::Disabled) => {
                    // USB disconnected or reset
                    embassy_time::Timer::after_millis(100).await;
                }
                Err(EndpointError::BufferOverflow) => {
                    warn!("Printer buffer overflow");
                }
            }
        }
    };

    join(usb_fut, print_fut).await;
}

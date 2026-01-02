//! This example shows how to use USB (Universal Serial Bus) in the RP235x chip.
//!
//! This creates a USB serial port that sends/receives data on uart (USB-to-Uart).
//!
//! No specific hardware is specified in this example. If you connect pin 0 (TX) and 1 (RX) you should get the same data back.
//! The Raspberry Pi Debug Probe (https://www.raspberrypi.com/products/debug-probe/) could be used
//! with its UART port.

#![no_std]
#![no_main]

use defmt::{Debug2Format, error, info};
use embassy_executor::Spawner;
use embassy_futures::join::join3;
use embassy_futures::select::{Either, select};
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::{UART0, USB};
use embassy_rp::uart::{BufferedInterruptHandler, BufferedUart, Config};
use embassy_rp::usb::{Driver as UsbDriver, InterruptHandler as UsbInterruptHandler};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embedded_io_async::{BufRead, Write};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
    USBCTRL_IRQ => UsbInterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("USB-to-uart example!");

    let p = embassy_rp::init(Default::default());

    let driver = UsbDriver::new(p.USB, Irqs);

    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("Usb-to-uart example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    let mut builder = {
        static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

        let builder = embassy_usb::Builder::new(
            driver,
            config,
            CONFIG_DESCRIPTOR.init([0; 256]),
            BOS_DESCRIPTOR.init([0; 256]),
            &mut [], // no msos descriptors
            CONTROL_BUF.init([0; 64]),
        );
        builder
    };

    // Create classes on the builder.
    let class = {
        static STATE: StaticCell<State> = StaticCell::new();
        let state = STATE.init(State::new());
        CdcAcmClass::new(&mut builder, state, 64)
    };

    let (mut usb_tx, mut usb_rx, usb_control) = class.split_with_control();

    let (tx_pin, rx_pin, uart) = (p.PIN_0, p.PIN_1, p.UART0);

    static TX_BUF: StaticCell<[u8; 64]> = StaticCell::new();
    let tx_buf = &mut TX_BUF.init([0; 64])[..];
    static RX_BUF: StaticCell<[u8; 64]> = StaticCell::new();
    let rx_buf = &mut RX_BUF.init([0; 64])[..];
    let uart = BufferedUart::new(uart, tx_pin, rx_pin, Irqs, tx_buf, rx_buf, Config::default());
    let (mut uart_tx, mut uart_rx) = uart.split();

    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    let usb_tx_uart_rx = async {
        loop {
            usb_tx.wait_connection().await;
            info!("USB Uart RX Connected");
            loop {
                match uart_rx.fill_buf().await {
                    Ok(usart_buf) => match usb_tx.write(usart_buf).await {
                        Err(err) => {
                            error!("Usb write error: {:?}. Assume disconnection", Debug2Format(&err));
                            break;
                        }
                        Ok(n) => uart_rx.consume(n),
                    },
                    Err(err) => {
                        error!("Uart read error: {:?}", Debug2Format(&err));
                    }
                }
            }
        }
    };

    let mut usb_buf = [0; 64];
    let usb_rx_uart_tx = async {
        loop {
            usb_rx.wait_connection().await;
            info!("USB Uart TX Connected");
            loop {
                match select(usb_control.control_changed(), usb_rx.read_packet(&mut usb_buf)).await {
                    Either::First(_) => {
                        let baud = usb_rx.line_coding().data_rate();
                        info!("Setting baud to: {}", baud);
                        uart_tx.set_baudrate(baud);
                    }
                    Either::Second(Err(err)) => {
                        error!("Usb read error: {:?}. Assume disconnection", Debug2Format(&err));
                        break;
                    }
                    Either::Second(Ok(n)) => {
                        let mut bytes_sent = 0;
                        loop {
                            match uart_tx.write(&usb_buf[bytes_sent..n]).await {
                                Err(err) => {
                                    error!(
                                        "Unable to write to usart: {:?}. Disguarding all remaining ({}) bytes",
                                        Debug2Format(&err),
                                        n - bytes_sent
                                    );
                                    break;
                                }
                                Ok(sent) => bytes_sent += sent,
                            }
                            if bytes_sent == n {
                                break;
                            }
                        }
                    }
                }
            }
        }
    };

    join3(usb_fut, usb_rx_uart_tx, usb_tx_uart_rx).await;
}

//! This example shows how to use USB Bluetooth Host Controller Interface (HCI) on the STM32WBA6 chip.
//!
//! Uses the High-Speed USB peripheral (USB_OTG_HS) to expose a Bluetooth HCI controller interface,
//! handling standard HCI control commands and echoing ACL data.

#![no_std]
#![no_main]

use defmt::{info, warn};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join3;
use embassy_stm32::usb::Driver;
use embassy_stm32::{Config, bind_interrupts, peripherals, usb};
use embassy_usb::Builder;
use embassy_usb::class::bth::device::{
    BluetoothHci, Config as BthConfig, State, USB_CLASS_WIRELESS_CONTROLLER, USB_PROTOCOL_BLUETOOTH_PRIMARY,
    USB_SUBCLASS_RF_CONTROLLER,
};
use embassy_usb::driver::EndpointError;
use panic_probe as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USB_OTG_HS => usb::InterruptHandler<peripherals::USB_OTG_HS>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Starting STM32WBA6 USB Bluetooth HCI example...");

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

    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0xcafe, 0x4010);
    config.manufacturer = Some("Embassy");
    config.product = Some("STM32WBA6 Bluetooth HCI");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;
    config.composite_with_iads = false;
    config.device_class = USB_CLASS_WIRELESS_CONTROLLER;
    config.device_sub_class = USB_SUBCLASS_RF_CONTROLLER;
    config.device_protocol = USB_PROTOCOL_BLUETOOTH_PRIMARY;

    let mut builder = {
        static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static CONTROL_BUF: StaticCell<[u8; 128]> = StaticCell::new();

        Builder::new(
            driver,
            config,
            CONFIG_DESCRIPTOR.init([0; 256]),
            BOS_DESCRIPTOR.init([0; 256]),
            &mut [],
            CONTROL_BUF.init([0; 128]),
        )
    };

    let bth_config = BthConfig::default();
    static STATE: StaticCell<State> = StaticCell::new();
    let state = STATE.init(State::new());

    let bth = BluetoothHci::new(&mut builder, state, bth_config);
    let (mut cmd_reader, mut event_writer, mut acl_reader, mut acl_writer) = bth.split();

    let mut usb = builder.build();

    let usb_fut = usb.run();

    let cmd_fut = async {
        let mut cmd_buf = [0u8; 258];
        loop {
            match cmd_reader.read_command(&mut cmd_buf).await {
                Ok(n) => {
                    if n < 3 {
                        warn!("Received truncated HCI command of len {}", n);
                        continue;
                    }

                    let opcode = u16::from_le_bytes([cmd_buf[0], cmd_buf[1]]);
                    let param_len = cmd_buf[2] as usize;
                    let _params = &cmd_buf[3..core::cmp::min(n, 3 + param_len)];

                    info!("HCI Command: opcode=0x{:04x}, param_len={}", opcode, param_len);

                    match opcode {
                        // HCI_Reset (0x0C03)
                        0x0C03 => {
                            let event = [0x0E, 0x04, 0x01, 0x03, 0x0C, 0x00];
                            let _ = event_writer.send_event(&event).await;
                        }
                        // HCI_Read_Local_Version_Information (0x1001)
                        0x1001 => {
                            let mut event = [0u8; 14];
                            event[0] = 0x0E;
                            event[1] = 0x0C;
                            event[2] = 0x01;
                            event[3] = 0x01;
                            event[4] = 0x10;
                            event[5] = 0x00;
                            event[6] = 0x0D; // HCI Version 5.4
                            event[7..9].copy_from_slice(&0x0001u16.to_le_bytes());
                            event[9] = 0x0D; // LMP Version
                            event[10..12].copy_from_slice(&0x0030u16.to_le_bytes()); // STMicroelectronics (0x0030)
                            event[12..14].copy_from_slice(&0x0001u16.to_le_bytes());
                            let _ = event_writer.send_event(&event).await;
                        }
                        // HCI_Read_BD_ADDR (0x1009)
                        0x1009 => {
                            let event = [0x0E, 0x0A, 0x01, 0x09, 0x10, 0x00, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
                            let _ = event_writer.send_event(&event).await;
                        }
                        // HCI_Read_Buffer_Size (0x1005)
                        0x1005 => {
                            let mut event = [0u8; 13];
                            event[0] = 0x0E;
                            event[1] = 0x0B;
                            event[2] = 0x01;
                            event[3] = 0x05;
                            event[4] = 0x10;
                            event[5] = 0x00;
                            event[6..8].copy_from_slice(&256u16.to_le_bytes());
                            event[8] = 0;
                            event[9..11].copy_from_slice(&4u16.to_le_bytes());
                            event[11..13].copy_from_slice(&0u16.to_le_bytes());
                            let _ = event_writer.send_event(&event).await;
                        }
                        _ => {
                            let opcode_bytes = opcode.to_le_bytes();
                            let event = [0x0E, 0x04, 0x01, opcode_bytes[0], opcode_bytes[1], 0x00];
                            let _ = event_writer.send_event(&event).await;
                        }
                    }
                }
                Err(e) => {
                    warn!("Error reading HCI command: {:?}", defmt::Debug2Format(&e));
                }
            }
        }
    };

    let acl_fut = async {
        let mut acl_buf = [0u8; 512];
        loop {
            acl_reader.wait_connection().await;
            info!("ACL endpoint connected");
            loop {
                match acl_reader.read_acl(&mut acl_buf).await {
                    Ok(n) => {
                        info!("Received ACL packet of {} bytes; echoing back", n);
                        let _ = acl_writer.send_acl(&acl_buf[..n]).await;
                    }
                    Err(EndpointError::Disabled) => {
                        info!("ACL endpoint disabled/disconnected");
                        break;
                    }
                    Err(e) => {
                        warn!("Error reading ACL data: {:?}", defmt::Debug2Format(&e));
                    }
                }
            }
        }
    };

    join3(usb_fut, cmd_fut, acl_fut).await;
}

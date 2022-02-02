#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use defmt::{info, unwrap};
use embassy::interrupt::InterruptExt;
use futures::pin_mut;

use embassy::executor::Spawner;
use embassy::io::{AsyncBufReadExt, AsyncWriteExt};
use embassy_stm32::usb::{Peripheral, State, Usb, UsbBus, UsbSerial};
use embassy_stm32::{interrupt, rcc, time::U32Ext, Peripherals};
use usb_device::device::{UsbDeviceBuilder, UsbVidPid};

fn config() -> embassy_stm32::Config {
    let mut config = embassy_stm32::Config::default();
    config.rcc.mux = rcc::ClockSrc::PLL(
        rcc::PLLSource::HSE(16.mhz().into()),
        rcc::PLLMul::Mul6,
        rcc::PLLDiv::Div4,
    );
    config
}

#[embassy::main(config = "config()")]
async fn main(_spawner: Spawner, _p: Peripherals) {
    let mut rx_buffer = [0u8; 64];
    // we send back input + cr + lf
    let mut tx_buffer = [0u8; 66];

    let usb_bus = UsbBus::new(Peripheral {});

    let serial = UsbSerial::new(&usb_bus, &mut rx_buffer, &mut tx_buffer);

    let device = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(0x02)
        .build();

    let irq = interrupt::take!(USB_LP);
    irq.set_priority(interrupt::Priority::P3);

    let mut state = State::new();
    let usb = unsafe { Usb::new(&mut state, device, serial, irq) };
    pin_mut!(usb);

    let (mut reader, mut writer) = usb.as_ref().take_serial_0();

    info!("usb initialized!");

    unwrap!(
        writer
            .write_all(b"\r\nInput returned upper cased on CR or LF\r\n")
            .await
    );

    let mut buf = [0u8; 64];
    loop {
        let mut n = 0;

        async {
            loop {
                let char = unwrap!(reader.read_byte().await);

                if char == b'\r' || char == b'\n' {
                    break;
                }

                buf[n] = char;
                n += 1;

                // stop if we're out of room
                if n == buf.len() {
                    break;
                }
            }
        }
        .await;

        if n > 0 {
            for char in buf[..n].iter_mut() {
                // upper case
                if 0x61 <= *char && *char <= 0x7a {
                    *char &= !0x20;
                }
            }
            unwrap!(writer.write_all(&buf[..n]).await);
            unwrap!(writer.write_all(b"\r\n").await);
            unwrap!(writer.flush().await);
        }
    }
}

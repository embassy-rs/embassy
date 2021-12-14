#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use defmt::{info, unwrap};
use defmt_rtt as _;
use embassy::interrupt::InterruptExt;
use futures::future::{select, Either};
use futures::pin_mut;
// global logger
use panic_probe as _; // print out panic messages

use embassy::executor::Spawner;
use embassy::io::{AsyncBufReadExt, AsyncWriteExt};
use embassy::time::{Duration, Timer};
use embassy_hal_common::usb::{State, Usb, UsbSerial};
use embassy_nrf::usb::{Usb as UsbDevice, UsbBus};
use embassy_nrf::{interrupt, Peripherals};
use usb_device::device::{UsbDeviceBuilder, UsbVidPid};

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let mut tx_buffer = [0u8; 1024];
    let mut rx_buffer = [0u8; 640];

    let _usb_dev = UsbDevice::new(p.USBD);

    let usb_bus = UsbBus::new();

    let serial = UsbSerial::new(&usb_bus, &mut rx_buffer, &mut tx_buffer);

    let device = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(0x02)
        .build();

    let irq = interrupt::take!(USBD);
    irq.set_priority(interrupt::Priority::P3);

    let mut state = State::new();

    let usb = unsafe { Usb::new(&mut state, device, serial, irq) };
    pin_mut!(usb);
    // usb.start();

    let (mut read_interface, mut write_interface) = usb.as_ref().take_serial_0();

    unwrap!(write_interface.write_all(b"\r\nSend something\r\n").await);

    info!("usb initialized!");

    let mut buf = [0u8; 64];
    loop {
        let mut n = 0;
        let left = {
            let recv_fut = async {
                loop {
                    let byte = unwrap!(read_interface.read_byte().await);
                    unwrap!(write_interface.write_byte(byte).await);
                    buf[n] = byte;

                    n += 1;
                    if byte == b'\n' || byte == b'\r' || n == buf.len() {
                        break;
                    }
                }
            };
            pin_mut!(recv_fut);

            let timeout = Timer::after(Duration::from_ticks(32768 * 10));

            match select(recv_fut, timeout).await {
                Either::Left(_) => true,
                Either::Right(_) => false,
            }
        };

        if left {
            for c in buf[..n].iter_mut() {
                if 0x61 <= *c && *c <= 0x7a {
                    *c &= !0x20;
                }
            }
            unwrap!(write_interface.write_byte(b'\n').await);
            unwrap!(write_interface.write_all(&buf[..n]).await);
            unwrap!(write_interface.write_byte(b'\n').await);
        } else {
            unwrap!(write_interface.write_all(b"\r\nSend something\r\n").await);
        }
    }
}

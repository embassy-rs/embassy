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
use embassy_nrf::usb::{State, Usb, UsbBus, UsbSerial};
use embassy_nrf::{interrupt, Peripherals};
use usb_device::device::{UsbDeviceBuilder, UsbVidPid};

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let mut tx_buffer = [0u8; 1024];
    let mut rx_buffer = [0u8; 640];

    let usb_bus = UsbBus::new(p.USBD);

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

    let (mut reader, mut writer) = usb.as_ref().take_serial_0();

    unwrap!(writer.write_all(b"\r\nSend something\r\n").await);

    info!("usb initialized!");

    let mut buf = [0u8; 64];
    loop {
        let mut n = 0;
        let timed_out = {
            let newline_fut = async {
                loop {
                    let char = unwrap!(reader.read_byte().await);
                    // echo input back to screen
                    unwrap!(writer.write_byte(char).await);
                    buf[n] = char;

                    n += 1;
                    if char == b'\n' || char == b'\r' || n == buf.len() {
                        break;
                    }
                }
            };
            pin_mut!(newline_fut);

            let timeout_fut = Timer::after(Duration::from_ticks(32768 * 10));

            // select chooses whichever returns first
            match select(newline_fut, timeout_fut).await {
                Either::Left(_) => false,
                Either::Right(_) => true,
            }
        };

        if timed_out {
            unwrap!(writer.write_all(b"\r\nTimed out\r\n").await);
        } else {
            for char in buf[..n].iter_mut() {
                // upper case
                if 0x61 <= *char && *char <= 0x7a {
                    *char &= !0x20;
                }
            }
            unwrap!(writer.write_byte(b'\n').await);
            unwrap!(writer.write_all(&buf[..n]).await);
            unwrap!(writer.write_byte(b'\n').await);
        }
    }
}

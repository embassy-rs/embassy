#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use defmt::{info, unwrap};
use defmt_rtt as _; // global logger
use embassy::interrupt::InterruptExt;
use embassy_nrf::usb::{ReadInterface, WriteInterface};
use futures::pin_mut;
use panic_probe as _; // print out panic messages

use embassy::executor::Spawner;
use embassy::io::{AsyncBufReadExt, AsyncWriteExt};
use embassy_nrf::usb::{ClassSet1, Index0, State, Usb, UsbBus, UsbSerial};
use embassy_nrf::{interrupt, Peripherals};
use usb_device::device::{UsbDeviceBuilder, UsbVidPid};

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let mut rx_buffer = [0u8; 64];
    // we send back input + cr + lf
    let mut tx_buffer = [0u8; 66];

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

    type NrfUsbDevice<'d> = nrf_usbd::Usbd<UsbBus<'d, embassy_nrf::peripherals::USBD>>;

    type NrfReadInterface<'a, 'bus, 'c, 'd> = ReadInterface<
        'a,
        'bus,
        'c,
        Index0,
        NrfUsbDevice<'d>,
        ClassSet1<NrfUsbDevice<'d>, UsbSerial<'bus, 'a, NrfUsbDevice<'d>>>,
        interrupt::USBD,
    >;

    type NrfWriteInterface<'a, 'bus, 'c, 'd> = WriteInterface<
        'a,
        'bus,
        'c,
        Index0,
        NrfUsbDevice<'d>,
        ClassSet1<NrfUsbDevice<'d>, UsbSerial<'bus, 'a, NrfUsbDevice<'d>>>,
        interrupt::USBD,
    >;

    let (mut reader, mut writer): (
        NrfReadInterface<'_, '_, '_, '_>,
        NrfWriteInterface<'_, '_, '_, '_>,
    ) = usb.as_ref().take_serial_0();

    info!("usb initialized!");

    unwrap!(
        writer
            .write_all(b"\r\nInput returned upper cased on CR+LF\r\n")
            .await
    );

    let mut buf = [0u8; 64];
    loop {
        let mut n = 0;

        async {
            loop {
                let char = unwrap!(reader.read_byte().await);

                // throw away, read more on cr, exit on lf
                if char == b'\r' {
                    continue;
                } else if char == b'\n' {
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

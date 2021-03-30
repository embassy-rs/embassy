#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(min_type_alias_impl_trait)]
#![feature(impl_trait_in_bindings)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use cortex_m_rt::entry;
use defmt::panic;
use embassy::executor::{Executor, Spawner};
use embassy::interrupt::InterruptExt;
use embassy::io::{AsyncBufReadExt, AsyncWriteExt};
use embassy::time::{Duration, Timer};
use embassy::util::Forever;
use embassy_extras::usb::usb_serial::UsbSerial;
use embassy_extras::usb::Usb;
use embassy_stm32::{interrupt, pac, rtc};
use futures::future::{select, Either};
use futures::pin_mut;
use stm32f4xx_hal::otg_fs::{UsbBus, USB};
use stm32f4xx_hal::prelude::*;
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;

#[embassy::task]
async fn run1(bus: &'static mut UsbBusAllocator<UsbBus<USB>>) {
    info!("Async task");

    let mut read_buf = [0u8; 128];
    let mut write_buf = [0u8; 128];
    let serial = UsbSerial::new(bus, &mut read_buf, &mut write_buf);

    let device = UsbDeviceBuilder::new(bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(0x02)
        .build();

    let irq = interrupt::take!(OTG_FS);
    irq.set_priority(interrupt::Priority::Level3);

    let usb = Usb::new(device, serial, irq);
    pin_mut!(usb);
    usb.as_mut().start();

    let (mut read_interface, mut write_interface) = usb.as_ref().take_serial_0();

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

static USB_BUS: Forever<UsbBusAllocator<UsbBus<USB>>> = Forever::new();

#[embassy::main(use_hse = 25, sysclk = 48, require_pll48clk)]
async fn main(spawner: Spawner) -> ! {
    static mut EP_MEMORY: [u32; 1024] = [0; 1024];

    info!("Hello World!");

    let (p, clocks) = embassy_stm32::Peripherals::take().unwrap();

    let gpioa = p.GPIOA.split();
    let usb = USB {
        usb_global: p.OTG_FS_GLOBAL,
        usb_device: p.OTG_FS_DEVICE,
        usb_pwrclk: p.OTG_FS_PWRCLK,
        pin_dm: gpioa.pa11.into_alternate_af10(),
        pin_dp: gpioa.pa12.into_alternate_af10(),
        hclk: clocks.hclk(),
    };
    // Rust analyzer isn't recognizing the static ref magic `cortex-m` does
    #[allow(unused_unsafe)]
    let usb_bus = USB_BUS.put(UsbBus::new(usb, unsafe { &mut EP_MEMORY }));

    spawner.spawn(run1(usb_bus)).unwrap();
}

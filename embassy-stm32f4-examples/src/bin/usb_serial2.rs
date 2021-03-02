#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use cortex_m_rt::entry;
use defmt::panic;
use embassy::executor::{task, Executor};
use embassy::io::{AsyncBufReadExt, AsyncWriteExt};
use embassy::util::Forever;
use embassy_stm32f4::interrupt::InterruptExt;
use embassy_stm32f4::usb::Usb;
use embassy_stm32f4::usb_serial::UsbSerial;
use embassy_stm32f4::{interrupt, pac};
use futures::future::{select, Either};
use futures::pin_mut;
use stm32f4xx_hal::otg_fs::{UsbBus, USB};
use stm32f4xx_hal::prelude::*;
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;

#[task]
async fn run1(bus: &'static mut UsbBusAllocator<UsbBus<USB>>) {
    info!("Async task");

    let mut read_buf1 = [0u8; 128];
    let mut write_buf1 = [0u8; 128];
    let serial1 = UsbSerial::new(bus, &mut read_buf1, &mut write_buf1);

    let mut read_buf2 = [0u8; 128];
    let mut write_buf2 = [0u8; 128];
    let serial2 = UsbSerial::new(bus, &mut read_buf2, &mut write_buf2);

    let device = UsbDeviceBuilder::new(bus, UsbVidPid(0x16c0, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        //.device_class(0x02)
        .build();

    let irq = interrupt::take!(OTG_FS);
    irq.set_priority(interrupt::Priority::Level3);

    let usb = Usb::new(device, (serial1, serial2), irq);
    pin_mut!(usb);

    let (mut read_interface1, mut write_interface1) = usb.as_ref().take_serial_0();
    let (mut read_interface2, mut write_interface2) = usb.as_ref().take_serial_1();

    let mut buf1 = [0u8; 64];
    let mut buf2 = [0u8; 64];

    loop {
        let mut n1 = 0;
        let mut n2 = 0;
        let left = {
            let read_line1 = async {
                loop {
                    let byte = unwrap!(read_interface1.read_byte().await);
                    unwrap!(write_interface1.write_byte(byte).await);
                    buf1[n1] = byte;

                    n1 += 1;
                    if byte == b'\n' || byte == b'\r' || n1 == buf1.len() {
                        break;
                    }
                }
            };
            pin_mut!(read_line1);

            let read_line2 = async {
                loop {
                    let byte = unwrap!(read_interface2.read_byte().await);
                    unwrap!(write_interface2.write_byte(byte).await);
                    buf2[n2] = byte;

                    n2 += 1;
                    if byte == b'\n' || byte == b'\r' || n2 == buf2.len() {
                        break;
                    }
                }
            };
            pin_mut!(read_line2);

            match select(read_line1, read_line2).await {
                Either::Left(_) => true,
                Either::Right(_) => false,
            }
        };

        if left {
            unwrap!(write_interface2.write_all(b"\r\n").await);
            unwrap!(write_interface2.write_all(&buf1[..n1]).await);
        } else {
            unwrap!(write_interface1.write_all(b"\r\n").await);
            unwrap!(write_interface1.write_all(&buf2[..n2]).await);
        }
    }
}

static EXECUTOR: Forever<Executor> = Forever::new();
static USB_BUS: Forever<UsbBusAllocator<UsbBus<USB>>> = Forever::new();

#[entry]
fn main() -> ! {
    static mut EP_MEMORY: [u32; 1024] = [0; 1024];

    info!("Hello World!");

    let p = unwrap!(pac::Peripherals::take());

    p.RCC.ahb1enr.modify(|_, w| w.dma1en().enabled());
    let rcc = p.RCC.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(25.mhz())
        .sysclk(48.mhz())
        .require_pll48clk()
        .freeze();

    p.DBGMCU.cr.modify(|_, w| {
        w.dbg_sleep().set_bit();
        w.dbg_standby().set_bit();
        w.dbg_stop().set_bit()
    });

    let executor = EXECUTOR.put(Executor::new());

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
    let usb_bus = USB_BUS.put(UsbBus::new(usb, unsafe { EP_MEMORY }));

    executor.run(move |spawner| {
        unwrap!(spawner.spawn(run1(usb_bus)));
    });
}

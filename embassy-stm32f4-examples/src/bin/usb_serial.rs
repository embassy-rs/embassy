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
use embassy::time::{Duration, Timer};
use embassy::util::Forever;
use embassy_stm32f4::interrupt::OwnedInterrupt;
use embassy_stm32f4::usb::Usb;
use embassy_stm32f4::usb_serial::UsbSerial;
use embassy_stm32f4::{interrupt, pac, rtc};
use futures::future::{select, Either};
use futures::pin_mut;
use stm32f4xx_hal::otg_fs::{UsbBus, USB};
use stm32f4xx_hal::prelude::*;
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;

#[task]
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

    let (mut read_interface, mut write_interface) = usb.as_mut().into_ref().take_serial();

    let mut buf = [0u8; 5];
    loop {
        let recv_fut = read_interface.read(&mut buf);
        let timeout = Timer::after(Duration::from_ticks(32768 * 3));

        match select(recv_fut, timeout).await {
            Either::Left((recv, _)) => {
                let recv = unwrap!(recv);
                unwrap!(write_interface.write_all(&buf[..recv]).await);
            }
            Either::Right(_) => {
                unwrap!(write_interface.write_all(b"Hello\r\n").await);
            }
        }
    }
}

static RTC: Forever<rtc::RTC<pac::TIM2>> = Forever::new();
static ALARM: Forever<rtc::Alarm<pac::TIM2>> = Forever::new();
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

    let rtc = RTC.put(rtc::RTC::new(p.TIM2, interrupt::take!(TIM2), clocks));
    rtc.start();

    unsafe { embassy::time::set_clock(rtc) };

    let alarm = ALARM.put(rtc.alarm1());
    let executor = EXECUTOR.put(Executor::new());
    executor.set_alarm(alarm);

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

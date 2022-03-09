#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../../example_common.rs"]
mod example_common;

mod cdc_acm;

use core::mem;
use defmt::*;
use embassy::executor::Spawner;
use embassy_nrf::interrupt;
use embassy_nrf::pac;
use embassy_nrf::usb::{self, Driver};
use embassy_nrf::Peripherals;
use embassy_usb::driver::EndpointOut;
use embassy_usb::{Config, UsbDeviceBuilder};
use futures::future::{join, select};

use crate::cdc_acm::CdcAcmClass;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let clock: pac::CLOCK = unsafe { mem::transmute(()) };
    let power: pac::POWER = unsafe { mem::transmute(()) };

    info!("Enabling ext hfosc...");
    clock.tasks_hfclkstart.write(|w| unsafe { w.bits(1) });
    while clock.events_hfclkstarted.read().bits() != 1 {}

    info!("Waiting for vbus...");
    while !power.usbregstatus.read().vbusdetect().is_vbus_present() {}
    info!("vbus OK");

    let irq = interrupt::take!(USBD);
    let driver = Driver::new(p.USBD, irq);
    let config = Config::new(0xc0de, 0xcafe);
    let mut device_descriptor = [0; 256];
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];

    let mut builder = UsbDeviceBuilder::new(
        driver,
        config,
        &mut device_descriptor,
        &mut config_descriptor,
        &mut bos_descriptor,
    );

    let mut class = CdcAcmClass::new(&mut builder, 64);

    let mut usb = builder.build();

    let fut1 = usb.run();
    let fut2 = async {
        let mut buf = [0; 64];
        loop {
            let n = class.read_ep.read(&mut buf).await.unwrap();
            let data = &buf[..n];
            info!("data: {:x}", data);
        }
    };

    join(fut1, fut2).await;
}

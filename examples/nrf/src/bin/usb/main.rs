#![no_std]
#![no_main]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

#[path = "../../example_common.rs"]
mod example_common;

mod cdc_acm;

use core::mem;
use defmt::*;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_nrf::interrupt;
use embassy_nrf::pac;
use embassy_nrf::usb::Driver;
use embassy_nrf::Peripherals;
use embassy_usb::driver::{EndpointIn, EndpointOut};
use embassy_usb::{Config, UsbDeviceBuilder};
use futures::future::join3;

use crate::cdc_acm::{CdcAcmClass, State};

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

    // Create the driver, from the HAL.
    let irq = interrupt::take!(USBD);
    let driver = Driver::new(p.USBD, irq);

    // Create embassy-usb Config
    let config = Config::new(0xc0de, 0xcafe);

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut device_descriptor = [0; 256];
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 7];

    let mut state = State::new();

    let mut builder = UsbDeviceBuilder::new(
        driver,
        config,
        &mut device_descriptor,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut control_buf,
    );

    // Create classes on the builder.
    let mut class = CdcAcmClass::new(&mut builder, &mut state, 64);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let fut1 = usb.run();

    // Do stuff with the classes
    let fut2 = async {
        let mut buf = [0; 64];
        loop {
            let n = class.read_ep.read(&mut buf).await.unwrap();
            let data = &buf[..n];
            info!("data: {:x}", data);
        }
    };
    let fut3 = async {
        loop {
            info!("writing...");
            class.write_ep.write(b"Hello World!\r\n").await.unwrap();
            info!("written");

            Timer::after(Duration::from_secs(1)).await;
        }
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join3(fut1, fut2, fut3).await;
}

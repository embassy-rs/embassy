#![no_std]
#![no_main]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use core::mem;
use defmt::*;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_nrf::interrupt;
use embassy_nrf::pac;
use embassy_nrf::usb::Driver;
use embassy_nrf::Peripherals;
use embassy_usb::control::OutResponse;
use embassy_usb::{Config, UsbDeviceBuilder};
use embassy_usb_hid::{HidClass, ReportId, RequestHandler, State};
use futures::future::join;
use usbd_hid::descriptor::{MouseReport, SerializedDescriptor};

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
    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Tactile Engineering");
    config.product = Some("Testy");
    config.serial_number = Some("12345678");
    config.max_power = 100;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut device_descriptor = [0; 256];
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut control_buf = [0; 16];
    let request_handler = MyRequestHandler {};

    let mut state = State::<5, 0>::new();

    let mut builder = UsbDeviceBuilder::new(
        driver,
        config,
        &mut device_descriptor,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut control_buf,
    );

    // Create classes on the builder.
    let mut hid = HidClass::new_ep_in(
        &mut builder,
        &mut state,
        MouseReport::desc(),
        Some(&request_handler),
        60,
    );

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    // Do stuff with the class!
    let hid_fut = async {
        loop {
            Timer::after(Duration::from_millis(500)).await;
            hid.input()
                .serialize(&MouseReport {
                    buttons: 0,
                    x: 0,
                    y: 4,
                    wheel: 0,
                    pan: 0,
                })
                .await
                .unwrap();

            Timer::after(Duration::from_millis(500)).await;
            hid.input()
                .serialize(&MouseReport {
                    buttons: 0,
                    x: 0,
                    y: -4,
                    wheel: 0,
                    pan: 0,
                })
                .await
                .unwrap();
        }
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, hid_fut).await;
}

struct MyRequestHandler {}

impl RequestHandler for MyRequestHandler {
    fn get_report(&self, id: ReportId, _buf: &mut [u8]) -> Option<usize> {
        info!("Get report for {:?}", id);
        None
    }

    fn set_report(&self, id: ReportId, data: &[u8]) -> OutResponse {
        info!("Set report for {:?}: {=[u8]}", id, data);
        OutResponse::Accepted
    }

    fn set_idle(&self, id: Option<ReportId>, dur: Duration) {
        info!("Set idle rate for {:?} to {:?}", id, dur);
    }

    fn get_idle(&self, id: Option<ReportId>) -> Option<Duration> {
        info!("Get idle rate for {:?}", id);
        None
    }
}

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{bind_interrupts, gpio, peripherals::USB};
use embassy_time::{Duration, Timer};
use embassy_usb::host::{Channel, ControlChannelExt, DeviceDescriptor, USBDescriptor, UsbDeviceRegistry, UsbHost};
use embassy_usb_driver::host::{channel, UsbHostDriver};
use gpio::{Level, Output};
use heapless::Vec;

use {defmt_rtt as _, panic_probe as _};

use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => embassy_rp::usb::host::InterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialise Peripherals
    let p = embassy_rp::init(Default::default());

    // Create the driver, from the HAL.
    let mut driver = embassy_rp::usb::host::Driver::new(p.USB, Irqs);
    usbhost.start();
    // let mut host = UsbHost::new(driver);

    debug!("Detecting device");
    // Wait for root-port to detect device
    let speed = loop {
        match usbhost.wait_for_device_event().await {
            Connected(speed) => break speed,
            _ => {}
        }
    };

    println!("Found device with speed = {:?}", speed);

    let mut descriptor_buf = [0u8; 512];
    let enum_info = usbhost.enumerate_root(speed, 1, &mut descriptor_buf).await.unwrap();
    let mut kbd = KbdHandler::try_register(&usbhost, enum_info)
        .await
        .expect("Couldn't register keyboard");

    loop {
        let result = kbd.wait_for_event().await;
        debug!("{}", result);
    }
}

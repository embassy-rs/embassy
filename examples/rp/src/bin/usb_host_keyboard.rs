#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{bind_interrupts, peripherals::USB};
use embassy_usb_driver::host::UsbHostDriver;
use embassy_usb_driver::host::DeviceEvent::Connected;
use embassy_usb::host::UsbHostBusExt;
use embassy_usb::handlers::{UsbHostHandler, kbd::KbdHandler};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => embassy_rp::usb::host::InterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialise Peripherals
    let p = embassy_rp::init(Default::default());

    // Create the driver, from the HAL.
    let mut usbhost = embassy_rp::usb::host::Driver::new(p.USB, Irqs);

    debug!("Detecting device");
    // Wait for root-port to detect device
    let speed = loop {
        match usbhost.wait_for_device_event().await {
            Connected(speed) => break speed,
            _ => {}
        }
    };

    println!("Found device with speed = {:?}", speed);

    let enum_info = usbhost.enumerate_root_bare(speed, 1).await.unwrap();
    let mut kbd = KbdHandler::try_register(&usbhost, &enum_info)
        .await
        .expect("Couldn't register keyboard");

    loop {
        let result = kbd.wait_for_event().await;
        debug!("{}", result);
    }
}

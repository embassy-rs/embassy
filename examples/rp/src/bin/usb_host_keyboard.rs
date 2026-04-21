#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_usb_host::class::hid::HidHost;
use embassy_usb_host::{BusRoute, UsbHost};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => embassy_rp::usb::host::InterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let driver = embassy_rp::usb::host::Driver::new(p.USB, Irqs);
    let mut host = UsbHost::new(driver);

    info!("USB host initialized, waiting for device...");

    loop {
        let speed = host.wait_for_connection().await;
        info!("Device connected at speed {:?}", speed);

        let mut config_buf = [0u8; 256];
        let result = host.enumerate(BusRoute::Direct(speed), &mut config_buf).await;

        let (enum_info, config_len) = match result {
            Ok(r) => r,
            Err(e) => {
                error!("Enumeration failed: {:?}", e);
                continue;
            }
        };

        info!(
            "Enumerated: VID={:04x} PID={:04x} addr={}",
            enum_info.device_desc.vendor_id, enum_info.device_desc.product_id, enum_info.device_address
        );

        let mut hid = match HidHost::new(host.driver(), &config_buf[..config_len], &enum_info) {
            Ok(h) => h,
            Err(e) => {
                error!("HID init failed: {:?}", e);
                continue;
            }
        };

        if let Err(e) = hid.set_idle(0, 0).await {
            error!("SET_IDLE failed: {:?}", e);
            continue;
        }

        info!("HID device ready, reading reports...");

        let mut buf = [0u8; 64];
        loop {
            match hid.read(&mut buf).await {
                Ok(n) if n > 0 => {
                    info!("HID report: {:x}", &buf[..n]);
                }
                Ok(_) => {}
                Err(e) => {
                    error!("HID read failed: {:?}", e);
                    break;
                }
            }
        }

        info!("Device disconnected, waiting for next...");
    }
}

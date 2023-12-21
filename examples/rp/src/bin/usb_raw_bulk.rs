//! Example of using USB without a pre-defined class, but instead using raw USB bulk transfers.
//!
//! Example code to send/receive data using `nusb`:
//!
//! ```ignore
//! use futures_lite::future::block_on;
//! use nusb::transfer::RequestBuffer;
//!
//! const BULK_OUT_EP: u8 = 0x01;
//! const BULK_IN_EP: u8 = 0x81;
//!
//! fn main() {
//!     let di = nusb::list_devices()
//!         .unwrap()
//!         .find(|d| d.vendor_id() == 0xc0de && d.product_id() == 0xcafe)
//!         .expect("no device found");
//!     let device = di.open().expect("error opening device");
//!     let interface = device.claim_interface(0).expect("error claiming interface");
//!
//!     let result = block_on(interface.bulk_out(BULK_OUT_EP, b"hello world".into()));
//!     println!("{result:?}");
//!     let result = block_on(interface.bulk_in(BULK_IN_EP, RequestBuffer::new(64)));
//!     println!("{result:?}");
//! }
//! ```

#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_usb::driver::{Endpoint, EndpointIn, EndpointOut};
use embassy_usb::msos::{self, windows_version};
use embassy_usb::{Builder, Config};
use {defmt_rtt as _, panic_probe as _};

// This is a randomly generated GUID to allow clients on Windows to find our device
const DEVICE_INTERFACE_GUIDS: &[&str] = &["{AFB9A6FB-30BA-44BC-9232-806CFC875321}"];

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello there!");

    let p = embassy_rp::init(Default::default());

    // Create the driver, from the HAL.
    let driver = Driver::new(p.USB, Irqs);

    // Create embassy-usb Config
    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB raw example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // // Required for windows compatibility.
    // // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut device_descriptor = [0; 256];
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut msos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let mut builder = Builder::new(
        driver,
        config,
        &mut device_descriptor,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );

    // Add the Microsoft OS Descriptor (MSOS/MOD) descriptor.
    // We tell Windows that this entire device is compatible with the "WINUSB" feature,
    // which causes it to use the built-in WinUSB driver automatically, which in turn
    // can be used by libusb/rusb software without needing a custom driver or INF file.
    // In principle you might want to call msos_feature() just on a specific function,
    // if your device also has other functions that still use standard class drivers.
    builder.msos_descriptor(windows_version::WIN8_1, 0);
    builder.msos_feature(msos::CompatibleIdFeatureDescriptor::new("WINUSB", ""));
    builder.msos_feature(msos::RegistryPropertyFeatureDescriptor::new(
        "DeviceInterfaceGUIDs",
        msos::PropertyData::RegMultiSz(DEVICE_INTERFACE_GUIDS),
    ));

    // Add a vendor-specific function (class 0xFF), and corresponding interface,
    // that uses our custom handler.
    let mut function = builder.function(0xFF, 0, 0);
    let mut interface = function.interface();
    let mut alt = interface.alt_setting(0xFF, 0, 0, None);
    let mut read_ep = alt.endpoint_bulk_out(64);
    let mut write_ep = alt.endpoint_bulk_in(64);
    drop(function);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    let usb_fut = usb.run();

    // Do stuff with the class!
    let echo_fut = async {
        loop {
            read_ep.wait_enabled().await;
            info!("Connected");
            loop {
                let mut data = [0; 64];
                match read_ep.read(&mut data).await {
                    Ok(n) => {
                        info!("Got bulk: {:a}", data[..n]);
                        // Echo back to the host:
                        write_ep.write(&data[..n]).await.ok();
                    }
                    Err(_) => break,
                }
            }
            info!("Disconnected");
        }
    };

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, echo_fut).await;
}

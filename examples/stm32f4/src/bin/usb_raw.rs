//! Example of using USB without a pre-defined class, but instead responding to
//! raw USB control requests.
//!
//! The host computer can either:
//! * send a command, with a 16-bit request ID, a 16-bit value, and an optional data buffer
//! * request some data, with a 16-bit request ID, a 16-bit value, and a length of data to receive
//!
//! For higher throughput data, you can add some bulk endpoints after creating the alternate,
//! but for low rate command/response, plain control transfers can be very simple and effective.
//!
//! Example code to send/receive data using `nusb`:
//!
//! ```ignore
//! use futures_lite::future::block_on;
//! use nusb::transfer::{ControlIn, ControlOut, ControlType, Recipient};
//!
//! fn main() {
//!     let di = nusb::list_devices()
//!         .unwrap()
//!         .find(|d| d.vendor_id() == 0xc0de && d.product_id() == 0xcafe)
//!         .expect("no device found");
//!     let device = di.open().expect("error opening device");
//!     let interface = device.claim_interface(0).expect("error claiming interface");
//!
//!     // Send "hello world" to device
//!     let result = block_on(interface.control_out(ControlOut {
//!         control_type: ControlType::Vendor,
//!         recipient: Recipient::Interface,
//!         request: 100,
//!         value: 200,
//!         index: 0,
//!         data: b"hello world",
//!     }));
//!     println!("{result:?}");
//!
//!     // Receive "hello" from device
//!     let result = block_on(interface.control_in(ControlIn {
//!         control_type: ControlType::Vendor,
//!         recipient: Recipient::Interface,
//!         request: 101,
//!         value: 201,
//!         index: 0,
//!         length: 5,
//!     }));
//!     println!("{result:?}");
//! }
//! ```

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::time::Hertz;
use embassy_stm32::usb_otg::Driver;
use embassy_stm32::{bind_interrupts, peripherals, usb_otg, Config};
use embassy_usb::control::{InResponse, OutResponse, Recipient, Request, RequestType};
use embassy_usb::msos::{self, windows_version};
use embassy_usb::types::InterfaceNumber;
use embassy_usb::{Builder, Handler};
use {defmt_rtt as _, panic_probe as _};

// Randomly generated UUID because Windows requires you provide one to use WinUSB.
// In principle WinUSB-using software could find this device (or a specific interface
// on it) by its GUID instead of using the VID/PID, but in practice that seems unhelpful.
const DEVICE_INTERFACE_GUIDS: &[&str] = &["{DAC2087C-63FA-458D-A55D-827C0762DEC7}"];

bind_interrupts!(struct Irqs {
    OTG_FS => usb_otg::InterruptHandler<peripherals::USB_OTG_FS>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Bypass,
        });
        config.rcc.pll_src = PllSource::HSE;
        config.rcc.pll = Some(Pll {
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL168,
            divp: Some(PllPDiv::DIV2), // 8mhz / 4 * 168 / 2 = 168Mhz.
            divq: Some(PllQDiv::DIV7), // 8mhz / 4 * 168 / 7 = 48Mhz.
            divr: None,
        });
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV4;
        config.rcc.apb2_pre = APBPrescaler::DIV2;
        config.rcc.sys = Sysclk::PLL1_P;
    }
    let p = embassy_stm32::init(config);

    // Create the driver, from the HAL.
    let mut ep_out_buffer = [0u8; 256];
    let mut config = embassy_stm32::usb_otg::Config::default();
    config.vbus_detection = true;
    let driver = Driver::new_fs(p.USB_OTG_FS, Irqs, p.PA12, p.PA11, &mut ep_out_buffer, config);

    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-raw example");
    config.serial_number = Some("12345678");

    // Required for windows compatibility.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
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

    let mut handler = ControlHandler {
        if_num: InterfaceNumber(0),
    };

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
    let _alternate = interface.alt_setting(0xFF, 0, 0, None);
    handler.if_num = interface.interface_number();
    drop(function);
    builder.handler(&mut handler);

    // Build the builder.
    let mut usb = builder.build();

    // Run the USB device.
    usb.run().await;
}

/// Handle CONTROL endpoint requests and responses. For many simple requests and responses
/// you can get away with only using the control endpoint.
struct ControlHandler {
    if_num: InterfaceNumber,
}

impl Handler for ControlHandler {
    /// Respond to HostToDevice control messages, where the host sends us a command and
    /// optionally some data, and we can only acknowledge or reject it.
    fn control_out<'a>(&'a mut self, req: Request, buf: &'a [u8]) -> Option<OutResponse> {
        // Log the request before filtering to help with debugging.
        info!("Got control_out, request={}, buf={:a}", req, buf);

        // Only handle Vendor request types to an Interface.
        if req.request_type != RequestType::Vendor || req.recipient != Recipient::Interface {
            return None;
        }

        // Ignore requests to other interfaces.
        if req.index != self.if_num.0 as u16 {
            return None;
        }

        // Accept request 100, value 200, reject others.
        if req.request == 100 && req.value == 200 {
            Some(OutResponse::Accepted)
        } else {
            Some(OutResponse::Rejected)
        }
    }

    /// Respond to DeviceToHost control messages, where the host requests some data from us.
    fn control_in<'a>(&'a mut self, req: Request, buf: &'a mut [u8]) -> Option<InResponse<'a>> {
        info!("Got control_in, request={}", req);

        // Only handle Vendor request types to an Interface.
        if req.request_type != RequestType::Vendor || req.recipient != Recipient::Interface {
            return None;
        }

        // Ignore requests to other interfaces.
        if req.index != self.if_num.0 as u16 {
            return None;
        }

        // Respond "hello" to request 101, value 201, when asked for 5 bytes, otherwise reject.
        if req.request == 101 && req.value == 201 && req.length == 5 {
            buf[..5].copy_from_slice(b"hello");
            Some(InResponse::Accepted(&buf[..5]))
        } else {
            Some(InResponse::Rejected)
        }
    }
}

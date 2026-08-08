#![no_std]
use defmt::panic;
use embassy_usb::UsbDeviceSpeed;
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::{Driver, EndpointError};
use embassy_usb::msos::{self, windows_version};
use embassy_usb::types::InterfaceNumber;

pub struct UsbResources<'d> {
    pub(crate) config_descriptor: [u8; 256],
    pub(crate) bos_descriptor: [u8; 256],
    pub(crate) msos_descriptor: [u8; 256],
    pub(crate) control_buf: [u8; 64],
    pub(crate) state: State<'d>,
}

impl<'d> UsbResources<'d> {
    pub const fn new() -> Self {
        Self {
            config_descriptor: [0; 256],
            bos_descriptor: [0; 256],
            msos_descriptor: [0; 256],
            control_buf: [0; 64],
            state: State::new(),
        }
    }
}

pub struct UsbParams {
    pub pid: u16,
    pub product: &'static str,
    pub serial: &'static str,
    pub max_speed: UsbDeviceSpeed,
}

/// Identifies the throughput peers' data interface to WinUSB.
const DEVICE_INTERFACE_GUIDS: &[&str] = &["{2F1B9A44-6C0E-4E1C-9C55-3B7A1D0E5F62}"];

/// A plain CDC-ACM device: `usbser.sys` on Windows, `cdc_acm` on Linux.
pub(crate) fn cdc<'d, D: Driver<'d>>(
    driver: D,
    resources: &'d mut UsbResources<'d>,
    params: UsbParams,
    mps: u16,
) -> (embassy_usb::UsbDevice<'d, D>, CdcAcmClass<'d, D>) {
    build(driver, resources, params, mps, false)
}

/// The same device, additionally advertising WinUSB compatibility.
///
/// Windows' CDC driver drops packets on a sustained high-speed bulk IN stream, so
/// `scripts/usb_throughput.py` reads the endpoints through libusb there and needs Windows
/// to bind WinUSB instead of `usbser.sys`. Linux ignores these descriptors and still binds
/// `cdc_acm`, so the port stays a normal CDC node there.
pub(crate) fn cdc_winusb<'d, D: Driver<'d>>(
    driver: D,
    resources: &'d mut UsbResources<'d>,
    params: UsbParams,
    mps: u16,
) -> (embassy_usb::UsbDevice<'d, D>, CdcAcmClass<'d, D>) {
    build(driver, resources, params, mps, true)
}

fn build<'d, D: Driver<'d>>(
    driver: D,
    resources: &'d mut UsbResources<'d>,
    params: UsbParams,
    mps: u16,
    winusb: bool,
) -> (embassy_usb::UsbDevice<'d, D>, CdcAcmClass<'d, D>) {
    let mut config = embassy_usb::Config::new(0xc0de, params.pid);
    config.manufacturer = Some("Embassy");
    config.product = Some(params.product);
    config.serial_number = Some(params.serial);
    config.max_power = 100;
    config.max_packet_size_0 = 64;
    config.max_speed = params.max_speed;
    let mut builder = embassy_usb::Builder::new(
        driver,
        config,
        &mut resources.config_descriptor,
        &mut resources.bos_descriptor,
        &mut resources.msos_descriptor,
        &mut resources.control_buf,
    );
    if winusb {
        builder.msos_descriptor(windows_version::WIN8_1, 0x20);
    }
    let class = CdcAcmClass::new(&mut builder, &mut resources.state, mps);
    if winusb {
        // CDC-ACM is a composite function, so `usbccgp` binds the function rather than the
        // device: the compatible ID has to sit in a function subset naming the function's
        // first interface. The class is already built, so the subset is written by hand.
        let msos_writer = builder.msos_writer();
        msos_writer.configuration(0);
        msos_writer.function(InterfaceNumber(0));
        msos_writer.function_feature(msos::CompatibleIdFeatureDescriptor::new("WINUSB", ""));
        msos_writer.function_feature(msos::RegistryPropertyFeatureDescriptor::new(
            "DeviceInterfaceGUIDs",
            msos::PropertyData::RegMultiSz(DEVICE_INTERFACE_GUIDS),
        ));
    }
    (builder.build(), class)
}

pub(crate) struct Disconnected;

impl From<EndpointError> for Disconnected {
    fn from(error: EndpointError) -> Self {
        match error {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected,
        }
    }
}

pub mod serial;
pub mod throughput;
pub mod throughput_device;
